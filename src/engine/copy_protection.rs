use crate::errors::MessageParseError;
use crate::{MessageParseErrorKind, parsing};
use core::fmt::{Display, Formatter};
use core::str::{FromStr, Split};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Engine's copy protection. GUIs should respect this.
///
/// Sent after [`UciOk`](super::UciOk).
///
/// <https://backscattering.de/chess/uci/#engine-copyprotection>
pub enum CopyProtection {
    Ok,
    Error,
}

impl CopyProtection {
    pub(crate) const NAME: &'static str = "copyprotection";

    pub(crate) fn from_str_parts(parts: Split<'_, char>) -> Result<Self, MessageParseError> {
        for part in parts {
            // TODO: Should it be lowercased (case insensitive)?
            match part.trim() {
                "ok" => return Ok(Self::Ok),
                "error" => return Ok(Self::Error),
                _ => (),
            }
        }

        Err(MessageParseError {
            expected: "ok or error",
            kind: MessageParseErrorKind::ValueParseError,
        })
    }
}

impl FromStr for CopyProtection {
    type Err = MessageParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = parsing::collect_message(Self::NAME, s)?;
        Self::from_str_parts(parts)
    }
}

impl Display for CopyProtection {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str("copyprotection ")?;

        match self {
            Self::Ok => f.write_str("ok")?,
            Self::Error => f.write_str("error")?,
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::CopyProtection;
    use crate::MessageParseErrorKind;
    use crate::errors::MessageParseError;

    #[test]
    fn to_from_str_ok() {
        let m = CopyProtection::Ok;
        let str = "copyprotection ok";

        assert_eq!(m.to_string(), str);
        assert_eq!(str.parse::<CopyProtection>(), Ok(m));
    }

    #[test]
    fn to_from_str_error() {
        let m = CopyProtection::Error;
        let str = "copyprotection error";

        assert_eq!(m.to_string(), str);
        assert_eq!(str.parse::<CopyProtection>(), Ok(m));
    }

    #[test]
    fn parse_error() {
        assert_eq!(
            "copyprotection why   \t are you here? 🤨\n\n".parse::<CopyProtection>(),
            Err(MessageParseError {
                expected: "ok or error",
                kind: MessageParseErrorKind::ValueParseError,
            })
        );
    }
}
