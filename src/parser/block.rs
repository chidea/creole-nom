//! Block-level parsers: constructs anchored at line starts (headings, lists,
//! tables, `{{{…}}}`, `---`), plus the paragraph fallback and the per-line
//! dispatchers used by `try_creoles`.

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

pub(crate) fn list_lines<'a>(
    head: char,
) -> impl Parser<&'a str, Output = Vec<&'a str>, Error = nom::error::Error<&'a str>> {
    separated_list0(
        char('\n'),
        recognize(separated_pair(
            preceded(space0, preceded(char(head), many0(list_head_char))),
            space0,
            alt((is_not("\n"), success(""))),
        )),
    )
}
/// One list run: consecutive `*`/`#` lines → `BulletList`/`NumberedList`,
/// nesting by marker depth (a deeper marker nests, a shallower closes).
pub(crate) fn list(input: &str) -> IResult<&str, ICreole<'_>> {
    let trimmed = input.trim_start();
    let (_, head) = list_head_char(trimmed)?;
    // a `*` run followed by another `*`, or a run >= 2 whose rest of the
    // first line holds a `**` closer, is emphasis, not a deeper bullet:
    // bail so the paragraph/text_style path claims the line
    let (_, run) = recognize(pair(char(head), many0(list_head_char))).parse(trimmed)?;
    if head == '*'
        && (trimmed[run.len()..].starts_with('*')
            || (run.len() >= 2 && trimmed[run.len()..].contains("**")))
    {
        return Err(err_tag(trimmed));
    }
    let (mut r, mut lines) = list_lines(head).parse(input)?;
    if lines.is_empty() || lines.iter().any(|l| l.trim().is_empty()) {
        return Err(err_tag(input)); // a blank line is never a list item
    }
    // the spec nests a blank-line separated block into the last item:
    // absorb `\n\n` + the following list block, "" marks the child boundary
    while r.starts_with("\n\n*") || r.starts_with("\n\n#") {
        match list_lines(head).parse(&r[2..]) {
            Ok((r2, more)) if !more.is_empty() && more.iter().all(|l| !l.trim().is_empty()) => {
                lines.push("");
                lines.extend(more);
                r = r2;
            }
            _ => break,
        }
    }
    Ok((r, _list(&trimmed[..1])(lines)))
}

pub(crate) fn _list<'a>(head_tag: &str) -> impl FnMut(Vec<&'a str>) -> ICreole<'a> {
    move |input: Vec<&str>| {
        let mut rst = vec![];
        let mut to_skip = 0;
        for i in 0..input.len() {
            if to_skip > 0 {
                to_skip -= 1;
                continue;
            }
            let line = input[i].trim_start();
            if line.is_empty() {
                // "" is the child-boundary sentinel list() inserts for a
                // blank-line separated continuation block, never an item
                continue;
            }
            // debug!("list line : {}", line);
            if starts_with_marker(line, head_tag) {
                // sibling
                let content = line[head_tag.len()..].trim_start();
                let (_, v) = map(collect_while_parser_fail0(lit, text), ICreole::ListItem)
                    .parse(content)
                    // the collector always succeeds (empty content -> empty
                    // item); the or_else only exists to keep the match total
                    .unwrap_or_else(|_| (line, ICreole::ListItem(vec![])));
                rst.push(v);
            } else {
                // child
                let child_head_tag = &line[..head_tag.len() + 1];
                let mut child_lines = vec![line];
                for line in &input[(i + 1)..] {
                    if line.starts_with(child_head_tag) {
                        child_lines.push(*line);
                    } else {
                        break;
                    }
                }
                to_skip = child_lines.len() - 1;
                // debug!("child lines : {:?}, skip : {}", child_lines, to_skip);
                rst.push(_list(child_head_tag)(child_lines));
            }
        }
        if head_tag.ends_with('*') {
            ICreole::BulletList(rst)
        } else {
            ICreole::NumberedList(rst)
        }
    }
}

/// `= text =` → `Heading(level, inline)`; level is the leading `=` run (1-6),
/// the trailing run is optional and may differ in length.
pub(crate) fn heading(input: &str) -> IResult<&str, ICreole<'_>> {
    map(
        separated_pair(
            map(many_m_n(1, 6, char('=')), |s| s.len()),
            space0,
            alt((take_until("\n"), rest)),
        ),
        |(level, body): (usize, &str)| {
            // a trailing '=' run closes the heading whatever its length
            // (a shorter closing run is spec-legal: `== a =` -> "a")
            let body = body.trim_end().trim_end_matches('=').trim_end();
            ICreole::Heading(
                level as u8,
                match take_dlit_text_until0().parse(body) {
                    Ok((_, v)) => v,
                    Err(_) => vec![],
                },
            )
        },
    )
    .parse(input)
}
/// `{{{raw}}}` preformatted block → `DontFormat` with the verbatim slice.
pub(crate) fn dont_format(input: &str) -> IResult<&str, ICreole<'_>> {
    map(
        delimited(tag("{{{"), take_until("}}}"), tag("}}}")),
        ICreole::DontFormat,
    )
    .parse(input)
}

pub(crate) fn table_header_cell_inner(input: &str) -> IResult<&str, ICreole<'_>> {
    map(
        take_dlit_text_until_peek_char0('|'),
        ICreole::TableHeaderCell,
    )
    .parse(input)
}
pub(crate) fn table_cell_inner(input: &str) -> IResult<&str, ICreole<'_>> {
    map(take_dlit_text_until_peek_char0('|'), ICreole::TableCell).parse(input)
}
pub(crate) fn table_header_row(input: &str) -> IResult<&str, ICreole<'_>> {
    let (left, line) = preceded(
        tag("|="),
        map(
            alt((
                terminated(
                    verify(map(is_not("\n"), |s: &str| s.trim_end()), |s: &str| {
                        s.ends_with('|')
                    }),
                    char('\n'),
                ),
                verify(rest, |s: &str| s.len() > 1 && s.ends_with('|')),
            )),
            |s: &str| &s[..(s.len() - 1)],
        ),
    )
    .parse(input)?;
    // debug!("table_header_row line : {}", line);

    let rst = split_cells(line, "|=")
        .into_iter()
        .filter_map(|c| table_header_cell_inner(c).ok().map(|(_, v)| v))
        .collect();
    Ok((left, ICreole::TableHeaderRow(rst)))
}
pub(crate) fn table_cell_row(input: &str) -> IResult<&str, ICreole<'_>> {
    let (left, line) = preceded(
        char('|'),
        map(
            alt((
                verify(map(is_not("\n"), |s: &str| s.trim_end()), |s: &str| {
                    s.ends_with('|')
                }),
                verify(map(rest, |s: &str| s.trim_end()), |s: &str| {
                    s.len() > 1 && s.ends_with('|')
                }),
            )),
            |s: &str| &s[..(s.len() - 1)],
        ),
    )
    .parse(input)?;
    // debug!("table_cell_row : {}", line);
    let rst = split_cells(line, "|")
        .into_iter()
        .filter_map(|c| table_cell_inner(c).ok().map(|(_, v)| v))
        .collect();
    Ok((left, ICreole::TableRow(rst)))
}
/// Consecutive `|`/`|=` lines → `Table` of `TableHeaderRow`/`TableRow`
/// (header row wins the run even mid-table; `||` separates cells).
pub(crate) fn table(input: &str) -> IResult<&str, ICreole<'_>> {
    let mut rst = vec![];
    // debug!("table input : {}", input);
    let (rest, body) = if let Ok((r, head)) = table_header_row(input) {
        // debug!("table head : {:?}", head);
        rst.push(head);
        match separated_list0(char('\n'), table_cell_row).parse(r) {
            Ok((rest, body)) => (rest, body),
            Err(_) => return Ok((r, ICreole::Table(rst))),
        }
    } else {
        // debug!("table head is empty");
        match separated_list1(char('\n'), table_cell_row).parse(input) {
            Ok(parsed) => parsed,
            Err(_) => return Err(err_tag(input)),
        }
    };
    Ok((rest, ICreole::Table([rst, body].concat())))
}

/// Fallback block: consecutive plain lines collected as one inline-run
/// `Paragraph` (stops at a blank or block-anchored line).
pub(crate) fn parse_paragraph(input: &str) -> IResult<&str, ICreole<'_>> {
    map(
        // many1(alt((dont_format, link, image, text_style))),
        collect_while_parser_fail_or0(
            alt((
                value(true, pair(char('\n'), peek(char('\n')))),
                value(true, peek(preceded(char('\n'), parse_creole_inner))),
            )),
            #[cfg(feature = "extend")]
            alt((dlit, extend)),
            #[cfg(not(feature = "extend"))]
            dlit,
            text,
        ),
        ICreole::Paragraph,
    )
    .parse(input)
}
/// parse things that should be at starting of a line which starts or continues a creole collection
pub(crate) fn parse_creole_inner(input: &str) -> IResult<&str, ICreole<'_>> {
    alt((
        value(
            ICreole::HorizontalLine,
            // tag("----"),
            terminated(tag("----"), alt((peek(line_ending), eof))),
        ),
        #[cfg(feature = "fold")]
        fold,
        #[cfg(feature = "link-button")]
        link_button,
        heading,
        dont_format,
        table,
        list,
    ))
    .parse(input)
}

// fn parse_element(input: &str) -> IResult<&str, ICreole> {
//     alt((parse_creole_inner, parse_paragraph)).parse(input)
// }
