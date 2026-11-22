//! # A note on "GUI"
//! Whenever you see "GUI" in the documentation, it doesn't mean "Graphical User Interface".
//! It means "whatever controls the engine".
//! It's called that because that's how it's referred to in the UCI protocol and elsewhere.
//!
//! # A note on message enums
//! If you write something like:
//! ```rust
//! # use suci::{EngineMessage, Info, UciMessage};
//! # use suci::gui;
//! # use suci::Go;
//! let _: EngineMessage = EngineMessage::Info(Box::new(Info::default()));
//! let _: UciMessage = UciMessage::Gui(gui::GuiMessage::Go(Go::default()));
//! ```
//!
//! Use instead:
//! ```rust
//! # use suci::{EngineMessage, Info, UciMessage};
//! # use suci::Go;
//! let _: EngineMessage = Info::default().into();
//! let _: UciMessage = Go::default().into();
//! ```
//!
//! In short, lower level messages can be converted to the higher level.
//!
//! # A note on [`Display`] impls
//! [`Display`] implementations of messages do **not** include the final newline (`\n`) character.

pub mod engine;
mod engine_connection;
mod errors;
pub mod gui;
mod parsing;
mod uci_moves;

use core::fmt::{Display, Formatter};
use core::str::FromStr;
pub use engine_connection::*;
pub use errors::*;
pub use {engine::*, gui::*};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Any message, sent from the engine or GUI.
pub enum UciMessage<'a> {
    Engine(EngineMessage<'a>),
    Gui(GuiMessage<'a>),
}

impl FromStr for UciMessage<'_> {
    type Err = MessageParseError;

    /// Tries to parse one line to a [`UciMessage`].
    /// If there's a newline character (`\n`) present, only the first line will be processed.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (message_pointer, parts) =
            parsing::collect_any_message::<UciMessagePointer>("any UCI message", s)?;

        match message_pointer {
            UciMessagePointer::Engine(engine) => match engine {
                // Value-less, parameter-less messages
                engine::pointers::MessagePointer::UciOk => Ok(UciOk.into()),
                engine::pointers::MessagePointer::ReadyOk => Ok(ReadyOk.into()),
                // Messages with values/parameters
                engine::pointers::MessagePointer::Id => Ok(Id::from_str_parts(parts)?.into()),
                engine::pointers::MessagePointer::BestMove => {
                    Ok(BestMove::from_str_parts(parts).into())
                }
                engine::pointers::MessagePointer::CopyProtection => {
                    Ok(CopyProtection::from_str_parts(parts)?.into())
                }
                engine::pointers::MessagePointer::Registration => {
                    Ok(Registration::from_str_parts(parts)?.into())
                }
                engine::pointers::MessagePointer::Info => Ok(Info::from_str_parts(parts).into()),
                engine::pointers::MessagePointer::Option => {
                    Ok(EngineOption::from_str_parts(parts)?.into())
                }
            },
            UciMessagePointer::Gui(gui) => match gui {
                // Value-less, parameter-less messages
                gui::pointers::MessagePointer::Uci => Ok(Uci.into()),
                gui::pointers::MessagePointer::IsReady => Ok(IsReady.into()),
                gui::pointers::MessagePointer::UciNewGame => Ok(UciNewGame.into()),
                gui::pointers::MessagePointer::Stop => Ok(Stop.into()),
                gui::pointers::MessagePointer::PonderHit => Ok(PonderHit.into()),
                gui::pointers::MessagePointer::Quit => Ok(Quit.into()),
                // Messages with values/parameters
                gui::pointers::MessagePointer::Debug => Ok(GuiDebug::from_str_parts(parts)?.into()),
                gui::pointers::MessagePointer::SetOption => {
                    Ok(SetOption::from_str_parts(parts)?.into())
                }
                gui::pointers::MessagePointer::Register => {
                    Ok(Register::from_str_parts(parts)?.into())
                }
                gui::pointers::MessagePointer::Position => {
                    Ok(Position::from_str_parts(parts)?.into())
                }
                gui::pointers::MessagePointer::Go => Ok(Go::from_str_parts(parts).into()),
            },
        }
    }
}

impl Display for UciMessage<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Engine(e) => e.fmt(f),
            Self::Gui(g) => g.fmt(f),
        }
    }
}

/// This is not an actual pointer; it is just a [`Copy`] enum for referencing messages.
enum UciMessagePointer {
    Engine(engine::pointers::MessagePointer),
    Gui(gui::pointers::MessagePointer),
}

impl FromStr for UciMessagePointer {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<engine::pointers::MessagePointer>().map_or_else(
            |()| s.parse::<gui::pointers::MessagePointer>().map(Self::Gui),
            |engine| Ok(Self::Engine(engine)),
        )
    }
}

pub(crate) trait OptionReplaceIf<T> {
    fn replace_if(&mut self, other: Option<T>);
}

impl<T> OptionReplaceIf<T> for Option<T> {
    fn replace_if(&mut self, other: Self) {
        if let Some(other) = other {
            *self = Some(other);
        }
    }
}
