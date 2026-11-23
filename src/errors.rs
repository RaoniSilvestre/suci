use core::error::Error;
use core::fmt::{Debug, Display, Formatter};
use std::io;

/// Parsing a UCI message failed.
///
/// Parsing is deliberately liberal and ignores errors unless they are critical.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct MessageParseError {
    /// A description of the message or parameter that was expected.
    pub expected: &'static str,
    /// The kind of parse failure.
    pub kind: MessageParseErrorKind,
}

impl Display for MessageParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let expected = self.expected;

        match self.kind {
            MessageParseErrorKind::NoMessage => {
                write!(f, "missing UCI message, expected {expected}")
            }
            MessageParseErrorKind::MissingParameters => {
                write!(f, "missing UCI parameters, expected {expected}")
            }
        }
    }
}

impl Error for MessageParseError {}

/// The kind of UCI parse error.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum MessageParseErrorKind {
    /// No message in the string was found.
    NoMessage,
    /// Required parameters are missing.
    MissingParameters,
}

/// Reading a message from the engine failed.
#[derive(Debug)]
pub enum ReadError {
    /// Reading failed due to an I/O error.
    Io(io::Error),
    /// Reading succeeded but parsing the engine response failed.
    Parse {
        /// The parse error.
        error: MessageParseError,
        /// The erroneous string. Includes the newline character (`\n`).
        got: String,
    },
}

impl Display for ReadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error reading UCI message: {e}"),
            Self::Parse { error, got } => {
                write!(f, "error parsing UCI message: {error}. Got: {got}")
            }
        }
    }
}

impl Error for ReadError {}

/// Reading or sending a message from or to the engine failed.
#[derive(Debug)]
pub enum ReadWriteError {
    /// Sending a message failed due to an I/O error.
    Write(io::Error),
    /// Reading a message failed.
    Read(ReadError),
}

impl Display for ReadWriteError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Write(e) => write!(f, "I/O error sending UCI message: {e}"),
            Self::Read(e) => write!(f, "error reading UCI message: {e}"),
        }
    }
}

impl Error for ReadWriteError {}

/// Capturing a child process's stdin or stdout failed.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum FromProcessError {
    /// See [`Child.stdout`](std::process::Child#structfield.stdout).
    StdoutNotCaptured,
    /// See [`Child.stdin`](std::process::Child#structfield.stdin).
    StdinNotCaptured,
}

impl Display for FromProcessError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StdoutNotCaptured => write!(f, "UCI process stdout is not captured"),
            Self::StdinNotCaptured => write!(f, "UCI process stdin is not captured"),
        }
    }
}

impl Error for FromProcessError {}

/// Starting an engine process or its UCI handshake failed.
#[derive(Debug)]
pub enum EngineStartError {
    /// The process could not be spawned.
    Spawn(io::Error),
    /// The process's stdin/stdout could not be captured.
    Pipes(FromProcessError),
    /// The UCI handshake failed.
    Handshake(ReadWriteError),
}

impl Display for EngineStartError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Spawn(e) => write!(f, "failed to spawn UCI engine: {e}"),
            Self::Pipes(e) => write!(f, "failed to capture UCI engine pipes: {e}"),
            Self::Handshake(e) => write!(f, "UCI handshake failed: {e}"),
        }
    }
}

impl Error for EngineStartError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_write() {
        const READ_ERROR_PARSE_STR: &str = "error parsing UCI message: ";
        const READ_WRITE_ERROR_READ_STR: &str = "error reading UCI message: ";
        let io_str = "screw your connection";
        assert_eq!(
            ReadWriteError::Write(io::Error::new(io::ErrorKind::ConnectionAborted, io_str))
                .to_string(),
            "I/O error sending UCI message: ".to_string() + io_str
        );

        let io_str = "i feel like the ErrorKind should be shown too :/";
        let read = ReadError::Io(io::Error::other(io_str));
        let read_str = "I/O error reading UCI message: ".to_string() + io_str;

        assert_eq!(read.to_string(), read_str);
        assert_eq!(
            ReadWriteError::Read(read).to_string(),
            READ_WRITE_ERROR_READ_STR.to_string() + &read_str
        );

        let expected = "apologies for being informal in these tests 😔 <- UTF-8 test right there, very serious!";
        let message_parse = MessageParseError {
            expected,
            kind: MessageParseErrorKind::NoMessage,
        };
        let read = ReadError::Parse {
            error: message_parse,
            got: "got".to_string(),
        };
        let read_str = READ_ERROR_PARSE_STR.to_string()
            + "missing UCI message, expected "
            + expected
            + ". Got: got";

        assert_eq!(message_parse.expected, expected);
        assert_eq!(read.to_string(), read_str);
        assert_eq!(
            ReadWriteError::Read(read).to_string(),
            READ_WRITE_ERROR_READ_STR.to_string() + &read_str
        );

        let expected = "�������";
        let message_parse = MessageParseError {
            expected,
            kind: MessageParseErrorKind::MissingParameters,
        };
        let read = ReadError::Parse {
            error: message_parse,
            got: String::new(),
        };
        let read_str = READ_ERROR_PARSE_STR.to_string()
            + "missing UCI parameters, expected "
            + expected
            + ". Got: ";

        assert_eq!(message_parse.expected, expected);
        assert_eq!(read.to_string(), read_str);
        assert_eq!(
            ReadWriteError::Read(read).to_string(),
            READ_WRITE_ERROR_READ_STR.to_string() + &read_str
        );
    }

    #[test]
    fn from_process() {
        assert_eq!(
            FromProcessError::StdoutNotCaptured.to_string(),
            "UCI process stdout is not captured"
        );
        assert_eq!(
            FromProcessError::StdinNotCaptured.to_string(),
            "UCI process stdin is not captured"
        );
    }
}
