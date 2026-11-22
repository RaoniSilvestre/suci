use crate::errors::MessageParseError;
use crate::{MessageParseErrorKind, parsing};
use core::fmt::{Display, Formatter};
use core::str::{FromStr, Split};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Tells the engine to be in debug mode.
///
/// <https://backscattering.de/chess/uci/#gui-debug>
pub struct GuiDebug(pub bool);

impl GuiDebug {
    pub(crate) const NAME: &'static str = "debug";

    pub(crate) fn from_str_parts(parts: Split<'_, char>) -> Result<Self, MessageParseError> {
        for part in parts {
            match part.trim() {
                "on" => return Ok(Self(true)),
                "off" => return Ok(Self(false)),
                _ => (),
            }
        }

        Err(MessageParseError {
            expected: "on or off",
            kind: MessageParseErrorKind::ValueParseError,
        })
    }
}

impl FromStr for GuiDebug {
    type Err = MessageParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = parsing::collect_message(Self::NAME, s)?;
        Self::from_str_parts(parts)
    }
}

impl Display for GuiDebug {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str("debug ")?;

        if self.0 {
            f.write_str("on")
        } else {
            f.write_str("off")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::GuiDebug;
    use crate::MessageParseErrorKind;
    use crate::errors::MessageParseError;

    #[test]
    fn to_from_str_on() {
        assert_eq!(GuiDebug(true).to_string(), "debug on");
        assert_eq!("debug on".parse::<GuiDebug>(), Ok(GuiDebug(true)));
    }

    #[test]
    fn to_from_str_off() {
        assert_eq!(GuiDebug(false).to_string(), "debug off");
        assert_eq!("debug off".parse::<GuiDebug>(), Ok(GuiDebug(false)));
        assert_eq!(
            "debug blah    off asffd\n".parse::<GuiDebug>(),
            Ok(GuiDebug(false))
        );
    }

    #[test]
    fn parse_error() {
        assert_eq!(
            "debug why   \t are you \nhere? 🤨".parse::<GuiDebug>(),
            Err(MessageParseError {
                expected: "on or off",
                kind: MessageParseErrorKind::ValueParseError,
            })
        );
    }
}
