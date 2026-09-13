//! Approximate terminal display width of a string, as opposed to its
//! `char` count. A combining accent takes zero columns and a CJK
//! ideograph takes two, so `chars().count()` is off for both in a way
//! that matters once a line mixes scripts.
//!
//! This is not a full port of the Unicode East Asian Width tables (that
//! would need a generated table too large to hand-maintain without a
//! dependency). It covers the ranges that actually show up in commit
//! messages and docs: combining marks, zero-width joiners/selectors, and
//! the common CJK/Hangul/fullwidth blocks. Anything outside those ranges
//! counts as one column, which is right for Latin, Greek, Cyrillic, and
//! most other scripts.

/// The display width of `s` in terminal columns.
pub fn display_width(s: &str) -> usize {
    s.chars().map(char_width).sum()
}

fn char_width(c: char) -> usize {
    let cp = c as u32;
    if is_zero_width(cp) {
        0
    } else if is_wide(cp) {
        2
    } else {
        1
    }
}

fn is_zero_width(cp: u32) -> bool {
    matches!(cp,
        0x0300..=0x036F   // combining diacritical marks
        | 0x0483..=0x0489 // combining Cyrillic
        | 0x200B..=0x200D // zero-width space / non-joiner / joiner
        | 0xFE00..=0xFE0F // variation selectors
        | 0xFEFF          // zero-width no-break space / BOM
    )
}

fn is_wide(cp: u32) -> bool {
    matches!(cp,
        0x1100..=0x115F   // Hangul jamo
        | 0x2E80..=0x303E // CJK radicals, symbols and punctuation
        | 0x3041..=0x33FF // hiragana through CJK compatibility
        | 0x3400..=0x4DBF // CJK unified ideographs extension A
        | 0x4E00..=0x9FFF // CJK unified ideographs
        | 0xA000..=0xA4CF // Yi
        | 0xAC00..=0xD7A3 // Hangul syllables
        | 0xF900..=0xFAFF // CJK compatibility ideographs
        | 0xFE30..=0xFE4F // CJK compatibility forms
        | 0xFF00..=0xFF60 // fullwidth forms
        | 0xFFE0..=0xFFE6 // fullwidth signs
        | 0x20000..=0x2FFFD // CJK unified ideographs extension B and beyond
        | 0x30000..=0x3FFFD // CJK unified ideographs extension G and beyond
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Case {
        name: &'static str,
        input: &'static str,
        want: usize,
    }

    #[test]
    fn widths() {
        let cases = [
            Case {
                name: "empty string",
                input: "",
                want: 0,
            },
            Case {
                name: "plain ascii",
                input: "hello",
                want: 5,
            },
            Case {
                name: "combining accent adds no width",
                input: "e\u{0301}", // e + combining acute accent
                want: 1,
            },
            Case {
                name: "precomposed accent counts as one char, one column",
                input: "\u{00E9}", // e with acute, precomposed
                want: 1,
            },
            Case {
                name: "cjk ideographs are double width",
                input: "\u{4E2D}\u{6587}", // "中文"
                want: 4,
            },
            Case {
                name: "hangul syllable is double width",
                input: "\u{D55C}", // "한"
                want: 2,
            },
            Case {
                name: "zero-width joiner does not add width",
                input: "a\u{200D}b",
                want: 2,
            },
            Case {
                name: "mixed ascii and wide text",
                input: "ok \u{4E2D}\u{6587} done",
                want: 3 + 4 + 5,
            },
        ];

        for c in cases {
            assert_eq!(display_width(c.input), c.want, "case: {}", c.name);
        }
    }
}
