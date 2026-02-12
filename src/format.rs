use once_cell::sync::Lazy;
use regex::Regex;

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
    /// indentation is auto-detected from the original text (if enabled).
    pub indent: Option<usize>,

    /// If `false`, indentation from the original text will not be
    /// auto-detected, even if a sample is present.
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

pub(crate) fn detect_format(text: &str, opts: &FormatOptions) -> FormatInfo {
    let sample = if opts.indent.is_none() && opts.preserve_indentation {
        Some(text.chars().take(opts.sample_size).collect::<String>())
    } else {
        None
    };

    static START_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\s+)").unwrap());
    static END_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\s+)$").unwrap());

    let (whitespace_start, whitespace_end) = if opts.preserve_whitespace {
        let ws_start = START_RE
            .captures(text)
            .and_then(|c| c.get(0))
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
        let ws_end = END_RE
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

pub(crate) fn compute_indent(info: &FormatInfo, opts: &FormatOptions) -> usize {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_format_captures_outer_whitespace_and_sample() {
        let text = "\n  {\"a\": 1}\n\n";
        let opts = FormatOptions::default();
        let info = detect_format(text, &opts);

        // 由于使用的是基于正则的 `^(\s+)`，这里会把换行符和紧随其后的两个空格
        // 一并视为“前导空白”捕获出来。
        assert_eq!(info.whitespace_start, "\n  ");
        assert_eq!(info.whitespace_end, "\n\n");
        assert!(info.sample.is_some());
        assert!(info.sample.as_ref().unwrap().contains("{\"a\": 1}"));
    }

    #[test]
    fn detect_format_respects_preserve_flags() {
        let text = "   {\"a\": 1}   ";
        let mut opts = FormatOptions::default();
        opts.preserve_whitespace = false;
        opts.preserve_indentation = false;

        let info = detect_format(text, &opts);
        assert!(info.sample.is_none());
        assert!(info.whitespace_start.is_empty());
        assert!(info.whitespace_end.is_empty());
    }

    #[test]
    fn compute_indent_prefers_explicit_indent() {
        let info = FormatInfo {
            sample: Some("  key: 1".into()),
            whitespace_start: String::new(),
            whitespace_end: String::new(),
        };
        let mut opts = FormatOptions::default();
        opts.indent = Some(4);

        assert_eq!(compute_indent(&info, &opts), 4);
    }

    #[test]
    fn compute_indent_detects_from_sample() {
        let info = FormatInfo {
            sample: Some("  key: 1\n    child: 2".into()),
            whitespace_start: String::new(),
            whitespace_end: String::new(),
        };
        let opts = FormatOptions::default();

        assert_eq!(compute_indent(&info, &opts), 2);
    }

    #[test]
    fn compute_indent_falls_back_to_default() {
        let info = FormatInfo {
            sample: Some("\n\n".into()),
            whitespace_start: String::new(),
            whitespace_end: String::new(),
        };
        let opts = FormatOptions::default();

        assert_eq!(compute_indent(&info, &opts), 2);
    }
}
