//! Inline-level parsers: constructs that may appear inside a paragraph, heading
//! or table cell. All `pub(crate)`, reached through the glob re-export in
//! `src/parser/mod.rs` by the sibling modules and the test suite.

use super::*;
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

/// Tilde escape, WikiCreole escape-character proposal
/// (http://www.wikicreole.org/wiki/EscapeCharacterProposal, spec §2.20):
/// `~` before markup yields that markup verbatim as text - `~**` -> `**`,
/// `~//` -> `//`, `~[[` `~]]` `~{{` `~}}` `~\\` `~----`, and for line-head
/// markup `~-` `~#` `~=` `~|` (escaping mid-line is harmless). `~~` and
/// `~<space>` escape the escape character itself -> a literal `~`. The
/// result borrows the escaped characters; no allocation.
pub(crate) fn tilde_escape(input: &str) -> IResult<&str, ICreole<'_>> {
    map(
        preceded(
            char('~'),
            alt((
                recognize(alt((char(' '), char('~')))),
                recognize(alt((
                    tag("**"),
                    tag("//"),
                    tag("[["),
                    tag("]]"),
                    tag("{{"),
                    tag("}}"),
                    tag("\\\\"),
                    tag("----"),
                    tag("-"),
                    tag("#"),
                    tag("="),
                    tag("|"),
                ))),
            )),
        ),
        ICreole::Text,
    )
    .parse(input)
}

/// `**strong**` → `Bold`, inline content up to a closing `**` run.
pub(crate) fn bold(input: &str) -> IResult<&str, ICreole<'_>> {
    map(
        delimited(
            tag("**"),
            collect_while_parser_fail_or0(
                alt((value(true, peek(tag("**"))), para_break())),
                alt((
                    tilde_escape,
                    italic,
                    link,
                    image,
                    value(ICreole::ForceLinebreak, tag("\\\\")),
                )),
                text,
            ),
            tag("**"),
        ),
        ICreole::Bold,
    )
    .parse(input)
}
/// `//emphasis//` → `Italic`, inline content up to a closing `//` run.
pub(crate) fn italic(input: &str) -> IResult<&str, ICreole<'_>> {
    map(
        delimited(
            tag("//"),
            collect_while_parser_fail_or0(
                alt((value(true, peek(tag("//"))), para_break())),
                alt((
                    tilde_escape,
                    bold,
                    link,
                    image,
                    value(ICreole::ForceLinebreak, tag("\\\\")),
                )),
                text,
            ),
            tag("//"),
        ),
        ICreole::Italic,
    )
    .parse(input)
}

/// Inline style dispatch: `bold` then `italic`.
pub(crate) fn text_style(input: &str) -> IResult<&str, ICreole<'_>> {
    alt((
        value(ICreole::ForceLinebreak, tag("\\\\")),
        bold,
        italic,
        #[cfg(feature = "tex")]
        inline_tex,
        // #[cfg(feature = "font-color")]
        // font_color,
    ))
    .parse(input)
}

/// `[[href]]` or `[[href|label]]` → `Link` with inline-parsed label.
pub(crate) fn link(input: &str) -> IResult<&str, ICreole<'_>> {
    // `[[[x]]]` carries no Creole meaning: rejecting a `[`-prefixed href
    // lets the leading bracket fall through as text and `[[x]]` parse.
    let (left, content) = delimited(
        tag("[["),
        verify(is_not("]\n"), |s: &str| !s.starts_with('[')),
        tag("]]"),
    )
    .parse(input)?;
    // `~|` inside a link is an escaped pipe: only an unescaped pipe splits
    // href from name, so `[[A pipe ~| within links]]` is one free link
    let (href, name) = match first_unescaped_pipe(content) {
        Some(i) => (&content[..i], &content[i + 1..]),
        None => (content, content),
    };

    #[cfg(feature = "extend")]
    let (_, v) = many0(alt((
        extend,
        tilde_escape,
        map(is_not("<~"), ICreole::Text),
    )))
    .parse(name)?;
    #[cfg(not(feature = "extend"))]
    let (_, v) = many0(alt((tilde_escape, map(is_not("<~"), ICreole::Text)))).parse(name)?;
    Ok((left, ICreole::Link(href, v)))
}
/// `{{href|alt}}` → `Image(href, alt)`; the title is kept verbatim (Creole
/// forbids markup inside image titles).
pub(crate) fn image(input: &str) -> IResult<&str, ICreole<'_>> {
    let (left, content) = delimited(tag("{{"), is_not("{}\n"), tag("}}")).parse(input)?;
    // same tilde rule as `link`: `~|` does not separate src from label
    let (src, label) = match first_unescaped_pipe(content) {
        Some(i) => (&content[..i], &content[i + 1..]),
        None => (content, ""),
    };
    Ok((left, ICreole::Image(src, label)))
}

/// `<<Name(text)>>` extension macro → `Extend(name, text)`. `extend`.
#[cfg(feature = "extend")]
pub(crate) fn extend(input: &str) -> IResult<&str, ICreole<'_>> {
    map(
        delimited(
            tag("<<"),
            separated_pair(is_not("<>(\n"), tag("("), is_not("<>)\n")),
            tag(")>>"),
        ),
        |(typ, val)| ICreole::Extend(typ, val),
    )
    .parse(input)
}
// #[cfg(feature="extend")]
// fn icon(input: &str) -> IResult<&str, ICreole> {
//     map(
//         delimited(
//             tag("<<Icon("),
//             is_not("<>)\n"),
//             // take_until(")>>"),
//             tag(")>>"),
//         ),
//         ICreole::Icon
//     ).parse(input)
// }
/// take any leftovers into plain text
pub(crate) fn text(input: &str) -> IResult<&str, ICreole<'_>> {
    map(verify(rest, |s: &str| !s.is_empty()), |s: &str| {
        ICreole::Text(s)
    })
    .parse(input)
}
/// `[{href|label}]` button link → `LinkButton(href, inline label nodes)`. `link-button`.
#[cfg(feature = "link-button")]
pub(crate) fn link_button(input: &str) -> IResult<&str, ICreole<'_>> {
    let (left, (href, name)) = delimited(
        tag("[{"),
        alt((
            separated_pair(is_not("[]{}|\n"), char('|'), is_not("[]{}\n")),
            map(is_not("[]{}\n"), |v: &str| (v, v)),
        )),
        tag("}]"),
    )
    .parse(input)?;
    #[cfg(feature = "extend")]
    let (_, v) = many0(alt((extend, map(is_not("<"), ICreole::Text)))).parse(name)?;
    #[cfg(not(feature = "extend"))]
    let v = vec![ICreole::Text(name)];
    Ok((left, ICreole::LinkButton(href, v)))
}
#[cfg(feature = "tex")]
pub(crate) fn inline_tex(input:&str) -> IResult<&str, ICreole<'_>> {
    map(
        delimited(
            tag("$$"),
            is_not("$\n"),
            tag("$$"),
        ),
        |body| ICreole::InlineTex(body),
    )
    .parse(input)
}
// #[cfg(feature = "font-color")]
// fn font_color(input: &str) -> IResult<&str, ICreole> {
//     map(
//         delimited(
//             tag("{["),
//             alt((
//                 separated_pair(is_not("|]}\n"), tag("|"), is_not("|]}\n")),
//                 map(is_not("]}\n"), |label: &str| -> (&str, &str) {
//                     (label, label)
//                 }),
//             )),
//             tag("]}"),
//         ),
//         |(link, label)| ICreole::Color(label, link),
//     )
//     .parse(input)
// }

/// `>><content>>` foldable region → `Fold(content)`. `fold`.
#[cfg(feature = "fold")]
pub(crate) fn fold(input: &str) -> IResult<&str, ICreole<'_>> {
    // map(map_parser(tag("---<"), take_until("----"), |rest| ICreole::Fold(rest))(input)
    map(
        delimited(tag("---<"), take_until("\n---<"), tag("\n---<")),
        ICreole::Fold,
    )
    .parse(input)
}

//     fail_parser: impl Parser<&'a str, Output = ICreole<'a>, Error = E>,
// ) -> impl Parser<&'a str, Output = Vec<ICreole<'a>>, Error = E> {
// }
// parse things that should be collected as paragraph
// fn take_text_on_parser_fail<'a, E: ParseError<&'a str>>(
//     parser: &mut impl Parser<&'a str, Output = ICreole<'a>, Error = E>,
//     input: &'a str,
// ) -> IResult<&'a str, Vec<ICreole<'a>>, E> {
//     let mut rst = vec![];
//     // let mut acc = String::new();
//     let mut len = 0;
//     let mut rem = input;
//     // let mut (a, b) = (0, 0);
//     loop {
//         if rem.is_empty() {
//             if len > 0 {
//                 rst.push(ICreole::Text(&rem[..len]));
//             }
//             break;
//         }
//         if let Ok((r, v)) = parser.parse(rem) {
//             if 0 < len {
//                 rst.push(ICreole::Text(&rem[..len]));
//                 len = 0;
//             }
//             rst.push(v);
//             rem = r;
//         } else {
//             len += 1;
//             rem = &rem[len..];
//         }
//     }
//     Ok((rem, rst))
// }
