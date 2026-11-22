use crate::errors::{ReadError, ReadWriteError};
use crate::{BestMove, Id, Info};
use crate::{FromProcessError, engine, gui};
use crate::{Go, MessageParseError};
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
    // CLIPPY: Message is implemented for borrows as well
    #[allow(clippy::needless_pass_by_value)]
    /// Sends a message.
    ///
    /// # Errors
    /// See [`Write::write_all`].
    pub fn send<M>(&mut self, message: M) -> io::Result<()>
    where
        M: gui::traits::Message,
    {
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
    /// Reads a line and attempts to parse it into a given engine message.
    ///
    /// Skips lines which are only composed of whitespace.
    ///
    /// Since this accepts the engine trait, not the enum ([`engine::Message`]), you can use a specific
    /// message type as the generic. Doing this is more performant, but it will treat
    /// the other valid engine messages the same way as it would treat an unrecognized string.
    pub fn read<M>(&mut self) -> Result<M, ReadError>
    where
        M: engine::traits::Message + FromStr<Err = MessageParseError>,
    {
        let mut line = String::with_capacity(100);

        loop {
            self.engine.read_line(&mut line).map_err(ReadError::Io)?;

            if line.trim().is_empty() {
                line.clear();
                continue;
            }

            match M::from_str(&line) {
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
    /// Sends the [`Uci`](gui::Uci) message and returns the engine's [`Id`]
    /// once the [`UciOk`](engine::UciOk) message is received.
    ///
    /// When an [`Option`](engine::Option) is encountered, the `option_receiver` function is called.
    pub fn use_uci<'m>(
        &mut self,
        mut option_receiver: impl FnMut(crate::Option<'m>),
    ) -> Result<Option<Id<'m>>, ReadWriteError> {
        self.send(crate::Uci).map_err(ReadWriteError::Write)?;

        let mut id = None::<Id>;

        loop {
            match self.read().map_err(ReadWriteError::Read)? {
                engine::Message::Option(option) => option_receiver(option),
                engine::Message::Id(new_id) => {
                    id = Some(if let Some(id) = id {
                        id.updated(new_id)
                    } else {
                        new_id
                    });
                }
                engine::Message::UciOk(_) => return Ok(id),
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
    pub fn go<'m>(
        &mut self,
        message: &Go,
        mut info_receiver: impl FnMut(Info<'m>),
    ) -> Result<BestMove, ReadWriteError> {
        self.send(message).map_err(ReadWriteError::Write)?;

        loop {
            match self.read().map_err(ReadWriteError::Read)? {
                engine::Message::Info(info) => info_receiver(*info),
                engine::Message::BestMove(best_move) => {
                    return Ok(best_move);
                }
                _ => (),
            }
        }
    }

    #[allow(clippy::missing_errors_doc)]
    /// Sends [`IsReady`](gui::IsReady) and waits for [`ReadyOk`](engine::ReadyOk).
    pub fn is_ready(&mut self) -> Result<(), ReadWriteError> {
        self.send(crate::IsReady).map_err(ReadWriteError::Write)?;

        loop {
            if let engine::Message::ReadyOk(_) = self.read().map_err(ReadWriteError::Read)? {
                return Ok(());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MessageParseErrorKind, NormalBestMove, OptionType};
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

    // CLIPPY: It's literally used???
    #[allow(clippy::extra_unused_lifetimes)]
    #[test]
    fn lifetimes<'a>() {
        let mut engine = Engine {
            engine: b"uciok\noption name n type button".as_slice(),
            gui: Vec::<u8>::new(),
            strict: false,
        };

        let _: engine::Message<'static> = engine.read::<engine::Message>().unwrap();

        assert_eq!(
            engine.read::<engine::Message>().unwrap(),
            engine::Message::Option(crate::Option {
                name: Cow::Borrowed::<'a>("n"),
                r#type: OptionType::Button,
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
                    crate::Option {
                        name: Cow::Borrowed("Horsey range"),
                        r#type: OptionType::String {
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

        assert_matches!(engine.read().unwrap(), engine::Message::ReadyOk(_));
    }

    #[test]
    fn skip_lines_typed() {
        let mut engine = input_engine(
            b"one\ntwo\nthree\noption name Hash type spin default 16 min 1 max 33554432\n",
        );

        engine.skip_lines(3).unwrap();

        assert_eq!(
            engine.read::<crate::Option>().unwrap(),
            crate::Option {
                name: Cow::Borrowed("Hash"),
                r#type: OptionType::Spin {
                    default: Some(16),
                    min: Some(1),
                    max: Some(33_554_432)
                },
            }
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
        use crate::{Id, Option};

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
            [Option {
                name: Cow::Borrowed("Hash"),
                r#type: OptionType::Spin {
                    default: Some(16),
                    min: Some(1),
                    max: Some(33_554_432)
                },
            }]
        );

        assert_eq!(engine.gui, b"uci\n");
    }
}
