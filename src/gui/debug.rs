use super::{pointers, traits};
use crate::MessageParseErrorKind;
use crate::dev_macros::{from_str_parts, impl_message, message_from_impl};
use crate::errors::MessageParseError;
use core::fmt::{Display, Formatter};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Tells the engine to be in debug mode.
///
/// <https://backscattering.de/chess/uci/#gui-debug>
pub struct Debug(pub bool);

impl_message!(Debug);
message_from_impl!(gui Debug);
from_str_parts!(impl Debug for parts -> Result {
    for part in parts {
        match part.trim() {
            "on" => return Ok(Self(true)),
            "off" => return Ok(Self(false)),
            _ => ()
        }
    }

    Err(MessageParseError {
        expected: "on or off",
        kind: MessageParseErrorKind::ValueParseError
    })
});

impl Display for Debug {
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
    use super::Debug;
    use crate::dev_macros::{assert_from_str_message, assert_message_to_from_str};
    use crate::{MessageParseError, MessageParseErrorKind};

    #[test]
    fn to_from_str_on() {
        assert_message_to_from_str!(gui Debug(true), "debug on");
    }

    #[test]
    fn to_from_str_off() {
        assert_message_to_from_str!(gui Debug(false), "debug off");
        assert_from_str_message!(gui "debug blah    off asffd\n", Ok(Debug(false)));
    }

    #[test]
    fn parse_error() {
        assert_from_str_message!(gui "debug why   \t are you \nhere? 🤨", Err::<Debug, MessageParseError>(MessageParseError {
            expected: "on or off",
            kind: MessageParseErrorKind::ValueParseError
        }));
    }
}
