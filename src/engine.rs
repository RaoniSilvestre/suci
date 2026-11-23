use crate::errors::{EngineStartError, ReadError, ReadWriteError};
use crate::message::Response;
use crate::{BestMove, EngineOption, Go, Id, Position, SetOption};
use core::fmt::Display;
use std::io;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, ExitStatus, Stdio};
use std::str::FromStr;
use tracing::{debug, trace};

/// Communicate with a chess engine.
///
/// [`Engine::spawn`] starts the process, captures its stdin/stdout, and performs the UCI
/// handshake. The engine owns the process and kills and reaps it when dropped.
///
/// # Important warning
/// UCI engines do not always follow the protocol completely. If an engine sends a line that
/// cannot be parsed, [`Engine::go`] and [`Engine::wait_until_ready`] can fail or hang unless
/// strict mode is disabled with [`Engine::set_strict`].
#[derive(Debug)]
pub struct Engine {
    child: Option<Child>,
    stream: EngineStream<BufReader<ChildStdout>, ChildStdin>,
    info: UciInfo,
}

/// The engine's UCI handshake information.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct UciInfo {
    pub id: Option<Id<'static>>,
    pub options: Vec<EngineOption<'static>>,
}

impl Engine {
    #[allow(clippy::missing_errors_doc)]
    /// Spawns an engine process and performs the UCI handshake.
    ///
    /// The process's stdin and stdout are set to [`Stdio::piped`].
    pub fn spawn(mut command: Command) -> Result<Self, EngineStartError> {
        command.stdin(Stdio::piped()).stdout(Stdio::piped());

        debug!(target: "suci", "starting engine");
        let mut child = command.spawn().map_err(EngineStartError::Spawn)?;

        let Some(stdin) = child.stdin.take() else {
            let _ = child.kill();
            let _ = child.wait();
            return Err(EngineStartError::Pipes(
                crate::FromProcessError::StdinNotCaptured,
            ));
        };

        let Some(stdout) = child.stdout.take() else {
            let _ = child.kill();
            let _ = child.wait();
            return Err(EngineStartError::Pipes(
                crate::FromProcessError::StdoutNotCaptured,
            ));
        };

        let mut engine = Self {
            child: Some(child),
            stream: EngineStream {
                engine: BufReader::new(stdout),
                gui: stdin,
                strict: true,
            },
            info: UciInfo::default(),
        };

        engine.info = engine
            .stream
            .handshake()
            .map_err(EngineStartError::Handshake)?;

        debug!(target: "suci", "engine started");
        Ok(engine)
    }

    /// The engine's identification, if it sent one during the handshake.
    pub const fn id(&self) -> Option<&Id<'static>> {
        self.info.id.as_ref()
    }

    /// The engine's advertised options from the handshake.
    pub const fn options(&self) -> &[EngineOption<'static>] {
        self.info.options.as_slice()
    }

    /// Enables or disables strict parsing of engine messages.
    pub const fn set_strict(&mut self, strict: bool) {
        self.stream.strict = strict;
    }

    /// Whether strict parsing is enabled.
    pub const fn strict(&self) -> bool {
        self.stream.strict
    }

    /// Sends a position to the engine.
    ///
    /// # Errors
    /// See [`Write::write_all`].
    pub fn set_position(&mut self, position: &Position<'_>) -> io::Result<()> {
        self.stream.send_message(position)
    }

    /// Sends an option to the engine.
    ///
    /// # Errors
    /// See [`Write::write_all`].
    pub fn set_option(&mut self, option: &SetOption<'_>) -> io::Result<()> {
        self.stream.send_message(option)
    }

    #[allow(clippy::missing_errors_doc)]
    /// Sends [`Go`] and waits for [`BestMove`].
    ///
    /// Received [`Info`](crate::Info) messages are logged at `trace`.
    pub fn go(&mut self, message: &Go) -> Result<BestMove, ReadWriteError> {
        self.stream.go(message)
    }

    #[allow(clippy::missing_errors_doc)]
    /// Sends `isready` and waits for `readyok`.
    pub fn wait_until_ready(&mut self) -> Result<(), ReadWriteError> {
        self.stream.wait_until_ready()
    }

    /// Tells the engine to start a new game.
    ///
    /// # Errors
    /// See [`Write::write_all`].
    pub fn new_game(&mut self) -> io::Result<()> {
        self.stream.send_command("ucinewgame")
    }

    /// Tells the engine to stop calculating.
    ///
    /// # Errors
    /// See [`Write::write_all`].
    pub fn stop(&mut self) -> io::Result<()> {
        self.stream.send_command("stop")
    }

    /// Tells the engine that pondering hit.
    ///
    /// # Errors
    /// See [`Write::write_all`].
    pub fn ponder_hit(&mut self) -> io::Result<()> {
        self.stream.send_command("ponderhit")
    }

    /// Tells the engine to quit.
    ///
    /// # Errors
    /// See [`Write::write_all`].
    pub fn quit(&mut self) -> io::Result<()> {
        self.stream.send_command("quit")
    }

    #[allow(clippy::missing_errors_doc)]
    /// Waits for the engine process to exit.
    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        let Some(child) = self.child.as_mut() else {
            return Err(io::Error::other("engine process already waited"));
        };

        debug!(target: "suci", "waiting for engine");
        let status = child.wait()?;
        self.child = None;
        debug!(target: "suci", "engine stopped");
        Ok(status)
    }

    #[allow(clippy::missing_errors_doc)]
    /// Kills the engine process.
    ///
    /// The process is reaped when the engine is dropped.
    pub fn kill(&mut self) -> io::Result<()> {
        if let Some(child) = self.child.as_mut() {
            debug!(target: "suci", "killing engine");
            child.kill()?;
        }

        Ok(())
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            debug!(target: "suci", "stopping engine");
            let _ = child.kill();
            let _ = child.wait();
            debug!(target: "suci", "engine stopped");
        }
    }
}

#[derive(Debug)]
struct EngineStream<E, G> {
    engine: E,
    gui: G,
    strict: bool,
}

impl<E, G> EngineStream<E, G>
where
    E: BufRead,
    G: Write,
{
    fn send_message(&mut self, message: &impl Display) -> io::Result<()> {
        let line = message.to_string();
        trace!(target: "suci", line = %line, "sent");
        self.gui.write_all((line + "\n").as_bytes())
    }

    fn send_command(&mut self, command: &str) -> io::Result<()> {
        trace!(target: "suci", command, "sent command");
        self.gui.write_all((command.to_string() + "\n").as_bytes())
    }

    fn read(&mut self) -> Result<Response<'static>, ReadError> {
        let mut line = String::with_capacity(100);

        loop {
            let bytes = self.engine.read_line(&mut line).map_err(ReadError::Io)?;

            if bytes == 0 {
                return Err(ReadError::Io(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "engine closed stdout",
                )));
            }

            if line.trim().is_empty() {
                line.clear();
                continue;
            }

            match Response::from_str(&line) {
                Ok(response) => {
                    trace!(target: "suci", ?response, "received");
                    return Ok(response);
                }
                Err(_) if !self.strict => {
                    trace!(target: "suci", line = %line, "ignored unrecognized line");
                    line.clear();
                }
                Err(error) => {
                    trace!(target: "suci", line = %line, "failed to parse line");
                    return Err(ReadError::Parse { error, got: line });
                }
            }
        }
    }

    fn handshake(&mut self) -> Result<UciInfo, ReadWriteError> {
        trace!(target: "suci", "handshake started");
        self.send_command("uci").map_err(ReadWriteError::Write)?;

        let mut id: Option<Id<'static>> = None;
        let mut options: Vec<EngineOption<'static>> = Vec::new();
        let mut line = String::with_capacity(100);

        loop {
            line.clear();

            let bytes = self
                .engine
                .read_line(&mut line)
                .map_err(|error| ReadWriteError::Read(ReadError::Io(error)))?;

            if bytes == 0 {
                return Err(ReadWriteError::Read(ReadError::Io(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "engine closed stdout during UCI handshake",
                ))));
            }

            if line.trim().is_empty() {
                continue;
            }

            match Response::from_str(&line) {
                Ok(Response::Id(new_id)) => {
                    trace!(target: "suci", ?new_id, "handshake id");
                    id = Some(if let Some(id) = id {
                        id.updated(new_id)
                    } else {
                        new_id
                    });
                }
                Ok(Response::Option(option)) => {
                    trace!(target: "suci", ?option, "handshake option");
                    options.push(option);
                }
                Ok(Response::UciOk) => {
                    trace!(target: "suci", "handshake uciok");
                    return Ok(UciInfo { id, options });
                }
                Ok(response) => {
                    trace!(target: "suci", ?response, "ignored handshake message");
                }
                Err(_) => {
                    trace!(target: "suci", line = %line, "ignored handshake line");
                }
            }
        }
    }

    fn go(&mut self, message: &Go) -> Result<BestMove, ReadWriteError> {
        trace!(target: "suci", ?message, "go started");
        self.send_message(message).map_err(ReadWriteError::Write)?;

        loop {
            if let Response::BestMove(best_move) = self.read().map_err(ReadWriteError::Read)? {
                trace!(target: "suci", ?best_move, "go best move");
                return Ok(best_move);
            }
        }
    }

    fn wait_until_ready(&mut self) -> Result<(), ReadWriteError> {
        trace!(target: "suci", "isready");
        self.send_command("isready")
            .map_err(ReadWriteError::Write)?;

        loop {
            if matches!(
                self.read().map_err(ReadWriteError::Read)?,
                Response::ReadyOk
            ) {
                return Ok(());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::MessageParseError;
    use crate::{EngineOption, EngineOptionType, Id, MessageParseErrorKind, NormalBestMove};
    use shakmaty::uci::UciMove;
    use std::borrow::Cow;

    fn input_engine(input: &[u8]) -> EngineStream<&[u8], Vec<u8>> {
        EngineStream {
            engine: input,
            gui: Vec::new(),
            strict: true,
        }
    }

    #[test]
    fn wait_until_ready() {
        let mut engine = input_engine(b"readyok\n");

        engine.wait_until_ready().unwrap();
        assert_eq!(engine.gui, b"isready\n");
    }

    #[test]
    fn strict() {
        let mut engine = EngineStream {
            engine: b"id name Big Tuna author Fischer\noption   name Horsey range type string default the biggest!!\nuciok\nisready?no".as_slice(),
            gui: Vec::new(),
            strict: true,
        };

        let info = engine.handshake().unwrap();

        assert_eq!(
            info.id,
            Some(Id::NameAndAuthor {
                name: "Big Tuna".into(),
                author: "Fischer".into()
            })
        );

        assert_eq!(
            info.options,
            vec![EngineOption {
                name: Cow::Borrowed("Horsey range"),
                option_type: EngineOptionType::String {
                    default: Some(Cow::Borrowed("the biggest!!"))
                }
            }]
        );

        assert_eq!(engine.gui, b"uci\n");

        let ReadWriteError::Read(ReadError::Parse { error, got }) =
            engine.wait_until_ready().unwrap_err()
        else {
            panic!("expected Parse ReadError");
        };
        let target_got = "isready?no".to_string();

        assert_eq!(
            error,
            MessageParseError {
                expected: "engine UCI message",
                kind: MessageParseErrorKind::NoMessage,
            },
        );

        assert_eq!(got, target_got);
    }

    /// See the [`BestMove::Other`](BestMove::Other) docs for what this tests.
    #[test]
    fn analyze_checkmate() {
        let mut engine = input_engine(b"info depth 5 score mate 1 pv e2e4\nbestmove (none)\n");

        let best_move = engine
            .go(&Go {
                depth: Some(5),
                ..Default::default()
            })
            .unwrap();

        assert_eq!(best_move, BestMove::Other);
    }

    #[test]
    fn go() {
        let mut engine =
            input_engine(b"info depth 15 score cp 20 pv e2e4\nbestmove e2e4 ponder c7c5\n");

        let best_move = engine
            .go(&Go {
                depth: Some(15),
                ..Default::default()
            })
            .unwrap();

        assert!(matches!(best_move, BestMove::Normal(_)));

        let best_move = best_move.normal().unwrap();

        assert_eq!(
            best_move,
            NormalBestMove {
                movement: UciMove::from_ascii(b"e2e4").unwrap(),
                ponder: Some(UciMove::from_ascii(b"c7c5").unwrap())
            }
        );
    }

    #[test]
    fn handshake() {
        let mut engine = input_engine(
            b"Stockfish 17\nid name Big Tuna author Fischer\noption name Hash type spin default 16 min 1 max 33554432\nuciok\n",
        );

        let info = engine.handshake().unwrap();

        assert_eq!(
            info.id,
            Some(Id::NameAndAuthor {
                name: "Big Tuna".into(),
                author: "Fischer".into()
            })
        );

        assert_eq!(
            info.options,
            vec![EngineOption {
                name: Cow::Borrowed("Hash"),
                option_type: EngineOptionType::Spin {
                    default: Some(16),
                    min: Some(1),
                    max: Some(33_554_432)
                },
            }]
        );

        assert_eq!(engine.gui, b"uci\n");
    }
}
