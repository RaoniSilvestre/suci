use crate::errors::MessageParseError;
use crate::parsing;
use core::fmt::{Display, Formatter};
use core::str::FromStr;

/// The engine is ready for UCI.
///
/// Sent after [`Uci`](crate::gui::Uci).
///
/// <https://backscattering.de/chess/uci/#engine-uciok>
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UciOk;

impl UciOk {
    pub(crate) const NAME: &'static str = "uciok";
}

impl FromStr for UciOk {
    type Err = MessageParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let _ = parsing::collect_message(Self::NAME, s)?;
        Ok(Self)
    }
}

impl Display for UciOk {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(Self::NAME)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// The engine is ready to accept new commands.
///
/// <https://backscattering.de/chess/uci/#engine-readyok>
pub struct ReadyOk;

impl ReadyOk {
    pub(crate) const NAME: &'static str = "readyok";
}

impl FromStr for ReadyOk {
    type Err = MessageParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let _ = parsing::collect_message(Self::NAME, s)?;
        Ok(Self)
    }
}

impl Display for ReadyOk {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(Self::NAME)
    }
}

#[cfg(test)]
mod tests {
    use super::{ReadyOk, UciOk};

    #[test]
    fn uci_ok() {
        assert_eq!(UciOk.to_string(), "uciok");
        assert_eq!("uciok".parse::<UciOk>(), Ok(UciOk));
    }

    #[test]
    fn ready_ok() {
        assert_eq!(ReadyOk.to_string(), "readyok");
        assert_eq!("readyok".parse::<ReadyOk>(), Ok(ReadyOk));
    }
}
