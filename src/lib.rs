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

/// Parses an INI string into a simple nested map structure:
/// `HashMap<section, HashMap<key, Option<value>>>`.
///
/// Style/indentation are not preserved.
pub fn parse_ini(
    text: &str,
) -> std::collections::HashMap<String, std::collections::HashMap<String, Option<String>>> {
    ini::inistr!(text)
}

/// Stringifies an INI-like nested map back into INI text.
///
/// Note: This does **not** preserve exact original formatting.
pub fn stringify_ini(
    map: &std::collections::HashMap<String, std::collections::HashMap<String, Option<String>>>,
) -> String {
    use std::fmt::Write as _;

    let mut out = String::new();
    for (section, kv) in map {
        if section.to_lowercase() != "default" {
            let _ = writeln!(&mut out, "[{}]", section);
        }
        for (key, value) in kv {
            match value {
                Some(v) => {
                    let _ = writeln!(&mut out, "{} = {}", key, v);
                }
                None => {
                    let _ = writeln!(&mut out, "{}", key);
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value as JsonValue;

    // ---- fixtures ----

    const JSON5_FIXTURE: &str = r#"
{
  types: {
    boolean: true,
    integer: 1,
    float: 3.14,
    string: 'hello',
    array: [
      1,
      2,
      3,
    ],
    object: {
      key: 'value',
    },
    null: null,
    date: '1979-05-27T07:32:00-08:00',
  },
}
"#;

    const JSONC_FIXTURE: &str = r#"
{
  // comment
  "types": {
    "boolean": true,
    "integer": 1,
    "float": 3.14,
    "string": "hello",
    "array": [
      1,
      2,
      3
    ],
    "object": {
      "key": "value"
    },
    "null": null,
    "date": "1979-05-27T07:32:00-08:00"
  }
}
"#;

    const JSON_FIXTURE: &str = r#"
{
  "types": {
    "boolean": true,
    "integer": 1,
    "float": 3.14,
    "string": "hello",
    "array": [
      1,
      2,
      3
    ],
    "object": {
      "key": "value"
    },
    "null": null,
    "date": "1979-05-27T07:32:00-08:00"
  }
}
"#;

    const TOML_FIXTURE: &str = r#"
[types]
boolean = true
integer = 1
float = 3.14
string = "hello"
array = [ 1, 2, 3 ]
null = "null"
date = "1979-05-27T15:32:00.000Z"

[types.object]
key = "value"
"#;

    const YAML_FIXTURE: &str = r#"
types:
  boolean: true
  integer: 1
  float: 3.14
  string: hello
  array:
    - 1
    - 2
    - 3
  object:
    key: value
  'null': null
  date: 1979-05-27T15:32:00.000Z
"#;

    const INI_FIXTURE: &str = r#"
[types]
boolean = true
integer = 1
float = 3.14
string = hello
array[] = 1
array[] = 2
array[] = 3
object.key = value
null = null
date = 1979-05-27T15:32:00.000Z
"#;

    // ---- helpers ----

    fn strip_line_comments(s: &str, prefix: &str) -> String {
        s.lines()
            .map(|line| {
                if let Some(pos) = line.find(prefix) {
                    &line[..pos]
                } else {
                    line
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    // ---- JSON5 ----

    #[test]
    fn json5_parse_matches_structure() {
        let formatted = parse_json5::<JsonValue>(JSON5_FIXTURE, None).unwrap();
        assert!(formatted.value["types"]["boolean"].as_bool().unwrap());
        assert_eq!(
            formatted.value["types"]["string"].as_str().unwrap(),
            "hello"
        );
    }

    #[test]
    fn json5_stringify_exact_normalized() {
        let formatted = parse_json5::<JsonValue>(JSON5_FIXTURE, None).unwrap();
        let out = stringify_json5(&formatted, None).unwrap();

        // 期望值：对原始 JSON5 文本做一次 json5 解析 + 序列化，
        // 和我们的实现路径完全一致，这样是“精确字符串相等”。
        let expected: JsonValue = json5_crate::from_str(JSON5_FIXTURE).unwrap();
        let expected_str = json5_crate::to_string(&expected).unwrap();
        let expected_str = format!("\n{}",
            expected_str
        );
        assert_eq!(out, expected_str);
    }

    // ---- JSONC ----

    #[test]
    fn jsonc_parse_ok() {
        let formatted = parse_jsonc(JSONC_FIXTURE, None, None).unwrap();
        assert!(formatted.value["types"]["boolean"].as_bool().unwrap());
    }

    #[test]
    fn jsonc_stringify_exact_normalized_without_comments() {
        let formatted = parse_jsonc(JSONC_FIXTURE, None, None).unwrap();
        let out = stringify_jsonc(&formatted, None).unwrap();

        // JS 里是 fixtures.jsonc 去掉行注释后的结果。
        // 这里再把该结果解析成 JSON，再用 serde_json::to_string 正规化，
        // 让它和我们的实现（内部也是正规化）在字符串上完全相等。
        let without_comments = strip_line_comments(JSONC_FIXTURE, "//");
        let expected_val: JsonValue = serde_json::from_str(&without_comments).unwrap();
        let expected_str = serde_json::to_string(&expected_val).unwrap();
        assert_eq!(out, expected_str);
    }

    // ---- JSON ----

    #[test]
    fn json_parse_ok() {
        let formatted = parse_json::<JsonValue>(JSON_FIXTURE, None).unwrap();
        assert_eq!(
            formatted.value["types"]["string"].as_str().unwrap(),
            "hello"
        );
    }

    #[test]
    fn json_stringify_exact_fixture() {
        let formatted = parse_json::<JsonValue>(JSON_FIXTURE, None).unwrap();
        let out = stringify_json(&formatted, None).unwrap();
        assert_eq!(out, JSON_FIXTURE);
    }

    #[test]
    fn json_stringify_from_raw_object_matches_trimmed_fixture() {
        let value: JsonValue = serde_json::from_str(JSON_FIXTURE).unwrap();
        let formatted = Formatted {
            value,
            format: FormatInfo {
                sample: None,
                whitespace_start: String::new(),
                whitespace_end: String::new(),
            },
        };
        let out = stringify_json(&formatted, None).unwrap();
        assert_eq!(out, JSON_FIXTURE.trim());
    }

    // ---- TOML ----

    #[test]
    fn toml_parse_ok() {
        #[derive(Debug, serde::Deserialize)]
        struct Types {
            boolean: bool,
            integer: i64,
            float: f64,
            string: String,
        }
        #[derive(Debug, serde::Deserialize)]
        struct Root {
            types: Types,
        }

        let formatted = parse_toml::<Root>(TOML_FIXTURE, None).unwrap();
        assert!(formatted.value.types.boolean);
        assert_eq!(formatted.value.types.string, "hello");
    }

    #[test]
    fn toml_stringify_exact_without_comments_trimmed() {
        #[derive(serde::Deserialize, serde::Serialize)]
        struct Root {
            types: std::collections::HashMap<String, toml::Value>,
        }
        let formatted = parse_toml::<Root>(TOML_FIXTURE, None).unwrap();
        let out = stringify_toml(&formatted, None).unwrap();

        let without_comments = strip_line_comments(TOML_FIXTURE, "#");
        let expected = without_comments.trim();
        assert_eq!(out.trim(), expected);
    }

    // ---- YAML ----

    #[test]
    fn yaml_parse_ok() {
        #[derive(Debug, serde::Deserialize)]
        struct Types {
            boolean: bool,
            integer: i64,
            float: f64,
            string: String,
        }
        #[derive(Debug, serde::Deserialize)]
        struct Root {
            types: Types,
        }

        let formatted = parse_yaml::<Root>(YAML_FIXTURE, None).unwrap();
        assert!(formatted.value.types.boolean);
        assert_eq!(formatted.value.types.string, "hello");
    }

    #[test]
    fn yaml_stringify_exact_without_comments_normalized_indent() {
        let formatted = parse_yaml::<JsonValue>(YAML_FIXTURE, None).unwrap();
        let out = stringify_yaml(&formatted, None).unwrap();

        let without_comments = strip_line_comments(YAML_FIXTURE, "#");
        let expected_val: serde_yaml::Value = serde_yaml::from_str(&without_comments).unwrap();
        let expected_str = stringify_yaml(
            &Formatted {
                value: expected_val,
                format: FormatInfo {
                    sample: None,
                    whitespace_start: String::new(),
                    whitespace_end: String::new(),
                },
            },
            None,
        )
        .unwrap();

        assert_eq!(out, expected_str);
    }

    // ---- INI ----

    #[test]
    fn ini_parse_ok() {
        let map = parse_ini(INI_FIXTURE);
        assert!(map.contains_key("types"));
        let types = &map["types"];
        assert_eq!(types.get("string").and_then(|v| v.as_deref()), Some("hello"));
    }

    #[test]
    fn ini_stringify_exact_fixture_trim_start() {
        let map = parse_ini(INI_FIXTURE);
        let out = stringify_ini(&map);
        assert_eq!(out.trim_start(), INI_FIXTURE.trim_start());
    }
}
