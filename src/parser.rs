#[cfg(debug_assertions)]
#[allow(unused_imports)]
use log::{debug, error, info};
use {
    crate::creole::ICreole,
    nom::{
        branch::alt,
        bytes::complete::{
            // take_until1,
            // take_while, // take while function result is true
            // take_while1,
            // take_while_m_n, // longest m<=len<=n
            is_not,
            tag, // exact match
            // tag_no_case, // same, case insensitive
            // take,      // blindly take by argument as count
            // take_till, // take before given function over input returns true
            // take_till1, // same, error on blank result
            take_until, // take before given pattern equals from remaining input
        },
        character::complete::{
            // space1,
            // anychar,
            // multispace0, multispace1,
            char,
            line_ending,
            // not_line_ending,
            // none_of,
            one_of,
            space0,
        },
        combinator::{
            // consumed, // returns tuple (consumed input, parser output) as result
            eof,
            map, // apply function to the result of parser
            // map_res,
            // opt,
            // rest_len, // return length of all remaining input
            peek,
            // not, // success when parser fails
            // cond, // take a bool and run parser when it's a true
            // all_consuming, // succeed when given parser remains no rest input
            recognize, // return consumed input by parser when its succeed
            rest,      // return all remaining input
            success,
            // map_opt, // same, returns Option<_>
            // map_res, // same, returns Result<_>
            // map_parser, // parser after parser
            value,  // return first argument as parser result if parser succeed
            verify, // same, if given verify function returns true over parser result
        },
        // Input,
        // Parser,
        // Err,
        error::{
            // Error,
            ErrorKind,
            ParseError,
        },
        multi::{
            // fill, // fill given slice or fail
            // fold_many0, // apply until fail
            // fold_many1, // same, does not allow empty result
            // fold_many_m_n, // m<=count<=n
            // length_count, // result of first parser is the count to repeat second parser
            // length_data, // result of first parser is the count to take
            // length_value, // result of first parser is the count to take and it's the input to second parser
            // many0_count, // repeat parser until fails and return the count of successful iteration
            // many1_count, // same, fail on zero
            many0,
            // many1,
            // many1,
            many_m_n,
            // many_till,
            separated_list0,
            separated_list1, // two parsers for list
        },
        sequence::{
            delimited, // "(a)" => "a"
            pair,      // "ab" => ("a", "b")
            preceded,  // "(a" => "a"
            // tuple, // same, up to 21 elements
            separated_pair, // "a,b" => ("a", "b")
            terminated,     // "a)" => "a"
        },
        IResult,
        Parser,
    },
};

// fn parse_tilde_escape<'a, E>(input: &'a str) -> IResult<&'a str, char, E>
// where
//     E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>,
// {
//     /*
//     from : http://www.wikicreole.org/wiki/EscapeCharacterProposal
//     Here's a complete list that would cause the escape character (e.g. tilde) to trigger escaping of the following character combinations. This can be extended as need be. Note also that an escape character can be escaped by putting a space after it, since a space cannot be escaped.

//     Escapes Markup      Combination Constraint              Example:
//     ==============      ==========================          =========
//     Bold                Anywhere                            ~**
//     Italics             Anywhere                            ~//
//     Unordered Lists     Hyphen, first chars in line         ~-
//     Ordered Lists       First chars in line                 ~#
//     Headings            Equal signs, first chars in line    ~=
//     Links Open          Anywhere                            ~[[
//     Inside Links        Within Links                        [[A pipe ~| is used within links]]
//     Links Close         Anywhere                            ~]]
//     Line Breaks         Anywhere                            ~\\
//     Horizontal Line     First chars in line                 ~----
//     Image Open          Anywhere                            ~{{
//     Image Close         Anywhere                            ~}}
//     Table Open          First chars in line                 ~| this is | not a table
//     Inside Tables       Pipe used within a table cell       | x~|y |
//     Escaping Escape     Anywhere                            ~~
//     Escaping Escape     Anywhere                            ~<space>
//     Nowiki Open         First chars in line                 ~{{{
//     Nowiki Close        First chars in line                 ~}}}
//         */
//     preceded(char('~'), alt(())).parse(input)
// }

fn bold(input: &str) -> IResult<&str, ICreole<'_>> {
    map(
        delimited(
            tag("**"),
            // map(
            //     alt((
            //         take_until("**"),
            //         recognize(many_till(anychar, alt((tag("\n"), tag("**"), eof)))),
            //     )),
            //     |s| vec![ICreole::Text(s)],
            // ),
            // map(recognize(many_till(peek(alt((tag("\n"), tag("**"), eof))))), |s| vec![ICreole::Text(s)])
            collect_while_parser_fail_or0(
                alt((value(true, peek(tag("**"))), value(true, peek(char('\n'))))),
                italic,
                text,
            ),
            tag("**"),
        ),
        ICreole::Bold,
    )
    .parse(input)
}
fn italic(input: &str) -> IResult<&str, ICreole<'_>> {
    map(
        delimited(
            tag("//"),
            collect_while_parser_fail_or0(
                alt((value(true, peek(tag("//"))), value(true, peek(char('\n'))))),
                alt((bold, link)),
                text,
            ),
            tag("//"),
        ),
        ICreole::Italic,
    )
    .parse(input)
}

fn text_style(input: &str) -> IResult<&str, ICreole<'_>> {
    alt((
        value(ICreole::ForceLinebreak, tag("\\\\")),
        bold,
        italic,
        // #[cfg(feature = "font-color")]
        // font_color,
    ))
    .parse(input)
}

fn list_head_char(input: &str) -> IResult<&str, char> {
    one_of("*#").parse(input)
}
fn list(input: &str) -> IResult<&str, ICreole<'_>> {
    let input = input.trim_start();
    let (_, head) = list_head_char(input)?;
    let (r, lines) = separated_list0(
        char('\n'),
        recognize(separated_pair(
            preceded(space0, preceded(char(head), many0(list_head_char))),
            char(' '),
            alt((is_not("\n"), success(""))),
        )),
    )
    .parse(input)?;

    // debug!("list recognized : {:?}", lines);
    if lines.is_empty() {
        return Err(nom::Err::Error(nom::error::Error {
            input,
            code: ErrorKind::Tag,
        }));
    }
    let v = _list(&input[..1])(lines)?;

    Ok((r, v))
}

fn _list<'a>(
    head_tag: &'a str,
) -> impl FnMut(Vec<&'a str>) -> Result<ICreole<'a>, nom::Err<nom::error::Error<&'a str>>> {
    move |input: Vec<&str>| {
        let head_space = format!("{} ", head_tag);
        let mut rst = vec![];
        let mut to_skip = 0;
        for i in 0..input.len() {
            if to_skip > 0 {
                to_skip -= 1;
                continue;
            }
            let line = input[i].trim_start();
            // debug!("list line : {}", line);
            if line.starts_with(&head_space) {
                // sibling
                if let Ok((_, v)) = map(collect_while_parser_fail0(lit, text), ICreole::ListItem)
                    .parse(&line[head_space.len()..])
                {
                    rst.push(v);
                }
            } else {
                // child
                let child_head_tag = &line[..head_space.len()];
                let mut child_lines = vec![line];
                for line in &input[(i + 1)..] {
                    if line.starts_with(&child_head_tag) {
                        child_lines.push(*line);
                    } else {
                        break;
                    }
                }
                to_skip = child_lines.len() - 1;
                // debug!("child lines : {:?}, skip : {}", child_lines, to_skip);
                if let Ok(v) = _list(child_head_tag)(child_lines) {
                    rst.push(v);
                }
            }
        }
        Ok(if head_tag.ends_with('*') {
            ICreole::BulletList(rst)
        } else {
            ICreole::NumberedList(rst)
        })
    }
}

fn link(input: &str) -> IResult<&str, ICreole<'_>> {
    let (left, (href, name)) = delimited(
        tag("[["),
        alt((
            separated_pair(is_not("]|\n"), char('|'), is_not("]\n")),
            map(is_not("]\n"), |v: &str| (v, v)),
        )),
        tag("]]"),
    )
    .parse(input)?;

    #[cfg(feature = "extend")]
    let (_, v) = many0(alt((extend, map(is_not("<"), ICreole::Text)))).parse(name)?;
    #[cfg(not(feature = "extend"))]
    let v = vec![ICreole::Text(name)];
    Ok((left, ICreole::Link(href, v)))
}
fn image(input: &str) -> IResult<&str, ICreole<'_>> {
    map(
        delimited(
            tag("{{"),
            alt((
                separated_pair(is_not("{}|\n"), tag("|"), is_not("{}\n")),
                map(is_not("{}\n"), |src: &str| (src, "")),
            )),
            tag("}}"),
        ),
        |(src, label)| ICreole::Image(src, label),
    )
    .parse(input)
}

#[cfg(feature = "extend")]
fn extend(input: &str) -> IResult<&str, ICreole<'_>> {
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
fn lit(input: &str) -> IResult<&str, ICreole<'_>> {
    alt((link, image, text_style)).parse(input)
}
fn dlit(input: &str) -> IResult<&str, ICreole<'_>> {
    // alt((dont_format, lit)).parse(input)
    alt((dont_format, link, image, text_style)).parse(input)
}
/// take any leftovers into plain text
fn text(input: &str) -> IResult<&str, ICreole<'_>> {
    map(verify(rest, |s: &str| !s.is_empty()), |s: &str| {
        ICreole::Text(s)
    })
    .parse(input)
}
fn take_dlit_text_until0<'a>() -> impl Parser<&'a str, Output = Vec<ICreole<'a>>> {
    collect_opt_pair0(take_while_parser_fail(dlit, text))
}
fn take_dlit_text_until_peek_char0<'a>(
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

fn take_while_parser_fail<'a, E: ParseError<&'a str>>(
    mut parser: impl Parser<&'a str, Output = ICreole<'a>, Error = E>,
    mut fail_parser: impl Parser<&'a str, Output = ICreole<'a>, Error = E>,
) -> impl Parser<&'a str, Output = (Option<ICreole<'a>>, Option<ICreole<'a>>), Error = E> {
    move |input: &'a str| {
        // debug!("take_while_parser_fail input : {}", input);
        // let mut i = input;
        let mut l = 0;
        for (i, c) in input.char_indices().by_ref() {
            // debug!("take_while_parser_fail i : {}", i);
            if let Ok((r, v)) = parser.parse(&input[l..]) {
                return Ok((
                    r,
                    (
                        if l > 0 {
                            if let Ok((_, v)) = fail_parser.parse(&input[..l]) {
                                Some(v)
                            } else {
                                None
                            }
                        } else {
                            None
                        },
                        Some(v),
                    ),
                ));
            } else {
                l = i + c.len_utf8();
            }
        }
        if l > 0 {
            if let Ok((_, f)) = fail_parser.parse(&input[..l]) {
                Ok((&input[input.len()..], (Some(f), None)))
            } else {
                Err(nom::Err::Error(ParseError::from_error_kind(
                    input,
                    ErrorKind::Tag,
                )))
            }
        } else {
            Err(nom::Err::Error(ParseError::from_error_kind(
                input,
                ErrorKind::Eof,
            )))
        }
    }
}

fn collect_opt_pair0<'a, E: ParseError<&'a str>>(
    parser: impl Parser<&'a str, Output = (Option<ICreole<'a>>, Option<ICreole<'a>>), Error = E>,
) -> impl Parser<&'a str, Output = Vec<ICreole<'a>>, Error = E> {
    alt((collect_opt_pair1(parser), success(vec![])))
}
fn collect_opt_pair1<'a, E: ParseError<&'a str>>(
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
            Err(nom::Err::Error(ParseError::from_error_kind(
                input,
                ErrorKind::Eof,
            )))
        } else {
            Ok((i, rst))
        }
    }
}

fn take_while_parser_fail_or<'a, E: ParseError<&'a str>>(
    mut term_parser: impl Parser<&'a str, Output = bool, Error = E>,
    mut parser: impl Parser<&'a str, Output = ICreole<'a>>,
    mut fail_parser: impl Parser<&'a str, Output = ICreole<'a>, Error = E>,
) -> impl Parser<&'a str, Output = (Option<ICreole<'a>>, Option<ICreole<'a>>), Error = E> {
    move |input: &'a str| {
        let mut l = 0;
        for (i, c) in input.char_indices().by_ref() {
            if let Ok((i, true)) = term_parser.parse(&input[l..]) {
                return Ok((
                    i,
                    (fail_parser.parse(&input[..l]).ok().map(|(_, v)| v), None),
                ));
            } else if let Ok((r, v)) = parser.parse(&input[l..]) {
                return Ok((
                    r,
                    (fail_parser.parse(&input[..l]).ok().map(|(_, v)| v), Some(v)),
                ));
            } else {
                l = i + c.len_utf8();
            }
        }
        if l > 0 {
            if let Ok((_, f)) = fail_parser.parse(&input[..l]) {
                Ok(("", (Some(f), None)))
            } else {
                Err(nom::Err::Error(ParseError::from_error_kind(
                    input,
                    ErrorKind::Tag,
                )))
            }
        } else {
            Err(nom::Err::Error(ParseError::from_error_kind(
                input,
                ErrorKind::Eof,
            )))
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

fn collect_while_parser_fail_or0<'a, E: ParseError<&'a str>>(
    term_parser: impl Parser<&'a str, Output = bool, Error = E>,
    parser: impl Parser<&'a str, Output = ICreole<'a>, Error = E>,
    fail_parser: impl Parser<&'a str, Output = ICreole<'a>, Error = E>,
) -> impl Parser<&'a str, Output = Vec<ICreole<'a>>, Error = E> {
    collect_opt_pair0(take_while_parser_fail_or(term_parser, parser, fail_parser))
}

fn collect_while_parser_fail0<'a, E: ParseError<&'a str>>(
    parser: impl Parser<&'a str, Output = ICreole<'a>, Error = E>,
    fail_parser: impl Parser<&'a str, Output = ICreole<'a>, Error = E>,
) -> impl Parser<&'a str, Output = Vec<ICreole<'a>>, Error = E> {
    collect_opt_pair0(take_while_parser_fail(parser, fail_parser))
}

#[cfg(feature = "link-button")]
fn link_button(input: &str) -> IResult<&str, ICreole<'_>> {
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
// #[cfg(feature="link-button")]
// fn link_button(input: &str) -> IResult<&str, ICreole> {
//   map(delimited(tag("[{"), alt((
//     separated_pair(is_not("|]}\n"), tag("|"), is_not("|]}\n")),
//     map(is_not("]}\n"), |label: &str| -> (&str, &str) { (label, label) }),
//   )), tag("]}")), |(link, label)| ICreole::LinkButton(label, link, ""))(input)
// }
// #[cfg(feature = "font-color")]
// fn font_color(input: &str) -> IResult<&str, ICreole> {
//     map(
//         delimited(
//             tag("[{"),
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

fn heading(input: &str) -> IResult<&str, ICreole<'_>> {
    map(
        separated_pair(
            map(many_m_n(1, 6, char('=')), |s| s.len()),
            char(' '),
            alt((take_until("\n"), rest)),
        ),
        |(level, body): (usize, &str)| {
            ICreole::Heading(
                level as u8,
                if let Ok((_, v)) = take_dlit_text_until0().parse({
                    let body = body.trim_end();
                    if body.ends_with(&input[..level]) {
                        // ignore ending space and '='s same as its opening
                        body[..body.len() - level].trim_end()
                    } else {
                        body
                    }
                }) {
                    v
                } else {
                    vec![]
                },
            )
        },
    )
    .parse(input)
}
fn dont_format(input: &str) -> IResult<&str, ICreole<'_>> {
    map(
        delimited(tag("{{{"), take_until("}}}"), tag("}}}")),
        ICreole::DontFormat,
    )
    .parse(input)
}

fn table_header_cell_inner(input: &str) -> IResult<&str, ICreole<'_>> {
    map(
        take_dlit_text_until_peek_char0('|'),
        ICreole::TableHeaderCell,
    )
    .parse(input)
}
fn table_cell_inner(input: &str) -> IResult<&str, ICreole<'_>> {
    map(take_dlit_text_until_peek_char0('|'), ICreole::TableCell).parse(input)
}
fn table_header_row(input: &str) -> IResult<&str, ICreole<'_>> {
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

    let (_, rst) = map(separated_list1(tag("|="), table_header_cell_inner), |v| {
        ICreole::TableHeaderRow(v)
    })
    .parse(line)?;
    Ok((left, rst))
}
fn table_cell_row(input: &str) -> IResult<&str, ICreole<'_>> {
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
    let (_, rst) = map(separated_list1(tag("|"), table_cell_inner), |v| {
        ICreole::TableRow(v)
    })
    .parse(line)?;
    Ok((left, rst))
}
fn table(input: &str) -> IResult<&str, ICreole<'_>> {
    let mut rst = vec![];
    let mut rest = input;
    // debug!("table input : {}", input);
    if let Ok((rest, body)) = if let Ok((r, head)) = table_header_row(input) {
        // debug!("table head : {:?}", head);
        rst.push(head);
        rest = r;
        separated_list0(char('\n'), table_cell_row).parse(r)
    } else {
        // debug!("table head is empty");
        separated_list1(char('\n'), table_cell_row).parse(input)
    } {
        // debug!("body : {:?}", body);
        return Ok((rest, ICreole::Table([rst, body].concat())));
    } else if !rst.is_empty() {
        // debug!("no body found, result : {:?}", rst);
        return Ok((rest, ICreole::Table(rst)));
    };
    // debug!("table : {:?}", rst);
    if rst.is_empty() {
        Err(nom::Err::Error(nom::error::Error {
            input,
            code: ErrorKind::Tag,
        }))
    } else {
        Ok((rest, ICreole::Table(rst)))
    }
}

#[cfg(feature = "fold")]
fn fold(input: &str) -> IResult<&str, ICreole<'_>> {
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
/// parse things that should be collected as paragraph
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

fn parse_paragraph(input: &str) -> IResult<&str, ICreole<'_>> {
    // let mut rst = vec![];
    // // let mut acc = String::new();
    // let mut len = 0;
    // let mut remainder = input;
    // // let mut (a, b) = (0, 0);
    // while !remainder.is_empty() {
    //     if let Ok((r, v)) = alt((dont_format, link, image, text_style)).parse(remainder) {
    //         if 0 < len {
    //             rst.push(ICreole::Text(&remainder[..len]));
    //             len = 0;
    //         }
    //         rst.push(v);
    //         remainder = r;
    //     } else {
    //         len += 1;
    //     }
    // }
    // Ok((remainder, ICreole::Paragraph(rst)))
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
fn parse_creole_inner(input: &str) -> IResult<&str, ICreole<'_>> {
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

pub fn try_creoles(input: &str) -> IResult<&str, Vec<ICreole<'_>>> {
    separated_list0(char('\n'), alt((parse_creole_inner, parse_paragraph)))
        // many0(terminated(
        //     alt((parse_creole_inner, parse_paragraph)),
        //     opt(line_ending),
        // ))
        .parse(input)
}

pub fn creoles(input: &str) -> Vec<ICreole<'_>> {
    if let Ok((_, v)) = try_creoles(input) {
        v
    } else {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::creole::ICreole;

    fn init() {
        let _ =
            env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
                .is_test(true)
                .try_init();
    }

    #[test]
    fn text_tests() {
        init();
        use ICreole::*;
        assert_eq!(creoles("ab1"), vec![Paragraph(vec![Text("ab1")])]);
    }

    #[test]
    fn text_style_tests() {
        init();
        use ICreole::*;
        assert_eq!(try_creoles("t"), Ok(("", vec![Paragraph(vec![Text("t")])])));

        assert_eq!(
            try_creoles("**b**"),
            Ok(("", vec![Paragraph(vec![Bold(vec![Text("b")])])]))
        );

        assert_eq!(
            try_creoles("//i//"),
            Ok(("", vec![Paragraph(vec![Italic(vec![Text("i")])])]))
        );

        assert_eq!(
            try_creoles("a**b**//c//d"),
            Ok((
                "",
                vec![Paragraph(vec![
                    Text("a"),
                    Bold(vec![Text("b")]),
                    Italic(vec![Text("c")]),
                    Text("d")
                ])]
            ))
        );
        assert_eq!(
            try_creoles("**a//b//c**"),
            Ok((
                "",
                vec![Paragraph(vec![Bold(vec![
                    Text("a"),
                    Italic(vec![Text("b")]),
                    Text("c")
                ])])]
            ))
        );
        assert_eq!(
            try_creoles("**bold,\nnot closed."),
            Ok(("", vec![Paragraph(vec![Text("**bold,\nnot closed.")])]))
        );
        assert_eq!(
            try_creoles("//italic,\nnot closed."),
            Ok(("", vec![Paragraph(vec![Text("//italic,\nnot closed.")])]))
        );
    }

    #[test]
    fn linebreak_tests() {
        init();
        use ICreole::*;
        assert_eq!(
            try_creoles("a\\\\b"),
            Ok((
                "",
                vec![Paragraph(vec![Text("a"), ForceLinebreak, Text("b"),])]
            ))
        );

        assert_eq!(creoles("a\\b"), vec![Paragraph(vec![Text("a\\b")])]);

        assert_eq!(
            try_creoles("a\nb\n\nc"),
            Ok((
                "",
                vec![Paragraph(vec![Text("a\nb")]), Paragraph(vec![Text("c")])]
            ))
        );
    }

    #[test]
    fn list_tests() {
        init();
        use ICreole::*;
        assert_eq!(list("* "), Ok(("", BulletList(vec![ListItem(vec![])]))));
        assert_eq!(
            list("* a"),
            Ok(("", BulletList(vec![ListItem(vec![Text("a")])])))
        );
        assert_eq!(
            list("** b"),
            Ok((
                "",
                BulletList(vec![BulletList(vec![ListItem(vec![Text("b")])])])
            ))
        );
        assert_eq!(
            list("*** c"),
            Ok((
                "",
                BulletList(vec![BulletList(vec![BulletList(vec![ListItem(vec![
                    Text("c")
                ])])])])
            ))
        );
        assert_eq!(
            list(
                "* a
** b"
            ),
            Ok((
                "",
                BulletList(vec![
                    ListItem(vec![Text("a")]),
                    BulletList(vec![ListItem(vec![Text("b")])]),
                ])
            ))
        );
        assert_eq!(
            list(
                "* a
** b
** c"
            ),
            Ok((
                "",
                BulletList(vec![
                    ListItem(vec![Text("a")]),
                    BulletList(vec![ListItem(vec![Text("b")]), ListItem(vec![Text("c")]),])
                ])
            ))
        );
        assert_eq!(
            list(
                "* a
** ab
* b
** ba"
            ),
            Ok((
                "",
                BulletList(vec![
                    ListItem(vec![Text("a")]),
                    BulletList(vec![ListItem(vec![Text("ab")])]),
                    ListItem(vec![Text("b")]),
                    BulletList(vec![ListItem(vec![Text("ba")])])
                ])
            ))
        );
        assert_eq!(
            list(
                "* [[a]]
** //b//
** **c**"
            ),
            Ok((
                "",
                BulletList(vec![
                    ListItem(vec![Link("a", vec![Text("a")])]),
                    BulletList(vec![
                        ListItem(vec![Italic(vec![Text("b")])]),
                        ListItem(vec![Bold(vec![Text("c")])]),
                    ]),
                ])
            ))
        );
        assert_eq!(
            list(
                "* a
** aa
** ab
* b
** ba"
            ),
            Ok((
                "",
                BulletList(vec![
                    ListItem(vec![Text("a")]),
                    BulletList(vec![ListItem(vec![Text("aa")]), ListItem(vec![Text("ab")]),]),
                    ListItem(vec![Text("b")]),
                    BulletList(vec![ListItem(vec![Text("ba")])]),
                ])
            ))
        );
        assert_eq!(
            creoles(
                "* a
** aa
** ab
* b
*# b1"
            ),
            vec![BulletList(vec![
                ListItem(vec![Text("a")]),
                BulletList(vec![ListItem(vec![Text("aa")]), ListItem(vec![Text("ab")]),]),
                ListItem(vec![Text("b")]),
                NumberedList(vec![ListItem(vec![Text("b1")])])
            ])]
        );
        assert_eq!(
            try_creoles(
                "* a
*# a1
*# a2
*## a31
#### 1111
#### 1112
### 112
##* 11a
##* 11b
##*# 11c1
##*# 11c2
* b"
            ),
            Ok((
                "",
                vec![
                    BulletList(vec![
                        ListItem(vec![Text("a")]),
                        NumberedList(vec![
                            ListItem(vec![Text("a1")]),
                            ListItem(vec![Text("a2")]),
                            NumberedList(vec![ListItem(vec![Text("a31")])])
                        ])
                    ]),
                    NumberedList(vec![NumberedList(vec![
                        NumberedList(vec![
                            NumberedList(vec![
                                ListItem(vec![Text("1111")]),
                                ListItem(vec![Text("1112")])
                            ]),
                            ListItem(vec![Text("112")])
                        ]),
                        BulletList(vec![
                            ListItem(vec![Text("11a")]),
                            ListItem(vec![Text("11b")]),
                            NumberedList(vec![
                                ListItem(vec![Text("11c1")]),
                                ListItem(vec![Text("11c2")])
                            ])
                        ])
                    ])]),
                    BulletList(vec![ListItem(vec![Text("b")])])
                ]
            ))
        );
        assert_eq!(
            list(" * a"),
            Ok(("", BulletList(vec![ListItem(vec![Text("a")])])))
        );
        assert_eq!(
            list("  * a"),
            Ok(("", BulletList(vec![ListItem(vec![Text("a")])])))
        );
        assert_eq!(
            list("***** 5"),
            Ok((
                "",
                BulletList(vec![BulletList(vec![BulletList(vec![BulletList(vec![
                    BulletList(vec![ListItem(vec![Text("5")])])
                ])])])])
            ))
        );
    }
    #[test]
    fn parser_tests() {
        init();
        use ICreole::*;
        assert_eq!(
            take_while_parser_fail(lit, text).parse("a b **a**"),
            Ok(("", (Some(Text("a b ")), Some(Bold(vec![Text("a")])),)))
        );
        assert_eq!(
            collect_opt_pair1(take_while_parser_fail(lit, text)).parse("a b **a**"),
            Ok(("", vec![Text("a b "), Bold(vec![Text("a")]),]))
        );
        assert_eq!(
            collect_opt_pair1(take_while_parser_fail(parse_creole_inner, text)).parse("a\n= b"),
            Ok(("", vec![Text("a\n"), Heading(1, vec![Text("b")]),]))
        );
        assert_eq!(
            collect_opt_pair1(take_while_parser_fail(lit, text)).parse("a b **a**"),
            Ok(("", vec![Text("a b "), Bold(vec![Text("a")]),]))
        );
        let (a, b) = collect_opt_pair1(take_while_parser_fail(lit, text))
            .parse("[[a|b]] //Live// Editor ([[c|d]])")
            .unwrap();
        assert_eq!(a, "");
        assert_eq!(
            b,
            vec![
                // Paragraph(vec![
                Link("a", vec![Text("b")]),
                Text(" "),
                Italic(vec![Text("Live")]),
                Text(" Editor ("),
                Link("c", vec![Text("d")]),
                Text(")"),
                // ])
            ]
        );
    }

    #[test]
    fn heading_tests() {
        init();
        use ICreole::*;
        assert_eq!(heading("= "), Ok(("", Heading(1, vec![]))));
        assert_eq!(heading("= a"), Ok(("", Heading(1, vec![Text("a")]))));
        assert_eq!(heading("= a ="), Ok(("", Heading(1, vec![Text("a")]))));
        assert_eq!(
            heading("= {{{a}}} ="),
            Ok(("", Heading(1, vec![DontFormat("a")])))
        );
        assert_eq!(
            heading("= [[a]]"),
            Ok(("", Heading(1, vec![Link("a", vec![Text("a")])])))
        );
        assert_eq!(
            heading("= a:[[a]]"),
            Ok(("", Heading(1, vec![Text("a:"), Link("a", vec![Text("a")])])))
        );
        assert_eq!(
            try_creoles("= a"),
            Ok(("", vec![Heading(1, vec![Text("a")])]))
        );
        assert_eq!(
            try_creoles("= a:[[a]]"),
            Ok(("", vec![Heading(1, vec![Text("a:"), Link("a", vec![Text("a")])])]))
        );

        assert_eq!(heading("=== b"), Ok(("", Heading(3, vec![Text("b")]))));
        assert_eq!(heading("==== c"), Ok(("", Heading(4, vec![Text("c")]))));
        assert_eq!(creoles("== b"), vec![Heading(2, vec![Text("b")])]);
        assert_eq!(creoles("=== c"), vec![Heading(3, vec![Text("c")])]);

        assert_eq!(
            try_creoles("= [[a]]//a"),
            Ok(("", vec![Heading(1, vec![Link("a", vec![Text("a")]), Text("//a")])]))
        );
        assert_eq!(try_creoles("= [[http://www.wikicreole.org|Creole]] //Live// Editor ([[https://github.com/chidea/wasm-creole-live-editor|github]])"), Ok(("", vec![
      Heading(1, vec![
        Link("http://www.wikicreole.org", vec![Text("Creole")]),
        Text(" "),
        Italic(vec![Text("Live")]),
        Text(" Editor ("),
        Link("https://github.com/chidea/wasm-creole-live-editor", vec![Text("github")]),
        Text(")"),
    ])])));
    }

    #[test]
    fn link_tests() {
        init();
        use ICreole::*;
        assert_eq!(creoles("[[a]]"), vec![Paragraph(vec![Link("a", vec![Text("a")])])]);
        assert_eq!(
            creoles("[[https://google.com|google]]"),
            vec![Paragraph(vec![Link("https://google.com", vec![Text("google")])])]
        );
        assert_eq!(link("[[a]]"), Ok(("", Link("a", vec![Text("a")]))));
        assert_eq!(creoles("[a]"), vec![Paragraph(vec![Text("[a]")])]);
        assert_eq!(link("[[link|label]]"), Ok(("", Link("link", vec![Text("label")]))));
        assert_eq!(
            link("[[https://google.com|google]]"),
            Ok(("", Link("https://google.com", vec![Text("google")])))
        );
        assert_eq!(
            creoles("//[[http://my.book.example/|My Book Title]]//"),
            vec![Paragraph(vec![Italic(vec![Link(
                "http://my.book.example/",
                vec![Text("My Book Title")]
            )])])]
        );
    }

    #[test]
    fn table_tests() {
        init();
        use ICreole::*;
        assert_eq!(
            table_header_row("|=a|=|=c|"),
            Ok((
                "",
                TableHeaderRow(vec![
                    TableHeaderCell(vec![Text("a")]),
                    TableHeaderCell(vec![]),
                    TableHeaderCell(vec![Text("c")]),
                ])
            ))
        );
        assert_eq!(
            table("|=a|=|=c|"),
            Ok((
                "",
                Table(vec![TableHeaderRow(vec![
                    TableHeaderCell(vec![Text("a")]),
                    TableHeaderCell(vec![]),
                    TableHeaderCell(vec![Text("c")]),
                ])])
            ))
        );
        // assert_eq!(table_cell_row("|a|b|c|\n"), Ok(("", TableRow(vec![
        //   TableCell(vec![Text("a")]),
        //   TableCell(vec![Text("b")]),
        //   TableCell(vec![Text("c")]),
        // ]))));

        assert_eq!(
            table(
                "|=|=table|=header| 
|a|table|row| 
|b|{{{ // don't format // }}}|{{{ ** me ** }}}|
|c||empty|"
            ),
            Ok((
                "",
                Table(vec![
                    TableHeaderRow(vec![
                        TableHeaderCell(vec![]),
                        TableHeaderCell(vec![Text("table")]),
                        TableHeaderCell(vec![Text("header")]),
                    ]),
                    TableRow(vec![
                        TableCell(vec![Text("a")]),
                        TableCell(vec![Text("table")]),
                        TableCell(vec![Text("row")]),
                    ]),
                    TableRow(vec![
                        TableCell(vec![Text("b")]),
                        TableCell(vec![DontFormat(" // don't format // ")]),
                        TableCell(vec![DontFormat(" ** me ** ")]),
                    ]),
                    TableRow(vec![
                        TableCell(vec![Text("c")]),
                        TableCell(vec![]),
                        TableCell(vec![Text("empty")]),
                    ]),
                ])
            ))
        );
        assert_eq!(
            try_creoles(
                "|=|=a|=b|
|0|1|2|
|3|4|5|"
            ),
            Ok((
                "",
                vec![Table(vec![
                    TableHeaderRow(vec![
                        TableHeaderCell(vec![]),
                        TableHeaderCell(vec![Text("a")]),
                        TableHeaderCell(vec![Text("b")])
                    ]),
                    TableRow(vec![
                        TableCell(vec![Text("0")]),
                        TableCell(vec![Text("1")]),
                        TableCell(vec![Text("2")])
                    ]),
                    TableRow(vec![
                        TableCell(vec![Text("3")]),
                        TableCell(vec![Text("4")]),
                        TableCell(vec![Text("5")])
                    ])
                ])]
            ))
        );
        assert_eq!(
            creoles(
                "|=h|=h2|
|[[a]]|**b**|
|x //y//|z\\\\w|"
            ),
            vec![Table(vec![
                TableHeaderRow(vec![
                    TableHeaderCell(vec![Text("h")]),
                    TableHeaderCell(vec![Text("h2")]),
                ]),
                TableRow(vec![
                    TableCell(vec![Link("a", vec![Text("a")])]),
                    TableCell(vec![Bold(vec![Text("b")])]),
                ]),
                TableRow(vec![
                    TableCell(vec![Text("x "), Italic(vec![Text("y")])]),
                    TableCell(vec![Text("z"), ForceLinebreak, Text("w")]),
                ]),
            ])]
        );
    }

    #[test]
    fn image_tests() {
        init();
        use ICreole::*;
        assert_eq!(image("{{a.jpg}}"), Ok(("", Image("a.jpg", ""))));
        assert_eq!(image("{{a.jpg|label}}"), Ok(("", Image("a.jpg", "label"))));
        assert_eq!(
            creoles("{{a.jpg}}"),
            vec![Paragraph(vec![Image("a.jpg", "")])]
        );
        assert_eq!(
            creoles("{{a.jpg|label}}"),
            vec![Paragraph(vec![Image("a.jpg", "label")])]
        );

        assert_eq!(
            creoles("{{a.jpg|[[label]]}}"),
            vec![Paragraph(vec![Image("a.jpg", "[[label]]")])]
        );
        assert_eq!(
            creoles("a {{i.png|t}} b"),
            vec![Paragraph(vec![Text("a "), Image("i.png", "t"), Text(" b")])]
        );
    }
    #[cfg(feature = "extend")]
    #[test]
    fn extend_tests() {
        init();
        use ICreole::*;
        assert_eq!(extend("<<Icon(home)>>"), Ok(("", Extend("Icon", "home"))))
    }
    #[test]
    fn other_tests() {
        init();
        use ICreole::*;
        assert_eq!(try_creoles(""), Ok(("", vec![Paragraph(vec![])])));
        assert_eq!(try_creoles("----"), Ok(("", vec![HorizontalLine])));
        assert_eq!(
            try_creoles("-----"),
            Ok(("", vec![Paragraph(vec![Text("-----")])]))
        );
        assert_eq!(creoles("----"), vec![HorizontalLine]);
        // assert_eq!(creoles("----a"), vec![Paragraph(vec![Text("----a")])]);
        assert_eq!(
            creoles("----\na"),
            vec![HorizontalLine, Paragraph(vec![Text("a")])]
        );
        assert_eq!(
            creoles("a\n----\nb"),
            vec![
                Paragraph(vec![Text("a")]),
                HorizontalLine,
                Paragraph(vec![Text("b")])
            ]
        );
        assert_eq!(
            creoles("x {{{ //a// **b** }}} y"),
            vec![Paragraph(vec![
                Text("x "),
                DontFormat(" //a// **b** "),
                Text(" y")
            ])]
        );
        assert_eq!(
            try_creoles("A paragraph.\n\n{{{\n//Some bold//\n}}}\n\nAnother."),
            Ok((
                "",
                vec![
                    Paragraph(vec![Text("A paragraph.")]),
                    DontFormat("\n//Some bold//\n"),
                    Paragraph(vec![Text("\nAnother.")])
                ]
            ))
        );
    }

    //   // #[cfg(feature="extended")]
    //   // #[test]
    //   // fn extended_tests() { init();
    //   //   // assert_eq!(creoles("[{a|b|c}]"), vec![LinkButton("a", "b", "c")]);
    //   //   assert_eq!(link_button("[{a|b|c}]"), LinkButton("a", "b", "c"));
    //   // }
    #[test]
    fn mixed_tests() {
        init();
        use ICreole::*;
        assert_eq!(
            try_creoles("= 大"),
            Ok(("", vec![Heading(1, vec![Text("大")])]))
        );
        assert_eq!(
            try_creoles("= a\n= b\n----"),
            Ok((
                "",
                vec![
                    Heading(1, vec![Text("a")]),
                    Heading(1, vec![Text("b")]),
                    HorizontalLine,
                ]
            ))
        );
        assert_eq!(
            try_creoles(
                "= t

= A"
            ),
            Ok((
                "",
                vec![
                    Heading(1, vec![Text("t")]),
                    Paragraph(vec![]),
                    Heading(1, vec![Text("A")]),
                ]
            ))
        );
        //     // assert_eq!(try_creoles("a[[/|home]]{{a.jpg}}"), Ok(("", vec![
        //     //   Text("a"),
        //     //   Link("/", "home"),
        //     //   Image("a.jpg", ""),
        //     // ])));
    }

    /// Per-test-case time profiling. Excluded from normal runs; invoke with:
    /// `cargo test --features extended -- --ignored --nocapture time_profile`
    mod time_profile {
        use super::*;
        use std::time::Instant;

        fn report(name: &str, f: fn()) {
            const N: usize = 20_000;
            for _ in 0..200 {
                f();
            }
            let mut ns: Vec<u64> = Vec::with_capacity(N);
            let t0 = Instant::now();
            for _ in 0..N {
                let t = Instant::now();
                f();
                ns.push(t.elapsed().as_nanos() as u64);
            }
            let total = t0.elapsed().as_nanos() as f64;
            ns.sort_unstable();
            println!(
                "{:<16} n={:<6} mean={:>8.2}us  p50={:>8.2}us  min={:>8.2}us  p95={:>8.2}us  max={:>9.2}us  total={:>7.1}ms",
                name,
                N,
                total / N as f64 / 1e3,
                ns[N / 2] as f64 / 1e3,
                ns[0] as f64 / 1e3,
                ns[N * 95 / 100] as f64 / 1e3,
                ns[N - 1] as f64 / 1e3,
                total / 1e6
            );
        }

        #[test]
        #[ignore]
        fn time_profile() {
            report("text_tests", text_tests);
            report("parser_tests", parser_tests);
            report("heading_tests", heading_tests);
            report("text_style_tests", text_style_tests);
            report("link_tests", link_tests);
            report("image_tests", image_tests);
            report("linebreak_tests", linebreak_tests);
            report("list_tests", list_tests);
            report("table_tests", table_tests);
            report("other_tests", other_tests);
            report("mixed_tests", mixed_tests);
            #[cfg(feature = "extend")]
            report("extend_tests", extend_tests);
        }
    }
}
