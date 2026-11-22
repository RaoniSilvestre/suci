use core::str::FromStr;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum MessagePointer {
    Debug,
    SetOption,
    Register,
    Position,
    Go,
    Uci,
    IsReady,
    UciNewGame,
    Stop,
    PonderHit,
    Quit,
}

impl FromStr for MessagePointer {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "debug" => Ok(Self::Debug),
            "setoption" => Ok(Self::SetOption),
            "register" => Ok(Self::Register),
            "position" => Ok(Self::Position),
            "go" => Ok(Self::Go),
            "uci" => Ok(Self::Uci),
            "isready" => Ok(Self::IsReady),
            "ucinewgame" => Ok(Self::UciNewGame),
            "stop" => Ok(Self::Stop),
            "ponderhit" => Ok(Self::PonderHit),
            "quit" => Ok(Self::Quit),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum SetOptionParameterPointer {
    Name,
    Value,
}

impl FromStr for SetOptionParameterPointer {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "name" => Ok(Self::Name),
            "value" => Ok(Self::Value),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum RegisterParameterPointer {
    Name,
    Code,
}

impl FromStr for RegisterParameterPointer {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "name" => Ok(Self::Name),
            "code" => Ok(Self::Code),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum PositionParameterPointer {
    Fen,
    Moves,
    StartPos,
}

impl FromStr for PositionParameterPointer {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "fen" => Ok(Self::Fen),
            "moves" => Ok(Self::Moves),
            "startpos" => Ok(Self::StartPos),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum GoParameterPointer {
    SearchMoves,
    Ponder,
    WTime,
    BTime,
    WInc,
    BInc,
    MovesToGo,
    Depth,
    Nodes,
    Mate,
    MoveTime,
    Infinite,
}

impl FromStr for GoParameterPointer {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "searchmoves" => Ok(Self::SearchMoves),
            "ponder" => Ok(Self::Ponder),
            "wtime" => Ok(Self::WTime),
            "btime" => Ok(Self::BTime),
            "winc" => Ok(Self::WInc),
            "binc" => Ok(Self::BInc),
            "movestogo" => Ok(Self::MovesToGo),
            "depth" => Ok(Self::Depth),
            "nodes" => Ok(Self::Nodes),
            "mate" => Ok(Self::Mate),
            "movetime" => Ok(Self::MoveTime),
            "infinite" => Ok(Self::Infinite),
            _ => Err(()),
        }
    }
}
