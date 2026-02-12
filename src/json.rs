use serde::{de::DeserializeOwned, Serialize};

use crate::format::{compute_indent, FormatOptions, Formatted};

/// Parses a JSON string into a value, capturing its formatting.
pub fn parse_json<T>(text: &str, options: Option<FormatOptions>) -> serde_json::Result<Formatted<T>>
where
    T: DeserializeOwned,
{
    let opts = options.unwrap_or_default();
    let value = serde_json::from_str(text)?;
    Ok(Formatted::new(text, value, &opts))
}

/// Stringifies a JSON value with preserved or configured formatting.
pub fn stringify_json<T>(
    formatted: &Formatted<T>,
    options: Option<FormatOptions>,
) -> serde_json::Result<String>
where
    T: Serialize,
{
    let opts = options.unwrap_or_default();
    let indent = compute_indent(&formatted.format, &opts);
    let json = serde_json::to_string_pretty(&formatted.value)?;
    let indented = json
        .lines()
        .map(|line| {
            if line.is_empty() {
                line.to_string()
            } else {
                let mut s = String::new();
                for _ in 0..indent {
                    s.push(' ');
                }
                s + line.trim_start()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    Ok(format!(
        "{}{}{}",
        formatted.format.whitespace_start, indented, formatted.format.whitespace_end
    ))
}

