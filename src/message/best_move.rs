use crate::errors::MessageParseError;
use crate::parsing;
use core::fmt::{Display, Formatter};
use core::str::{FromStr, Split};
use shakmaty::uci::UciMove;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum BestMove {
    /// This variant exists because engines can send "valid" [`BestMove`] messages, which will fail to parse.
    ///
    /// For example, this happens with Stockfish when trying to analyze a game over position, it will send back `bestmove (none)`.
    /// However, Komodo Dragon sends back a null move; `bestmove 0000`.
    ///
    /// This variant just means that the `bestmove` string
    /// was encountered, but the rest of the message was not understood.
    ///
    /// The [`Display`] impl of this variant is just `"bestmove"`.
    ///
    /// This case is not covered by the protocol description, which is why this solution
    /// is improvised and isn't great.
    Other,
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
/// The engine's best move, with an optional pondering move.
///
/// Sent after [`Go`](crate::Go) is received and calculation is finished.
///
/// <https://backscattering.de/chess/uci/#engine-bestmove>
pub struct NormalBestMove {
    pub movement: UciMove,
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
