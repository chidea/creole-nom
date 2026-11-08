# Upstream Test Coverage Audit

Cross-check of `src/parser.rs` tests against the reference WikiCreole test suites of
`paulyg/WikiCreole` (PHP implementation, pinned at `f969cba`, 2014-10-18):

- `tests/CreoleTestInput.txt` / `CreoleTestExpected.html` — the official Creole 1.0 test
  cases (the same suite the README's "Strict 1.0" checklist links).
- `tests/WikiCreoleTest.php` — ~40 PHPUnit unit tests (input + expected HTML per method).

Upstream asserts HTML; this crate emits `ICreole`. Comparison is therefore semantic:
each upstream input was run through `creoles()` with `--features extended` and the
resulting AST checked against the expected HTML structure.

## Verdict

For markup this crate implements, most upstream behavior matches and was already tested.
The audit added the missing assertions (all green under `--features extended` and
`--features extend`; 12/12 tests pass):

| Added assert | File location | Upstream case proven |
| --- | --- | --- |
| unclosed `**` / `//` across a line stay literal | `text_style_tests` | official §"Character formatting" (bold must not span lines) |
| `//[[http://my.book.example/\|My Book Title]]//` → `Italic([Link])` | `link_tests` | italic link (README Strict `[x]`, upstream `//[[...]]//` test) — was claimed but untested |
| `***** 5` → 5-level nesting | `list_tests` | official §"up to five levels" (existing tests reached level 4) |
| inline image between text: `a {{i.png\|t}} b` | `image_tests` | upstream `testImagesWithAlt` inline usage |
| markup in table cells (`[[a]]`, `**b**`, `//y//`) + `\\` cell → `ForceLinebreak` | `table_tests` | upstream table-markup test + official §"Tables" `align\\ it.` — README claims it, untested before |
| `a\n----\nb` paragraph–hr–paragraph; inline + block nowiki | `other_tests` | official §hr, §nowiki; replaced commented-out `dont_format` test whose expectation (`Text("a\n")`) contradicts both the parser and upstream |

## Gaps: upstream-valid markup this crate misparses, untested either way

Repro = input; `A:` = current AST, `U:` = upstream expected. Root causes are line refs in
`src/parser.rs`. Not fixed here — each is a parser-behavior decision; the README either
doesn't claim the case or contradicts it.

1. **Heading requires a space after `=`.** `=Title=` → `Paragraph([Text("=Title=")])`;
   upstream → `Heading(1, ["Title"])`. `===Title===` likewise literal. A heading with a
   leading space (` ==T==`) also fails (`list` trims, `heading` doesn't).
   Root: `heading` has a hard `char(' ')` after the `=` run (`src/parser.rs:537`);
   `parse_creole_inner` (`src/parser.rs:739`) feeds it untrimmed lines.
   Upstream cases: `testHeadingsH1/H3`, `testHeadingsLeadingWhitespace`, reset input
   `=Title1=`. Official suite only uses space-prefixed headings, so the spec ambiguity is
   real but upstream (and every other impl) accepts `=Title=`.
2. **List item without space after marker fails.** `*One` / `#One` → literal paragraph
   text; upstream → list item. Worse, a mixed run (` #One\n # Two\n #Three\n# Four`)
   splits into paragraph/list/paragraph/list.
   Root: `list` requires `char(' ')` after the marker run (`src/parser.rs:183`).
   Upstream cases: `testOrderedListFunkyWhitespace`, reset input `*One\n**Two\n*Three`.
3. **Indented nested list at same marker-depth splits.** Official input
   `# i1\n# i2\n# i3\n    ## s1\n  ## s2` must produce ONE nested `ol` containing s1+s2;
   the 4-space → 2-space indent change breaks the child grouping into two sibling lists.
   Root: child lookahead uses raw untrimmed lines (`src/parser.rs:227`) while the sibling
   check trims (`src/parser.rs:213`).
4. **Whitespace after list marker is kept in the item text.** `  *   item a` →
   `Text("  item a")`; official expected is `<li>item a</li>`.
   Root: item body is `is_not("\n")` verbatim (`src/parser.rs:184`).
5. **Multi-line list item not supported.** `* You can have\nmultiline list items` →
   `BulletList([li])` + separate `Paragraph`; upstream keeps both lines in ONE `li`
   (official §"multiline list items").
6. **Vertical table headers leak `=`.** `|=Name|Irene|` then `|=Relationship|Wife|` —
   only the first row yields header cells; body-row header markers stay text
   (`TableCell([Text("=Relationship")])`). Upstream `testTableVerticalHeader` → th per
   row. `=` stripping exists only in the header-row path (`src/parser.rs:569-578` cell
   parsers take content verbatim).
7. **`**` does not contain links; `//` does.** `**[[documentation]]**` → bold text with
   literal `[[...]]`; upstream → `Bold([Link])`. Asymmetry is structural: italic's inner
   alt is `alt((bold, link))` (`src/parser.rs:152`) but bold's inner alt is
   `italic, text` only (`src/parser.rs:135-139`). Upstream cases: `testWikiLinkWithTextFormatting`,
   `testExternalLinkWithTextFormatting`. README Extended claims "link/image/bold/italic in
   heading/list/table" but not link-in-bold, so this is an unclaimed gap, not a
   regression.
8. **`//` inside bare URLs starts italic.** `http://a and ftp://b` →
   `Text("http:") + Italic(["a and ftp:"]) + Text(...)`. Upstream never italicizes
   (official §"http://bar … should not render italic"). Root: no URL guard before the
   `lit` → `text_style` alternative (`src/parser.rs:304`). Independent of the unclaimed
   free-link feature: any paragraph mentioning two URLs gets mangled.

## Documented deviations (intentional, no action)

- **Nested list AST shape**: nested `BulletList` is a sibling element inside the outer
  list's vector, not wrapped inside the preceding `ListItem` (upstream HTML nests
  `<ul>` inside `<li>`). A conforming HTML renderer must reconstruct the wrap;
  `list_tests` deliberately pins the current shape.
- **Formatting inside headings parses** (README Extended `[x]`); upstream leaves
  `**`/`//` literal in headings. `==T **b**?==` without space-after-`=` additionally hits
  gap 1, so today it renders as a paragraph with bold.
- **Free links / tilde escapes**: README Strict `[ ]`; upstream renders both. Tilde input
  additionally trips gap 8 (`~http://x` → italic) — expected while escapes are unclaimed.
- **Interwiki links, strikethrough `--x--`, underline `__x__`, sub `,,x,,`, sup `^^x^^`,
  monospace `##x##`, free-standing URLs, block macros, `urlBase`/`imgBase` href
  rewriting**: out of scope; all upstream inputs using them parse as literal text here.
- **Whitespace details**: text after a forced break keeps the consumed `\n`
  (`Text("\nPone: xyz")`), a paragraph after a block nowiki may start with `\n`, trailing
  `\n` after `}}}` yields an empty `Paragraph([])`, and empty input yields
  `Paragraph([])` vs upstream's empty output. Harmless to HTML renderers; affects strict
  string-diff only.
- **Block-position image** is wrapped `Paragraph([Image])`; upstream emits `<img>`
  un-wrapped. Renderer-level concern.

## Upstream expectations contradicting each other

`WikiCreoleTest.php::testMultilineBold/testMultilineItalic` assert bold/italic DO span a
line break, while the official `CreoleTestExpected.html` asserts the same constructs stay
literal. This crate follows the official expectation (README Strict `[ ]`
"italic/bold extend across line"); the new unclosed-marker asserts pin the official side.
