# Repository Guidelines

## Project Overview

`creole-nom` (v1.0.9, MIT, author `chidea`) is a Rust library that parses WikiCreole 1.0
wiki markup into a zero-copy AST using **nom 8**. Only three items form the public API:
`creoles`, `try_creoles`, `ICreole` (re-exported via `creole_nom::prelude`).
The README is a checkbox support matrix against the
[WikiCreole 1.0 Test Cases](http://www.wikicreole.org/wiki/Creole1.0TestCases) — no install,
usage, or contributor examples exist anywhere (zero code blocks); the only usage examples in
the tree are test assertions. Still unchecked: tilde escape, free link, italic/bold across
line, interwiki link, link-in-image-title, image-in-link-title, text colors. The "Extended"
section is not mapped to Cargo features ("text colors" unchecked, yet `font-color` exists);
companion project `chidea/wasm-creole-live-editor` appears only in test fixtures
(`src/parser.rs:1108-1114`).

## Architecture & Data Flow

```
&str ──try_creoles──▶ Vec<ICreole<'_>>      one element per '\n'-separated block
        separated_list0(char('\n'), alt((parse_creole_inner, parse_paragraph)))
        block parsers (heading|table|list|hrule|dont_format|…) tried before paragraph fallback
creoles = infallible wrapper: Ok((_, v)) => v, Err => vec![]   (errors are swallowed!)
```

- `src/creole.rs` — AST only: `ICreole<'a>` enum holding `&'a str` slices (no allocation),
  derives `Debug, Clone, Serialize, Deserialize, PartialEq`, manual `impl Eq`.
- `src/parser.rs` — all parsing + all tests (lines 1–777 parsers, 780–1333 `mod tests`).
- `src/strings.rs` — **orphaned: no `mod strings;` anywhere, never compiled.** Small-string
  `CowStr`/`InlineStr` types, referenced by nothing; its serde impls even gate on a
  `feature = "serde"` that Cargo.toml never declares.
- `src/main.rs` — scratch nom demo (`tag("hello")`); does not use the crate. Not a CLI.
- `src/lexer.rs` — `logos` tokenizer experiment (`Tok`, `lex_vec`, `lex_count`) for the
  lexer-vs-nom benchmark; zero-alloc token slices, not wired into `creoles`.
- Feature-gated AST variants + parsers: `extend` (`<<Icon(x)>>` → `Extend`), `font-color`
  (`Color`), `fold` (`Fold`), `link-button` (`[{…}]` → `LinkButton`); `extended` = all four.
  Caveat: `Color`'s parser is fully commented out (`src/parser.rs:515-531`) — constructible
  but never produced; the `icon` feature has zero live consumers.
- Targets are autodiscovered: lib `creole_nom` **and** bin `creole-nom` (`src/main.rs`);
  Cargo.toml declares `rust-version = "1.85"`; no `[lib]`/`[[bin]]`/`[lints]` sections.

## Key Directories

- `src/` — 6 files, flat; no `tests/`, `examples/`, `benches/`, no CI, no `unsafe`.
- Root scripts `test.sh`, `watch_test.sh`, `debug_test.sh`, `t.ps1` are test wrappers,
  excluded from the published crate (`exclude = ["*.sh", "*.ps1", "*.log"]` in Cargo.toml).

## Development Commands

```bash
cargo check                      # passes for every feature set, default included
cargo test  --features extended -- --nocapture   # what the author runs (t.ps1)
./test.sh                         # clear && cargo test -- --nocapture
./debug_test.sh                   # RUST_LOG=debug variant (env_logger, test-only)
./watch_test.sh                   # needs `cargo-watch`
t.ps1 [filter]                    # PowerShell: cargo test --features extended -j 1 <filter> -- --nocapture
```

**Baseline repaired 2026-11-08** (before: no feature set built without `extend`, and
`cargo test` never compiled anywhere):

- `src/parser.rs:259-262` — `extend` now cfg-paired inside `link`, mirroring the house
  pattern in `link_button` (:502-505); `take_dlit_text_until_peek_char0` (:326-331) gained
  the missing `not(extend) + link-button` arm that caused the `E0061`.
- The 14 stale `Link(href, text: &str)` test assertions now use the AST shape
  `Link(href, vec![…])`.
- `extend_tests` is gated `#[cfg(feature = "extend")]`; `other_tests`' empty-input assert
  matches real behavior — `try_creoles("") == Ok(("", vec![Paragraph(vec![])]))`, the same
  empty-line convention `mixed_tests` asserts for blank lines.

Green matrix: `cargo test` 11 passed; `--features extend`/`extended` 12 passed; each of
`link-button`, `font-color`, `fold` 11; `cargo check` clean for all seven combinations.

## Code Conventions & Common Patterns

- nom 8 **function form** only (no macros): plain `fn bold(input: &str) -> IResult<&str, ICreole<'_>>`,
  invoked through the `Parser` trait (`.parse(input)`); `nom::IResult` imported directly, never aliased.
- Custom combinator *factories* return `impl Parser<&'a str, Output = …, Error = nom::error::Error<&'a str>>`,
  generic over `E: ParseError<&'a str>`: `take_while_parser_fail`, `collect_opt_pair0/1`,
  `collect_while_parser_fail0`, `take_dlit_text_until0`. They scan char boundaries, accumulate
  skipped prefix as `ICreole::Text` — this is how mixed inline content (`**a**b`) is built.
- Error handling: pure nom `Err`, zero `unwrap`/`panic` outside tests; two spellings —
  struct literal `nom::Err::Error(nom::error::Error { input, code: ErrorKind::Tag })` and
  trait form `ParseError::from_error_kind(input, ErrorKind::Tag)`; only `Tag`/`Eof` kinds.
- `#[cfg(feature = "…")]` is applied both to whole fns and *inside* `alt((…))` tuples,
  always paired with `#[cfg(not(feature = "…"))]` / `all(...)` arms so **every feature
  combination keeps exactly one active arm** (e.g. `src/parser.rs:326-331`; a missing
  `not(extend)+link-button` arm was the `E0061` fixed 2026-11-08) — keep the matrix
  complete when adding feature variants.
- Naming: bare-noun parser fns (`bold`, `link`, `heading`); `try_` prefix = IResult-returning
  twin of an infallible fn; `parse_` = block-level; `take_`/`collect_` = combinator factories;
  `*_inner` = cell/line bodies; `lit`/`dlit` = inline-text parsers (dlit is dont_format-aware);
  leading `_` for private helpers (`_list` builds nested lists recursively on `Vec<&str>` lines).
- Logging: `#[cfg(debug_assertions)]`-gated `log` import; all `debug!` call sites are
  currently commented out.
- The tree keeps large commented-out blocks as a menu of tried alternatives (e.g.
  `src/parser.rs:82-121` tilde-escape sketch) — read them, they document abandoned designs.

## Important Files

- `Cargo.toml` — features (`extended` = `link-button,font-color,fold,extend`), deps:
  `nom = "8"`, `log = "0"`, `serde = { version = "1", features = ["derive"] }`;
  dev-dep `env_logger = "0"` (resolved: nom 8.0.0, log 0.4.25, serde 1.0.217, env_logger 0.11.6;
  the on-disk `Cargo.lock` is v4 format, i.e. written by Cargo ≥ 1.78).
- `src/lib.rs` — module wiring + `prelude`; add new public surface here.
- `src/parser.rs` — every parser, entry points `try_creoles`/`creoles` (:762/:771), inline tests.
- `src/creole.rs` — AST; a new markup node = variant here + parser fn + feature cfg if optional.
- `.gitignore` ignores `Cargo.lock` (library convention); tree is a git checkout since
  2026-11-08 (`4dd0019` green baseline, `3c770f8` logos experiment).

## Runtime/Tooling Preferences

- Rust **edition 2024** (migrated from 2021 on 2026-11-08; `cargo fix --edition` changed
  no source — already 2024-clean; MSRV `rust-version = "1.85"`, verified on rustc 1.97.1);
  plain stable `cargo`. No rustfmt.toml, no clippy config, no CI.
- Tests assume `--features extended` (t.ps1), but **every feature set compiles and passes**
  (verified 2026-11-08) — keep all cfg combinations green when adding markup.
- Crate publishes to the same name `creole-nom`; scripts are excluded from the package.

## Testing & QA

- Inline `#[cfg(test)] mod tests` in `src/parser.rs` (12 active `#[test]`s under
  `--features extend`/`extended`, 11 without — `extend_tests` is `#[cfg(feature = "extend")]`;
  list: `text_tests`, `text_style_tests`, `linebreak_tests`, `list_tests`, `parser_tests`,
  `heading_tests`, `link_tests`, `table_tests`, `image_tests`, `extend_tests`,
  `other_tests`, `mixed_tests`; one commented-out) and `src/strings.rs`
  (module `test_special_string`, 11 `#[test]`s —
  never compiled, the module is orphaned). Tests reach **private** combinators (`link`,
  `heading`, `lit`, `take_while_parser_fail`) via `use super::*;` — no `pub` needed to
  test a new parser fn.
- Test idiom — start with `init(); use ICreole::*;` (`init()` = `env_logger::Builder::from_env(
  Env::default().default_filter_or("debug")).is_test(true).try_init()`), then assert on full AST:

  ```rust
  #[test]
  fn text_tests() {
      init();
      use ICreole::*;
      assert_eq!(creoles("ab1"), vec![Paragraph(vec![Text("ab1")])]);
  }
  // verbatim, src/parser.rs:786-791
  ```

- Assertion shapes: `Ok(("", vec![…]))` / `Ok(("", ICreole…))` from `try_creoles` or direct
  combinator calls (remainder asserted as `""`), bare `vec![…]` from `creoles`. Suites are
  batched per concept (`list_tests` spans src/parser.rs:857-1031, ~13 `assert_eq!`s);
  fixtures are inline literals incl. multi-line and CJK/emoji — keep Unicode paths covered.
  Run one concept: `cargo test --features extended linebreak -- --nocapture` or `t.ps1 heading_tests`.
- No integration tests exist; add `tests/*.rs` if a public-API regression test is needed.
- `parser::tests::time_profile` (kept in `src/parser.rs`, `#[ignore]`d — normal suite stays
  fast): calls every test fn 20k× with per-iteration timing. Invoke
  `cargo test --features extended -- --ignored --nocapture time_profile` (add `--release`
  for release numbers — add `--exact` to run one profile fn alone); last run captured to
  `creole-nom-profile.log` (repo root, 2026-11-08).
- `parser::tests::time_profile_logos` (also `#[ignore]`d): identical fixture strings through
  the `logos` lexer — `lex vec` = tokenize into `Vec<Tok>`, `lex scan` = DFA pass without
  materializing, `nom` = `creoles()` full parse. Run with `--exact` (bare `time_profile`
  substring also matches this fn and double-prints into one log).
