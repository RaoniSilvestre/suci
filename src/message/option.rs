use crate::MessageParseErrorKind;
use crate::errors::MessageParseError;
use crate::parsing;
use core::fmt::{Display, Formatter};
use core::str::{FromStr, Split};

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum OptionParameter {
    Name,
    Type,
    Default,
    Min,
    Max,
    Var,
}

impl FromStr for OptionParameter {
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

/// An engine configuration option.
///
/// Engines advertise their options during the UCI handshake. The resulting list is available
/// through [`Engine::options`](crate::Engine::options). To set an option, send a
/// [`SetOption`](crate::SetOption) with the option's name and value.
///
/// # Example
/// ```rust
/// use suci::{EngineOption, EngineOptionType};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let option: EngineOption =
///     "option name Hash type spin default 16 min 1 max 33554432".parse()?;
///
/// assert_eq!(option.name, "Hash");
/// assert!(matches!(
///     option.option_type,
///     EngineOptionType::Spin {
///         default: Some(16),
///         min: Some(1),
///         max: Some(33_554_432),
///     }
/// ));
/// # Ok(())
/// # }
/// ```
///
/// <https://backscattering.de/chess/uci/#engine-option>
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EngineOption {
    /// The option name, as used by [`SetOption`](crate::SetOption).
    ///
    /// <https://backscattering.de/chess/uci/#engine-option-name>
    pub name: String,
    /// The option type and its default, range, or allowed values.
    ///
    /// <https://backscattering.de/chess/uci/#engine-option-type>
    pub option_type: EngineOptionType,
}

impl EngineOption {
    pub(crate) const NAME: &'static str = "option";

    pub(crate) fn from_str_parts(parts: Split<'_, char>) -> Result<Self, MessageParseError> {
        let mut name = None::<String>;
        let mut kind = None::<BlankOption>;
        let mut default = None::<String>;
        let mut min = None::<i64>;
        let mut max = None::<i64>;
        let mut var = Vec::new();
        let parameter_fn = |parameter, _, value: &str, parts| {
            match parameter {
                OptionParameter::Name => name = Some(value.to_string()),
                OptionParameter::Type => kind = value.parse().ok().or(kind),
                OptionParameter::Default => default = Some(value.to_string()),
                OptionParameter::Min => min = value.parse().ok().or(min),
                OptionParameter::Max => max = value.parse().ok().or(max),
                OptionParameter::Var => var.push(value.to_string()),
            }

            Some(parts)
        };

        let mut value = String::with_capacity(200);
        parsing::apply_parameters(parts, &mut value, parameter_fn);

        let Some(name) = name else {
            return Err(MessageParseError {
                expected: "name",
                kind: MessageParseErrorKind::MissingParameters,
            });
        };

        let Some(kind) = kind else {
            return Err(MessageParseError {
                expected: "a type parameter; check, spin, combo, button or string",
                kind: MessageParseErrorKind::MissingParameters,
            });
        };

        match kind {
            BlankOption::Check => Ok(Self {
                name,
                option_type: EngineOptionType::Check {
                    default: default.and_then(|d| d.parse().ok()),
                },
            }),
            BlankOption::Spin => Ok(Self {
                name,
                option_type: EngineOptionType::Spin {
                    default: default.and_then(|d| d.parse().ok()),
                    min,
                    max,
                },
            }),
            BlankOption::Combo => Ok(Self {
                name,
                option_type: EngineOptionType::Combo { default, var },
            }),
            BlankOption::Button => Ok(Self {
                name,
                option_type: EngineOptionType::Button,
            }),
            BlankOption::String => Ok(Self {
                name,
                option_type: EngineOptionType::String { default },
            }),
        }
    }
}

impl FromStr for EngineOption {
    type Err = MessageParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = parsing::collect_message(Self::NAME, s)?;
        Self::from_str_parts(parts)
    }
}

impl Display for EngineOption {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "option name {} {}", self.name, self.option_type)
    }
}

/// The type of an engine [`EngineOption`].
///
/// <https://backscattering.de/chess/uci/#engine-option-type>
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EngineOptionType {
    /// A boolean option.
    ///
    /// <https://backscattering.de/chess/uci/#engine-option-type-check>
    Check {
        /// The default value, if the engine reports one.
        ///
        /// <https://backscattering.de/chess/uci/#engine-option-default>
        default: Option<bool>,
    },
    /// An integer option with an optional range.
    ///
    /// <https://backscattering.de/chess/uci/#engine-option-type-spin>
    Spin {
        /// The default value, if the engine reports one.
        ///
        /// <https://backscattering.de/chess/uci/#engine-option-default>
        default: Option<i64>,
        /// The minimum allowed value, if the engine reports one.
        ///
        /// <https://backscattering.de/chess/uci/#engine-option-min>
        min: Option<i64>,
        /// The maximum allowed value, if the engine reports one.
        ///
        /// <https://backscattering.de/chess/uci/#engine-option-max>
        max: Option<i64>,
    },
    /// A string option restricted to a set of allowed values.
    ///
    /// <https://backscattering.de/chess/uci/#engine-option-type-combo>
    Combo {
        /// The default value, if the engine reports one.
        ///
        /// <https://backscattering.de/chess/uci/#engine-option-default>
        default: Option<String>,
        /// The allowed values.
        ///
        /// <https://backscattering.de/chess/uci/#engine-option-var>
        var: Vec<String>,
    },
    /// A button that runs an action when pressed.
    ///
    /// <https://backscattering.de/chess/uci/#engine-option-type-button>
    Button,
    /// A free-form string option.
    ///
    /// <https://backscattering.de/chess/uci/#engine-option-type-string>
    String {
        /// The default value, if the engine reports one.
        ///
        /// <https://backscattering.de/chess/uci/#engine-option-default>
        default: Option<String>,
    },
}

impl Display for EngineOptionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str("type ")?;

        match self {
            Self::Check { default } => {
                f.write_str("check")?;

                if let Some(default) = default {
                    write!(f, " default {default}")?;
                }
            }
            Self::Spin { default, min, max } => {
                f.write_str("spin")?;

                if let Some(default) = default {
                    write!(f, " default {default}")?;
                }

                if let Some(min) = min {
                    write!(f, " min {min}")?;
                }

                if let Some(max) = max {
                    write!(f, " max {max}")?;
                }
            }
            Self::Combo { default, var } => {
                f.write_str("combo")?;

                if let Some(default) = default {
                    write!(f, " default {default}")?;
                }

                for variation in var {
                    write!(f, " var {variation}")?;
                }
            }
            Self::Button => f.write_str("button")?,
            Self::String { default } => {
                f.write_str("string")?;

                if let Some(default) = default {
                    write!(f, " default {default}")?;
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum BlankOption {
    Check,
    Spin,
    Combo,
    Button,
    String,
}

impl FromStr for BlankOption {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "check" => Ok(Self::Check),
            "spin" => Ok(Self::Spin),
            "combo" => Ok(Self::Combo),
            "button" => Ok(Self::Button),
            "string" => Ok(Self::String),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{EngineOption, EngineOptionType};

    #[test]
    fn to_from_str_min_max() {
        let m = EngineOption {
            name: "Skill Level".into(),
            option_type: EngineOptionType::Spin {
                default: Some(20),
                min: Some(-10),
                max: Some(20),
            },
        };

        assert_eq!(
            "option name Skill Level type spin type INVALID default 20 min -10 max 20"
                .parse::<EngineOption>(),
            Ok(m.clone())
        );
        assert_eq!(
            m.to_string(),
            "option name Skill Level type spin default 20 min -10 max 20"
        );

        let m = EngineOption {
            name: "Personality".into(),
            option_type: EngineOptionType::String {
                default: Some("Aggressive".into()),
            },
        };

        assert_eq!(
            "option name Personality type spin type string default Aggressive"
                .parse::<EngineOption>(),
            Ok(m.clone())
        );
        assert_eq!(
            m.to_string(),
            "option name Personality type string default Aggressive"
        );
    }

    #[test]
    fn to_from_str_var() {
        let var = vec![
            "Foo bar fighter".into(),
            "Aggressive p".into(),
            "Defensive p".into(),
            "Positional".into(),
            "Endgame".into(),
        ];

        let m = EngineOption {
            name: "K Personality".into(),
            option_type: EngineOptionType::Combo {
                default: Some("Default p".into()),
                var,
            },
        };
        let str_in = "option var Foo bar fighter name K Personality type combo default Default p var Aggressive p var Defensive p var Positional var Endgame";
        // Output has a different order, which is fine, but can't use the same string.
        let str_out = "option name K Personality type combo default Default p var Foo bar fighter var Aggressive p var Defensive p var Positional var Endgame";

        assert_eq!(str_in.parse::<EngineOption>(), Ok(m.clone()));
        assert_eq!(m.to_string(), str_out);
    }

    #[test]
    fn button() {
        let m = EngineOption {
            name: "Execute   order\t 66".into(),
            option_type: EngineOptionType::Button,
        };

        assert_eq!(
            "option name   Execute   order\t 66   type button".parse::<EngineOption>(),
            Ok(m.clone())
        );
        assert_eq!(
            m.to_string(),
            "option name Execute   order\t 66 type button"
        );
    }
}
