use serde::{Serialize, de::DeserializeOwned};

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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn toml_parse_ok() {
        #[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq)]
        struct Types {
            boolean: bool,
            integer: i64,
            float: f64,
            string: String,
            array: Vec<i64>,
            null: String,
            date: String,
            object: Object,
        }

        #[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq)]
        struct Object {
            key: String,
        }

        #[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq)]
        struct Root {
            types: Types,
        }

        let formatted = parse_toml::<Root>(TOML_FIXTURE, None).unwrap();

        // Manually verify each field to avoid relying on `toml::from_str` for expectations.
        assert_eq!(formatted.value.types.boolean, true);
        assert_eq!(formatted.value.types.integer, 1);
        assert!((formatted.value.types.float - 3.14).abs() < f64::EPSILON);
        assert_eq!(formatted.value.types.string, "hello");
        assert_eq!(formatted.value.types.array, vec![1, 2, 3]);
        assert_eq!(formatted.value.types.null, "null");
        assert_eq!(formatted.value.types.date, "1979-05-27T15:32:00.000Z");
        assert_eq!(formatted.value.types.object.key, "value");
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

        // 通过解析成 toml::Value 比较语义是否等价，避免键顺序和空格风格差异。
        let expected_val: toml::Value = toml::from_str(expected).unwrap();
        let out_val: toml::Value = toml::from_str(out.trim()).unwrap();
        assert_eq!(out_val, expected_val);
    }

    #[test]
    fn toml_preserves_outer_whitespace() {
        let text = " \n[section]\nkey = 1\n\n";
        #[derive(serde::Deserialize, serde::Serialize)]
        struct Sectioned {
            section: std::collections::HashMap<String, toml::Value>,
        }

        let formatted = parse_toml::<Sectioned>(text, None).unwrap();
        let out = stringify_toml(&formatted, None).unwrap();

        assert!(out.starts_with(" \n"));
        assert!(out.ends_with("\n\n"));
    }
}
