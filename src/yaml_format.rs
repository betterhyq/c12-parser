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
        let formatted = parse_yaml::<serde_yaml::Value>(YAML_FIXTURE, None).unwrap();
        let root = formatted.value;

        let types = root
            .get("types")
            .expect("types key should exist")
            .as_mapping()
            .expect("types should be a mapping");

        assert_eq!(
            types.get(&serde_yaml::Value::String("boolean".into())),
            Some(&serde_yaml::Value::Bool(true))
        );
        assert_eq!(
            types.get(&serde_yaml::Value::String("integer".into())),
            Some(&serde_yaml::Value::Number(1.into()))
        );
        assert_eq!(
            types.get(&serde_yaml::Value::String("float".into())),
            Some(&serde_yaml::Value::Number(serde_yaml::Number::from(3.14)))
        );
        assert_eq!(
            types.get(&serde_yaml::Value::String("string".into())),
            Some(&serde_yaml::Value::String("hello".into()))
        );
        assert_eq!(
            types.get(&serde_yaml::Value::String("array".into())),
            Some(&serde_yaml::Value::Sequence(vec![
                serde_yaml::Value::Number(1.into()),
                serde_yaml::Value::Number(2.into()),
                serde_yaml::Value::Number(3.into()),
            ]))
        );
        // `'null'` is a string key whose value is YAML null.
        assert_eq!(
            types.get(&serde_yaml::Value::String("null".into())),
            Some(&serde_yaml::Value::Null)
        );
        assert_eq!(
            types.get(&serde_yaml::Value::String("date".into())),
            Some(&serde_yaml::Value::String(
                "1979-05-27T15:32:00.000Z".into()
            ))
        );
    }

    #[test]
    fn yaml_stringify_exact_without_comments_normalized_indent() {
        let formatted = parse_yaml::<JsonValue>(YAML_FIXTURE, None).unwrap();
        let out = stringify_yaml(&formatted, None).unwrap();

        let without_comments = strip_line_comments(YAML_FIXTURE, "#");
        let expected_val: serde_yaml::Value = serde_yaml::from_str(&without_comments).unwrap();

        let out_val: serde_yaml::Value = serde_yaml::from_str(&out).unwrap();
        assert_eq!(out_val, expected_val);
    }

    #[test]
    fn yaml_preserves_outer_whitespace() {
        let text = " \ntypes:\n  key: value\n\n";
        let formatted = parse_yaml::<JsonValue>(text, None).unwrap();
        let out = stringify_yaml(&formatted, None).unwrap();

        assert!(out.starts_with(" \n"));
        assert!(out.ends_with("\n\n"));
    }
}
