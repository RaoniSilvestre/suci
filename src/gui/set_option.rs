use super::pointers::SetOptionParameterPointer;
use crate::errors::MessageParseError;
use crate::{MessageParseErrorKind, parsing};
use core::fmt::{Display, Formatter};
use core::str::{FromStr, Split};
use std::borrow::Cow;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Sets a configuration option ([`EngineOption`](crate::engine::EngineOption)) of the engine.
/// They are retrieved with [`Uci`](super::Uci).
///
/// <https://backscattering.de/chess/uci/#gui-setoption>
pub struct SetOption<'a> {
    pub name: Cow<'a, str>,
    pub value: Option<Cow<'a, str>>,
}

impl SetOption<'_> {
    pub(crate) const NAME: &'static str = "setoption";

    /// Calls [`Cow::into_owned`] on each [`Cow`] field.
    /// The resulting value has a `'static` lifetime.
    #[must_use]
    pub fn into_owned(self) -> SetOption<'static> {
        SetOption {
            name: Cow::Owned(self.name.into_owned()),
            value: self.value.map(Cow::into_owned).map(Cow::Owned),
        }
    }

    pub(crate) fn from_str_parts(parts: Split<'_, char>) -> Result<Self, MessageParseError> {
        let mut name = None;
        let mut value_parameter = None;
        let parameter_fn = |parameter, _, value: &str, parts| {
            match parameter {
                SetOptionParameterPointer::Name => name = Some(value.to_owned()),
                SetOptionParameterPointer::Value => {
                    value_parameter = Some(Cow::Owned(value.to_owned()));
                }
            }

            Some(parts)
        };

        let mut value = String::with_capacity(200);
        parsing::apply_parameters(parts, &mut value, parameter_fn);

        Ok(Self {
            name: Cow::Owned(name.ok_or(MessageParseError {
                expected: "name",
                kind: MessageParseErrorKind::MissingParameters,
            })?),
            value: value_parameter,
        })
    }
}

impl FromStr for SetOption<'_> {
    type Err = MessageParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = parsing::collect_message(Self::NAME, s)?;
        Self::from_str_parts(parts)
    }
}

impl Display for SetOption<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match &self.value {
            Some(value) => write!(f, "setoption name {} value {value}", self.name),
            None => write!(f, "setoption name {}", self.name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SetOption;
    use std::borrow::Cow;

    #[test]
    fn to_from_str() {
        let m = SetOption {
            name: Cow::from("Skill Level"),
            value: Some(Cow::from("1")),
        };

        let s = "setoption name Skill Level value 1";

        assert_eq!(m.to_string(), s);
        assert_eq!(s.parse::<SetOption>(), Ok(m));

        let m = SetOption {
            name: Cow::from("Skill     Level"),
            value: Some(Cow::from("test   \tfoo")),
        };

        assert_eq!(
            "setoption value test   \tfoo   \t name     Skill     Level    ".parse::<SetOption>(),
            Ok(m.clone())
        );
        assert_eq!(
            m.to_string(),
            "setoption name Skill     Level value test   \tfoo"
        );
    }
}
