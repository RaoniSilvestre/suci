use core::fmt::{Display, Formatter};
use shakmaty::fen::Fen;
use shakmaty::uci::UciMove;
use std::borrow::Cow;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Changes the position to analyze.
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
}

impl Display for Position<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str("position")?;

        match self {
            Self::StartPos { moves } => {
                f.write_str(" startpos")?;

                if !moves.is_empty() {
                    f.write_str(" moves ")?;
                    super::format_moves(moves, f)?;
                }
            }
            Self::Fen { fen, moves } => {
                write!(f, " fen {fen}")?;

                if !moves.is_empty() {
                    f.write_str(" moves ")?;
                    super::format_moves(moves, f)?;
                }
            }
        }

        Ok(())
    }
}
