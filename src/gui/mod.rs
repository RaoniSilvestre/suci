mod debug;
mod empty;
mod go;
pub(crate) mod pointers;
mod position;
mod register;
mod set_option;

pub use debug::*;
pub use empty::*;
pub use go::*;
pub use position::*;
pub use register::*;
pub use set_option::*;

use crate::errors::MessageParseError;
use crate::{UciMessage, parsing};
use core::fmt::{Display, Formatter};
use core::str::FromStr;

/// A message sent from the GUI to the engine.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GuiMessage<'a> {
    Debug(GuiDebug),
    SetOption(SetOption<'a>),
    Register(Register<'a>),
    Position(Position<'a>),
    Go(Go<'a>),
    Uci(Uci),
    IsReady(IsReady),
    UciNewGame(UciNewGame),
    Stop(Stop),
    PonderHit(PonderHit),
    Quit(Quit),
}

impl FromStr for GuiMessage<'_> {
    type Err = MessageParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (message_pointer, parts) =
            parsing::collect_any_message::<pointers::MessagePointer>("GUI UCI message", s)?;

        match message_pointer {
            pointers::MessagePointer::Debug => Ok(GuiDebug::from_str_parts(parts)?.into()),
            pointers::MessagePointer::SetOption => Ok(SetOption::from_str_parts(parts)?.into()),
            pointers::MessagePointer::Register => Ok(Register::from_str_parts(parts)?.into()),
            pointers::MessagePointer::Position => Ok(Position::from_str_parts(parts)?.into()),
            pointers::MessagePointer::Go => Ok(Go::from_str_parts(parts).into()),
            pointers::MessagePointer::Uci => Ok(Uci.into()),
            pointers::MessagePointer::IsReady => Ok(IsReady.into()),
            pointers::MessagePointer::UciNewGame => Ok(UciNewGame.into()),
            pointers::MessagePointer::Stop => Ok(Stop.into()),
            pointers::MessagePointer::PonderHit => Ok(PonderHit.into()),
            pointers::MessagePointer::Quit => Ok(Quit.into()),
        }
    }
}

impl Display for GuiMessage<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Debug(message) => message.fmt(f),
            Self::SetOption(message) => message.fmt(f),
            Self::Register(message) => message.fmt(f),
            Self::Position(message) => message.fmt(f),
            Self::Go(message) => message.fmt(f),
            Self::Uci(message) => message.fmt(f),
            Self::IsReady(message) => message.fmt(f),
            Self::UciNewGame(message) => message.fmt(f),
            Self::Stop(message) => message.fmt(f),
            Self::PonderHit(message) => message.fmt(f),
            Self::Quit(message) => message.fmt(f),
        }
    }
}

impl From<GuiDebug> for GuiMessage<'_> {
    fn from(value: GuiDebug) -> Self {
        Self::Debug(value)
    }
}

impl From<GuiDebug> for UciMessage<'_> {
    fn from(value: GuiDebug) -> Self {
        Self::Gui(value.into())
    }
}

impl<'a> From<SetOption<'a>> for GuiMessage<'a> {
    fn from(value: SetOption<'a>) -> Self {
        Self::SetOption(value)
    }
}

impl<'a> From<SetOption<'a>> for UciMessage<'a> {
    fn from(value: SetOption<'a>) -> Self {
        Self::Gui(value.into())
    }
}

impl<'a> From<Register<'a>> for GuiMessage<'a> {
    fn from(value: Register<'a>) -> Self {
        Self::Register(value)
    }
}

impl<'a> From<Register<'a>> for UciMessage<'a> {
    fn from(value: Register<'a>) -> Self {
        Self::Gui(value.into())
    }
}

impl<'a> From<Position<'a>> for GuiMessage<'a> {
    fn from(value: Position<'a>) -> Self {
        Self::Position(value)
    }
}

impl<'a> From<Position<'a>> for UciMessage<'a> {
    fn from(value: Position<'a>) -> Self {
        Self::Gui(value.into())
    }
}

impl<'a> From<Go<'a>> for GuiMessage<'a> {
    fn from(value: Go<'a>) -> Self {
        Self::Go(value)
    }
}

impl<'a> From<Go<'a>> for UciMessage<'a> {
    fn from(value: Go<'a>) -> Self {
        Self::Gui(value.into())
    }
}

impl From<Uci> for GuiMessage<'_> {
    fn from(value: Uci) -> Self {
        Self::Uci(value)
    }
}

impl From<Uci> for UciMessage<'_> {
    fn from(value: Uci) -> Self {
        Self::Gui(value.into())
    }
}

impl From<IsReady> for GuiMessage<'_> {
    fn from(value: IsReady) -> Self {
        Self::IsReady(value)
    }
}

impl From<IsReady> for UciMessage<'_> {
    fn from(value: IsReady) -> Self {
        Self::Gui(value.into())
    }
}

impl From<UciNewGame> for GuiMessage<'_> {
    fn from(value: UciNewGame) -> Self {
        Self::UciNewGame(value)
    }
}

impl From<UciNewGame> for UciMessage<'_> {
    fn from(value: UciNewGame) -> Self {
        Self::Gui(value.into())
    }
}

impl From<Stop> for GuiMessage<'_> {
    fn from(value: Stop) -> Self {
        Self::Stop(value)
    }
}

impl From<Stop> for UciMessage<'_> {
    fn from(value: Stop) -> Self {
        Self::Gui(value.into())
    }
}

impl From<PonderHit> for GuiMessage<'_> {
    fn from(value: PonderHit) -> Self {
        Self::PonderHit(value)
    }
}

impl From<PonderHit> for UciMessage<'_> {
    fn from(value: PonderHit) -> Self {
        Self::Gui(value.into())
    }
}

impl From<Quit> for GuiMessage<'_> {
    fn from(value: Quit) -> Self {
        Self::Quit(value)
    }
}

impl From<Quit> for UciMessage<'_> {
    fn from(value: Quit) -> Self {
        Self::Gui(value.into())
    }
}

impl<'a> From<GuiMessage<'a>> for UciMessage<'a> {
    fn from(value: GuiMessage<'a>) -> Self {
        Self::Gui(value)
    }
}

impl<T> From<&T> for GuiMessage<'_>
where
    T: Clone + Into<Self>,
{
    fn from(value: &T) -> Self {
        value.clone().into()
    }
}

#[cfg(test)]
mod tests {
    use super::{Go, GuiMessage, Uci};
    use crate::UciMessage;

    #[test]
    fn message_wiring() {
        let go = Go::default();

        assert_eq!(GuiMessage::from(go.clone()).to_string(), "go");
        assert_eq!(UciMessage::from(go.clone()).to_string(), "go");
        assert_eq!("go".parse::<GuiMessage>(), Ok(GuiMessage::from(go.clone())));
        assert_eq!("go".parse::<UciMessage>(), Ok(UciMessage::from(go)));

        assert_eq!(GuiMessage::from(Uci).to_string(), "uci");
        assert_eq!(UciMessage::from(Uci).to_string(), "uci");
        assert_eq!("uci".parse::<GuiMessage>(), Ok(GuiMessage::from(Uci)));
        assert_eq!("uci".parse::<UciMessage>(), Ok(UciMessage::from(Uci)));
    }
}
