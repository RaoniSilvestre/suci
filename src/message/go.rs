use core::fmt::{Display, Formatter};
use core::num::NonZeroUsize;
use shakmaty::uci::UciMove;
use std::borrow::Cow;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
/// Tells the engine to start calculating.
///
/// <https://backscattering.de/chess/uci/#gui-go>
pub struct Go<'a> {
    /// <https://backscattering.de/chess/uci/#gui-go-searchmoves>
    pub search_moves: Cow<'a, [UciMove]>,
    /// <https://backscattering.de/chess/uci/#gui-go-ponder>
    pub ponder: bool,
    /// White's time.
    ///
    /// <https://backscattering.de/chess/uci/#gui-go-wtime>
    pub w_time: Option<usize>,
    /// Black's time.
    ///
    /// <https://backscattering.de/chess/uci/#gui-go-btime>
    pub b_time: Option<usize>,
    /// White's increment.
    ///
    /// <https://backscattering.de/chess/uci/#gui-go-winc>
    pub w_inc: Option<NonZeroUsize>,
    /// Black's increment.
    ///
    /// <https://backscattering.de/chess/uci/#gui-go-binc>
    pub b_inc: Option<NonZeroUsize>,
    /// <https://backscattering.de/chess/uci/#gui-go-movestogo>
    pub moves_to_go: Option<NonZeroUsize>,
    /// <https://backscattering.de/chess/uci/#gui-go-depth>
    pub depth: Option<usize>,
    /// <https://backscattering.de/chess/uci/#gui-go-nodes>
    pub nodes: Option<usize>,
    /// <https://backscattering.de/chess/uci/#gui-go-mate>
    pub mate: Option<usize>,
    /// <https://backscattering.de/chess/uci/#gui-go-movetime>
    pub move_time: Option<usize>,
    /// <https://backscattering.de/chess/uci/#gui-go-infinite>
    pub infinite: bool,
}

impl Go<'_> {
    pub fn into_owned(self) -> Go<'static> {
        let Go {
            search_moves,
            ponder,
            w_time,
            b_time,
            w_inc,
            b_inc,
            moves_to_go,
            depth,
            nodes,
            mate,
            move_time,
            infinite,
        } = self;

        Go {
            search_moves: Cow::Owned(search_moves.into_owned()),
            ponder,
            w_time,
            b_time,
            w_inc,
            b_inc,
            moves_to_go,
            depth,
            nodes,
            mate,
            move_time,
            infinite,
        }
    }
}

impl Display for Go<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str("go")?;

        if !self.search_moves.is_empty() {
            f.write_str(" searchmoves ")?;
            super::format_moves(&self.search_moves, f)?;
        }

        if self.ponder {
            f.write_str(" ponder")?;
        }

        if let Some(white_time) = self.w_time {
            write!(f, " wtime {white_time}")?;
        }

        if let Some(black_time) = self.b_time {
            write!(f, " btime {black_time}")?;
        }

        if let Some(white_increment) = self.w_inc {
            write!(f, " winc {white_increment}")?;
        }

        if let Some(black_increment) = self.b_inc {
            write!(f, " binc {black_increment}")?;
        }

        if let Some(moves_to_go) = self.moves_to_go {
            write!(f, " movestogo {moves_to_go}")?;
        }

        if let Some(depth) = self.depth {
            write!(f, " depth {depth}")?;
        }

        if let Some(nodes) = self.nodes {
            write!(f, " nodes {nodes}")?;
        }

        if let Some(mate) = self.mate {
            write!(f, " mate {mate}")?;
        }

        if let Some(move_time) = self.move_time {
            write!(f, " movetime {move_time}")?;
        }

        if self.infinite {
            f.write_str(" infinite")?;
        }

        Ok(())
    }
}
