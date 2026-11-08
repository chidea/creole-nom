//! Logos-based tokenizer experiment: tokenizes WikiCreole markup into a flat
//! token stream. Benchmark comparison against the nom pipeline (see
//! `parser::tests::time_profile::time_profile_logos`); NOT wired into
//! `creoles`/`try_creoles` output.
//!
//! Marker runs (`=+`, `#+`, `\*+`, `/+`) are lexed without line context; a
//! real pipeline would disambiguate heading/list markers by line position.
//! Characters the stream cannot match (stray `[`, single `\`, ...) surface as
//! `Tok::Text` via the error slice.

use logos::Logos;

#[derive(Debug, Clone, PartialEq, Logos)]
pub enum Tok<'a> {
    #[token("[[")]
    LinkOpen,
    #[token("]]")]
    LinkClose,
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
    #[regex(r"[^\[\]\{\}\|\*\\\/\=#\n]+", |lex| lex.slice())]
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
}
