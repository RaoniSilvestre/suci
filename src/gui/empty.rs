use crate::errors::MessageParseError;
use crate::parsing;
use core::fmt::{Display, Formatter};
use core::str::FromStr;

fn parse_empty(name: &'static str, s: &str) -> Result<(), MessageParseError> {
    let _ = parsing::collect_message(name, s)?;
    Ok(())
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Tells the engine to use UCI.
///
/// <https://backscattering.de/chess/uci/#gui-uci>
pub struct Uci;

impl Uci {
    pub(crate) const NAME: &'static str = "uci";
}

impl FromStr for Uci {
    type Err = MessageParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_empty(Self::NAME, s)?;
        Ok(Self)
    }
}

impl Display for Uci {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(Self::NAME)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Asks the engine if it is ready.
///
/// Once ready, engine replies with [`ReadyOk`](crate::engine::ReadyOk).
///
/// <https://backscattering.de/chess/uci/#gui-isready>
pub struct IsReady;

impl IsReady {
    pub(crate) const NAME: &'static str = "isready";
}

impl FromStr for IsReady {
    type Err = MessageParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_empty(Self::NAME, s)?;
        Ok(Self)
    }
}

impl Display for IsReady {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(Self::NAME)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Tells the engine to start a new game with the next [`Position`](crate::gui::Position) or [`Go`](crate::gui::Go).
///
/// <https://backscattering.de/chess/uci/#gui-ucinewgame>
pub struct UciNewGame;

impl UciNewGame {
    pub(crate) const NAME: &'static str = "ucinewgame";
}

impl FromStr for UciNewGame {
    type Err = MessageParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_empty(Self::NAME, s)?;
        Ok(Self)
    }
}

impl Display for UciNewGame {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(Self::NAME)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Tells the engine to stop calculating.
///
/// <https://backscattering.de/chess/uci/#gui-stop>
pub struct Stop;

impl Stop {
    pub(crate) const NAME: &'static str = "stop";
}

impl FromStr for Stop {
    type Err = MessageParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_empty(Self::NAME, s)?;
        Ok(Self)
    }
}

impl Display for Stop {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(Self::NAME)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// <https://backscattering.de/chess/uci/#gui-ponderhit>
pub struct PonderHit;

impl PonderHit {
    pub(crate) const NAME: &'static str = "ponderhit";
}

impl FromStr for PonderHit {
    type Err = MessageParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_empty(Self::NAME, s)?;
        Ok(Self)
    }
}

impl Display for PonderHit {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(Self::NAME)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Tells the engine to stop everything.
///
/// <https://backscattering.de/chess/uci/#gui-quit>
pub struct Quit;

impl Quit {
    pub(crate) const NAME: &'static str = "quit";
}

impl FromStr for Quit {
    type Err = MessageParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_empty(Self::NAME, s)?;
        Ok(Self)
    }
}

impl Display for Quit {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(Self::NAME)
    }
}

#[cfg(test)]
mod tests {
    use super::{IsReady, PonderHit, Quit, Stop, Uci, UciNewGame};

    #[test]
    fn uci() {
        assert_eq!(Uci.to_string(), "uci");
        assert_eq!("uci".parse::<Uci>(), Ok(Uci));
    }

    #[test]
    fn is_ready() {
        assert_eq!(IsReady.to_string(), "isready");
        assert_eq!("isready".parse::<IsReady>(), Ok(IsReady));
    }

    #[test]
    fn uci_new_game() {
        assert_eq!(UciNewGame.to_string(), "ucinewgame");
        assert_eq!("ucinewgame".parse::<UciNewGame>(), Ok(UciNewGame));
    }

    #[test]
    fn stop() {
        assert_eq!(Stop.to_string(), "stop");
        assert_eq!("stop".parse::<Stop>(), Ok(Stop));
    }

    #[test]
    fn ponder_hit() {
        assert_eq!(PonderHit.to_string(), "ponderhit");
        assert_eq!("ponderhit".parse::<PonderHit>(), Ok(PonderHit));
    }

    #[test]
    fn quit() {
        assert_eq!(Quit.to_string(), "quit");
        assert_eq!("quit".parse::<Quit>(), Ok(Quit));
    }
}
