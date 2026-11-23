use core::fmt::{Display, Formatter};

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
pub struct SetOption {
    /// The option name, matching an option advertised by the engine.
    pub name: String,
    /// The new value, or `None` for a button-style option.
    pub value: Option<String>,
}

impl Display for SetOption {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match &self.value {
            Some(value) => write!(f, "setoption name {} value {value}", self.name),
            None => write!(f, "setoption name {}", self.name),
        }
    }
}
