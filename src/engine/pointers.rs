use core::str::FromStr;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum MessagePointer {
    Id,
    BestMove,
    CopyProtection,
    Registration,
    Info,
    Option,
    UciOk,
    ReadyOk,
}

impl FromStr for MessagePointer {
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

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum IdParameterPointer {
    Name,
    Author,
}

impl FromStr for IdParameterPointer {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "name" => Ok(Self::Name),
            "author" => Ok(Self::Author),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum InfoParameterPointer {
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
    Refutation,
    CurrLine,
}

impl InfoParameterPointer {
    /// The string representation of the parameter, surrounded by spaces.
    pub const fn as_str_spaced(self) -> &'static str {
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
            Self::Refutation => " refutation ",
            Self::CurrLine => " currline ",
        }
    }
}

impl FromStr for InfoParameterPointer {
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
            "refutation" => Ok(Self::Refutation),
            "currline" => Ok(Self::CurrLine),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum OptionParameterPointer {
    Name,
    Type,
    Default,
    Min,
    Max,
    Var,
}

impl FromStr for OptionParameterPointer {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "name" => Ok(Self::Name),
            "type" => Ok(Self::Type),
            "default" => Ok(Self::Default),
            "min" => Ok(Self::Min),
            "max" => Ok(Self::Max),
            "var" => Ok(Self::Var),
            _ => Err(()),
        }
    }
}
