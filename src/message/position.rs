use core::fmt::{Display, Formatter};
use shakmaty::fen::Fen;
use shakmaty::uci::UciMove;
use std::borrow::Cow;

/// Changes the position to analyze.
///
/// UCI has no command that applies a single move. Each turn sends the complete position, either
/// as the initial position plus a move history or as a FEN plus a move history. Pass the result
/// to [`Engine::set_position`](crate::Engine::set_position).
///
/// # Example
/// ```rust
/// use shakmaty::fen::Fen;
/// use std::borrow::Cow;
/// use suci::Position;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let start = Position::StartPos {
///     moves: Cow::Borrowed(&[]),
/// };
/// assert_eq!(start.to_string(), "position startpos");
///
/// let fen = Fen::from_ascii(b"4k3/8/8/8/8/8/4P3/4K3 w - - 0 1")?;
/// let position = Position::Fen {
///     fen: Cow::Owned(fen),
///     moves: Cow::Borrowed(&[]),
/// };
/// assert!(position.to_string().starts_with("position fen 4k3/8/8/8/8/8/4P3/4K3"));
/// # Ok(())
/// # }
/// ```
///
/// <https://backscattering.de/chess/uci/#gui-position>
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Position<'a> {
    /// The initial chess position, followed by a list of moves.
    StartPos {
        /// Moves played from the initial position, in UCI long algebraic notation such as
        /// `e2e4`.
        moves: Cow<'a, [UciMove]>,
    },
    /// A position given as a FEN, followed by a list of moves.
    Fen {
        /// The position from which `moves` starts.
        fen: Cow<'a, Fen>,
        /// Moves played from `fen`, in UCI long algebraic notation such as `e2e4`.
        moves: Cow<'a, [UciMove]>,
    },
}

impl Position<'_> {
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
