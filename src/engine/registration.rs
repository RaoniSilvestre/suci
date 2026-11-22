use crate::errors::MessageParseError;
use crate::{MessageParseErrorKind, parsing};
use core::fmt::{Display, Formatter};
use core::str::{FromStr, Split};

/// Engine's registration status.
///
/// Sent after [`Uci`](crate::gui::Uci) or [`Register`](crate::gui::Register).
///
/// <https://backscattering.de/chess/uci/#engine-registration>
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Registration {
    Checking,
    Ok,
    Error,
}

impl Registration {
    pub(crate) const NAME: &'static str = "registration";

    pub(crate) fn from_str_parts(parts: Split<'_, char>) -> Result<Self, MessageParseError> {
        for part in parts {
            match part.trim() {
                "checking" => return Ok(Self::Checking),
                "ok" => return Ok(Self::Ok),
                "error" => return Ok(Self::Error),
                _ => (),
            }
        }

        Err(MessageParseError {
            expected: "checking, ok or error",
            kind: MessageParseErrorKind::ValueParseError,
        })
    }
}

impl FromStr for Registration {
    type Err = MessageParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = parsing::collect_message(Self::NAME, s)?;
        Self::from_str_parts(parts)
    }
}

impl Display for Registration {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str("registration ")?;

        match self {
            Self::Checking => f.write_str("checking")?,
            Self::Ok => f.write_str("ok")?,
            Self::Error => f.write_str("error")?,
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Registration;
    use crate::MessageParseErrorKind;
    use crate::errors::MessageParseError;

    #[test]
    fn to_from_str_checking() {
        let m = Registration::Checking;
        let str = "registration checking";

        assert_eq!(m.to_string(), str);
        assert_eq!(str.parse::<Registration>(), Ok(m));
    }

    #[test]
    fn to_from_str_ok() {
        let m = Registration::Ok;
        let str = "registration ok";

        assert_eq!(m.to_string(), str);
        assert_eq!(str.parse::<Registration>(), Ok(m));
    }

    #[test]
    fn to_from_str_error() {
        let m = Registration::Error;

        assert_eq!("registration tes error\n".parse::<Registration>(), Ok(m));
        assert_eq!(m.to_string(), "registration error");
    }

    #[test]
    fn parse_error() {
        assert_eq!(
            "registration invalid".parse::<Registration>(),
            Err(MessageParseError {
                expected: "checking, ok or error",
                kind: MessageParseErrorKind::ValueParseError,
            })
        );
    }
}
