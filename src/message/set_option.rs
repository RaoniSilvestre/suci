use core::fmt::{Display, Formatter};
use std::borrow::Cow;

/// Sets a configuration option on the engine.
///
/// Engines advertise their available options during the handshake; see
/// [`Engine::options`](crate::Engine::options). Send this through
/// [`Engine::set_option`](crate::Engine::set_option).
///
/// # Example
/// ```rust
/// use suci::SetOption;
///
/// let option = SetOption {
///     name: "Threads".into(),
///     value: Some("1".into()),
/// };
///
/// assert_eq!(option.to_string(), "setoption name Threads value 1");
/// ```
///
/// <https://backscattering.de/chess/uci/#gui-setoption>
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SetOption<'a> {
    /// The option name, matching an option advertised by the engine.
    pub name: Cow<'a, str>,
    /// The new value, or `None` for a button-style option.
    pub value: Option<Cow<'a, str>>,
}

impl SetOption<'_> {
    /// Calls [`Cow::into_owned`] on each [`Cow`] field.
    /// The resulting value has a `'static` lifetime.
    #[must_use]
    pub fn into_owned(self) -> SetOption<'static> {
        SetOption {
            name: Cow::Owned(self.name.into_owned()),
            value: self.value.map(Cow::into_owned).map(Cow::Owned),
        }
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
