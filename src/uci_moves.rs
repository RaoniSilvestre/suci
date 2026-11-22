use core::fmt::{Display, Formatter, Write};
use shakmaty::uci::UciMove;

/// Separates the string by spaces and parses moves while it can.
/// When it encounters an unparseable move, it stops.
pub fn from_str(s: &str) -> Vec<UciMove> {
    s.split(' ').map_while(|part| part.parse().ok()).collect()
}

/// Just joins the moves with a space.
pub fn fmt(this: &[UciMove], f: &mut Formatter<'_>) -> core::fmt::Result {
    let Some((first, rest)) = this.split_first() else {
        return Ok(());
    };

    first.fmt(f)?;

    for r#move in rest {
        f.write_char(' ')?;
        r#move.fmt(f)?;
    }

    Ok(())
}
