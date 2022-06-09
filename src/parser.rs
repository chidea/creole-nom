use {
    // log::{debug, error, info},
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
            // take, // blindly take by argument as count
            // take_till, // take before given function over input returns true
            // take_till1, // same, error on blank result
            take_until, // take before given pattern equals from remaining input
        },
        character::complete::{
            char,
            // not_line_ending, line_ending,
            // none_of,
            one_of,
            // space0, space1,
            // multispace0, multispace1,
        },
        combinator::{
            // consumed, // same, returns tuple (consumed input, parser output) as result
            map, // apply function to the result of parser
            // rest_len, // return length of all remaining input
            peek,
            // not, // success when parser fails
            // cond, // take a bool and run parser when it's a true
            // all_consuming, // succeed when given parser remains no rest input
            recognize, // return consumed input by parser when its succeed
            rest,      // return all remaining input
            success,
            // eof,
            // map_opt, // same, returns Option<_>
            // map_res, // same, returns Result<_>
            // map_parser, // parser after parser
            value,  // return first argument as parser result if parser succeed
            verify, // same, if given verify function returns true over parser result
        },
        // InputLength,
        // InputTakeAtPosition,
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
    },
};

fn bold(input: &str) -> IResult<&str, ICreole<'_>> {
    map(
        delimited(
            tag("**"),
            collect_opt_pair0(take_while_parser_fail_or(
                alt((peek(tag("**")), peek(tag("\n")))),
                italic,
                text,
            )),
            tag("**"),
        ),
        ICreole::Bold,
    )(input)
}
fn italic(input: &str) -> IResult<&str, ICreole<'_>> {
    map(
        delimited(
            tag("//"),
            collect_opt_pair0(take_while_parser_fail_or(
                alt((peek(tag("//")), peek(tag("\n")))),
                bold,
                text,
            )),
            tag("//"),
        ),
        ICreole::Italic,
    )(input)
}

fn text_style(input: &str) -> IResult<&str, ICreole<'_>> {
    alt((
        value(ICreole::ForceLinebreak, tag("\\\\")),
        bold,
        italic,
        // #[cfg(feature="font-color")]
        // font_color,
    ))(input)
}

fn list_head_char(input: &str) -> IResult<&str, char> {
    one_of("*#")(input)
}
fn list(input: &str) -> IResult<&str, ICreole> {
    let (_, head) = list_head_char(input)?;
    let (r, lines) = separated_list0(
        char('\n'),
        recognize(separated_pair(
            preceded(char(head), many0(list_head_char)),
            char(' '),
            alt((is_not("\n"), success(""))),
        )),
    )(input)?;

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
) -> impl FnMut(Vec<&'a str>) -> Result<ICreole, nom::Err<nom::error::Error<&'a str>>> {
    move |input: Vec<&str>| {
        let head_space = format!("{} ", head_tag);
        let mut rst = vec![];
        let mut to_skip = 0;
        for i in 0..input.len() {
            if to_skip > 0 {
                to_skip -= 1;
                continue;
            }
            let line = input[i];
            // debug!("list line : {}", line);
            if line.starts_with(&head_space) {
                // sibling
                if let Ok((_, v)) = map(
                    alt((
                        collect_opt_pair1(take_while_parser_fail(lit, text)),
                        success(vec![]),
                    )),
                    ICreole::ListItem,
                )(&line[head_space.len()..])
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

fn link(input: &str) -> IResult<&str, ICreole> {
    map(
        delimited(
            tag("[["),
            alt((
                separated_pair(is_not("[]|\n"), tag("|"), is_not("[]\n")),
                map(is_not("[]\n"), |v: &str| (v, v)),
            )),
            tag("]]"),
        ),
        |(link, label)| ICreole::Link(link, label),
    )(input)
}
fn image(input: &str) -> IResult<&str, ICreole> {
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
    )(input)
}

fn lit(input: &str) -> IResult<&str, ICreole> {
    alt((link, image, text_style))(input)
}
fn text(input: &str) -> IResult<&str, ICreole> {
    map(verify(rest, |s: &str| !s.is_empty()), |s: &str| {
        ICreole::Text(s)
    })(input)
}
fn take_lit_text_until0(
    until_tag: &'static str,
) -> impl FnMut(&str) -> IResult<&str, Vec<ICreole>> {
    move |input: &str| {
        collect_opt_pair0(take_while_parser_fail_or_peek_tag(until_tag, lit, text))(input)
    }
}

fn take_while_parser_fail<'a, F, G>(
    mut parser: F,
    mut fail_parser: G,
) -> impl FnMut(&'a str) -> IResult<&'a str, (Option<ICreole<'a>>, Option<ICreole<'a>>)>
where
    F: FnMut(&str) -> IResult<&str, ICreole>,
    G: FnMut(&'a str) -> IResult<&'a str, ICreole<'a>>,
{
    move |input: &str| {
        // debug!("take_while_parser_fail input : {}", input);
        // let mut i = input;
        let mut l = 0;
        for (i, c) in input.char_indices().by_ref() {
            // debug!("take_while_parser_fail i : {}", i);
            if let Ok((r, v)) = parser(&input[l..]) {
                return Ok((
                    r,
                    (
                        if l > 0 {
                            if let Ok((_, v)) = fail_parser(&input[..l]) {
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
            if let Ok((_, f)) = fail_parser(&input[..l]) {
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

fn collect_opt_pair0<'a, F>(
    mut parser: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, Vec<ICreole<'a>>>
where
    F: FnMut(&'a str) -> IResult<&'a str, (Option<ICreole<'a>>, Option<ICreole<'a>>)>,
{
    move |input: &str| {
        let mut rst = vec![];
        let mut i = input;
        // debug!("collect_opt_pair input : {}", i);
        while let Ok((r, t)) = parser(i) {
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
        Ok((i, rst))
    }
}
fn collect_opt_pair1<'a, F>(
    mut parser: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, Vec<ICreole<'a>>>
where
    F: FnMut(&'a str) -> IResult<&'a str, (Option<ICreole<'a>>, Option<ICreole<'a>>)>,
{
    move |input: &str| {
        let mut rst = vec![];
        let mut i = input;
        // debug!("collect_opt_pair input : {}", i);
        while let Ok((r, t)) = parser(i) {
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

fn take_while_parser_fail_or<'a, F, G, H>(
    mut term_parser: F,
    mut parser: G,
    mut fail_parser: H,
) -> impl FnMut(&'a str) -> IResult<&'a str, (Option<ICreole<'a>>, Option<ICreole<'a>>)>
where
    F: FnMut(&'a str) -> IResult<&'a str, &'a str>,
    G: FnMut(&'a str) -> IResult<&'a str, ICreole<'a>>,
    H: FnMut(&'a str) -> IResult<&'a str, ICreole<'a>>,
{
    move |input: &str| {
        let mut l = 0;
        for (i, c) in input.char_indices().by_ref() {
            if let Ok((i, _v)) = term_parser(&input[l..]) {
                return Ok((i, (fail_parser(&input[..l]).ok().map(|(_, v)| v), None)));
            } else if let Ok((r, v)) = parser(&input[l..]) {
                return Ok((r, (fail_parser(&input[..l]).ok().map(|(_, v)| v), Some(v))));
            } else {
                l = i + c.len_utf8();
            }
        }
        if l > 0 {
            if let Ok((_, f)) = fail_parser(&input[..l]) {
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

fn take_while_parser_fail_or_peek_tag<'a, F, G>(
    term_tag: &'static str,
    parser: F,
    fail_parser: G,
) -> impl FnMut(&'a str) -> IResult<&'a str, (Option<ICreole<'a>>, Option<ICreole<'a>>)>
where
    F: FnMut(&'a str) -> IResult<&'a str, ICreole<'a>> + Copy,
    G: FnMut(&'a str) -> IResult<&'a str, ICreole<'a>> + Copy,
{
    move |input: &str| take_while_parser_fail_or(peek(tag(term_tag)), parser, fail_parser)(input)
}

// #[cfg(feature="link-button")]
// fn link_button(input: &str) -> IResult<&str, ICreole> {
//   map(delimited(tag("[{"), alt((
//     separated_pair(is_not("|]}\n"), tag("|"), is_not("|]}\n")),
//     map(is_not("]}\n"), |label: &str| -> (&str, &str) { (label, label) }),
//   )), tag("]}")), |(link, label)| ICreole::LinkButton(label, link, ""))(input)
// }
// #[cfg(feature="font-color")]
// fn font_color(input: &str) -> IResult<&str, ICreole> {
//   map(delimited(tag("[{"), alt((
//     separated_pair(is_not("|]}\n"), tag("|"), is_not("|]}\n")),
//     map(is_not("]}\n"), |label: &str| -> (&str, &str) { (label, label) }),
//   )), tag("]}")), |(link, label)| ICreole::Link(label, link))(input)
// }

fn heading(input: &str) -> IResult<&str, ICreole> {
    map(
        separated_pair(
            map(many_m_n(1, 6, char('=')), |s| s.len()),
            char(' '),
            take_lit_text_until0("\n"),
        ),
        |(level, body)| ICreole::Heading(level as u8, body),
    )(input)
}
fn dont_format(input: &str) -> IResult<&str, ICreole> {
    map(
        delimited(tag("{{{\n"), take_until("\n}}}"), tag("\n}}}")),
        ICreole::DontFormat,
    )(input)
}

fn table_header_cell_inner(input: &str) -> IResult<&str, ICreole> {
    map(take_lit_text_until0("|"), ICreole::TableHeaderCell)(input)
}
fn table_cell_inner(input: &str) -> IResult<&str, ICreole> {
    map(take_lit_text_until0("|"), ICreole::TableCell)(input)
}
fn table_header_row(input: &str) -> IResult<&str, ICreole> {
    let (left, line) = preceded(
        tag("|="),
        map(
            alt((
                terminated(verify(is_not("\n"), |s: &str| s.ends_with('|')), tag("\n")),
                verify(rest, |s: &str| s.len() > 1 && s.ends_with('|')),
            )),
            |s: &str| &s[..(s.len() - 1)],
        ),
    )(input)?;
    // debug!("table_header_row line : {}", line);

    let (_, rst) = map(separated_list1(tag("|="), table_header_cell_inner), |v| {
        ICreole::TableHeaderRow(v)
    })(line)?;
    Ok((left, rst))
}
fn table_cell_row(input: &str) -> IResult<&str, ICreole> {
    let (left, line) = preceded(
        tag("|"),
        map(
            alt((
                verify(is_not("\n"), |s: &str| s.ends_with('|')),
                verify(rest, |s: &str| s.len() > 1 && s.ends_with('|')),
            )),
            |s: &str| &s[..(s.len() - 1)],
        ),
    )(input)?;
    // debug!("table_cell_row : {}", line);
    let (_, rst) = map(separated_list1(tag("|"), table_cell_inner), |v| {
        ICreole::TableRow(v)
    })(line)?;
    Ok((left, rst))
}
fn table(input: &str) -> IResult<&str, ICreole> {
    let mut rst = vec![];
    let mut rest = input;
    // debug!("table input : {}", input);
    if let Ok((rest, body)) = if let Ok((r, head)) = table_header_row(input) {
        // debug!("table head : {:?}", head);
        rst.push(head);
        rest = r;
        separated_list0(char('\n'), table_cell_row)(r)
    } else {
        // debug!("table head is empty");
        separated_list1(char('\n'), table_cell_row)(input)
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

// #[cfg(feature="fold")]
// fn fold(input: &str) -> IResult<&str, ICreole> {
//   map(map_parser(tag("---<"), rest), |rest| ICreole::Fold(rest))(input)
// }

fn line(input: &str) -> IResult<&str, ICreole> {
    map(
        collect_opt_pair0(take_while_parser_fail_or(
            alt((
                recognize(pair(char('\n'), peek(char('\n')))),
                recognize(peek(preceded(char('\n'), creole_inner))),
            )),
            lit,
            text,
        )),
        ICreole::Line,
    )(input)
}
fn creole_inner(input: &str) -> IResult<&str, ICreole> {
    alt((
        value(ICreole::HorizontalLine, tag("----")),
        heading,
        dont_format,
        table,
        list,
    ))(input)
}

pub fn try_creoles(input: &str) -> IResult<&str, Vec<ICreole>> {
    separated_list0(char('\n'), alt((creole_inner, line)))(input)
}

pub fn creoles(input: &str) -> Vec<ICreole> {
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
        assert_eq!(creoles("ab1"), vec![Line(vec![Text("ab1")])]);
    }

    #[test]
    fn text_style_tests() {
        init();
        use ICreole::*;
        assert_eq!(try_creoles("t"), Ok(("", vec![Line(vec![Text("t")])])));

        assert_eq!(
            try_creoles("**b**"),
            Ok(("", vec![Line(vec![Bold(vec![Text("b")])])]))
        );

        assert_eq!(
            try_creoles("//i//"),
            Ok(("", vec![Line(vec![Italic(vec![Text("i")])])]))
        );

        assert_eq!(
            try_creoles("a**b**//c//d"),
            Ok((
                "",
                vec![Line(vec![
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
                vec![Line(vec![Bold(vec![
                    Text("a"),
                    Italic(vec![Text("b")]),
                    Text("c")
                ])])]
            ))
        );
    }

    #[test]
    fn linebreak_tests() {
        init();
        use ICreole::*;
        assert_eq!(
            try_creoles("a\\\\b"),
            Ok(("", vec![Line(vec![Text("a"), ForceLinebreak, Text("b"),])]))
        );

        assert_eq!(creoles("a\\b"), vec![Line(vec![Text("a\\b")])]);

        assert_eq!(
            try_creoles("a\nb\n\nc"),
            Ok(("", vec![Line(vec![Text("a\nb")]), Line(vec![Text("c")])]))
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
                    ListItem(vec![Link("a", "a")]),
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
    }
    #[test]
    fn parser_tests() {
        init();
        use ICreole::*;
        assert_eq!(
            take_while_parser_fail(lit, text)("a b **a**"),
            Ok(("", (Some(Text("a b ")), Some(Bold(vec![Text("a")])),)))
        );
        assert_eq!(
            collect_opt_pair1(take_while_parser_fail(lit, text))("a b **a**"),
            Ok(("", vec![Text("a b "), Bold(vec![Text("a")]),]))
        );
        assert_eq!(
            collect_opt_pair1(take_while_parser_fail(creole_inner, text))("a\n= b"),
            Ok(("", vec![Text("a\n"), Heading(1, vec![Text("b")]),]))
        );
        assert_eq!(
            collect_opt_pair1(take_while_parser_fail(lit, text))("a b **a**"),
            Ok(("", vec![Text("a b "), Bold(vec![Text("a")]),]))
        );
        assert_eq!(
            collect_opt_pair1(take_while_parser_fail(lit, text))(
                "[[a|b]] //Live// Editor ([[c|d]])"
            ),
            Ok((
                "",
                vec![
                    // Line(vec![
                    Link("a", "b"),
                    Text(" "),
                    Italic(vec![Text("Live")]),
                    Text(" Editor ("),
                    Link("c", "d"),
                    Text(")"),
                    // ])
                ]
            ))
        );
    }

    #[test]
    fn heading_tests() {
        init();
        use ICreole::*;
        assert_eq!(heading("= "), Ok(("", Heading(1, vec![]))));
        assert_eq!(heading("= a"), Ok(("", Heading(1, vec![Text("a")]))));
        assert_eq!(
            heading("= [[a]]"),
            Ok(("", Heading(1, vec![Link("a", "a")])))
        );
        assert_eq!(
            heading("= a:[[a]]"),
            Ok(("", Heading(1, vec![Text("a:"), Link("a", "a")])))
        );
        assert_eq!(
            try_creoles("= a"),
            Ok(("", vec![Heading(1, vec![Text("a")])]))
        );
        assert_eq!(
            try_creoles("= a:[[a]]"),
            Ok(("", vec![Heading(1, vec![Text("a:"), Link("a", "a")])]))
        );

        assert_eq!(heading("=== b"), Ok(("", Heading(3, vec![Text("b")]))));
        assert_eq!(heading("==== c"), Ok(("", Heading(4, vec![Text("c")]))));
        assert_eq!(creoles("== b"), vec![Heading(2, vec![Text("b")])]);
        assert_eq!(creoles("=== c"), vec![Heading(3, vec![Text("c")])]);

        assert_eq!(
            try_creoles("= [[a]]//a"),
            Ok(("", vec![Heading(1, vec![Link("a", "a"), Text("//a")])]))
        );
        assert_eq!(try_creoles("= [[http://www.wikicreole.org|Creole]] //Live// Editor ([[https://github.com/chidea/wasm-creole-live-editor|github]])"), Ok(("", vec![
      Heading(1, vec![
        Link("http://www.wikicreole.org", "Creole"),
        Text(" "),
        Italic(vec![Text("Live")]),
        Text(" Editor ("),
        Link("https://github.com/chidea/wasm-creole-live-editor", "github"),
        Text(")"),
    ])])));
    }

    #[test]
    fn link_tests() {
        init();
        use ICreole::*;
        assert_eq!(creoles("[[a]]"), vec![Line(vec![Link("a", "a")])]);
        assert_eq!(
            creoles("[[https://google.com|google]]"),
            vec![Line(vec![Link("https://google.com", "google")])]
        );
        assert_eq!(link("[[a]]"), Ok(("", Link("a", "a"))));
        assert_eq!(creoles("[a]"), vec![Line(vec![Text("[a]")])]);
        assert_eq!(link("[[link|label]]"), Ok(("", Link("link", "label"))));
        assert_eq!(
            link("[[https://google.com|google]]"),
            Ok(("", Link("https://google.com", "google")))
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
|b|table|row|
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
                        TableCell(vec![Text("table")]),
                        TableCell(vec![Text("row")]),
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
    }

    #[test]
    fn image_tests() {
        init();
        use ICreole::*;
        assert_eq!(image("{{a.jpg}}"), Ok(("", Image("a.jpg", ""))));
        assert_eq!(image("{{a.jpg|label}}"), Ok(("", Image("a.jpg", "label"))));
        assert_eq!(creoles("{{a.jpg}}"), vec![Line(vec![Image("a.jpg", "")])]);
        assert_eq!(
            creoles("{{a.jpg|label}}"),
            vec![Line(vec![Image("a.jpg", "label")])]
        );

        assert_eq!(
            creoles("{{a.jpg|[[label]]}}"),
            vec![Line(vec![Image("a.jpg", "[[label]]")])]
        );
    }
    #[test]
    fn other_tests() {
        init();
        use ICreole::*;
        assert_eq!(try_creoles(""), Ok(("", vec![Line(vec![])])));
        assert_eq!(try_creoles("----"), Ok(("", vec![HorizontalLine])));
        assert_eq!(creoles("----"), vec![HorizontalLine]);
        // assert_eq!(creoles("----a"), vec![Line(vec![Text("----a")])]);
        assert_eq!(
            creoles("----\na"),
            vec![HorizontalLine, Line(vec![Text("a")])]
        );
        // assert_eq!(try_creoles("a\n----\nb"), Ok(("", vec![Line(vec![Text("a\n")]), HorizontalLine, Line(vec![Text("b")])])));
        // //     assert_eq!(creoles("{{a.jpg|b}}"), vec![Image("a.jpg", "b")]);
        //     assert_eq!(dont_format("{{{
        // == [[no]]:\n//**don't** format//
        // }}}"), Ok(("", DontFormat("\n== [[no]]:\n//**don't** format//"))));
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
                    Line(vec![]),
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
}
