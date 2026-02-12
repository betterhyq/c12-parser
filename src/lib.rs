use ini as ini_crate;
use json5 as json5_crate;
use jsonc_parser::{parse_to_serde_value, ParseOptions as JsoncParseOptions};
use regex::Regex;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value as JsonValue;

/// Information about formatting (indentation and outer whitespace)
/// captured from the original text.
#[derive(Clone, Debug)]
pub struct FormatInfo {
    pub sample: Option<String>,
    pub whitespace_start: String,
    pub whitespace_end: String,
}

/// Options that control how formatting is detected and preserved.
#[derive(Clone, Debug)]
pub struct FormatOptions {
    /// Explicit indent to use when stringifying. When `None`,
    /// indentation may be auto-detected from the original text.
    pub indent: Option<usize>,

    /// If `false`, indentation from the original text will not be
    /// auto-detected.
    pub preserve_indentation: bool,

    /// If `false`, leading and trailing whitespace around the value
    /// will not be preserved.
    pub preserve_whitespace: bool,

    /// Number of characters to sample from the start of the text
    /// when detecting indentation.
    pub sample_size: usize,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            indent: None,
            preserve_indentation: true,
            preserve_whitespace: true,
            sample_size: 1024,
        }
    }
}

fn detect_format(text: &str, opts: &FormatOptions) -> FormatInfo {
    let sample = if opts.indent.is_none() && opts.preserve_indentation {
        Some(
            text.chars()
                .take(opts.sample_size)
                .collect::<String>(),
        )
    } else {
        None
    };

    let (whitespace_start, whitespace_end) = if opts.preserve_whitespace {
        // Leading whitespace
        let start_re = Regex::new(r"^(\s+)").unwrap();
        let end_re = Regex::new(r"(\s+)$").unwrap();

        let ws_start = start_re
            .captures(text)
            .and_then(|c| c.get(0))
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
        let ws_end = end_re
            .captures(text)
            .and_then(|c| c.get(0))
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();

        (ws_start, ws_end)
    } else {
        (String::new(), String::new())
    };

    FormatInfo {
        sample,
        whitespace_start,
        whitespace_end,
    }
}

fn compute_indent(info: &FormatInfo, opts: &FormatOptions) -> usize {
    if let Some(explicit) = opts.indent {
        return explicit;
    }

    if let Some(sample) = &info.sample {
        // Naive indent detection: find the first non-empty line and
        // count its leading spaces.
        for line in sample.lines() {
            let trimmed = line.trim_start();
            if trimmed.is_empty() {
                continue;
            }
            let indent_len = line.len() - trimmed.len();
            if indent_len > 0 {
                return indent_len;
            }
        }
    }

    // Default indent size if nothing else is detected
    2
}

/// A value bundled with its detected formatting information.
#[derive(Clone, Debug)]
pub struct Formatted<T> {
    pub value: T,
    pub format: FormatInfo,
}

impl<T> Formatted<T> {
    pub fn new(text: &str, value: T, opts: &FormatOptions) -> Self {
        let format = detect_format(text, opts);
        Self { value, format }
    }
}

// ===== JSON =====

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

// ===== JSON5 =====

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
    let indent = compute_indent(&formatted.format, &opts);
    let mut serializer = json5_crate::Serializer::new(String::new());
    serializer
        .indent(Some(indent))
        .serialize(&formatted.value)?;
    let json5 = serializer.into_inner();
    Ok(format!(
        "{}{}{}",
        formatted.format.whitespace_start, json5, formatted.format.whitespace_end
    ))
}

// ===== JSONC =====

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
    stringify_json(&Formatted {
        value: &formatted.value,
        format: formatted.format.clone(),
    }, options)
}

// ===== TOML =====

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

// ===== YAML =====

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
    let opts = options.unwrap_or_default();
    let indent = compute_indent(&formatted.format, &opts);

    // serde_yaml doesn't expose indent size directly, but respects
    // configuration via emitter. We approximate by using default and
    // not attempting to perfectly match JS behavior; outer whitespace
    // is preserved exactly, and inner indentation is best-effort.
    let yaml_str = serde_yaml::to_string(&formatted.value)?;
    let adjusted = yaml_str
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
        formatted.format.whitespace_start,
        adjusted.trim(),
        formatted.format.whitespace_end
    ))
}

// ===== INI =====

/// Parses an INI string into a value. Style/indentation are not preserved.
pub fn parse_ini(
    text: &str,
    options: Option<ini_crate::IniOptions>,
) -> ini_crate::Ini {
    // The `ini` crate doesn't expose formatting hooks; we just parse.
    ini_crate::Ini::load_from_str_opt(text, options)
        .unwrap_or_else(|_| ini_crate::Ini::new())
}

/// Stringifies an INI value. Style/indentation are not preserved.
pub fn stringify_ini(ini: &ini_crate::Ini) -> String {
    let mut output = Vec::new();
    ini.write_to(&mut output).ok();
    String::from_utf8_lossy(&output).into_owned()
}

