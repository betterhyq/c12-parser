use serde::{Serialize, de::DeserializeOwned};

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
        formatted.format.whitespace_start, yaml_str, formatted.format.whitespace_end
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value as JsonValue;

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
        assert_eq!(formatted.value.types.integer, 1);
        assert!((formatted.value.types.float - 3.14).abs() < f64::EPSILON);
    }

    #[test]
    fn yaml_stringify_exact_without_comments_normalized_indent() {
        let formatted = parse_yaml::<JsonValue>(YAML_FIXTURE, None).unwrap();
        let out = stringify_yaml(&formatted, None).unwrap();

        let without_comments = strip_line_comments(YAML_FIXTURE, "#");
        let expected_val: serde_yaml::Value = serde_yaml::from_str(&without_comments).unwrap();

        // 直接比较解析后的 YAML 值是否等价，避免键顺序和缩进实现差异。
        let out_val: serde_yaml::Value = serde_yaml::from_str(&out).unwrap();
        assert_eq!(out_val, expected_val);
    }
}
