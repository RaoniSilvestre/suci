use crate::errors::MessageParseError;
use crate::parsing;
use core::fmt::{Display, Formatter};
use core::str::{FromStr, Split};
use shakmaty::uci::UciMove;

/// The engine's best move, or an unparseable `bestmove` response.
///
/// [`Engine::go`](crate::Engine::go) returns this when the engine finishes searching.
///
/// # Example
/// ```rust
/// use suci::BestMove;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let best_move: BestMove = "bestmove e2e4 ponder c7c5".parse()?;
///
/// if let Some(normal) = best_move.normal() {
///     assert_eq!(normal.movement.to_string(), "e2e4");
///     assert_eq!(normal.ponder.map(|movement| movement.to_string()), Some("c7c5".to_owned()));
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum BestMove {
    /// A `bestmove` response whose move could not be parsed.
    ///
    /// Engines can send a syntactically valid `bestmove` message without a usable move. Stockfish
    /// sends `bestmove (none)` at game over, while Komodo Dragon sends the null move
    /// `bestmove 0000`. This variant records that the `bestmove` line was received even though
    /// the move was not understood.
    ///
    /// <https://backscattering.de/chess/uci/#move-null>
    ///
    /// The protocol does not define this case. Its [`Display`] implementation is just
    /// `"bestmove"`.
    Other,
    /// A normal UCI move, optionally with a move to ponder.
    Normal(NormalBestMove),
}

impl BestMove {
    pub(crate) const NAME: &'static str = "bestmove";

    /// Returns the inner [`NormalBestMove`], if [`Self`] matches [`Self::Normal`].
    pub const fn normal(self) -> Option<NormalBestMove> {
        match self {
            Self::Other => None,
            Self::Normal(n) => Some(n),
        }
    }

    pub(crate) fn from_str_parts(parts: Split<'_, char>) -> Self {
        let mut movement = None;
        let mut ponder_encountered = false;
        let mut ponder = None;

        for part in parts {
            if ponder_encountered {
                ponder = part.parse().ok().or(ponder);
            } else if part.trim() == "ponder" {
                ponder_encountered = true;
            } else {
                movement = part.parse().ok().or(movement);
            }
        }

        movement.map_or(Self::Other, |movement| {
            Self::Normal(NormalBestMove { movement, ponder })
        })
    }
}

impl FromStr for BestMove {
    type Err = MessageParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = parsing::collect_message(Self::NAME, s)?;
        Ok(Self::from_str_parts(parts))
    }
}

/// The engine's best move, with an optional pondering move.
///
/// Sent after [`Go`](crate::Go) is received and calculation is finished.
///
/// <https://backscattering.de/chess/uci/#engine-bestmove>
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct NormalBestMove {
    /// The move chosen by the engine.
    ///
    /// <https://backscattering.de/chess/uci/#engine-bestmove>
    pub movement: UciMove,
    /// The move the engine suggests pondering, if provided.
    ///
    /// <https://backscattering.de/chess/uci/#engine-bestmove-ponder>
    pub ponder: Option<UciMove>,
}

impl From<NormalBestMove> for BestMove {
    fn from(value: NormalBestMove) -> Self {
        Self::Normal(value)
    }
}

impl Display for BestMove {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str("bestmove")?;

        match &self {
            Self::Other => Ok(()),
            Self::Normal(best_move) => {
                write!(f, " {}", best_move.movement)?;

                best_move
                    .ponder
                    .as_ref()
                    .map_or(Ok(()), |ponder| write!(f, " ponder {ponder}"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BestMove, NormalBestMove};
    use shakmaty::uci::UciMove;

    #[test]
    fn to_from_str() {
        let m: BestMove = NormalBestMove {
            movement: UciMove::from_ascii(b"e2e4").unwrap(),
            ponder: Some(UciMove::from_ascii(b"c7c5").unwrap()),
        }
        .into();

        assert_eq!(m.to_string(), "bestmove e2e4 ponder c7c5");
        assert_eq!("bestmove e2e4 ponder c7c5".parse::<BestMove>(), Ok(m));

        let m: BestMove = NormalBestMove {
            movement: UciMove::from_ascii(b"d2d4").unwrap(),
            ponder: Some(UciMove::from_ascii(b"c7c5").unwrap()),
        }
        .into();

        assert_eq!(
            "bestmove oops d2d4 ponder c7c5 ignorethis".parse::<BestMove>(),
            Ok(m)
        );
        assert_eq!(m.to_string(), "bestmove d2d4 ponder c7c5");
    }

    #[test]
    fn to_from_str_bad_value() {
        let m: BestMove = NormalBestMove {
            movement: UciMove::from_ascii(b"e2e4").unwrap(),
            ponder: Some(UciMove::from_ascii(b"c7c5").unwrap()),
        }
        .into();

        assert_eq!(
            "bestmove junk e2e4 ponder c7c5\n".parse::<BestMove>(),
            Ok(m)
        );
        assert_eq!(m.to_string(), "bestmove e2e4 ponder c7c5");
    }

    #[test]
    fn to_from_str_other() {
        let m: BestMove = BestMove::Other;

        assert_eq!("bestmove (none)\n".parse::<BestMove>(), Ok(m));
        assert_eq!(m.to_string(), "bestmove");
    }
}
