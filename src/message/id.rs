use crate::errors::MessageParseError;
use crate::{MessageParseErrorKind, parsing};
use core::fmt::{Display, Formatter};
use core::str::{FromStr, Split};

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum IdParameter {
    Name,
    Author,
}

impl FromStr for IdParameter {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "name" => Ok(Self::Name),
            "author" => Ok(Self::Author),
            _ => Err(()),
        }
    }
}

/// The engine's identification.
///
/// Engines may send the name and author in separate `id` messages. The handshake combines them
/// into [`Id::NameAndAuthor`] when both are present. The result is available through
/// [`Engine::id`](crate::Engine::id).
///
/// # Example
/// ```rust
/// use suci::Id;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let id: Id = "id name Stockfish 19 author the Stockfish developers".parse()?;
///
/// assert_eq!(
///     id,
///     Id::NameAndAuthor {
///         name: "Stockfish 19".into(),
///         author: "the Stockfish developers".into(),
///     }
/// );
/// # Ok(())
/// # }
/// ```
///
/// <https://backscattering.de/chess/uci/#engine-id>
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Id {
    /// The engine's name.
    ///
    /// <https://backscattering.de/chess/uci/#engine-id-name>
    Name(String),
    /// The engine's author.
    ///
    /// <https://backscattering.de/chess/uci/#engine-id-author>
    Author(String),
    /// Both the engine's name and author.
    NameAndAuthor {
        /// The engine's name.
        name: String,
        /// The engine's author.
        author: String,
    },
}

impl Id {
    pub(crate) const NAME: &'static str = "id";

    #[must_use]
    /// Returns a new [`Id`] that is updated with values of the `new` parameter.
    ///
    /// # Examples
    /// ```rust
    /// # use suci::Id;
    /// let mut id = Id::Name("Carp".into());
    ///
    /// id = id.updated(Id::Name("Salmon".into()));
    /// assert_eq!(id, Id::Name("Salmon".into()));
    ///
    /// id = id.updated(Id::Author("Fischerman".into()));
    /// assert_eq!(
    ///     id,
    ///     Id::NameAndAuthor {
    ///         name: "Salmon".into(),
    ///         author: "Fischerman".into()
    ///     }
    /// );
    ///
    /// id = id.updated(Id::Author("Garry Chess".into()));
    /// assert_eq!(
    ///     id,
    ///     Id::NameAndAuthor {
    ///         name: "Salmon".into(),
    ///         author: "Garry Chess".into()
    ///     }
    /// );
    ///
    /// id = id.updated(Id::Name("Big Tuna".into()));
    /// assert_eq!(
    ///     id,
    ///     Id::NameAndAuthor {
    ///         name: "Big Tuna".into(),
    ///         author: "Garry Chess".into()
    ///     }
    /// );
    /// ```
    pub fn updated(self, new: Self) -> Self {
        match (self, new) {
            (Self::Author(_), Self::Author(author)) => Self::Author(author),
            (Self::Name(_), Self::Name(name)) => Self::Name(name),
            (
                Self::NameAndAuthor { .. } | Self::Author(_) | Self::Name(_),
                Self::NameAndAuthor { name, author },
            )
            | (Self::NameAndAuthor { author: _, name } | Self::Name(name), Self::Author(author))
            | (Self::NameAndAuthor { author, name: _ } | Self::Author(author), Self::Name(name)) => {
                Self::NameAndAuthor { name, author }
            }
        }
    }

    pub(crate) fn from_str_parts(parts: Split<'_, char>) -> Result<Self, MessageParseError> {
        let mut name = None::<String>;
        let mut author = None::<String>;
        let parameter_fn = |parameter, value: &str| match parameter {
            IdParameter::Name => name = Some(value.to_owned()),
            IdParameter::Author => author = Some(value.to_owned()),
        };

        parsing::for_each_parameter(parts, parameter_fn);

        match (name, author) {
            (Some(name), Some(author)) => Ok(Self::NameAndAuthor { name, author }),
            (Some(name), None) => Ok(Self::Name(name)),
            (None, Some(author)) => Ok(Self::Author(author)),
            (None, None) => Err(MessageParseError {
                expected: "name or author",
                kind: MessageParseErrorKind::MissingParameters,
            }),
        }
    }
}

impl FromStr for Id {
    type Err = MessageParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = parsing::collect_message(Self::NAME, s)?;
        Self::from_str_parts(parts)
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str("id ")?;

        match self {
            Self::Name(name) => write!(f, "name {name}")?,
            Self::Author(author) => write!(f, "author {author}")?,
            Self::NameAndAuthor { name, author } => {
                write!(f, "name {name} author {author}")?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Id;

    #[test]
    fn to_from_str() {
        let m = Id::NameAndAuthor {
            name: "Stockfish 16.1".into(),
            author: "The stockfish developers".into(),
        };

        assert_eq!(
            m.to_string(),
            "id name Stockfish 16.1 author The stockfish developers"
        );
        assert_eq!(
            "id name Stockfish 16.1 author The stockfish developers".parse::<Id>(),
            Ok(m)
        );

        let m = Id::Name("Stockfish 16.1".into());

        assert_eq!(
            "id name Stockfish 16.1 \n ignored".parse::<Id>(),
            Ok(m.clone())
        );
        assert_eq!(m.to_string(), "id name Stockfish 16.1");
    }
}
