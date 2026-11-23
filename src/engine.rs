use crate::errors::{EngineStartError, MessageParseError, ReadError, ReadWriteError};
use crate::parsing;
use crate::{BestMove, EngineOption, Go, Id, Info, Position, SetOption};
use core::fmt::Display;
use std::io;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::str::FromStr;
use tracing::{debug, trace};

/// Communicate with a chess engine.
///
/// [`Engine::spawn`] starts the process, captures its stdin/stdout, and performs the UCI
/// handshake. The engine owns the process for its entire lifetime; there is no public way to
/// detach it or to keep using the engine after shutdown.
///
/// A session normally looks like this:
///
/// 1. Spawn the engine once with [`Engine::spawn`].
/// 2. Start a game with [`Engine::new_game`].
/// 3. Send a position with [`Engine::set_position`] and ask for a move with [`Engine::go`].
/// 4. Repeat step 3 for every move.
/// 5. Call [`Engine::quit`] for graceful shutdown, or simply drop the engine.
///
/// Do not spawn a new engine for every move. UCI engines are stateful sessions; the process
/// and handshake are the expensive part.
///
/// Dropping the engine kills and reaps the child process, so dropping is a safe fallback even
/// if [`Engine::quit`] is not called.
///
/// <https://backscattering.de/chess/uci/#gui>
///
/// # Example
/// ```rust,no_run
/// # use std::borrow::Cow;
/// # use std::process::Command;
/// # use suci::{Engine, Go, Position};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Spawns the process and performs the UCI handshake.
/// let mut engine = Engine::spawn(Command::new("stockfish"))?;
///
/// println!("ID: {:?}", engine.id());
///
/// engine.new_game()?;
/// engine.wait_until_ready()?;
///
/// engine.set_position(&Position::StartPos {
///     moves: Cow::Borrowed(&[]),
/// })?;
///
/// let best_move = engine.go(&Go {
///     depth: Some(12),
///     ..Default::default()
/// })?;
/// println!("best move: {best_move:?}");
///
/// // Ask the engine to exit and wait for it. Dropping would also clean it up.
/// engine.quit()?;
/// # Ok(())
/// # }
/// ```
///
/// # Strict parsing
/// UCI engines do not always follow the protocol completely. By default, an unparseable line
/// makes [`Engine::go`] and [`Engine::wait_until_ready`] return an error. Disabling strict mode
/// with [`Engine::set_strict`] skips such lines instead. If the skipped line was required for the
/// current command, the call can then wait indefinitely for a response that never arrives.
#[derive(Debug)]
pub struct Engine {
    child: Child,
    stream: EngineStream<BufReader<ChildStdout>, ChildStdin>,
    info: UciInfo,
}

/// The engine's UCI handshake information.
///
/// <https://backscattering.de/chess/uci/#gui-uci>
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct UciInfo {
    /// The engine's identification, if it sent an `id` message during the handshake.
    pub id: Option<Id<'static>>,
    /// The options advertised by the engine during the handshake.
    pub options: Vec<EngineOption<'static>>,
}

#[derive(Debug)]
enum Response<'a> {
    Id(Id<'a>),
    BestMove(BestMove),
    Info(Box<Info<'a>>),
    Option(EngineOption<'a>),
    UciOk,
    ReadyOk,
    Other,
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum ResponseKind {
    Id,
    BestMove,
    CopyProtection,
    Registration,
    Info,
    Option,
    UciOk,
    ReadyOk,
}

impl FromStr for ResponseKind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "id" => Ok(Self::Id),
            "bestmove" => Ok(Self::BestMove),
            "copyprotection" => Ok(Self::CopyProtection),
            "registration" => Ok(Self::Registration),
            "info" => Ok(Self::Info),
            "option" => Ok(Self::Option),
            "uciok" => Ok(Self::UciOk),
            "readyok" => Ok(Self::ReadyOk),
            _ => Err(()),
        }
    }
}

impl FromStr for Response<'_> {
    type Err = MessageParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (kind, parts) = parsing::collect_any_message::<ResponseKind>("engine UCI message", s)?;

        match kind {
            ResponseKind::Id => Ok(Self::Id(Id::from_str_parts(parts)?)),
            ResponseKind::BestMove => Ok(Self::BestMove(BestMove::from_str_parts(parts))),
            ResponseKind::Info => Ok(Self::Info(Box::new(Info::from_str_parts(parts)))),
            ResponseKind::Option => Ok(Self::Option(EngineOption::from_str_parts(parts)?)),
            ResponseKind::UciOk => Ok(Self::UciOk),
            ResponseKind::ReadyOk => Ok(Self::ReadyOk),
            ResponseKind::CopyProtection | ResponseKind::Registration => Ok(Self::Other),
        }
    }
}

impl Engine {
    /// Starts an engine process and performs the UCI handshake.
    ///
    /// The process's stdin and stdout are set to [`Stdio::piped`].
    ///
    /// The handshake sends `uci`, collects the engine's `id` and `option` messages, and waits
    /// for `uciok`. The resulting data is available through [`Engine::id`] and
    /// [`Engine::options`].
    ///
    /// <https://backscattering.de/chess/uci/#gui-uci>
    ///
    /// # Errors
    /// Returns [`EngineStartError::Spawn`] if the process cannot be started,
    /// [`EngineStartError::Pipes`] if stdin/stdout are missing, or
    /// [`EngineStartError::Handshake`] if the UCI handshake fails.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use std::process::Command;
    /// # use suci::Engine;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut engine = Engine::spawn(Command::new("stockfish"))?;
    /// println!("ID: {:?}", engine.id());
    /// engine.quit()?;
    /// # Ok(())
    /// # }
    /// ```
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
            child,
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
    ///
    /// <https://backscattering.de/chess/uci/#engine-id>
    pub const fn id(&self) -> Option<&Id<'static>> {
        self.info.id.as_ref()
    }

    /// The engine's advertised options from the handshake.
    ///
    /// <https://backscattering.de/chess/uci/#engine-option>
    pub const fn options(&self) -> &[EngineOption<'static>] {
        self.info.options.as_slice()
    }

    /// Enables or disables strict parsing of engine messages.
    ///
    /// Strict mode is enabled by default. When it is disabled, lines that cannot be parsed are
    /// skipped instead of returning an error. If a skipped line was required for the current
    /// command, the next read can wait indefinitely. Use this only for engines known to emit extra
    /// non-UCI lines.
    pub const fn set_strict(&mut self, strict: bool) {
        self.stream.strict = strict;
    }

    /// Whether strict parsing is enabled.
    pub const fn strict(&self) -> bool {
        self.stream.strict
    }

    /// Sends a position to the engine.
    ///
    /// UCI has no command for applying a single move. Send the complete position each turn:
    /// either [`Position::StartPos`] with the move history, or [`Position::Fen`] with a FEN and
    /// any moves played from it.
    ///
    /// <https://backscattering.de/chess/uci/#gui-position>
    ///
    /// # Errors
    /// See [`Write::write_all`].
    ///
    /// # Example
    /// ```rust,no_run
    /// # use std::borrow::Cow;
    /// # use suci::{Engine, Position};
    /// # fn example(engine: &mut Engine) -> Result<(), Box<dyn std::error::Error>> {
    /// engine.set_position(&Position::StartPos {
    ///     moves: Cow::Borrowed(&[]),
    /// })?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_position(&mut self, position: &Position<'_>) -> io::Result<()> {
        self.stream.send_message(position)
    }

    /// Sends an option to the engine.
    ///
    /// Options are usually set once after [`Engine::spawn`]. Call
    /// [`Engine::wait_until_ready`] afterward if the next command depends on the option being
    /// applied.
    ///
    /// <https://backscattering.de/chess/uci/#gui-setoption>
    ///
    /// # Errors
    /// See [`Write::write_all`].
    ///
    /// # Example
    /// ```rust,no_run
    /// # use suci::{Engine, SetOption};
    /// # fn example(engine: &mut Engine) -> Result<(), Box<dyn std::error::Error>> {
    /// engine.set_option(&SetOption {
    ///     name: "Threads".into(),
    ///     value: Some("1".into()),
    /// })?;
    /// engine.wait_until_ready()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_option(&mut self, option: &SetOption<'_>) -> io::Result<()> {
        self.stream.send_message(option)
    }

    /// Sends [`Go`] and waits for the engine's [`BestMove`].
    ///
    /// This call blocks until the engine sends `bestmove`. Received
    /// [`Info`](crate::Info) messages are logged at `trace`.
    ///
    /// <https://backscattering.de/chess/uci/#gui-go>
    ///
    /// # Errors
    /// Returns [`ReadWriteError`] if sending `go` fails or if reading the engine's response
    /// fails.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use suci::{Engine, Go};
    /// # fn example(engine: &mut Engine) -> Result<(), Box<dyn std::error::Error>> {
    /// let best_move = engine.go(&Go {
    ///     depth: Some(12),
    ///     ..Default::default()
    /// })?;
    ///
    /// if let Some(normal) = best_move.normal() {
    ///     println!("engine played: {:?}", normal.movement);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn go(&mut self, message: &Go) -> Result<BestMove, ReadWriteError> {
        self.stream.go(message)
    }

    /// Sends `isready` and waits for `readyok`.
    ///
    /// This is the UCI synchronization barrier. It is useful after changing options or starting
    /// a new game, and before a command whose result depends on earlier commands having been
    /// processed.
    ///
    /// It is not required between [`Engine::set_position`] and [`Engine::go`]; UCI commands are
    /// processed in order.
    ///
    /// <https://backscattering.de/chess/uci/#gui-isready>
    ///
    /// # Errors
    /// Returns [`ReadWriteError`] if sending or receiving fails.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use suci::Engine;
    /// # fn example(engine: &mut Engine) -> Result<(), Box<dyn std::error::Error>> {
    /// engine.new_game()?;
    /// engine.wait_until_ready()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn wait_until_ready(&mut self) -> Result<(), ReadWriteError> {
        self.stream.wait_until_ready()
    }

    /// Tells the engine to start a new game.
    ///
    /// This sends `ucinewgame`. It does not reset options or the process.
    ///
    /// <https://backscattering.de/chess/uci/#gui-ucinewgame>
    ///
    /// # Errors
    /// See [`Write::write_all`].
    pub fn new_game(&mut self) -> io::Result<()> {
        self.stream.send_command("ucinewgame")
    }

    /// Tells the engine to stop calculating.
    ///
    /// The engine should still answer the current [`Engine::go`] with `bestmove`.
    ///
    /// <https://backscattering.de/chess/uci/#gui-stop>
    ///
    /// # Errors
    /// See [`Write::write_all`].
    pub fn stop(&mut self) -> io::Result<()> {
        self.stream.send_command("stop")
    }

    /// Tells the engine that the pondered move was played.
    ///
    /// This sends `ponderhit` and is only meaningful when `go ponder` is active.
    ///
    /// <https://backscattering.de/chess/uci/#gui-ponderhit>
    ///
    /// # Errors
    /// See [`Write::write_all`].
    pub fn ponder_hit(&mut self) -> io::Result<()> {
        self.stream.send_command("ponderhit")
    }

    /// Asks the engine to exit and consumes it.
    ///
    /// This sends `quit` and waits for the process to exit. It returns any I/O error from sending
    /// the command or waiting for the process. Because it consumes the engine, the engine cannot
    /// be used after this call.
    ///
    /// Dropping an engine also kills and reaps the process. Use this method when you want an
    /// explicit graceful shutdown with error reporting; otherwise, dropping is sufficient.
    ///
    /// This method blocks until the engine exits. An engine that ignores `quit` can make it wait
    /// indefinitely; drop the engine instead if you need a forced shutdown.
    ///
    /// <https://backscattering.de/chess/uci/#gui-quit>
    ///
    /// # Errors
    /// Returns an I/O error if sending `quit` fails or if waiting for the process fails.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use suci::Engine;
    /// # fn example(engine: Engine) -> Result<(), Box<dyn std::error::Error>> {
    /// engine.quit()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn quit(mut self) -> io::Result<()> {
        debug!(target: "suci", "quitting engine");
        self.stream.send_command("quit")?;
        self.child.wait()?;
        debug!(target: "suci", "engine stopped");
        Ok(())
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        if matches!(self.child.try_wait(), Ok(Some(_))) {
            return;
        }

        debug!(target: "suci", "stopping engine");
        let _ = self.child.kill();
        let _ = self.child.wait();
        debug!(target: "suci", "engine stopped");
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
            match self.read().map_err(ReadWriteError::Read)? {
                Response::BestMove(best_move) => {
                    trace!(target: "suci", ?best_move, "go best move");
                    return Ok(best_move);
                }
                Response::Info(info) => {
                    trace!(target: "suci", ?info, "go info");
                }
                _ => (),
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
