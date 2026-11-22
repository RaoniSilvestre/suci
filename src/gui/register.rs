use super::pointers::RegisterParameterPointer;
use crate::errors::MessageParseError;
use crate::{MessageParseErrorKind, parsing};
use core::fmt::{Display, Formatter};
use core::str::{FromStr, Split};
use std::borrow::Cow;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Tries to register with the engine.
///
/// <https://backscattering.de/chess/uci/#gui-register>
pub enum Register<'a> {
    Later,
    Name(Cow<'a, str>),
    Code(Cow<'a, str>),
    NameAndCode {
        name: Cow<'a, str>,
        code: Cow<'a, str>,
    },
}

impl Register<'_> {
    pub(crate) const NAME: &'static str = "register";

    /// Calls [`Cow::into_owned`] on each [`Cow`] field.
    /// The resulting value has a `'static` lifetime.
    #[must_use]
    pub fn into_owned(self) -> Register<'static> {
        match self {
            Self::Later => Register::Later,
            Self::Name(name) => Register::Name(Cow::Owned(name.into_owned())),
            Self::Code(code) => Register::Code(Cow::Owned(code.into_owned())),
            Self::NameAndCode { name, code } => Register::NameAndCode {
                name: Cow::Owned(name.into_owned()),
                code: Cow::Owned(code.into_owned()),
            },
        }
    }

    pub(crate) fn from_str_parts(parts: Split<'_, char>) -> Result<Self, MessageParseError> {
        let mut name = None;
        let mut code = None;
        let parameter_fn = |parameter, _, value: &str, parts| {
            match parameter {
                RegisterParameterPointer::Name => name = Some(value.to_owned()),
                RegisterParameterPointer::Code => code = Some(value.to_owned()),
            }

            Some(parts)
        };

        let mut value = String::with_capacity(200);
        let value = parsing::apply_parameters(parts, &mut value, parameter_fn).unwrap_or_default();

        // CLIPPY: It is less readable and doesn't work
        #[allow(clippy::option_if_let_else)]
        if let Some(name) = name {
            if let Some(code) = code {
                Ok(Self::NameAndCode {
                    name: Cow::Owned(name),
                    code: Cow::Owned(code),
                })
            } else {
                Ok(Self::Name(Cow::Owned(name)))
            }
        } else if let Some(code) = code {
            Ok(Self::Code(Cow::Owned(code)))
        } else if value.split(' ').any(|s| s == "later") {
            Ok(Self::Later)
        } else {
            Err(MessageParseError {
                expected: "name, code, or the \"later\" value",
                kind: MessageParseErrorKind::MissingParameters,
            })
        }
    }
}

impl FromStr for Register<'_> {
    type Err = MessageParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = parsing::collect_message(Self::NAME, s)?;
        Self::from_str_parts(parts)
    }
}

impl Display for Register<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Later => f.write_str("register later"),
            Self::Name(name) => write!(f, "register name {name}"),
            Self::Code(code) => write!(f, "register code {code}"),
            Self::NameAndCode { name, code } => {
                write!(f, "register name {name} code {code}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Register;
    use std::borrow::Cow;

    #[test]
    fn to_from_str() {
        let m = Register::NameAndCode {
            name: Cow::Borrowed("john later smith"),
            code: Cow::Borrowed("31 tango"),
        };

        let s = "register name john later smith code 31 tango";

        assert_eq!(m.to_string(), s);
        assert_eq!(s.parse::<Register>(), Ok(m));
    }

    #[test]
    fn to_from_str_later() {
        let m = Register::Later;

        assert_eq!(
            "register   later   ff\n\n".parse::<Register>(),
            Ok(m.clone())
        );
        assert_eq!(m.to_string(), "register later");
    }

    #[test]
    fn invalid_parameter() {
        let m = Register::Name(Cow::Borrowed("a l o t o f s p a c e s"));

        assert_eq!(
            "register blahblah woo name a l o t o f s p a c e s\n".parse::<Register>(),
            Ok(m.clone())
        );
        assert_eq!(m.to_string(), "register name a l o t o f s p a c e s");
    }
}
