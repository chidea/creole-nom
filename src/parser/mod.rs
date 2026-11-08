//! The public parse API. Submodules split by concern; their items are
//! `pub(crate)`-glob-re-exported here, and the crate's two entry points are
//! defined at the bottom of this file.

mod block;
mod combinators;
mod inline;
#[cfg(test)]
mod tests;
pub(crate) use block::*;
pub(crate) use combinators::*;
pub(crate) use inline::*;

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

/// Parse a whole document: one node per `\n`-separated block (blank lines
/// become empty `Paragraph`s); block parsers are tried before the paragraph
/// fallback.
pub fn try_creoles(input: &str) -> IResult<&str, Vec<ICreole<'_>>> {
    separated_list0(char('\n'), alt((parse_creole_inner, parse_paragraph))).parse(input)
}

/// Infallible [`try_creoles`]: nodes on success, an empty `Vec` on any error.
pub fn creoles(input: &str) -> Vec<ICreole<'_>> {
    if let Ok((_, v)) = try_creoles(input) {
        v
    } else {
        vec![]
    }
}
