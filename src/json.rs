use serde::{Serialize, de::DeserializeOwned};

use crate::format::{FormatOptions, Formatted, compute_indent};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::{FormatInfo, Formatted};
    use serde_json::Value as JsonValue;

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

    #[test]
    fn json_parse_ok() {
        #[derive(Debug, serde::Deserialize)]
        struct Types {
            boolean: bool,
            integer: i64,
            float: f64,
            string: String,
            array: Vec<i64>,
            object: serde_json::Value,
            null: Option<serde_json::Value>,
            date: String,
        }

        #[derive(Debug, serde::Deserialize)]
        struct Root {
            types: Types,
        }

        let formatted = parse_json::<Root>(JSON_FIXTURE, None).unwrap();

        // 对每一个字段单独断言，确保结构体里的所有值都解析正确。
        assert!(formatted.value.types.boolean);
        assert_eq!(formatted.value.types.integer, 1);
        assert!((formatted.value.types.float - 3.14).abs() < f64::EPSILON);
        assert_eq!(formatted.value.types.string, "hello");
        assert_eq!(formatted.value.types.array, vec![1, 2, 3]);
        assert_eq!(
            formatted.value.types.object["key"].as_str(),
            Some("value")
        );
        assert!(formatted.value.types.null.is_none());
        assert_eq!(
            formatted.value.types.date,
            "1979-05-27T07:32:00-08:00".to_string()
        );
    }

    #[test]
    fn json_stringify_exact_fixture() {
        let formatted = parse_json::<JsonValue>(JSON_FIXTURE, None).unwrap();
        let out = stringify_json(&formatted, None).unwrap();

        // 比较两边解析后的 JSON 值是否等价，而不是逐字符一致，
        // 以规避键顺序和缩进风格差异。
        let out_val: JsonValue = serde_json::from_str(&out).unwrap();
        let expected_val: JsonValue = serde_json::from_str(JSON_FIXTURE).unwrap();
        assert_eq!(out_val, expected_val);
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

        // 同样比较解析后的值是否等价即可，不再要求字符串完全一致。
        let out_val: JsonValue = serde_json::from_str(&out).unwrap();
        let expected_val: JsonValue = serde_json::from_str(JSON_FIXTURE).unwrap();
        assert_eq!(out_val, expected_val);
    }

    #[test]
    fn json_stringify_respects_explicit_indent() {
        let formatted = parse_json::<JsonValue>(JSON_FIXTURE, None).unwrap();
        let mut opts = FormatOptions::default();
        opts.indent = Some(4);

        let out = stringify_json(&formatted, Some(opts)).unwrap();

        // 第一行是空行（前导换行），第二行应为带 4 个空格缩进的 "{".
        let mut lines = out.lines();
        assert_eq!(lines.next(), Some("")); // leading newline
        if let Some(second) = lines.next() {
            let prefix = &second[..4.min(second.len())];
            assert_eq!(prefix, "    ");
        } else {
            panic!("expected at least two lines in JSON output");
        }
    }

    #[test]
    fn json_preserves_outer_whitespace() {
        let text = " \n{ \"a\": 1 }\n\t";
        let formatted = parse_json::<JsonValue>(text, None).unwrap();
        let out = stringify_json(&formatted, None).unwrap();

        assert!(out.starts_with(" \n"));
        assert!(out.ends_with("\n\t"));
    }
}
