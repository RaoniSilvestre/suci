use core::fmt::{Display, Formatter};
use std::borrow::Cow;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Sets a configuration option (`EngineOption`) of the engine.
/// They are retrieved during the UCI handshake.
///
/// <https://backscattering.de/chess/uci/#gui-setoption>
pub struct SetOption<'a> {
    pub name: Cow<'a, str>,
    pub value: Option<Cow<'a, str>>,
}

impl SetOption<'_> {
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
