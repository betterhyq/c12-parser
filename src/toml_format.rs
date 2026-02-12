use serde::{de::DeserializeOwned, Serialize};

use crate::format::{FormatOptions, Formatted};

/// Parses a TOML string into a value, capturing outer whitespace only.
pub fn parse_toml<T>(
    text: &str,
    options: Option<FormatOptions>,
) -> Result<Formatted<T>, toml::de::Error>
where
    T: DeserializeOwned,
{
    let mut opts = options.unwrap_or_default();
    // Match JS version: comments/indentation are not preserved, but whitespace is.
    opts.preserve_indentation = false;
    let value = toml::from_str(text)?;
    Ok(Formatted::new(text, value, &opts))
}

/// Stringifies a TOML value with preserved outer whitespace.
pub fn stringify_toml<T>(
    formatted: &Formatted<T>,
    _options: Option<FormatOptions>,
) -> Result<String, toml::ser::Error>
where
    T: Serialize,
{
    let toml_str = toml::to_string(&formatted.value)?;
    Ok(format!(
        "{}{}{}",
        formatted.format.whitespace_start, toml_str, formatted.format.whitespace_end
    ))
}

