use core::fmt::{Display, Formatter};
use core::num::NonZeroUsize;
use shakmaty::uci::UciMove;

/// Tells the engine to start calculating.
///
/// Every field is optional. A [`Default`] value sends a bare `go`, which asks the engine to
/// choose its own search limits. [`Engine::go`](crate::Engine::go) blocks until the engine sends
/// its `bestmove` response.
///
/// # Example
/// ```rust
/// use suci::Go;
///
/// // Search to depth 12 plies.
/// let go = Go {
///     depth: Some(12),
///     ..Default::default()
/// };
///
/// assert_eq!(go.to_string(), "go depth 12");
/// ```
///
/// <https://backscattering.de/chess/uci/#gui-go>
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Go {
    /// Restricts the root search to these moves.
    ///
    /// Leave it empty to let the engine search all legal moves.
    ///
    /// <https://backscattering.de/chess/uci/#gui-go-searchmoves>
    pub search_moves: Vec<UciMove>,
    /// Starts a pondering search.
    ///
    /// The engine keeps searching after the expected move and does not send `bestmove` until it
    /// receives `stop` or `ponderhit`.
    ///
    /// <https://backscattering.de/chess/uci/#gui-go-ponder>
    pub ponder: bool,
    /// White's remaining time in milliseconds.
    ///
    /// <https://backscattering.de/chess/uci/#gui-go-wtime>
    pub w_time: Option<usize>,
    /// Black's remaining time in milliseconds.
    ///
    /// <https://backscattering.de/chess/uci/#gui-go-btime>
    pub b_time: Option<usize>,
    /// White's increment per move in milliseconds.
    ///
    /// <https://backscattering.de/chess/uci/#gui-go-winc>
    pub w_inc: Option<NonZeroUsize>,
    /// Black's increment per move in milliseconds.
    ///
    /// <https://backscattering.de/chess/uci/#gui-go-binc>
    pub b_inc: Option<NonZeroUsize>,
    /// The number of moves remaining until the next time control.
    ///
    /// <https://backscattering.de/chess/uci/#gui-go-movestogo>
    pub moves_to_go: Option<NonZeroUsize>,
    /// The search depth in plies.
    ///
    /// <https://backscattering.de/chess/uci/#gui-go-depth>
    pub depth: Option<usize>,
    /// The maximum number of nodes to search.
    ///
    /// <https://backscattering.de/chess/uci/#gui-go-nodes>
    pub nodes: Option<usize>,
    /// Searches for a mate in this many moves.
    ///
    /// <https://backscattering.de/chess/uci/#gui-go-mate>
    pub mate: Option<usize>,
    /// Limits the search to this many milliseconds.
    ///
    /// <https://backscattering.de/chess/uci/#gui-go-movetime>
    pub move_time: Option<usize>,
    /// Searches until [`Engine::stop`](crate::Engine::stop) is called.
    ///
    /// <https://backscattering.de/chess/uci/#gui-go-infinite>
    pub infinite: bool,
}

impl Display for Go {
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
