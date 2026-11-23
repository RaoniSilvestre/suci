use crate::errors::MessageParseError;
use crate::parsing;
use core::fmt::{Display, Formatter};
use core::str::{FromStr, Split};
use shakmaty::uci::UciMove;
use std::borrow::Cow;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum InfoParameter {
    Depth,
    SelDepth,
    Time,
    Nodes,
    Pv,
    MultiPv,
    Score,
    CurrMove,
    CurrMoveNumber,
    HashFull,
    Nps,
    TbHits,
    SbHits,
    CpuLoad,
    String,
}

impl InfoParameter {
    /// The string representation of the parameter, surrounded by spaces.
    const fn as_str_spaced(self) -> &'static str {
        match self {
            Self::Depth => " depth ",
            Self::SelDepth => " seldepth ",
            Self::Time => " time ",
            Self::Nodes => " nodes ",
            Self::Pv => " pv ",
            Self::MultiPv => " multipv ",
            Self::Score => " score ",
            Self::CurrMove => " currmove ",
            Self::CurrMoveNumber => " currmovenumber ",
            Self::HashFull => " hashfull ",
            Self::Nps => " nps ",
            Self::TbHits => " tbhits ",
            Self::SbHits => " sbhits ",
            Self::CpuLoad => " cpuload ",
            Self::String => " string ",
        }
    }
}

impl FromStr for InfoParameter {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "depth" => Ok(Self::Depth),
            "seldepth" => Ok(Self::SelDepth),
            "time" => Ok(Self::Time),
            "nodes" => Ok(Self::Nodes),
            "pv" => Ok(Self::Pv),
            "multipv" => Ok(Self::MultiPv),
            "score" => Ok(Self::Score),
            "currmove" => Ok(Self::CurrMove),
            "currmovenumber" => Ok(Self::CurrMoveNumber),
            "hashfull" => Ok(Self::HashFull),
            "nps" => Ok(Self::Nps),
            "tbhits" => Ok(Self::TbHits),
            "sbhits" => Ok(Self::SbHits),
            "cpuload" => Ok(Self::CpuLoad),
            "string" => Ok(Self::String),
            _ => Err(()),
        }
    }
}

/// Information about the engine's calculation.
///
/// Engines send `info` messages while searching. [`Engine::go`](crate::Engine::go) logs them at
/// `trace` and returns only the final [`BestMove`](crate::BestMove), so this type is useful when
/// parsing UCI output directly.
///
/// # Example
/// ```rust
/// use suci::{Info, Score};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let info: Info = "info depth 20 score cp 35 pv e2e4 e7e5".parse()?;
///
/// assert_eq!(info.depth.map(|depth| depth.depth), Some(20));
/// assert_eq!(info.score.map(|score| score.kind), Some(Score::Centipawns(35)));
/// assert_eq!(info.pv.len(), 2);
/// # Ok(())
/// # }
/// ```
///
/// <https://backscattering.de/chess/uci/#engine-info>
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Info<'a> {
    /// The search depth and optional selective depth.
    ///
    /// <https://backscattering.de/chess/uci/#engine-info-depth>
    pub depth: Option<Depth>,
    /// The elapsed search time in milliseconds.
    ///
    /// <https://backscattering.de/chess/uci/#engine-info-time>
    pub time: Option<usize>,
    /// The number of nodes searched.
    ///
    /// <https://backscattering.de/chess/uci/#engine-info-nodes>
    pub nodes: Option<usize>,
    /// The principal variation: the line the engine expects to be best.
    ///
    /// <https://backscattering.de/chess/uci/#engine-info-pv>
    pub pv: Cow<'a, [UciMove]>,
    /// The index of this line when multi-PV is enabled, starting at 1.
    ///
    /// <https://backscattering.de/chess/uci/#engine-info-multipv>
    pub multi_pv: Option<usize>,
    /// The engine's evaluation of the position.
    ///
    /// <https://backscattering.de/chess/uci/#engine-info-score>
    pub score: Option<ScoreWithBound>,
    /// The move currently being searched at the root.
    ///
    /// <https://backscattering.de/chess/uci/#engine-info-currmove>
    pub curr_move: Option<UciMove>,
    /// The ordinal number of `curr_move` at the root, starting at 1.
    ///
    /// <https://backscattering.de/chess/uci/#engine-info-currmovenumber>
    pub curr_move_number: Option<usize>,
    /// Transposition-table usage in permill (0 to 1000).
    ///
    /// <https://backscattering.de/chess/uci/#engine-info-hashfull>
    pub hash_full: Option<usize>,
    /// The search speed in nodes per second.
    ///
    /// <https://backscattering.de/chess/uci/#engine-info-nps>
    pub nps: Option<usize>,
    /// The number of tablebase hits.
    ///
    /// <https://backscattering.de/chess/uci/#engine-info-tbhits>
    pub tb_hits: Option<usize>,
    /// The number of Shredderbase hits.
    ///
    /// <https://backscattering.de/chess/uci/#engine-info-sbhits>
    pub sb_hits: Option<usize>,
    /// The engine's CPU load in permill (0 to 1000).
    ///
    /// <https://backscattering.de/chess/uci/#engine-info-cpuload>
    pub cpu_load: Option<usize>,
    /// Free-form text from the engine.
    ///
    /// <https://backscattering.de/chess/uci/#engine-info-string>
    pub string: Option<Cow<'a, str>>,
}

impl Info<'_> {
    /// Calls [`Cow::into_owned`] on each [`Cow`] field.
    /// The resulting value has a `'static` lifetime.
    #[must_use]
    pub fn into_owned(self) -> Info<'static> {
        let Info {
            depth,
            time,
            nodes,
            pv,
            multi_pv,
            score,
            curr_move,
            curr_move_number,
            hash_full,
            nps,
            tb_hits,
            sb_hits,
            cpu_load,
            string,
        } = self;

        Info {
            depth,
            time,
            nodes,
            pv: Cow::Owned(pv.into_owned()),
            multi_pv,
            score,
            curr_move,
            curr_move_number,
            hash_full,
            nps,
            tb_hits,
            sb_hits,
            cpu_load,
            string: string.map(Cow::into_owned).map(Cow::Owned),
        }
    }
}

impl Info<'_> {
    pub(crate) const NAME: &'static str = "info";

    pub(crate) fn from_str_parts<'p>(parts: Split<'p, char>) -> Self {
        let mut info = Self::default();
        // Need to handle depth like this in case the seldepth argument comes before the depth argument.
        let mut depth = None::<usize>;
        let mut seldepth = None::<usize>;
        let mut pv = Vec::new();
        let mut string = String::new();

        let parameter_fn = |parameter,
                            next_parameter: Option<InfoParameter>,
                            value: &str,
                            parts: core::str::Split<'p, char>| {
            match parameter {
                InfoParameter::Depth => depth = value.parse().ok().or(depth),
                InfoParameter::SelDepth => seldepth = value.parse().ok().or(seldepth),
                InfoParameter::Time => info.time = value.parse().ok().or(info.time),
                InfoParameter::Nodes => info.nodes = value.parse().ok().or(info.nodes),
                InfoParameter::Pv => {
                    let parsed = super::parse_moves(value);

                    if !parsed.is_empty() {
                        pv = parsed;
                    }
                }
                InfoParameter::MultiPv => info.multi_pv = value.parse().ok().or(info.multi_pv),
                InfoParameter::Score => {
                    info.score = ScoreWithBound::from_str(value).or(info.score);
                }
                InfoParameter::CurrMove => info.curr_move = value.parse().ok().or(info.curr_move),
                InfoParameter::CurrMoveNumber => {
                    info.curr_move_number = value.parse().ok().or(info.curr_move_number);
                }
                InfoParameter::HashFull => info.hash_full = value.parse().ok().or(info.hash_full),
                InfoParameter::Nps => info.nps = value.parse().ok().or(info.nps),
                InfoParameter::TbHits => info.tb_hits = value.parse().ok().or(info.tb_hits),
                InfoParameter::SbHits => info.sb_hits = value.parse().ok().or(info.sb_hits),
                InfoParameter::CpuLoad => info.cpu_load = value.parse().ok().or(info.cpu_load),
                InfoParameter::String => {
                    string = value.to_string();
                    let size_hint = parts.size_hint();
                    string.reserve(size_hint.1.unwrap_or(size_hint.0).saturating_mul(20));

                    if let Some(next_parameter) = next_parameter {
                        string.push_str(next_parameter.as_str_spaced());
                    }

                    let mut first = true;

                    for part in parts {
                        if first {
                            first = false;
                        } else {
                            string.push(' ');
                        }

                        string.push_str(part);
                    }

                    return None;
                }
            }

            Some(parts)
        };

        let mut value = String::with_capacity(200);
        parsing::apply_parameters(parts, &mut value, parameter_fn);

        info.pv = Cow::Owned(pv);

        if let Some(depth) = depth {
            info.depth = Some(Depth { depth, seldepth });
        }

        if !string.is_empty() {
            info.string = Some(Cow::Owned(string));
        }

        info
    }
}

impl FromStr for Info<'_> {
    type Err = MessageParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = parsing::collect_message(Self::NAME, s)?;
        Ok(Self::from_str_parts(parts))
    }
}

impl Display for Info<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str("info")?;

        if let Some(depth) = &self.depth {
            write!(f, " depth {}", depth.depth)?;

            if let Some(selective_search_depth) = depth.seldepth {
                write!(f, " seldepth {selective_search_depth}")?;
            }
        }

        if let Some(time) = self.time {
            write!(f, " time {time}")?;
        }

        if let Some(nodes) = self.nodes {
            write!(f, " nodes {nodes}")?;
        }

        if !self.pv.is_empty() {
            f.write_str(" pv ")?;
            super::format_moves(&self.pv, f)?;
        }

        if let Some(multi_primary_variation) = self.multi_pv {
            write!(f, " multipv {multi_primary_variation}")?;
        }

        if let Some(score) = &self.score {
            f.write_str(" score")?;

            match score.kind {
                Score::Centipawns(centipawns) => write!(f, " cp {centipawns}")?,
                Score::MateIn(mate_in) => write!(f, " mate {mate_in}")?,
            }

            match score.bound {
                Some(ScoreBound::UpperBound) => {
                    f.write_str(" upperbound")?;
                }
                Some(ScoreBound::LowerBound) => {
                    f.write_str(" lowerbound")?;
                }
                None => {}
            }
        }

        if let Some(current_move) = &self.curr_move {
            write!(f, " currmove {current_move}")?;
        }

        if let Some(current_move_number) = self.curr_move_number {
            write!(f, " currmovenumber {current_move_number}")?;
        }

        if let Some(hash_full) = self.hash_full {
            write!(f, " hashfull {hash_full}")?;
        }

        if let Some(nodes_per_second) = self.nps {
            write!(f, " nps {nodes_per_second}")?;
        }

        if let Some(table_base_hits) = self.tb_hits {
            write!(f, " tbhits {table_base_hits}")?;
        }

        if let Some(shredder_base_hits) = self.sb_hits {
            write!(f, " sbhits {shredder_base_hits}")?;
        }

        if let Some(cpu_load) = self.cpu_load {
            write!(f, " cpuload {cpu_load}")?;
        }

        if let Some(string) = &self.string {
            write!(f, " string {string}")?;
        }

        Ok(())
    }
}

/// The depth reached by an engine's search.
///
/// <https://backscattering.de/chess/uci/#engine-info-depth>
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Depth {
    /// The completed search depth in plies.
    ///
    /// <https://backscattering.de/chess/uci/#engine-info-depth>
    pub depth: usize,
    /// The selective search depth in plies, if the engine reports one.
    ///
    /// <https://backscattering.de/chess/uci/#engine-info-seldepth>
    pub seldepth: Option<usize>,
}

/// Whether a score is a lower bound or an upper bound.
///
/// Engines use bounds when they stop searching before the score is exact.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ScoreBound {
    /// The real score is at least the reported score.
    ///
    /// <https://backscattering.de/chess/uci/#engine-info-score-lowerbound>
    LowerBound,
    /// The real score is at most the reported score.
    ///
    /// <https://backscattering.de/chess/uci/#engine-info-score-upperbound>
    UpperBound,
}

/// Player advantage or disadvantage from the engine's perspective.
///
/// <https://backscattering.de/chess/uci/#engine-info-score>
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Score {
    /// A score in centipawns, where 100 is roughly one pawn.
    ///
    /// <https://backscattering.de/chess/uci/#engine-info-score-centipawns>
    Centipawns(isize),
    /// A forced mate in this many moves.
    ///
    /// A positive value means the engine delivers mate; a negative value means it is mated.
    ///
    /// <https://backscattering.de/chess/uci/#engine-info-score-mate>
    MateIn(isize),
}

/// A score together with its optional bound.
///
/// <https://backscattering.de/chess/uci/#engine-info-score>
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ScoreWithBound {
    /// The score itself.
    pub kind: Score,
    /// Whether the score is a lower or upper bound, if the engine reported one.
    ///
    /// <https://backscattering.de/chess/uci/#engine-info-score-lowerbound>
    /// <https://backscattering.de/chess/uci/#engine-info-score-upperbound>
    pub bound: Option<ScoreBound>,
}

impl ScoreWithBound {
    fn from_str(s: &str) -> Option<Self> {
        fn value_after(split: &[&str], position: Option<usize>) -> Option<isize> {
            split
                .get(position?.checked_add(1)?)
                .and_then(|s| s.parse().ok())
        }

        let split = s.split(' ').collect::<Vec<_>>();
        let centipawns = value_after(&split, split.iter().position(|&part| part == "cp"));
        let mate_in = value_after(&split, split.iter().position(|&part| part == "mate"));
        let kind = match (centipawns, mate_in) {
            (Some(centipawns), _) => Score::Centipawns(centipawns),
            (None, Some(mate_in)) => Score::MateIn(mate_in),
            (None, None) => return None,
        };
        let bound = match (split.contains(&"lowerbound"), split.contains(&"upperbound")) {
            (true, false) => Some(ScoreBound::LowerBound),
            (false, true) => Some(ScoreBound::UpperBound),
            _ => None,
        };

        Some(Self { kind, bound })
    }
}

#[cfg(test)]
mod tests {
    use super::{Depth, Info, Score, ScoreBound, ScoreWithBound};
    use shakmaty::uci::UciMove;
    use std::borrow::Cow;

    #[test]
    fn to_from_str() {
        let pv: &[UciMove] = &[
            UciMove::from_ascii(b"e2e4").unwrap(),
            UciMove::from_ascii(b"c7c5").unwrap(),
        ];

        let m = Info {
            depth: Some(Depth {
                depth: 20,
                seldepth: Some(31),
            }),
            time: Some(12),
            nodes: Some(4),
            pv: Cow::Borrowed(pv),
            multi_pv: Some(1),
            score: Some(ScoreWithBound {
                kind: Score::Centipawns(22),
                bound: Some(ScoreBound::LowerBound),
            }),
            curr_move: Some(UciMove::from_ascii(b"e2e4").unwrap()),
            curr_move_number: None,
            hash_full: None,
            nps: None,
            tb_hits: Some(2),
            sb_hits: None,
            cpu_load: None,
            string: Some(Cow::Borrowed("blabla")),
        };

        let s = "info depth 20 seldepth 31 time 12 nodes 4 pv e2e4 c7c5 multipv 1 score cp 22 lowerbound currmove e2e4 tbhits 2 string blabla";

        assert_eq!(m.to_string(), s);
        assert_eq!(s.parse::<Info>(), Ok(m));
    }

    #[test]
    fn to_from_str_bad_parameters() {
        let m = Info {
            depth: Some(Depth {
                depth: 20,
                seldepth: Some(31),
            }),
            time: Some(12),
            nodes: Some(4),
            pv: Cow::Borrowed(&[]),
            multi_pv: Some(1),
            score: Some(ScoreWithBound {
                kind: Score::Centipawns(22),
                bound: Some(ScoreBound::LowerBound),
            }),
            curr_move: Some(UciMove::from_ascii(b"e2e4").unwrap()),
            curr_move_number: None,
            hash_full: None,
            nps: None,
            tb_hits: Some(4),
            sb_hits: None,
            cpu_load: None,
            string: Some(Cow::Borrowed("blabla")),
        };

        assert_eq!(
            "info depth BAD depth 20 seldepth 31 time 12 depth also bad nodes 4 multipv 1 score cp 22 lowerbound currmove e2e4 tbhits 2 tbhits 4 string blabla".parse::<Info>(),
            Ok(m.clone())
        );
        assert_eq!(
            m.to_string(),
            "info depth 20 seldepth 31 time 12 nodes 4 multipv 1 score cp 22 lowerbound currmove e2e4 tbhits 4 string blabla"
        );
    }

    #[test]
    fn to_from_str_string() {
        let m = Info {
            depth: Some(Depth {
                depth: 13,
                seldepth: None,
            }),
            string: Some(Cow::Borrowed(
                "the parameters in this string should be ignored! depth 20 (psych) time 2 nodes 4 score cp 22 lowerbound",
            )),
            ..Default::default()
        };

        let s = "info depth 13 string the parameters in this string should be ignored! depth 20 (psych) time 2 nodes 4 score cp 22 lowerbound";

        assert_eq!(m.to_string(), s);
        assert_eq!(s.parse::<Info>(), Ok(m));
    }
}
