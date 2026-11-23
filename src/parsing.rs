use crate::MessageParseErrorKind;
use crate::errors::MessageParseError;
use core::str::FromStr;

/// Returns the first line of `s`, trimmed.
fn first_line(s: &str) -> &str {
    let s = s.trim();
    s.split_once('\n').map_or(s, |(line, _)| line)
}

/// Finds the target message and returns the rest of the string, as separated by spaces.
///
/// # Errors
/// No message was found.
pub fn collect_message<'s>(
    target: &'static str,
    s: &'s str,
) -> Result<core::str::Split<'s, char>, MessageParseError> {
    let mut parts = first_line(s).split(' ');
    parts
        .find(|&part| part == target)
        .ok_or(MessageParseError {
            expected: target,
            kind: MessageParseErrorKind::NoMessage,
        })?;

    Ok(parts)
}

/// Finds any message and returns the rest of the string, as separated by spaces.
///
/// # Errors
/// No message was found.
pub fn collect_any_message<'s, M>(
    expected: &'static str,
    s: &'s str,
) -> Result<(M, core::str::Split<'s, char>), MessageParseError>
where
    M: FromStr,
{
    let mut parts = first_line(s).split(' ');
    let message = parts
        .find_map(|part| M::from_str(part).ok())
        .ok_or(MessageParseError {
            expected,
            kind: MessageParseErrorKind::NoMessage,
        })?;

    Ok((message, parts))
}

/// Calls `parameter_fn` for every recognized parameter and its value.
///
/// A value is the run of unrecognized tokens up to the next recognized parameter,
/// with surrounding whitespace removed. Tokens before the first recognized
/// parameter are ignored.
pub fn for_each_parameter<'p, Parameter>(
    parts: impl Iterator<Item = &'p str>,
    mut parameter_fn: impl FnMut(Parameter, &str),
) where
    Parameter: FromStr + Copy,
{
    let mut value = String::with_capacity(200);
    let mut last_parameter = None;

    for part in parts {
        match Parameter::from_str(part) {
            Ok(parameter) => {
                if let Some(last_parameter) = last_parameter {
                    parameter_fn(last_parameter, value.trim());
                    value.clear();
                }

                last_parameter = Some(parameter);
            }
            Err(_) => {
                if last_parameter.is_some() {
                    value.push_str(part);
                    value.push(' ');
                }
            }
        }
    }

    if let Some(last_parameter) = last_parameter {
        parameter_fn(last_parameter, value.trim());
    }
}
