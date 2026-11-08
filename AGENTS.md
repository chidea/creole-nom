# Repository Guidelines

## Project Overview

`creole-nom` (v1.0.9, MIT, author `chidea`) is a Rust library that parses WikiCreole 1.0
wiki markup into a zero-copy AST using **nom 8**. Only three items form the public API:
`creoles`, `try_creoles`, `ICreole` (re-exported via `creole_nom::prelude`).
The README is a checkbox support matrix against the
[WikiCreole 1.0 Test Cases](http://www.wikicreole.org/wiki/Creole1.0TestCases) — no install,
usage, or contributor examples exist anywhere (zero code blocks); the only usage examples in
the tree are test assertions. Still unchecked: free link, interwiki link, link-in-image-
title, image-in-link-title, text colors (tilde escape + across-line emphasis landed 2026-11-08).
section is not mapped to Cargo features ("text colors" unchecked, yet `font-color` exists);
companion project `chidea/wasm-creole-live-editor` appears only in test fixtures
(`src/parser.rs:1194`).

## Architecture & Data Flow

```
&str ──try_creoles──▶ Vec<ICreole<'_>>      one element per '\n'-separated block
        separated_list0(char('\n'), alt((parse_creole_inner, parse_paragraph)))
        block parsers (heading|table|list|hrule|dont_format|…) tried before paragraph fallback
creoles = infallible wrapper: Ok((_, v)) => v, Err => vec![]   (errors are swallowed!)
```
        block parsers (heading|table|list|hrule|dont_format|…) tried before paragraph fallback
        list() is line-level: deeper marker nests, blank line closes the run; paragraphs span
        lines until a blank one (para_break); inline text scans ride the lexer DFA
- Inline behavior (Batch B, 2026-11-08): emphasis crosses newlines (terminator sets reach
  `para_break` = blank line / inline delimiter / line end), heading closing `=`-run of any
  length, `tilde_escape` renders `~` escapes (space, `~`, markup opens, `|` `#` `=` `-`),
  `split_cells` + `first_unescaped_pipe` give `~|` the tilde rule, and `link`/`image`
  render names through `tilde_escape`.
- `src/creole.rs` — AST only: `ICreole<'a>` enum holding `&'a str` slices (no allocation),
  derives `Debug, Clone, Serialize, Deserialize, PartialEq`, manual `impl Eq`.
- `src/parser.rs` — all parsing + all inline tests: parsers :1-807 (public entry points at the
  end: `try_creoles` :809, `creoles` :822), `mod tests` from :830 (1703 lines total).
- `src/main.rs` — scratch nom demo (`tag("hello")`); does not use the crate. Not a CLI.
- `src/lexer.rs` — `logos` tokenizer (`Tok`, `lex_vec`, `lex_count` from the lexer-vs-nom
  bench) plus `next_inline_stop`/`is_scan_stop`, which now **drive** the inline text
  collectors in `parser.rs` (DFA jump between markup candidates; `lit`/`dlit` semantics
  unchanged). `Text` regex excludes `<`/`>`/`~` so `<<extend>>` opens and tilde escapes are stops.
- Feature-gated AST variants + parsers: `extend` (`<<Icon(x)>>` → `Extend`), `font-color`
  (`Color`), `fold` (`Fold`), `link-button` (`[{…}]` → `LinkButton`); `extended` = all four.
  Caveat: `Color`'s parser is fully commented out — constructible but never produced;
  decided 2026-11-08: implement later as `[[color:red|text]]`, designed in one pass with
  free link so all `[[…]]` sub-syntaxes land together. The `icon` feature has zero live consumers.
- Targets are autodiscovered: lib `creole_nom` **and** bin `creole-nom` (`src/main.rs`);
  Cargo.toml declares `rust-version = "1.85"` and a `[lints]` section (see Important Files).

## Key Directories

- `src/` — 5 files, flat (`lib.rs`, `creole.rs`, `parser.rs`, `main.rs`, `lexer.rs`); no
  `benches/`, no `unsafe` (`[lints.rust] unsafe_code = "forbid"`).
- `examples/creole_test_input.rs` + `examples/CreoleTestInput.txt` — upstream WikiCreole 1.0
  corpus smoke runner: `cargo run --release --example creole_test_input` must print
  `37 blocks … 0 empty parses` and exit 0.
- `tests/public_api.rs` — integration suite over the published surface only (see Testing & QA).
- `.github/workflows/ci.yml` — CI: per-feature `cargo test` matrix, `cargo hack` powerset
  check, `cargo fmt --check` + `cargo clippy -- -D warnings`.

## Development Commands

```bash
cargo check                      # passes for every feature set, default included
cargo test --features extended   # what the author runs: 13 passed + 2 ignored (profiles)
cargo test --features extended -- --nocapture
cargo test --features extended --test public_api   # public-surface suite only
cargo fmt --all && cargo clippy --all-targets --features extended   # both CI-enforced clean
cargo hack --feature-powerset --depth 2 check      # full feature matrix (needs cargo-hack)
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
- Error handling: pure nom `Err`, zero `unwrap`/`panic` outside tests; construct through the
  helpers `err_tag` (`src/parser.rs:91`) / `err_eof` (`:95`), generic over
  `E: ParseError<&'a str>`; only `Tag`/`Eof` kinds.
- `#[cfg(feature = "…")]` is applied both to whole fns and *inside* `alt((…))` tuples,
  always paired with `#[cfg(not(feature = "…"))]` / `all(...)` arms so **every feature
  combination keeps exactly one active arm** (the cfg arms in `link`, `link_button`,
  `take_dlit_text_until_peek_char0`; a missing `not(extend)+link-button` arm was the
  `E0061` fixed 2026-11-08) — keep the matrix complete when adding feature variants.
- Naming: bare-noun parser fns (`bold`, `link`, `heading`); `try_` prefix = IResult-returning
  twin of an infallible fn; `parse_` = block-level; `take_`/`collect_` = combinator factories;
  `*_inner` = cell/line bodies; `lit`/`dlit` = inline-text parsers (dlit is dont_format-aware);
  leading `_` for private helpers (`_list(head_tag)` nests over `Vec<&str>` lines, infallible).
- Logging: `#[cfg(debug_assertions)]`-gated `log` import; all `debug!` call sites are
  currently commented out.
- The tree keeps large commented-out blocks as a menu of tried alternatives (e.g. the
  disabled `Color` parser, `src/parser.rs:526-531`) — read them, they document abandoned designs.

## Important Files

- `Cargo.toml` — features (`extended` = `link-button,font-color,fold,extend`), deps:
  `nom = "8"`, `log = "0"`, `serde = { version = "1", features = ["derive"] }`,
  `logos = "0.16.1"` (benchmark-only); dev-dep `env_logger = "0"`; `[lints]` sets
  `unsafe_code = "forbid"` + clippy `all = "warn"`. Resolved: nom 8.0.0, log 0.4.25,
  serde 1.0.217, env_logger 0.11.6; on-disk `Cargo.lock` is v4 (Cargo ≥ 1.78).
- `src/lib.rs` — module wiring + `prelude`; add new public surface here.
- `src/parser.rs` — every parser, entry points `try_creoles`/`creoles` (:809/:822), inline tests.
- `src/creole.rs` — AST; a new markup node = variant here + parser fn + feature cfg if optional.
- `.gitignore` ignores `Cargo.lock` (library convention); tree is a git checkout since
  2026-11-08 (`4dd0019` green baseline, `3c770f8` logos experiment, `cf559f7` cleanup).

## Runtime/Tooling Preferences

- Rust **edition 2024** (migrated from 2021 on 2026-11-08; `cargo fix --edition` changed
  no source — already 2024-clean; MSRV `rust-version = "1.85"`, verified on rustc 1.97.1);
  plain stable `cargo`; source is rustfmt-formatted since 2026-11-08 (CI pins it via
  `cargo fmt --check`), clippy-clean (`-D warnings` in CI). CI workflow exists now.
- Tests assume `--features extended`, but **every feature set compiles and passes**
  (CI matrix + powerset check enforce it) — keep all cfg combinations green when adding markup.
- Crate publishes to the same name `creole-nom`; root test wrappers (`test.sh`, `t.ps1`,
  `watch_test.sh`, `debug_test.sh`) were deleted 2026-11-08, `exclude = ["*.sh", "*.ps1", "*.log"]`
  remains in Cargo.toml for stray artifacts.

## Testing & QA

- Inline `#[cfg(test)] mod tests` in `src/parser.rs:830-1703` (15 active `#[test]`s under
  `--features extend`/`extended`, 14 without — `extend_tests` is `#[cfg(feature = "extend")]`;
  list: `text_tests`, `text_style_tests`, `linebreak_tests`, `list_tests`, `parser_tests`,
  `heading_tests`, `link_tests`, `table_tests`, `image_tests`, `extend_tests`, `escape_tests`,
  `other_tests`, `mixed_tests`; one commented-out) plus 2 tests in `src/lexer.rs`
  (`lexer_smoke`, `inline_stops` — the latter pins jump-candidate semantics).
  Tests reach **private** combinators (`link`,
  `heading`, `lit`, `take_while_parser_fail`) via `use super::*;` — no `pub` needed to
  test a new parser fn.
- `tests/public_api.rs` — 10 integration tests reaching only `creole_nom::prelude`
  (`creoles`/`try_creoles`/`ICreole`), mirroring the README support matrix incl. the
  escape cases and the space-less-bold-list convention. New public-API regressions go here.
- Test idiom — start with `init(); use ICreole::*;` (`init()` = `env_logger::Builder::from_env(
  Env::default().default_filter_or("debug")).is_test(true).try_init()`), then assert on full AST:

  ```rust
  #[test]
  fn text_tests() {
      init();
      use ICreole::*;
      assert_eq!(creoles("ab1"), vec![Paragraph(vec![Text("ab1")])]);
  }
  // verbatim, src/parser.rs:772-776
  ```

- Assertion shapes: `Ok(("", vec![…]))` / `Ok(("", ICreole…))` from `try_creoles` or direct
  combinator calls (remainder asserted as `""`), bare `vec![…]` from `creoles`. Suites are
  batched per concept (`list_tests` spans src/parser.rs:851-1033, ~13 `assert_eq!`s);
  fixtures are inline literals incl. multi-line and CJK/emoji — keep Unicode paths covered.
  Run one concept: `cargo test --features extended linebreak -- --nocapture`.
- `parser::tests::time_profile` (kept in `src/parser.rs`, `#[ignore]`d — normal suite stays
  fast): calls every test fn 20k× with per-iteration timing. Invoke
  `cargo test --features extended -- --ignored --nocapture time_profile` (add `--release`
  for release numbers — add `--exact` to run one profile fn alone); last run captured to
  `creole-nom-profile.log` (repo root, 2026-11-08).
- `parser::tests::time_profile_logos` (also `#[ignore]`d): identical fixture strings through
  the `logos` lexer — `lex vec` = tokenize into `Vec<Tok>`, `lex scan` = DFA pass without
  materializing, `nom` = `creoles()` full parse. Run with `--exact` (bare `time_profile`
  substring also matches this fn and double-prints into one log).
