use serde::{de::DeserializeOwned, Serialize};

use crate::format::{FormatOptions, Formatted};

/// Parses a YAML string into a value, capturing outer whitespace only.
pub fn parse_yaml<T>(
    text: &str,
    options: Option<FormatOptions>,
) -> Result<Formatted<T>, serde_yaml::Error>
where
    T: DeserializeOwned,
{
    let mut opts = options.unwrap_or_default();
    // Comments are not preserved; indentation is not preserved in the JS version.
    opts.preserve_indentation = false;
    let value = serde_yaml::from_str(text)?;
    Ok(Formatted::new(text, value, &opts))
}

/// Stringifies a YAML value with preserved outer whitespace.
pub fn stringify_yaml<T>(
    formatted: &Formatted<T>,
    options: Option<FormatOptions>,
) -> Result<String, serde_yaml::Error>
where
    T: Serialize,
{
    let _opts = options.unwrap_or_default();

    // We let serde_yaml handle inner indentation and only restore the
    // outer whitespace captured during parsing.
    let yaml_str = serde_yaml::to_string(&formatted.value)?;

    Ok(format!(
        "{}{}{}",
        formatted.format.whitespace_start,
        yaml_str,
        formatted.format.whitespace_end
    ))
}

