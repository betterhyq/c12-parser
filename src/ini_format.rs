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

