#[cfg(test)]
use super::*;
use crate::creole::ICreole;

fn init() {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
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
        // `**` at line start is the bullet-list marker (spec precedence):
        // unclosed emphasis no longer falls back to plain paragraph text
        try_creoles("**bold,\nnot closed."),
        Ok((
            "",
            vec![
                BulletList(vec![BulletList(vec![ListItem(vec![Text("bold,")])])]),
                Paragraph(vec![Text("not closed.")])
            ]
        ))
    );
    assert_eq!(
        try_creoles("//italic,\nnot closed."),
        Ok(("", vec![Paragraph(vec![Text("//italic,\nnot closed.")])]))
    );

    assert_eq!(
        try_creoles("**a\nb**"),
        Ok(("", vec![Paragraph(vec![Bold(vec![Text("a\nb")])])]))
    );
    assert_eq!(
        try_creoles("//i\nj//"),
        Ok(("", vec![Paragraph(vec![Italic(vec![Text("i\nj")])])]))
    );
    assert_eq!(
        try_creoles("**//bi//**"),
        Ok((
            "",
            vec![Paragraph(vec![Bold(vec![Italic(vec![Text("bi")])])])]
        ))
    );
    assert_eq!(
        try_creoles("**a [[l|t]] b**"),
        Ok((
            "",
            vec![Paragraph(vec![Bold(vec![
                Text("a "),
                Link("l", vec![Text("t")]),
                Text(" b")
            ])])]
        ))
    );
    assert_eq!(
        try_creoles("**a {{i.png|t}} b**"),
        Ok((
            "",
            vec![Paragraph(vec![Bold(vec![
                Text("a "),
                Image("i.png", "t"),
                Text(" b")
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
        Ok((
            "",
            vec![Heading(1, vec![Text("a:"), Link("a", vec![Text("a")])])]
        ))
    );

    assert_eq!(heading("=== b"), Ok(("", Heading(3, vec![Text("b")]))));
    assert_eq!(heading("==== c"), Ok(("", Heading(4, vec![Text("c")]))));
    assert_eq!(creoles("== b"), vec![Heading(2, vec![Text("b")])]);
    assert_eq!(creoles("=== c"), vec![Heading(3, vec![Text("c")])]);

    assert_eq!(
        try_creoles("= [[a]]//a"),
        Ok((
            "",
            vec![Heading(1, vec![Link("a", vec![Text("a")]), Text("//a")])]
        ))
    );
    assert_eq!(
        try_creoles(
            "= [[http://www.wikicreole.org|Creole]] //Live// Editor ([[https://github.com/chidea/wasm-creole-live-editor|github]])"
        ),
        Ok((
            "",
            vec![Heading(
                1,
                vec![
                    Link("http://www.wikicreole.org", vec![Text("Creole")]),
                    Text(" "),
                    Italic(vec![Text("Live")]),
                    Text(" Editor ("),
                    Link(
                        "https://github.com/chidea/wasm-creole-live-editor",
                        vec![Text("github")]
                    ),
                    Text(")"),
                ]
            )]
        ))
    );

    assert_eq!(heading("== a ="), Ok(("", Heading(2, vec![Text("a")]))));
    assert_eq!(heading("=== a =="), Ok(("", Heading(3, vec![Text("a")]))));
}

#[test]
fn link_tests() {
    init();
    use ICreole::*;
    assert_eq!(
        creoles("[[a]]"),
        vec![Paragraph(vec![Link("a", vec![Text("a")])])]
    );
    assert_eq!(
        creoles("[[https://google.com|google]]"),
        vec![Paragraph(vec![Link(
            "https://google.com",
            vec![Text("google")]
        )])]
    );
    assert_eq!(link("[[a]]"), Ok(("", Link("a", vec![Text("a")]))));
    assert_eq!(creoles("[a]"), vec![Paragraph(vec![Text("[a]")])]);
    assert_eq!(
        link("[[link|label]]"),
        Ok(("", Link("link", vec![Text("label")])))
    );
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
    assert_eq!(
        creoles("[[A ~| pipe]]"),
        vec![Paragraph(vec![Link(
            "A ~| pipe",
            vec![Text("A "), Text("|"), Text(" pipe")]
        )])]
    );
    assert_eq!(link("[[a~|b|c]]"), Ok(("", Link("a~|b", vec![Text("c")]))));
}

#[test]
fn escape_tests() {
    init();
    use ICreole::*;
    assert_eq!(
        creoles("~**not bold**"),
        vec![Paragraph(vec![Text("**not bold**")])]
    );
    assert_eq!(
        creoles("~~esc ~~x"),
        vec![Paragraph(vec![Text("~esc "), Text("~x")])]
    );
    assert_eq!(creoles("~ bold?"), vec![Paragraph(vec![Text(" bold?")])]);
    assert_eq!(
        creoles("~- item\nplain"),
        vec![Paragraph(vec![Text("- item\nplain")])]
    );
    assert_eq!(creoles("~= t"), vec![Paragraph(vec![Text("= t")])]);
    assert_eq!(
        creoles("~~~|x"),
        vec![Paragraph(vec![Text("~"), Text("|x")])]
    );
    assert_eq!(
        creoles("**a\\\\b**"),
        vec![Paragraph(vec![Bold(vec![
            Text("a"),
            ForceLinebreak,
            Text("b")
        ])])]
    );
}

#[test]
fn adversarial_tests() {
    init();
    use ICreole::*;
    // deeper-than-two markers and unclosed runs degrade to text, not error
    assert_eq!(
        creoles("***c"),
        // one nesting level per marker char, unbounded, no error
        vec![BulletList(vec![BulletList(vec![BulletList(vec![
            ListItem(vec![Text("c")])
        ])])])]
    );
    assert_eq!(
        creoles("**a*]]b{{c"),
        vec![BulletList(vec![BulletList(vec![ListItem(vec![Text(
            "a*]]b{{c"
        )])])])]
    );
    for junk in ["[[[[[[[[[[", "]]]]]]]]]]", "{{{{{{{{{{", "}}}}}}}}}}"] {
        assert_eq!(creoles(junk), vec![Paragraph(vec![Text(junk)])]);
    }
    assert_eq!(creoles("= = = = ="), vec![Heading(1, vec![Text("= = =")])]);
    assert_eq!(
        creoles("*a\n**b\n***c"),
        vec![BulletList(vec![
            ListItem(vec![Text("a")]),
            BulletList(vec![
                ListItem(vec![Text("b")]),
                BulletList(vec![ListItem(vec![Text("c")])])
            ])
        ])]
    );
    assert_eq!(
        creoles("|| || ||"),
        vec![Table(vec![TableRow(vec![
            TableCell(vec![]),
            TableCell(vec![Text(" ")]),
            TableCell(vec![]),
            TableCell(vec![Text(" ")]),
            TableCell(vec![])
        ])])]
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

    assert_eq!(
        try_creoles("| x~|y |"),
        Ok((
            "",
            vec![Table(vec![TableRow(vec![TableCell(vec![
                Text(" x"),
                Text("|y ")
            ])])])]
        ))
    );
    assert_eq!(
        try_creoles("|= h~|=h2 |"),
        Ok((
            "",
            vec![Table(vec![TableHeaderRow(vec![TableHeaderCell(vec![
                Text(" h"),
                Text("|=h2 ")
            ])])])]
        ))
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
    assert_eq!(image("{{i.png|a~|b}}"), Ok(("", Image("i.png", "a~|b"))));
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

    fn report(name: &str, f: impl Fn()) {
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
        report("escape_tests", escape_tests);
        report("adversarial_tests", adversarial_tests);
        #[cfg(feature = "extend")]
        report("extend_tests", extend_tests);
    }

    /// Logos lexer vs nom pipeline on identical fixture strings:
    /// `lex vec` = full tokenization into Vec<Tok>, `lex scan` = DFA pass
    /// without materializing, `nom` = current full parse via creoles().
    #[test]
    #[ignore]
    fn time_profile_logos() {
        use crate::lexer::{lex_count, lex_vec};
        use std::hint::black_box;
        let samples: &[(&str, &str)] = &[
            ("text", "The quick brown fox jumps over the lazy dog 123"),
            ("style", "**bold** and //italic// mixed with plain text"),
            ("heading", "== A medium heading =="),
            (
                "link",
                "[[http://example.com|Example]] and [[Internal link]]",
            ),
            ("image", "{{img/photo.jpg|alt text}} inline {{b.png|}}"),
            (
                "list",
                "* item a\n** item b\n*** item c\n# ordered 1\n## ordered 2\n### deep",
            ),
            (
                "table",
                "|=Name|=Irene\n|=Relationship|Wife\n|=Children|Kat, Peter",
            ),
            (
                "mixed",
                "a [[http://x|t]] **b** //i// {{f.png|alt}}\\\ntail **B** ~~esc~~ end",
            ),
            ("cjk", "文本测试 **粗体** //斜体// [[链接|标题]] {{图|alt}}"),
        ];
        for (name, s) in samples {
            report(&format!("lex vec  {name}"), {
                let s = *s;
                move || {
                    black_box(&lex_vec(s));
                }
            });
            report(&format!("lex scan {name}"), {
                let s = *s;
                move || {
                    black_box(lex_count(s));
                }
            });
            report(&format!("nom      {name}"), {
                let s = *s;
                move || {
                    black_box(&creoles(s));
                }
            });
        }
    }
}
