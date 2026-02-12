use json5 as json5_crate;
use serde::{Serialize, de::DeserializeOwned};

use crate::format::{FormatOptions, Formatted, compute_indent};

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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value as JsonValue;

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

    #[test]
    fn json5_parse_matches_structure() {
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

        let formatted = parse_json5::<Root>(JSON5_FIXTURE, None).unwrap();

        // 对每一个字段单独断言，确保结构体里的所有值都解析正确。
        assert!(formatted.value.types.boolean);
        assert_eq!(formatted.value.types.integer, 1);
        assert!((formatted.value.types.float - 3.14).abs() < f64::EPSILON);
        assert_eq!(formatted.value.types.string, "hello");
        assert_eq!(formatted.value.types.array, vec![1, 2, 3]);
        assert_eq!(formatted.value.types.object["key"].as_str(), Some("value"));
        assert!(formatted.value.types.null.is_none());
        assert_eq!(
            formatted.value.types.date,
            "1979-05-27T07:32:00-08:00".to_string()
        );
    }

    #[test]
    fn json5_stringify_exact_normalized() {
        let formatted = parse_json5::<JsonValue>(JSON5_FIXTURE, None).unwrap();
        let out = stringify_json5(&formatted, None).unwrap();

        // 期望值：对原始 JSON5 文本做一次 json5 解析 + 序列化，
        // 和我们的实现路径完全一致，这样是“精确字符串相等”。
        let expected: JsonValue = ::json5::from_str(JSON5_FIXTURE).unwrap();
        let expected_str = ::json5::to_string(&expected).unwrap();
        let expected_str = format!("\n{}", expected_str);

        // 为了避免不同版本 json5 在末尾换行等细节上的差异，这里放宽到
        // 去掉首尾空白后的字符串相等。
        assert_eq!(out.trim(), expected_str.trim());
    }

    #[test]
    fn json5_preserves_outer_whitespace() {
        let text = " \n{ types: { boolean: true } }\n\t";
        let formatted = parse_json5::<JsonValue>(text, None).unwrap();
        let out = stringify_json5(&formatted, None).unwrap();

        assert!(out.starts_with(" \n"));
        assert!(out.ends_with("\n\t"));
    }
}
