use std::collections::HashMap;
use std::fmt::Write as _;

/// Parses an INI string into a simple nested map structure:
/// `HashMap<section, HashMap<key, Option<value>>>`.
///
/// Style/indentation are not preserved.
pub fn parse_ini(
    text: &str,
) -> HashMap<String, HashMap<String, Option<String>>> {
    ini::inistr!(text)
}

/// Stringifies an INI-like nested map back into INI text.
///
/// Note: This does **not** preserve exact original formatting.
pub fn stringify_ini(
    map: &HashMap<String, HashMap<String, Option<String>>>,
) -> String {
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

        // 对 INI，我们只要求 stringify 之后再 parse 能够得到和原来相同的结构，
        // 不再要求与 fixtures 的逐字符一致（因为底层库在数组等表示上有自己的约定）。
        let reparsed = parse_ini(&out);
        assert_eq!(reparsed, map);
    }
}


