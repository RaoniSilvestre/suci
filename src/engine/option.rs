use super::pointers::OptionParameterPointer;
use crate::errors::MessageParseError;
use crate::{MessageParseErrorKind, OptionReplaceIf, parsing};
use core::fmt::{Display, Formatter};
use core::str::{FromStr, Split};
use std::borrow::Cow;

/// Engine configuration option.
///
/// Sent after [`Uci`](crate::gui::Uci).
///
/// <https://backscattering.de/chess/uci/#engine-option>
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EngineOption<'a> {
    pub name: Cow<'a, str>,
    pub r#type: EngineOptionType<'a>,
}

impl EngineOption<'_> {
    pub(crate) const NAME: &'static str = "option";

    pub(crate) fn from_str_parts(parts: Split<'_, char>) -> Result<Self, MessageParseError> {
        let mut name = None::<String>;
        let mut r#type = None::<BlankOptionType>;
        let mut default = None::<String>;
        let mut min = None::<i64>;
        let mut max = None::<i64>;
        let mut var = Vec::new();
        let parameter_fn = |parameter, _, value: &str, parts| {
            match parameter {
                OptionParameterPointer::Name => name = Some(value.to_string()),
                OptionParameterPointer::Type => r#type.replace_if(value.parse().ok()),
                OptionParameterPointer::Default => default = Some(value.to_string()),
                OptionParameterPointer::Min => min.replace_if(value.parse().ok()),
                OptionParameterPointer::Max => max.replace_if(value.parse().ok()),
                OptionParameterPointer::Var => var.push(Cow::Owned(value.to_string())),
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

        let Some(r#type) = r#type else {
            return Err(MessageParseError {
                expected: "a type parameter; check, spin, combo, button or string",
                kind: MessageParseErrorKind::MissingParameters,
            });
        };

        let name = Cow::Owned(name);

        match r#type {
            BlankOptionType::Check => Ok(Self {
                name,
                r#type: EngineOptionType::Check {
                    default: default.and_then(|d| d.parse().ok()),
                },
            }),
            BlankOptionType::Spin => Ok(Self {
                name,
                r#type: EngineOptionType::Spin {
                    default: default.and_then(|d| d.parse().ok()),
                    min,
                    max,
                },
            }),
            BlankOptionType::Combo => Ok(Self {
                name,
                r#type: EngineOptionType::Combo {
                    default: default.map(Cow::Owned),
                    var: Cow::Owned(var),
                },
            }),
            BlankOptionType::Button => Ok(Self {
                name,
                r#type: EngineOptionType::Button,
            }),
            BlankOptionType::String => Ok(Self {
                name,
                r#type: EngineOptionType::String {
                    default: default.map(Cow::Owned),
                },
            }),
        }
    }
}

impl FromStr for EngineOption<'_> {
    type Err = MessageParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = parsing::collect_message(Self::NAME, s)?;
        Self::from_str_parts(parts)
    }
}

impl Display for EngineOption<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "option name {} {}", self.name, self.r#type)
    }
}

/// Type of an engine [`EngineOption`].
///
/// <https://backscattering.de/chess/uci/#engine-option-type>
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EngineOptionType<'a> {
    /// <https://backscattering.de/chess/uci/#engine-option-type-check>
    Check {
        /// <https://backscattering.de/chess/uci/#engine-option-default>
        default: Option<bool>,
    },
    /// <https://backscattering.de/chess/uci/#engine-option-type-spin>
    Spin {
        /// <https://backscattering.de/chess/uci/#engine-option-default>
        default: Option<i64>,
        /// <https://backscattering.de/chess/uci/#engine-option-min>
        min: Option<i64>,
        /// <https://backscattering.de/chess/uci/#engine-option-max>
        max: Option<i64>,
    },
    /// <https://backscattering.de/chess/uci/#engine-option-type-combo>
    Combo {
        /// <https://backscattering.de/chess/uci/#engine-option-default>
        default: Option<Cow<'a, str>>,
        /// <https://backscattering.de/chess/uci/#engine-option-var>
        var: Cow<'a, [Cow<'a, str>]>,
    },
    /// <https://backscattering.de/chess/uci/#engine-option-type-button>
    Button,
    /// <https://backscattering.de/chess/uci/#engine-option-type-string>
    String {
        /// <https://backscattering.de/chess/uci/#engine-option-default>
        default: Option<Cow<'a, str>>,
    },
}

impl EngineOptionType<'_> {
    /// Calls [`Cow::into_owned`] on each [`Cow`] field.
    /// The resulting value has a `'static` lifetime.
    #[must_use]
    pub fn into_owned(self) -> EngineOptionType<'static> {
        match self {
            Self::Check { default } => EngineOptionType::Check { default },
            Self::Spin { default, min, max } => EngineOptionType::Spin { default, min, max },
            Self::Combo { default, var } => EngineOptionType::Combo {
                default: default.map(Cow::into_owned).map(Cow::Owned),
                var: Cow::Owned(
                    var.into_owned()
                        .into_iter()
                        .map(Cow::into_owned)
                        .map(Cow::Owned)
                        .collect::<Vec<_>>(),
                ),
            },
            Self::Button => EngineOptionType::Button,
            Self::String { default } => EngineOptionType::String {
                default: default.map(Cow::into_owned).map(Cow::Owned),
            },
        }
    }
}

impl Display for EngineOptionType<'_> {
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

                for variation in var.iter() {
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

enum BlankOptionType {
    Check,
    Spin,
    Combo,
    Button,
    String,
}

impl FromStr for BlankOptionType {
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

impl Display for BlankOptionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Check => "check",
            Self::Spin => "spin",
            Self::Combo => "combo",
            Self::Button => "button",
            Self::String => "string",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{EngineOption, EngineOptionType};
    use std::borrow::Cow;

    #[test]
    fn to_from_str_min_max() {
        let m = EngineOption {
            name: Cow::Borrowed("Skill Level"),
            r#type: EngineOptionType::Spin {
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
            name: Cow::Borrowed("Personality"),
            r#type: EngineOptionType::String {
                default: Some(Cow::Borrowed("Aggressive")),
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
        let var = &[
            Cow::Borrowed("Foo bar fighter"),
            Cow::Borrowed("Aggressive p"),
            Cow::Borrowed("Defensive p"),
            Cow::Borrowed("Positional"),
            Cow::Borrowed("Endgame"),
        ];

        let m = EngineOption {
            name: Cow::Borrowed("K Personality"),
            r#type: EngineOptionType::Combo {
                default: Some(Cow::Borrowed("Default p")),
                var: Cow::Borrowed(var),
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
            r#type: EngineOptionType::Button,
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
