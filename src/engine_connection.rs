use crate::errors::{ReadError, ReadWriteError};
use crate::{BestMove, EngineMessage, EngineOption, FromProcessError, Go, GuiMessage, Id, Info};
use std::io;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout};
use std::str::FromStr;

/// Communicate with a chess engine.
///
/// # Important warning
/// This struct contains convenience functions for messages like [`Go`], which follow the UCI
/// protocol, but engines might not follow it completely.
/// If that's the case, the functions might fail inappropriately or hang.
///
/// Some engines send unrecognized messages, which can safely be ignored, but since this crate
/// can't parse them, they will result in an error.
///
/// If you're dealing with an engine like that, you can set [`Engine.strict`](Engine#structfield.strict)
/// to `false`. **Keep reading.**
///
/// Engines are not the only ones to blame.
/// A major crutch of the protocol is that there's no way for the engine to tell the GUI
/// about errors. So, engines will probably send a string, which this crate cannot recognize,
/// which will result in a parsing error if [`Engine.strict`](Engine#structfield.strict) is `true`,
/// but if it's `false`, the function will hang.
///
/// For this reason, you should leave [`Engine.strict`](Engine#structfield.strict) set to `true`
/// if possible. If you know when the engine sends an unrecognized string, set it to `false`
/// temporarily, or better yet, read a line and discard the unrecognized string, while keeping
/// [`Engine.strict`](Engine#structfield.strict) set to `true`.
///
/// You can see this discarding in the examples when working with Stockfish,
/// because Stockfish sends an unrecognized string when started.
#[derive(Debug)]
pub struct Engine<E, G> {
    /// The output of the engine.
    pub engine: E,
    /// The output of the GUI.
    pub gui: G,
    /// Whether message parsing errors should be ignored.
    ///
    /// See [`Engine#important-warning`](Engine#important-warning) for more.
    ///
    /// If this is `false`, [`ReadError::Parse`] is guaranteed not to occur, including inside
    /// of [`ReadWriteError`]s.
    pub strict: bool,
}

impl Engine<BufReader<ChildStdout>, ChildStdin> {
    #[allow(clippy::missing_errors_doc)]
    /// Uses the `stdin` and `stdout` from an existing process.
    ///
    /// See also [`Engine.strict`](Engine#structfield.strict).
    pub fn from_process(process: &mut Child, strict: bool) -> Result<Self, FromProcessError> {
        let Some(stdout) = process.stdout.take() else {
            return Err(FromProcessError::StdoutNotCaptured);
        };

        let Some(stdin) = process.stdin.take() else {
            return Err(FromProcessError::StdinNotCaptured);
        };

        let stdout = BufReader::new(stdout);

        Ok(Self {
            engine: stdout,
            gui: stdin,
            strict,
        })
    }
}

impl<E, G> Engine<E, G>
where
    G: Write,
{
    /// Sends a message.
    ///
    /// # Errors
    /// See [`Write::write_all`].
    pub fn send<'a, M>(&mut self, message: M) -> io::Result<()>
    where
        M: Into<GuiMessage<'a>>,
    {
        let message = message.into();
        self.gui.write_all((message.to_string() + "\n").as_bytes())
    }
}

impl<E, G> Engine<E, G>
where
    E: BufRead,
{
    /// Skips some lines.
    ///
    /// # Errors
    /// See [`BufRead::read_until`].
    pub fn skip_lines(&mut self, count: usize) -> io::Result<()> {
        let mut buf = Vec::with_capacity(512);

        for _ in 0..count {
            let bytes = self.engine.read_until(b'\n', &mut buf)?;

            if bytes == 0 {
                break;
            }

            buf.clear();
        }

        Ok(())
    }

    #[allow(clippy::missing_errors_doc)]
    /// Reads a line and attempts to parse it into an [`EngineMessage`].
    ///
    /// Skips lines which are only composed of whitespace.
    pub fn read(&mut self) -> Result<EngineMessage<'static>, ReadError> {
        let mut line = String::with_capacity(100);

        loop {
            self.engine.read_line(&mut line).map_err(ReadError::Io)?;

            if line.trim().is_empty() {
                line.clear();
                continue;
            }

            match EngineMessage::from_str(&line) {
                Ok(message) => return Ok(message),
                Err(_) if !self.strict => line.clear(),
                Err(error) => return Err(ReadError::Parse { error, got: line }),
            }
        }
    }
}

impl<E, G> Engine<E, G>
where
    E: BufRead,
    G: Write,
{
    #[allow(clippy::missing_errors_doc)]
    /// Sends the [`Uci`](crate::Uci) message and returns the engine's [`Id`]
    /// once the [`UciOk`](crate::UciOk) message is received.
    ///
    /// When an [`EngineOption`] is encountered, the `option_receiver` function is called.
    pub fn use_uci(
        &mut self,
        mut option_receiver: impl FnMut(EngineOption<'static>),
    ) -> Result<Option<Id<'static>>, ReadWriteError> {
        self.send(crate::Uci).map_err(ReadWriteError::Write)?;

        let mut id = None::<Id>;

        loop {
            match self.read().map_err(ReadWriteError::Read)? {
                EngineMessage::Option(option) => option_receiver(option),
                EngineMessage::Id(new_id) => {
                    id = Some(if let Some(id) = id {
                        id.updated(new_id)
                    } else {
                        new_id
                    });
                }
                EngineMessage::UciOk(_) => return Ok(id),
                _ => (),
            }
        }
    }

    #[allow(clippy::missing_errors_doc)]
    /// Sends [`Go`] to the engine and waits for [`BestMove`].
    /// You pass in a function through which [`Info`]s will be sent.
    ///
    /// Note that the engine will only send [`BestMove`]
    /// if you configure the message to set a constraint on the engine's calculation.
    pub fn go(
        &mut self,
        message: &Go,
        mut info_receiver: impl FnMut(Info<'static>),
    ) -> Result<BestMove, ReadWriteError> {
        self.send(message).map_err(ReadWriteError::Write)?;

        loop {
            match self.read().map_err(ReadWriteError::Read)? {
                EngineMessage::Info(info) => info_receiver(*info),
                EngineMessage::BestMove(best_move) => {
                    return Ok(best_move);
                }
                _ => (),
            }
        }
    }

    #[allow(clippy::missing_errors_doc)]
    /// Sends [`IsReady`](crate::IsReady) and waits for [`ReadyOk`](crate::ReadyOk).
    pub fn is_ready(&mut self) -> Result<(), ReadWriteError> {
        self.send(crate::IsReady).map_err(ReadWriteError::Write)?;

        loop {
            if let EngineMessage::ReadyOk(_) = self.read().map_err(ReadWriteError::Read)? {
                return Ok(());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::MessageParseError;
    use crate::{
        EngineMessage, EngineOption, EngineOptionType, MessageParseErrorKind, NormalBestMove,
    };
    use pretty_assertions::{assert_eq, assert_matches};
    use shakmaty::uci::UciMove;
    use std::borrow::Cow;

    fn input_engine(input: &[u8]) -> Engine<&[u8], Vec<u8>> {
        Engine {
            engine: input,
            gui: Vec::new(),
            strict: true,
        }
    }

    #[test]
    fn is_ready() {
        let mut engine = input_engine(b"readyok\n");

        engine.is_ready().unwrap();
        assert_eq!(engine.gui, b"isready\n");
    }

    #[test]
    fn lifetimes() {
        let mut engine = Engine {
            engine: b"uciok\noption name n type button".as_slice(),
            gui: Vec::<u8>::new(),
            strict: false,
        };

        let _: EngineMessage<'static> = engine.read().unwrap();

        assert_eq!(
            engine.read().unwrap(),
            EngineMessage::Option(EngineOption {
                name: Cow::Borrowed("n"),
                r#type: EngineOptionType::Button,
            })
        );
    }

    #[test]
    fn strict() {
        let mut engine = Engine {
            engine: b"id name Big Tuna author Fischer\noption   name Horsey range type string default the biggest!!\nuciok\nisready?no".as_slice(),
            gui: Vec::new(),
            strict: true,
        };

        engine
            .use_uci(|option| {
                assert_eq!(
                    option,
                    EngineOption {
                        name: Cow::Borrowed("Horsey range"),
                        r#type: EngineOptionType::String {
                            default: Some(Cow::Borrowed("the biggest!!"))
                        }
                    }
                );
            })
            .unwrap();

        assert_eq!(engine.gui, b"uci\n");
        let ReadWriteError::Read(ReadError::Parse { error, got }) = engine.is_ready().unwrap_err()
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

    #[test]
    fn skip_lines() {
        let mut engine = input_engine(b"one\ntwo\nthree\nreadyok\n");

        engine.skip_lines(3).unwrap();

        assert_matches!(engine.read().unwrap(), EngineMessage::ReadyOk(_));
    }

    #[test]
    fn skip_lines_typed() {
        let mut engine = input_engine(
            b"one\ntwo\nthree\noption name Hash type spin default 16 min 1 max 33554432\n",
        );

        engine.skip_lines(3).unwrap();

        assert_eq!(
            engine.read().unwrap(),
            EngineMessage::Option(EngineOption {
                name: Cow::Borrowed("Hash"),
                r#type: EngineOptionType::Spin {
                    default: Some(16),
                    min: Some(1),
                    max: Some(33_554_432)
                },
            })
        );
    }

    /// See the [`BestMove::Other`](BestMove::Other) docs for what this tests.
    #[test]
    fn analyze_checkmate() {
        let mut engine = input_engine(b"info depth 5 score mate 1 pv e2e4\nbestmove (none)\n");

        let best_move = engine
            .go(
                &Go {
                    depth: Some(5),
                    ..Default::default()
                },
                |_| {},
            )
            .unwrap();

        assert_eq!(best_move, BestMove::Other);
    }

    #[test]
    fn go() {
        let mut engine =
            input_engine(b"info depth 15 score cp 20 pv e2e4\nbestmove e2e4 ponder c7c5\n");

        let best_move = engine
            .go(
                &Go {
                    depth: Some(15),
                    ..Default::default()
                },
                |_| {},
            )
            .unwrap();

        assert_matches!(best_move, BestMove::Normal(_));

        let best_move = best_move.normal().unwrap();

        assert_eq!(
            best_move,
            NormalBestMove {
                r#move: UciMove::from_ascii(b"e2e4").unwrap(),
                ponder: Some(UciMove::from_ascii(b"c7c5").unwrap())
            }
        );
    }

    #[test]
    fn use_uci() {
        use crate::Id;

        let mut engine = input_engine(
            b"id name Big Tuna author Fischer\noption name Hash type spin default 16 min 1 max 33554432\nuciok\n",
        );

        let mut options = Vec::new();
        let id = engine.use_uci(|option| options.push(option)).unwrap();

        assert_eq!(
            id,
            Some(Id::NameAndAuthor {
                name: "Big Tuna".into(),
                author: "Fischer".into()
            })
        );

        assert_eq!(
            options,
            [EngineOption {
                name: Cow::Borrowed("Hash"),
                r#type: EngineOptionType::Spin {
                    default: Some(16),
                    min: Some(1),
                    max: Some(33_554_432)
                },
            }]
        );

        assert_eq!(engine.gui, b"uci\n");
    }
}
