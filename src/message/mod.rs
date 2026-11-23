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
use std::fmt::{Display, Formatter, Write};

/// Separates the string by spaces and parses moves while it can.
/// When it encounters an unparseable move, it stops.
fn parse_moves(s: &str) -> Vec<UciMove> {
    s.split(' ').map_while(|part| part.parse().ok()).collect()
}

/// Just joins the moves with a space.
///
/// # Errors
/// Returns [`core::fmt::Error`] if the formatter fails.
fn format_moves(this: &[UciMove], f: &mut Formatter<'_>) -> core::fmt::Result {
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
