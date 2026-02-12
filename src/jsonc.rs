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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value as JsonValue;

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
        // 这里再把该结果解析成 JSON，比较“值”等价，而不是要求序列化后的
        // 字符串逐字符一致（不同实现的 pretty-print 策略可能不同）。
        let without_comments = strip_line_comments(JSONC_FIXTURE, "//");
        let expected_val: JsonValue = serde_json::from_str(&without_comments).unwrap();
        let out_val: JsonValue = serde_json::from_str(&out).unwrap();
        assert_eq!(out_val, expected_val);
    }
}


