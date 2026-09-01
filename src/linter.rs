//! Rules for linting hard-wrapped plain text: paragraphs that a human
//! wrapped by hand (or with a "reflow" command) at a fixed column width,
//! the way git commit bodies, RFCs, and man pages usually are.

use std::fmt;

/// A single rule violation, tied to a 1-indexed line number.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub line: usize,
    pub rule: Rule,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rule {
    LineTooLong,
    TrailingWhitespace,
    HardTab,
    RaggedWrap,
}

impl Rule {
    pub fn name(&self) -> &'static str {
        match self {
            Rule::LineTooLong => "line-too-long",
            Rule::TrailingWhitespace => "trailing-whitespace",
            Rule::HardTab => "hard-tab",
            Rule::RaggedWrap => "ragged-wrap",
        }
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}: {}", self.line, self.rule.name(), self.message)
    }
}

pub struct Options {
    pub max_width: usize,
}

impl Default for Options {
    fn default() -> Self {
        Options { max_width: 72 }
    }
}

pub fn lint(text: &str, opts: &Options) -> Vec<Finding> {
    let lines: Vec<&str> = text.lines().collect();
    let mut findings = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        let n = i + 1;
        check_trailing_whitespace(line, n, &mut findings);
        check_hard_tab(line, n, &mut findings);
        check_line_too_long(line, n, opts.max_width, &mut findings);
    }

    check_ragged_wraps(&lines, opts.max_width, &mut findings);

    findings.sort_by_key(|f| f.line);
    findings
}

fn check_trailing_whitespace(line: &str, n: usize, out: &mut Vec<Finding>) {
    if line != line.trim_end_matches(|c: char| c == ' ' || c == '\t') {
        out.push(Finding {
            line: n,
            rule: Rule::TrailingWhitespace,
            message: "line has trailing whitespace".to_string(),
        });
    }
}

fn check_hard_tab(line: &str, n: usize, out: &mut Vec<Finding>) {
    if line.contains('\t') {
        out.push(Finding {
            line: n,
            rule: Rule::HardTab,
            message: "line contains a hard tab; width depends on the reader's tab size"
                .to_string(),
        });
    }
}

fn check_line_too_long(line: &str, n: usize, max_width: usize, out: &mut Vec<Finding>) {
    let width = line.chars().count();
    if width <= max_width {
        return;
    }
    // A line with no whitespace is a single unbreakable token (a URL, a
    // path, a long identifier). Wrapping can't fix that, so don't ask
    // for it.
    if !line.trim().contains(char::is_whitespace) {
        return;
    }
    out.push(Finding {
        line: n,
        rule: Rule::LineTooLong,
        message: format!(
            "line is {} characters wide, over the {} limit",
            width, max_width
        ),
    });
}

/// Find lines that end well short of `max_width` in the middle of a
/// paragraph, where the next line's first word would have fit on this
/// one. That pattern usually means the paragraph was edited after it
/// was wrapped and never re-flowed. The last line of a paragraph is
/// exempt: it is supposed to be short.
fn check_ragged_wraps(lines: &[&str], max_width: usize, out: &mut Vec<Finding>) {
    let mut i = 0;
    while i < lines.len() {
        if lines[i].trim().is_empty() {
            i += 1;
            continue;
        }
        // Collect the run of non-blank lines that make up one paragraph.
        let start = i;
        let mut end = i;
        while end + 1 < lines.len() && !lines[end + 1].trim().is_empty() {
            end += 1;
        }

        for j in start..end {
            let line = lines[j];
            let single_token = !line.trim().contains(char::is_whitespace);
            if single_token {
                continue;
            }
            let width = line.chars().count();
            let next_word = lines[j + 1].split_whitespace().next().unwrap_or("");
            if next_word.is_empty() {
                continue;
            }
            if width + 1 + next_word.chars().count() <= max_width {
                out.push(Finding {
                    line: j + 1,
                    rule: Rule::RaggedWrap,
                    message: format!(
                        "line wraps at {} characters though the next word ('{}') would fit within the {} limit",
                        width, next_word, max_width
                    ),
                });
            }
        }

        i = end + 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Case {
        name: &'static str,
        input: &'static str,
        max_width: usize,
        want: &'static [(usize, Rule)],
    }

    #[test]
    fn per_line_rules() {
        let cases = [
            Case {
                name: "trailing space",
                input: "hello \n",
                max_width: 72,
                want: &[(1, Rule::TrailingWhitespace)],
            },
            Case {
                name: "trailing tab",
                input: "hello\t\n",
                max_width: 72,
                want: &[(1, Rule::TrailingWhitespace), (1, Rule::HardTab)],
            },
            Case {
                name: "hard tab mid-line",
                input: "a\tb\n",
                max_width: 72,
                want: &[(1, Rule::HardTab)],
            },
            Case {
                name: "clean short line",
                input: "hello world\n",
                max_width: 72,
                want: &[],
            },
        ];

        for c in cases {
            let findings = lint(c.input, &Options { max_width: c.max_width });
            let got: Vec<(usize, Rule)> = findings.iter().map(|f| (f.line, f.rule)).collect();
            assert_eq!(got, c.want, "case: {}", c.name);
        }
    }
}
