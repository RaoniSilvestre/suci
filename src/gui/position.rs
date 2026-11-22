use super::pointers::PositionParameterPointer;
use crate::errors::MessageParseError;
use crate::{MessageParseErrorKind, OptionReplaceIf, parsing, uci_moves};
use core::fmt::{Display, Formatter};
use core::str::{FromStr, Split};
use shakmaty::fen::Fen;
use shakmaty::uci::UciMove;
use std::borrow::Cow;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Changes the position to analyze.
///
/// Returns an error when parsing if neither `startpos` nor `fen` parameters are present.
/// If both are present, the first one takes precedence (because Stockfish and Dragon do it like that).
///
/// <https://backscattering.de/chess/uci/#gui-position>
pub enum Position<'a> {
    StartPos {
        moves: Cow<'a, [UciMove]>,
    },
    Fen {
        fen: Cow<'a, Fen>,
        moves: Cow<'a, [UciMove]>,
    },
}

impl Position<'_> {
    pub(crate) const NAME: &'static str = "position";

    /// Calls [`Cow::into_owned`] on each [`Cow`] field.
    /// The resulting value has a `'static` lifetime.
    #[must_use]
    pub fn into_owned(self) -> Position<'static> {
        match self {
            Self::StartPos { moves } => Position::StartPos {
                moves: Cow::Owned(moves.into_owned()),
            },
            Self::Fen { fen, moves } => Position::Fen {
                fen: Cow::Owned(fen.into_owned()),
                moves: Cow::Owned(moves.into_owned()),
            },
        }
    }

    pub(crate) fn from_str_parts(parts: Split<'_, char>) -> Result<Self, MessageParseError> {
        let mut startpos = false;
        let mut fen = None::<Fen>;
        let mut moves = Vec::new();
        let parameter_fn = |parameter, _, value: &str, parts| {
            match parameter {
                PositionParameterPointer::Fen => {
                    if !startpos {
                        fen.replace_if(value.parse().ok());
                    }
                }
                PositionParameterPointer::Moves => {
                    let parsed = uci_moves::from_str(value);

                    if !parsed.is_empty() {
                        moves = parsed;
                    }
                }
                PositionParameterPointer::StartPos => {
                    if fen.is_none() {
                        startpos = true;
                    }
                }
            }
            Some(parts)
        };

        let mut value = String::with_capacity(200);
        parsing::apply_parameters(parts, &mut value, parameter_fn);

        if let Some(fen) = fen {
            Ok(Self::Fen {
                fen: Cow::Owned(fen),
                moves: Cow::Owned(moves),
            })
        } else if startpos {
            Ok(Self::StartPos {
                moves: Cow::Owned(moves),
            })
        } else {
            Err(MessageParseError {
                expected: "startpos",
                kind: MessageParseErrorKind::MissingParameters,
            })
        }
    }
}

impl FromStr for Position<'_> {
    type Err = MessageParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = parsing::collect_message(Self::NAME, s)?;
        Self::from_str_parts(parts)
    }
}

impl Display for Position<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str("position")?;

        match self {
            Self::StartPos { moves } => {
                f.write_str(" startpos")?;

                if !moves.is_empty() {
                    f.write_str(" moves ")?;
                    uci_moves::fmt(moves, f)?;
                }
            }
            Self::Fen { fen, moves } => {
                write!(f, " fen {fen}")?;

                if !moves.is_empty() {
                    f.write_str(" moves ")?;
                    uci_moves::fmt(moves, f)?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Position;
    use crate::MessageParseErrorKind;
    use crate::errors::MessageParseError;
    use shakmaty::fen::Fen;
    use shakmaty::uci::UciMove;
    use std::borrow::Cow;

    #[test]
    fn to_from_str_start_pos() {
        let moves = [
            UciMove::from_ascii(b"d2d4").unwrap(),
            UciMove::from_ascii(b"d7d5").unwrap(),
        ];
        let repr = Position::StartPos {
            moves: Cow::Borrowed(&moves),
        };

        let s = "position startpos moves d2d4 d7d5";

        assert_eq!(repr.to_string(), s);
        assert_eq!(s.parse::<Position>(), Ok(repr));

        let moves = [
            UciMove::from_ascii(b"d2d4").unwrap(),
            UciMove::from_ascii(b"d7d5").unwrap(),
        ];
        let m = Position::StartPos {
            moves: Cow::Borrowed(&moves),
        };

        assert_eq!(
            "position    startpos fen rnbqk2r/ppppp1bp/5np1/5p2/2PP4/6P1/PP2PPBP/RNBQK1NR w KQkq - 1 5 moves \t d2d4 d7d5".parse::<Position>(),
            Ok(m.clone())
        );
        assert_eq!(m.to_string(), "position startpos moves d2d4 d7d5");
    }

    #[test]
    fn to_from_str_fen() {
        let fen =
            Fen::from_ascii(b"rnbqk2r/ppppp1bp/5np1/5p2/2PP4/6P1/PP2PPBP/RNBQK1NR w KQkq - 1 5")
                .unwrap();
        let moves = [UciMove::from_ascii(b"b1c3").unwrap()];
        let m = Position::Fen {
            fen: Cow::Borrowed(&fen),
            moves: Cow::Borrowed(&moves),
        };

        let s = "position fen rnbqk2r/ppppp1bp/5np1/5p2/2PP4/6P1/PP2PPBP/RNBQK1NR w KQkq - 1 5 moves b1c3";

        assert_eq!(m.to_string(), s);
        assert_eq!(s.parse::<Position>(), Ok(m));

        let fen =
            Fen::from_ascii(b"rnbqk2r/ppppp1bp/5np1/5p2/2PP4/6P1/PP2PPBP/RNBQK1NR w KQkq - 1 5")
                .unwrap();
        let moves = [UciMove::from_ascii(b"b1c3").unwrap()];
        let m = Position::Fen {
            fen: Cow::Borrowed(&fen),
            moves: Cow::Borrowed(&moves),
        };

        assert_eq!(
            "position fen   rnbqk2r/ppppp1bp/5np1/5p2/2PP4/6P1/PP2PPBP/RNBQK1NR w KQkq - 1 5 startpos \t moves b1c3".parse::<Position>(),
            Ok(m.clone())
        );
        assert_eq!(
            m.to_string(),
            "position fen rnbqk2r/ppppp1bp/5np1/5p2/2PP4/6P1/PP2PPBP/RNBQK1NR w KQkq - 1 5 moves b1c3"
        );
    }

    #[test]
    fn invalid_tail() {
        let moves = [UciMove::from_ascii(b"d2d4").unwrap()];

        let m = Position::StartPos {
            moves: Cow::Borrowed(&moves),
        };

        assert_eq!(
            "position startpos moves d2d4 this ain't a move buddy pal".parse::<Position>(),
            Ok(m.clone())
        );
        assert_eq!(m.to_string(), "position startpos moves d2d4");
    }

    #[test]
    fn parse_error() {
        assert_eq!(
            "position moves e2e4".parse::<Position>(),
            Err(MessageParseError {
                expected: "startpos",
                kind: MessageParseErrorKind::MissingParameters,
            })
        );
    }
}
