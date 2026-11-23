#![allow(clippy::redundant_pub_crate)]

mod best_move;
mod go;
mod id;
mod info;
mod option;
mod position;
mod set_option;

pub use best_move::*;
pub use go::*;
pub use id::*;
pub use info::*;
pub use option::*;
pub use position::*;
pub use set_option::*;
use shakmaty::uci::UciMove;

use crate::errors::MessageParseError;
use crate::parsing;
use core::str::FromStr;
use std::fmt::{Display, Formatter, Write};

#[derive(Debug)]
pub(crate) enum Response<'a> {
    Id(Id<'a>),
    BestMove(BestMove),
    #[allow(dead_code)]
    Info(Box<Info<'a>>),
    Option(EngineOption<'a>),
    UciOk,
    ReadyOk,
    Other,
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum ResponseKind {
    Id,
    BestMove,
    CopyProtection,
    Registration,
    Info,
    Option,
    UciOk,
    ReadyOk,
}

impl FromStr for ResponseKind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "id" => Ok(Self::Id),
            "bestmove" => Ok(Self::BestMove),
            "copyprotection" => Ok(Self::CopyProtection),
            "registration" => Ok(Self::Registration),
            "info" => Ok(Self::Info),
            "option" => Ok(Self::Option),
            "uciok" => Ok(Self::UciOk),
            "readyok" => Ok(Self::ReadyOk),
            _ => Err(()),
        }
    }
}

impl FromStr for Response<'_> {
    type Err = MessageParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (kind, parts) = parsing::collect_any_message::<ResponseKind>("engine UCI message", s)?;

        match kind {
            ResponseKind::Id => Ok(Self::Id(Id::from_str_parts(parts)?)),
            ResponseKind::BestMove => Ok(Self::BestMove(BestMove::from_str_parts(parts))),
            ResponseKind::Info => Ok(Self::Info(Box::new(Info::from_str_parts(parts)))),
            ResponseKind::Option => Ok(Self::Option(EngineOption::from_str_parts(parts)?)),
            ResponseKind::UciOk => Ok(Self::UciOk),
            ResponseKind::ReadyOk => Ok(Self::ReadyOk),
            ResponseKind::CopyProtection | ResponseKind::Registration => Ok(Self::Other),
        }
    }
}

/// Separates the string by spaces and parses moves while it can.
/// When it encounters an unparseable move, it stops.
pub(crate) fn parse_moves(s: &str) -> Vec<UciMove> {
    s.split(' ').map_while(|part| part.parse().ok()).collect()
}

/// Just joins the moves with a space.
pub(crate) fn format_moves(this: &[UciMove], f: &mut Formatter<'_>) -> core::fmt::Result {
    let Some((first, rest)) = this.split_first() else {
        return Ok(());
    };

    first.fmt(f)?;

    for movement in rest {
        f.write_char(' ')?;
        movement.fmt(f)?;
    }

    Ok(())
}
