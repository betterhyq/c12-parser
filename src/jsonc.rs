use jsonc_parser::{parse_to_serde_value, ParseOptions as JsoncParseOptions};
use serde_json::Value as JsonValue;

use crate::format::{FormatOptions, Formatted};
use crate::json::stringify_json;

/// Extra options for JSONC parsing.
#[derive(Clone, Debug, Default)]
pub struct JsoncExtraOptions {
    pub disallow_comments: bool,
    pub allow_trailing_comma: bool,
}

/// Parses a JSONC string into a serde_json::Value, capturing formatting.
pub fn parse_jsonc(
    text: &str,
    fmt_options: Option<FormatOptions>,
    jsonc_options: Option<JsoncExtraOptions>,
) -> Result<Formatted<JsonValue>, Box<dyn std::error::Error>> {
    let fmt_opts = fmt_options.unwrap_or_default();
    let extra = jsonc_options.unwrap_or_default();

    let parse_opts = JsoncParseOptions {
        allow_comments: !extra.disallow_comments,
        allow_trailing_commas: extra.allow_trailing_comma,
        ..Default::default()
    };

    let value_opt = parse_to_serde_value(text, &parse_opts)?;
    let value = value_opt.unwrap_or(JsonValue::Null);
    Ok(Formatted::new(text, value, &fmt_opts))
}

/// Stringifies a JSONC value (as plain JSON) with preserved formatting.
pub fn stringify_jsonc(
    formatted: &Formatted<JsonValue>,
    options: Option<FormatOptions>,
) -> serde_json::Result<String> {
    // JSONC comments/trailing commas are not preserved; we emit plain JSON.
    stringify_json(
        &Formatted {
            value: &formatted.value,
            format: formatted.format.clone(),
        },
        options,
    )
}

