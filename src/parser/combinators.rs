use super::*;
use crate::lexer;
#[cfg(any(feature = "extend", feature = "fold", feature = "link-button"))]
#[cfg(debug_assertions)]
#[allow(unused_imports)]
use log::{debug, error, info};
#[allow(unused_imports)]
use {
    crate::creole::ICreole,
    nom::{
        IResult, Parser,
        branch::alt,
        bytes::complete::{is_not, tag, take_until},
        character::complete::{char, line_ending, one_of, space0},
        combinator::{eof, map, peek, recognize, rest, success, value, verify},
        error::{ErrorKind, ParseError},
        multi::{many_m_n, many0, separated_list0, separated_list1},
        sequence::{delimited, pair, preceded, separated_pair, terminated},
    },
};

/// Uniform error construction: the only kinds produced by this parser are Tag and Eof.
pub(crate) fn err_tag<'a, E: ParseError<&'a str>>(input: &'a str) -> nom::Err<E> {
    nom::Err::Error(E::from_error_kind(input, ErrorKind::Tag))
}

pub(crate) fn err_eof<'a, E: ParseError<&'a str>>(input: &'a str) -> nom::Err<E> {
    nom::Err::Error(E::from_error_kind(input, ErrorKind::Eof))
}

/// Paragraph boundary that emphasis may not reach across: a blank line or a
/// block parser starting on the next line - the same terminators
/// `parse_paragraph` uses, so an unclosed `**` cannot eat past its own
/// paragraph into a later closer.
pub(crate) fn para_break<'a>()
-> impl Parser<&'a str, Output = bool, Error = nom::error::Error<&'a str>> {
    alt((
        value(true, peek(pair(char('\n'), char('\n')))),
        value(true, peek(preceded(char('\n'), parse_creole_inner))),
    ))
}

pub(crate) fn list_head_char(input: &str) -> IResult<&str, char> {
    one_of("*#").parse(input)
}
/// `head_tag` is the bare marker run (`*`, `##`) without the trailing space,
/// so membership needs no `format!("{} ")` allocation per recursion.
pub(crate) fn starts_with_marker(line: &str, head_tag: &str) -> bool {
    line.starts_with(head_tag) && !line[head_tag.len()..].starts_with(['*', '#'])
}

/// Byte index of the first `|` not preceded by an odd run of `~`: `~|` is
/// the escaped pipe (literal), `~~|` still separates.
pub(crate) fn first_unescaped_pipe(s: &str) -> Option<usize> {
    let mut tilde_run = 0usize;
    for (i, c) in s.char_indices() {
        if c == '~' {
            tilde_run += 1;
            continue;
        }
        let sep = c == '|' && tilde_run % 2 == 0;
        tilde_run = 0;
        if sep {
            return Some(i);
        }
    }
    None
}

pub(crate) fn lit(input: &str) -> IResult<&str, ICreole<'_>> {
    alt((tilde_escape, link, image, text_style)).parse(input)
}
pub(crate) fn dlit(input: &str) -> IResult<&str, ICreole<'_>> {
    alt((dont_format, tilde_escape, link, image, text_style)).parse(input)
}
pub(crate) fn take_dlit_text_until0<'a>() -> impl Parser<&'a str, Output = Vec<ICreole<'a>>> {
    collect_opt_pair0(take_while_parser_fail(dlit, text))
}
pub(crate) fn take_dlit_text_until_peek_char0<'a>(
    until_char: char,
) -> impl Parser<&'a str, Output = Vec<ICreole<'a>>, Error = nom::error::Error<&'a str>> {
    collect_opt_pair0(take_while_parser_fail_or(
        value(true, peek(char(until_char))),
        #[cfg(all(feature = "extend", feature = "link-button"))]
        alt((dlit, link_button, extend)),
        #[cfg(all(not(feature = "extend"), feature = "link-button"))]
        alt((dlit, link_button)),
        #[cfg(not(feature = "link-button"))]
        dlit,
        text,
    ))
}

pub(crate) fn take_while_parser_fail<'a, E: ParseError<&'a str>>(
    mut parser: impl Parser<&'a str, Output = ICreole<'a>, Error = E>,
    mut fail_parser: impl Parser<&'a str, Output = ICreole<'a>, Error = E>,
) -> impl Parser<&'a str, Output = (Option<ICreole<'a>>, Option<ICreole<'a>>), Error = E> {
    move |input: &'a str| {
        // lexer fast-forward: see take_while_parser_fail_or for the
        // token-boundary argument (lexer::next_inline_stop).
        let mut off = 0usize;
        while let Some(t) = lexer::next_inline_stop(input, off) {
            if input[t..].starts_with("[[[") {
                // `[[[` has no Creole meaning: skip the run so the inner
                // `[[` never parses as a link
                off = t + 2;
                continue;
            } else if let Ok((r, v)) = parser.parse(&input[t..]) {
                let prefix = if t > 0 {
                    fail_parser.parse(&input[..t]).ok().map(|(_, v)| v)
                } else {
                    None
                };
                return Ok((r, (prefix, Some(v))));
            }
            off = t + 1;
        }
        if let Ok((_, f)) = fail_parser.parse(input) {
            Ok(("", (Some(f), None)))
        } else if input.is_empty() {
            Err(err_eof(input))
        } else {
            Err(err_tag(input))
        }
    }
}

pub(crate) fn collect_opt_pair0<'a, E: ParseError<&'a str>>(
    parser: impl Parser<&'a str, Output = (Option<ICreole<'a>>, Option<ICreole<'a>>), Error = E>,
) -> impl Parser<&'a str, Output = Vec<ICreole<'a>>, Error = E> {
    alt((collect_opt_pair1(parser), success(vec![])))
}
/// Coalesce adjacent `Text` nodes whose content slices are physically
/// contiguous in `input` (raw runs split only at scan stops). A merged
/// slice then equals `prev.content ++ next.content` verbatim; escaped or
/// spaced-apart texts have a pointer gap and stay separate nodes.
pub(crate) fn merge_text<'a>(input: &'a str, v: Vec<ICreole<'a>>) -> Vec<ICreole<'a>> {
    let base = input.as_ptr() as usize;
    let mut out: Vec<ICreole<'a>> = Vec::with_capacity(v.len());
    for node in v {
        match (&node, out.last_mut()) {
            (ICreole::Text(t), Some(ICreole::Text(prev)))
                if !t.is_empty()
                    && !prev.is_empty()
                    && t.as_ptr() as usize == prev.as_ptr() as usize + prev.len() =>
            {
                let end = t.as_ptr() as usize - base + t.len();
                let start = end - prev.len() - t.len();
                *prev = &input[start..end];
            }
            _ => out.push(node),
        }
    }
    out
}

pub(crate) fn collect_opt_pair1<'a, E: ParseError<&'a str>>(
    mut parser: impl Parser<&'a str, Output = (Option<ICreole<'a>>, Option<ICreole<'a>>), Error = E>,
) -> impl Parser<&'a str, Output = Vec<ICreole<'a>>, Error = E> {
    move |input: &'a str| {
        let mut rst = vec![];
        let mut i = input;
        // debug!("collect_opt_pair input : {}", i);
        while let Ok((r, t)) = parser.parse(i) {
            // debug!("collect_opt_pair i : {}, r: {}", i, r);
            i = r;
            match t {
                (Some(a), Some(b)) => {
                    rst.push(a);
                    rst.push(b);
                }
                (None, Some(b)) => {
                    rst.push(b);
                }
                (Some(a), None) => {
                    rst.push(a);
                    break;
                }
                _ => {
                    break;
                }
            }
        }
        // debug!("collect_opt_pair rst : {:?}", rst);
        if rst.is_empty() {
            Err(err_eof(input))
        } else {
            Ok((i, merge_text(input, rst)))
        }
    }
}

pub(crate) fn take_while_parser_fail_or<'a, E: ParseError<&'a str>>(
    mut term_parser: impl Parser<&'a str, Output = bool, Error = E>,
    mut parser: impl Parser<&'a str, Output = ICreole<'a>>,
    mut fail_parser: impl Parser<&'a str, Output = ICreole<'a>, Error = E>,
) -> impl Parser<&'a str, Output = (Option<ICreole<'a>>, Option<ICreole<'a>>), Error = E> {
    move |input: &'a str| {
        // lexer fast-forward: markup/terminators only ever start at token
        // boundaries; failed candidates retry from candidate + 1 so nested
        // openers (`{{{{y}}`) stay reachable (see lexer::next_inline_stop).
        let mut off = 0usize;
        while let Some(t) = lexer::next_inline_stop(input, off) {
            if let Ok((i, true)) = term_parser.parse(&input[t..]) {
                return Ok((
                    i,
                    (fail_parser.parse(&input[..t]).ok().map(|(_, v)| v), None),
                ));
            } else if input[t..].starts_with("[[[") {
                // `[[[` has no Creole meaning; skip the run (see
                // take_while_parser_fail)
                off = t + 2;
                continue;
            } else if let Ok((r, v)) = parser.parse(&input[t..]) {
                return Ok((
                    r,
                    (fail_parser.parse(&input[..t]).ok().map(|(_, v)| v), Some(v)),
                ));
            } else {
                off = t + 1;
            }
        }
        if let Ok((_, f)) = fail_parser.parse(input) {
            Ok(("", (Some(f), None)))
        } else if input.is_empty() {
            Err(err_eof(input))
        } else {
            Err(err_tag(input))
        }
    }
}

// fn take_while_parser_fail_or_peek_tag<'a>(
//     term_tag: &'static str,
//     parser: impl FnMut(&'a str) -> IResult<&'a str, ICreole<'a>>,
//     fail_parser: impl FnMut(&'a str) -> IResult<&'a str, ICreole<'a>>,
// ) -> impl FnMut(&'a str) -> IResult<&'a str, (Option<ICreole<'a>>, Option<ICreole<'a>>)> {
//     take_while_parser_fail_or(value(true, peek(tag(term_tag))), parser, fail_parser)
// }

pub(crate) fn collect_while_parser_fail_or0<'a, E: ParseError<&'a str>>(
    term_parser: impl Parser<&'a str, Output = bool, Error = E>,
    parser: impl Parser<&'a str, Output = ICreole<'a>, Error = E>,
    fail_parser: impl Parser<&'a str, Output = ICreole<'a>, Error = E>,
) -> impl Parser<&'a str, Output = Vec<ICreole<'a>>, Error = E> {
    collect_opt_pair0(take_while_parser_fail_or(term_parser, parser, fail_parser))
}

pub(crate) fn collect_while_parser_fail0<'a, E: ParseError<&'a str>>(
    parser: impl Parser<&'a str, Output = ICreole<'a>, Error = E>,
    fail_parser: impl Parser<&'a str, Output = ICreole<'a>, Error = E>,
) -> impl Parser<&'a str, Output = Vec<ICreole<'a>>, Error = E> {
    collect_opt_pair0(take_while_parser_fail(parser, fail_parser))
}

/// Split a table row body into cells at `sep` boundaries that are not
/// escaped by an odd run of `~` (tilde escape: `| x~|y |` is one cell,
/// `~~|` separates). Escaped pipes stay in the raw cell slice; the cell
/// parser's `tilde_escape` renders them as a literal `|`.
pub(crate) fn split_cells<'a>(line: &'a str, sep: &str) -> Vec<&'a str> {
    let mut cells = Vec::new();
    let mut start = 0;
    let mut tilde_run = 0usize;
    for (i, c) in line.char_indices() {
        if c == '~' {
            tilde_run += 1;
            continue;
        }
        if line[i..].starts_with(sep) && tilde_run % 2 == 0 {
            cells.push(&line[start..i]);
            start = i + sep.len();
        }
        tilde_run = 0;
    }
    cells.push(&line[start..]);
    cells
}
