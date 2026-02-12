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

pub(crate) fn detect_format(text: &str, opts: &FormatOptions) -> FormatInfo {
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

