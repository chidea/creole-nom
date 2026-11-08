//! Logos-based tokenizer experiment: tokenizes WikiCreole markup into a flat
//! token stream; the inline scanner `next_inline_stop` is wired into
//! `parser`'s text collectors (see `parser::tests::time_profile::time_profile_logos`
//! for the standalone benchmark comparison).
//!
//! Marker runs (`=+`, `#+`, `\*+`, `/+`) are lexed without line context; a
//! real pipeline would disambiguate heading/list markers by line position.
//! Inline markup opens (`[[`, `{{`, `[{`, `<<`, `\\`, `**`, `//`, `~`) are token
//! starts, which is what `next_inline_stop` uses to fast-forward inline text
//! scans (wired into `parser`'s text collectors).
//! Characters the stream cannot match (stray `[`, single `\`, ...) surface as
//! `Tok::Text` via the error slice.

use logos::Logos;

#[derive(Debug, Clone, PartialEq, Logos)]
pub enum Tok<'a> {
    #[token("[[")]
    LinkOpen,
    #[token("]]")]
    LinkClose,
    /// free-link button open `[{` (link-button feature)
    #[token("[{")]
    LinkBtnOpen,
    /// extend open `<<` (extend feature)
    #[token("<<")]
    LtLt,
    #[token("{{")]
    ImgOpen,
    #[token("}}")]
    ImgClose,
    /// forced linebreak `\\`
    #[token("\\\\")]
    ForcedBreak,
    /// cell / free-link separator
    #[token("|")]
    Pipe,
    /// heading delimiters and horizontal rules (line position decides)
    #[regex("=+", |lex| lex.slice())]
    EqRun(&'a str),
    /// list markers or monospace delimiters
    #[regex(r"#+", |lex| lex.slice())]
    HashRun(&'a str),
    #[regex(r"\*+", |lex| lex.slice())]
    StarRun(&'a str),
    /// italic delimiters or URL scheme colon-slashes
    #[regex(r"/+", |lex| lex.slice())]
    SlashRun(&'a str),
    #[token("\n")]
    Newline,
    #[regex(r"[^\[\]\{\}\|\*\\\/\=#<>\n~]+", |lex| lex.slice())]
    Text(&'a str),
}

/// Full tokenization into a vector (what a token-based pipeline would hold).
pub fn lex_vec(input: &str) -> Vec<Tok<'_>> {
    let mut lexer = Tok::lexer(input);
    let mut out = Vec::new();
    loop {
        match lexer.next() {
            Some(Ok(t)) => out.push(t),
            Some(Err(_)) => out.push(Tok::Text(lexer.slice())),
            None => break,
        }
    }
    out
}

/// Pure scan cost: run the DFA to end without materializing anything.
pub fn lex_count(input: &str) -> usize {
    let mut lexer = Tok::lexer(input);
    let mut n = 0;
    while lexer.next().is_some() {
        n += 1;
    }
    n
}

/// Offset of the next position any parser passed to the text collectors can
/// start at.
///
/// The text collectors used to re-attempt their parser at every char
/// boundary. `Tok::Text` is the only token whose first character can open
/// nothing (its regex excludes `[ ] { } | * \ / = # ~` and `\n`; `~` opens
/// the tilde escape), so every non-`Text` token start —
/// including error slices — is a superset of the positions where the inline
/// parsers, block parsers (`=`, `#`, `*` line heads) and terminators
/// (`**`, `//`, `|`, `\n`) can ever match; prose in particular is skipped
/// one whole token at a time. Positions needed after a failed candidate
/// (markup nested inside markup starts, e.g. `{{{{y}}`) stay reachable
/// because callers resume at candidate + 1.
pub fn next_inline_stop(input: &str, from: usize) -> Option<usize> {
    let mut lexer = Tok::lexer(&input[from..]);
    while let Some(tok) = lexer.next() {
        let start = from + lexer.span().start;
        if is_scan_stop(tok) {
            return Some(start);
        }
    }
    None
}

fn is_scan_stop(tok: Result<Tok<'_>, ()>) -> bool {
    match tok {
        Ok(Tok::Text(_)) => false,
        // markup token, or stray single marker chars (`{`, `[`, ...) that
        // surface as error slices: a parser may start there
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Tok::*;

    #[test]
    fn lexer_smoke() {
        assert_eq!(lex_vec("= H ="), vec![EqRun("="), Text(" H "), EqRun("=")]);
        assert_eq!(
            lex_vec("* a\n**[[l|t]]\\\\x"),
            vec![
                StarRun("*"),
                Text(" a"),
                Newline,
                StarRun("**"),
                LinkOpen,
                Text("l"),
                Pipe,
                Text("t"),
                LinkClose,
                ForcedBreak,
                Text("x")
            ]
        );
        // stray single marker chars survive as Text, scan never fails
        assert_eq!(lex_count(r"[a{b}c\d"), 8);
    }

    #[test]
    fn inline_stops() {
        // long prose run: nothing worth retrying until the marker
        assert_eq!(next_inline_stop("plain words only", 0), None);
        assert_eq!(next_inline_stop("plain **b**", 0), Some(6));
        // list heads / heading markers are stop candidates too (block parsers)
        assert_eq!(next_inline_stop("3 * 4 = 12", 0), Some(2));
        // newline and pipe are terminator stops
        assert_eq!(next_inline_stop("a\nb", 0), Some(1));
        assert_eq!(next_inline_stop("a|b", 0), Some(1));
        // nested inside an opener: resume at candidate + 1 exposes it
        assert_eq!(next_inline_stop("{{{", 0), Some(0));
        assert_eq!(next_inline_stop("{{{", 2), Some(2)); // Err slice: one redundant probe, then the +1 guard ends it
        assert_eq!(next_inline_stop("{{{{y}}", 1), Some(1));
        assert_eq!(next_inline_stop("{{{{y}}", 2), Some(2));
        // feature markers: link-button opens on Err slices, extend on the
        // excluded angle brackets, forced break on its token
        assert_eq!(next_inline_stop("a[{x}]b", 0), Some(1));
        assert_eq!(next_inline_stop("a<<x>>b", 0), Some(1));
        assert_eq!(next_inline_stop("a\\\\b", 0), Some(1));
        assert_eq!(next_inline_stop("a~**b", 0), Some(1));
    }
}
