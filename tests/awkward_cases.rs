//! The rules in `linter.rs` are simple individually, but their interaction
//! with paragraph boundaries is where a naive implementation goes wrong.
//! This is a table of the cases that actually broke earlier drafts:
//! the last line of a paragraph, a paragraph that's only one line long,
//! and a single unbreakable token (a URL) that can't be wrapped further.

use wraplint::linter::{lint, Options, Rule};

struct Case {
    name: &'static str,
    input: &'static str,
    max_width: usize,
    want: &'static [(usize, Rule)],
}

#[test]
fn awkward_cases() {
    let cases = [
        Case {
            name: "last line of a paragraph is short on purpose",
            input: "abcdefghij klmnopqrs\ntuvwxyz1234 567890AB\nok\n",
            max_width: 20,
            want: &[],
        },
        Case {
            name: "single-line paragraph, far under width, not flagged",
            input: "hi\n",
            max_width: 72,
            want: &[],
        },
        Case {
            name: "a bare URL longer than the width is an unbreakable token, exempt",
            input: "https://example.com/a/very/long/path/that/cannot/be/split/at/all\n",
            max_width: 40,
            want: &[],
        },
        Case {
            name: "ragged wrap: next word would clearly have fit",
            input: "hello world\nfoo bar baz\n",
            max_width: 20,
            want: &[(1, Rule::RaggedWrap)],
        },
        Case {
            name: "not ragged: next word is too long to have fit",
            input: "hello world\nreallylongword\n",
            max_width: 15,
            want: &[],
        },
        Case {
            name: "blank line separates paragraphs; ragged wrap only inside the second",
            input: "para one line\n\npara two is here\nshort\n",
            max_width: 30,
            want: &[(3, Rule::RaggedWrap)],
        },
        Case {
            name: "line over width with no whitespace is a token, not a wrap failure",
            input: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n",
            max_width: 40,
            want: &[],
        },
        Case {
            name: "line over width with whitespace is flagged",
            input: "abcdefg hijklmno\n",
            max_width: 10,
            want: &[(1, Rule::LineTooLong)],
        },
    ];

    for c in cases {
        let findings = lint(c.input, &Options { max_width: c.max_width });
        let got: Vec<(usize, Rule)> = findings.iter().map(|f| (f.line, f.rule)).collect();
        assert_eq!(got, c.want, "case: {}", c.name);
    }
}
