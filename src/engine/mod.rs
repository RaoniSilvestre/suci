mod best_move;
mod copy_protection;
mod empty;
mod id;
mod info;
mod option;
pub(crate) mod pointers;
mod registration;

pub use best_move::*;
pub use copy_protection::*;
pub use empty::*;
pub use id::*;
pub use info::*;
pub use option::*;
pub use registration::*;

use crate::errors::MessageParseError;
use crate::{UciMessage, parsing};
use core::fmt::{Display, Formatter};
use core::str::FromStr;

/// A message sent from the engine to the GUI.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EngineMessage<'a> {
    Id(Id<'a>),
    BestMove(BestMove),
    CopyProtection(CopyProtection),
    Registration(Registration),
    Info(Box<Info<'a>>),
    Option(EngineOption<'a>),
    UciOk(UciOk),
    ReadyOk(ReadyOk),
}

impl FromStr for EngineMessage<'_> {
    type Err = MessageParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (message_pointer, parts) =
            parsing::collect_any_message::<pointers::MessagePointer>("engine UCI message", s)?;

        match message_pointer {
            pointers::MessagePointer::Id => Ok(Id::from_str_parts(parts)?.into()),
            pointers::MessagePointer::BestMove => Ok(BestMove::from_str_parts(parts).into()),
            pointers::MessagePointer::CopyProtection => {
                Ok(CopyProtection::from_str_parts(parts)?.into())
            }
            pointers::MessagePointer::Registration => {
                Ok(Registration::from_str_parts(parts)?.into())
            }
            pointers::MessagePointer::Info => Ok(Info::from_str_parts(parts).into()),
            pointers::MessagePointer::Option => Ok(EngineOption::from_str_parts(parts)?.into()),
            pointers::MessagePointer::UciOk => Ok(UciOk.into()),
            pointers::MessagePointer::ReadyOk => Ok(ReadyOk.into()),
        }
    }
}

impl Display for EngineMessage<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Id(message) => message.fmt(f),
            Self::BestMove(message) => message.fmt(f),
            Self::CopyProtection(message) => message.fmt(f),
            Self::Registration(message) => message.fmt(f),
            Self::Info(message) => message.fmt(f),
            Self::Option(message) => message.fmt(f),
            Self::UciOk(message) => message.fmt(f),
            Self::ReadyOk(message) => message.fmt(f),
        }
    }
}

impl<'a> From<Id<'a>> for EngineMessage<'a> {
    fn from(value: Id<'a>) -> Self {
        Self::Id(value)
    }
}

impl<'a> From<Id<'a>> for UciMessage<'a> {
    fn from(value: Id<'a>) -> Self {
        Self::Engine(value.into())
    }
}

impl From<BestMove> for EngineMessage<'_> {
    fn from(value: BestMove) -> Self {
        Self::BestMove(value)
    }
}

impl From<BestMove> for UciMessage<'_> {
    fn from(value: BestMove) -> Self {
        Self::Engine(value.into())
    }
}

impl From<CopyProtection> for EngineMessage<'_> {
    fn from(value: CopyProtection) -> Self {
        Self::CopyProtection(value)
    }
}

impl From<CopyProtection> for UciMessage<'_> {
    fn from(value: CopyProtection) -> Self {
        Self::Engine(value.into())
    }
}

impl From<Registration> for EngineMessage<'_> {
    fn from(value: Registration) -> Self {
        Self::Registration(value)
    }
}

impl From<Registration> for UciMessage<'_> {
    fn from(value: Registration) -> Self {
        Self::Engine(value.into())
    }
}

impl<'a> From<Info<'a>> for EngineMessage<'a> {
    fn from(value: Info<'a>) -> Self {
        Self::Info(Box::new(value))
    }
}

impl<'a> From<Info<'a>> for UciMessage<'a> {
    fn from(value: Info<'a>) -> Self {
        Self::Engine(value.into())
    }
}

impl<'a> From<EngineOption<'a>> for EngineMessage<'a> {
    fn from(value: EngineOption<'a>) -> Self {
        Self::Option(value)
    }
}

impl<'a> From<EngineOption<'a>> for UciMessage<'a> {
    fn from(value: EngineOption<'a>) -> Self {
        Self::Engine(value.into())
    }
}

impl From<UciOk> for EngineMessage<'_> {
    fn from(value: UciOk) -> Self {
        Self::UciOk(value)
    }
}

impl From<UciOk> for UciMessage<'_> {
    fn from(value: UciOk) -> Self {
        Self::Engine(value.into())
    }
}

impl From<ReadyOk> for EngineMessage<'_> {
    fn from(value: ReadyOk) -> Self {
        Self::ReadyOk(value)
    }
}

impl From<ReadyOk> for UciMessage<'_> {
    fn from(value: ReadyOk) -> Self {
        Self::Engine(value.into())
    }
}

impl<'a> From<EngineMessage<'a>> for UciMessage<'a> {
    fn from(value: EngineMessage<'a>) -> Self {
        Self::Engine(value)
    }
}

#[cfg(test)]
mod tests {
    use super::{EngineMessage, Id, UciOk};
    use crate::UciMessage;

    #[test]
    fn message_wiring() {
        let id = Id::Name("Big Tuna".into());

        assert_eq!(
            EngineMessage::from(id.clone()).to_string(),
            "id name Big Tuna"
        );
        assert_eq!(UciMessage::from(id.clone()).to_string(), "id name Big Tuna");
        assert_eq!(
            "id name Big Tuna".parse::<EngineMessage>(),
            Ok(EngineMessage::from(id.clone()))
        );
        assert_eq!(
            "id name Big Tuna".parse::<UciMessage>(),
            Ok(UciMessage::from(id))
        );

        assert_eq!(EngineMessage::from(UciOk).to_string(), "uciok");
        assert_eq!(UciMessage::from(UciOk).to_string(), "uciok");
        assert_eq!(
            "uciok".parse::<EngineMessage>(),
            Ok(EngineMessage::from(UciOk))
        );
        assert_eq!("uciok".parse::<UciMessage>(), Ok(UciMessage::from(UciOk)));
    }
}
