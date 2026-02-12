use json5 as json5_crate;
use serde::{de::DeserializeOwned, Serialize};

use crate::format::{compute_indent, FormatOptions, Formatted};

/// Parses a JSON5 string into a value, capturing its formatting.
pub fn parse_json5<T>(
    text: &str,
    options: Option<FormatOptions>,
) -> Result<Formatted<T>, json5_crate::Error>
where
    T: DeserializeOwned,
{
    let opts = options.unwrap_or_default();
    let value = json5_crate::from_str(text)?;
    Ok(Formatted::new(text, value, &opts))
}

/// Stringifies a JSON5 value with preserved or configured formatting.
pub fn stringify_json5<T>(
    formatted: &Formatted<T>,
    options: Option<FormatOptions>,
) -> Result<String, json5_crate::Error>
where
    T: Serialize,
{
    let opts = options.unwrap_or_default();
    let _indent = compute_indent(&formatted.format, &opts);
    // json5 crate does not currently expose a configurable pretty printer
    // in the same way as the JS version. We fall back to its default
    // serialization behavior and only preserve outer whitespace.
    let json5 = json5_crate::to_string(&formatted.value)?;
    Ok(format!(
        "{}{}{}",
        formatted.format.whitespace_start, json5, formatted.format.whitespace_end
    ))
}

