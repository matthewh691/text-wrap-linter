//! Minimal JSON array output for findings, so `--json` doesn't need a
//! serde dependency. The findings themselves are small and flat enough
//! that a hand-rolled encoder is less code than wiring up a crate would
//! be, as long as the escaping is right.

use crate::linter::Finding;

/// Render `(path, finding)` pairs as a single JSON array, in the order
/// given. Callers are expected to have already sorted findings the way
/// they want them reported.
pub fn findings_to_json(items: &[(&str, &Finding)]) -> String {
    let mut out = String::from("[");
    for (idx, (path, finding)) in items.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        out.push_str("{\"path\":");
        out.push_str(&escape(path));
        out.push_str(",\"line\":");
        out.push_str(&finding.line.to_string());
        out.push_str(",\"rule\":");
        out.push_str(&escape(finding.rule.name()));
        out.push_str(",\"message\":");
        out.push_str(&escape(&finding.message));
        out.push('}');
    }
    out.push(']');
    out
}

/// Quote and escape a string for use as a JSON string literal. Covers
/// the characters that would otherwise break the JSON (quote,
/// backslash) or produce invalid JSON (raw control characters).
fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linter::Rule;

    #[test]
    fn escapes_quotes_and_backslashes() {
        assert_eq!(escape("a\"b\\c"), "\"a\\\"b\\\\c\"");
    }

    #[test]
    fn escapes_control_characters() {
        assert_eq!(escape("a\nb\tc\rd"), "\"a\\nb\\tc\\rd\"");
        assert_eq!(escape("\u{1}"), "\"\\u0001\"");
    }

    #[test]
    fn leaves_plain_text_alone() {
        assert_eq!(escape("hello world"), "\"hello world\"");
    }

    #[test]
    fn empty_array_for_no_findings() {
        assert_eq!(findings_to_json(&[]), "[]");
    }

    #[test]
    fn one_finding_serializes_all_fields() {
        let finding = Finding {
            line: 3,
            rule: Rule::LineTooLong,
            message: "line is 80 columns wide, over the 72 limit".to_string(),
        };
        let items = [("CHANGELOG.txt", &finding)];
        assert_eq!(
            findings_to_json(&items),
            "[{\"path\":\"CHANGELOG.txt\",\"line\":3,\"rule\":\"line-too-long\",\"message\":\"line is 80 columns wide, over the 72 limit\"}]"
        );
    }

    #[test]
    fn multiple_findings_are_comma_separated() {
        let a = Finding {
            line: 1,
            rule: Rule::HardTab,
            message: "line contains a hard tab; width depends on the reader's tab size"
                .to_string(),
        };
        let b = Finding {
            line: 2,
            rule: Rule::TrailingWhitespace,
            message: "line has trailing whitespace".to_string(),
        };
        let items = [("a.txt", &a), ("b.txt", &b)];
        let json = findings_to_json(&items);
        assert!(json.starts_with('['));
        assert!(json.ends_with(']'));
        assert_eq!(json.matches("\"path\":\"a.txt\"").count(), 1);
        assert_eq!(json.matches("\"path\":\"b.txt\"").count(), 1);
        assert_eq!(json.matches("},{").count(), 1);
    }
}
