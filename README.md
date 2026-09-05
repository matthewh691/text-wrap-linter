# wraplint

A linter for hard-wrapped plain text.

## The problem

Some text is still wrapped by hand, or by a "reflow paragraph" editor
command, at a fixed column width: git commit bodies, RFCs, man pages,
CHANGELOGs, plenty of READMEs. That works fine until someone edits a
paragraph later and doesn't re-wrap it. You end up with:

- lines that run past the column limit the rest of the file uses
- a line that stops short in the middle of a paragraph even though the
  next word would have fit on it (a classic sign the paragraph was
  edited and never re-flowed)
- trailing whitespace and stray tabs left over from an editor

None of that breaks a Markdown renderer, so it tends to sit unnoticed
until someone reads the raw file in a terminal at 80 columns and it
looks ragged. wraplint checks a file's wrapping and reports the line
number of each problem so you can fix it before it ships.

## Usage

```
cargo build --release
./target/release/wraplint --width 72 CHANGELOG.txt
```

Given a file like:

```
This line is fine.
This one however goes on for entirely too
long. It ran past the wrap column because someone tacked a sentence
onto the end of the paragraph without rewrapping the rest of it.
```

wraplint reports:

```
CHANGELOG.txt:2: ragged-wrap: line wraps at 43 characters though the next word ('long.') would fit within the 72 limit
```

`--width` defaults to 72, the usual width for commit message bodies.
The exit code is 0 when a file is clean and 1 when there's at least
one finding, so it can be dropped into a pre-commit hook or CI step.

Pass `-` in place of a filename to read from stdin instead, which is
what you want in a `commit-msg` hook:

```
./target/release/wraplint --width 72 - < "$1"
```

Findings from stdin are reported under the path `-`.

## Rules

- `line-too-long` — a line is wider than the configured limit. A line
  that's a single unbreakable token (a bare URL, for instance) is
  exempt, since wrapping it wouldn't help.
- `ragged-wrap` — a line inside a paragraph stops well short of the
  limit even though the next line's first word would have fit on it.
  The last line of a paragraph is exempt, since it's supposed to be
  short.
- `trailing-whitespace` — a line ends in a space or tab.
- `hard-tab` — a line contains a tab character, whose rendered width
  isn't fixed.

Lines inside a fenced code block (delimited by ``` ``` ``` or `~~~`)
are exempt from every rule. Code isn't wrapped by hand and its width
is whatever the code needs it to be.

## Known limitations

Line width is measured in `char`s, not display columns, so wide or
combining Unicode characters will throw the count off. Blockquote
prefixes (`> `) aren't recognized yet, so a quoted paragraph is linted
as if it were prose.

## Development

```
cargo test
```

The paragraph-boundary rules (`ragged-wrap` especially) are covered by
a table of cases in `tests/awkward_cases.rs`, since that's where a
naive implementation tends to go wrong: the last line of a paragraph,
a paragraph that's only one line long, a run of blank-line-separated
paragraphs, and a token too long to wrap regardless of width.

## License

MIT, see `LICENSE`.
