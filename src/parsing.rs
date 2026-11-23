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

/// Applies a closure to all parameters and their values.
///
/// `parameter_fn` accepts these parameters:
/// - `Parameter`: the parameter kind associated with the passed in value.
/// - `Option<Parameter>`: the next parameter, if there is one.
/// - `Split<char>`: the parts that this function parses. You should return these, as is given
///   by the `parameter_fn` signature, unless you want to stop parsing. In that case, return `None`
///   and keep the parts to yourself.
pub fn apply_parameters<'p, Parameter>(
    mut parts: core::str::Split<'p, char>,
    value: &mut String,
    mut parameter_fn: impl FnMut(
        Parameter,
        Option<Parameter>,
        &str,
        core::str::Split<'p, char>,
    ) -> Option<core::str::Split<'p, char>>,
) where
    Parameter: FromStr + Copy,
{
    let mut first_parameter_encountered = false;
    let mut last_parameter = None;

    while let Some(part) = parts.next() {
        let Ok(parameter) = Parameter::from_str(part) else {
            value.push_str(part);
            value.push(' ');
            continue;
        };

        if !first_parameter_encountered {
            first_parameter_encountered = true;
            value.clear();
        }

        if let Some(last_parameter) = last_parameter {
            let Some(new_parts) =
                parameter_fn(last_parameter, Some(parameter), value.trim(), parts)
            else {
                return;
            };
            parts = new_parts;
            value.clear();
        }

        last_parameter = Some(parameter);
    }

    if let Some(last_parameter) = last_parameter {
        parameter_fn(last_parameter, None, value.trim(), parts);
    }
}
