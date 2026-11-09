//! Public-surface regression tests: only `creole_nom::prelude` is reachable here,
//! so every assertion exercises the published contract (`creoles` / `try_creoles` /
//! `ICreole`) — never private combinators. Fixtures mirror the README support matrix.
use creole_nom::prelude::*;

#[test]
fn text_and_paragraphs() {
    use ICreole::*;
    assert_eq!(creoles("ab1"), vec![Paragraph(vec![Text("ab1")])]);
    assert_eq!(try_creoles("t"), Ok(("", vec![Paragraph(vec![Text("t")])])));
    // empty input yields one empty paragraph, matching the blank-line convention
    assert_eq!(try_creoles(""), Ok(("", vec![Paragraph(vec![])])));
    assert_eq!(
        creoles("a\nb\n\nc"),
        vec![Paragraph(vec![Text("a\nb")]), Paragraph(vec![Text("c")])]
    );
}

#[test]
fn text_styles() {
    use ICreole::*;
    assert_eq!(
        creoles("**b**"),
        vec![Paragraph(vec![Bold(vec![Text("b")])])]
    );
    assert_eq!(
        creoles("//i//"),
        vec![Paragraph(vec![Italic(vec![Text("i")])])]
    );
    assert_eq!(
        creoles("a**b**//c//d"),
        vec![Paragraph(vec![
            Text("a"),
            Bold(vec![Text("b")]),
            Italic(vec![Text("c")]),
            Text("d"),
        ])]
    );
    assert_eq!(
        creoles("**a//b//c**"),
        vec![Paragraph(vec![Bold(vec![
            Text("a"),
            Italic(vec![Text("b")]),
            Text("c"),
        ])])]
    );
    // space-less `**` is a bold list (spec), not unclosed emphasis
    assert_eq!(
        creoles("**bold,\nnot closed."),
        vec![
            BulletList(vec![BulletList(vec![ListItem(vec![Text("bold,")])])]),
            Paragraph(vec![Text("not closed.")]),
        ]
    );
}

#[test]
fn linebreaks() {
    use ICreole::*;
    assert_eq!(
        creoles("a\\\\b"),
        vec![Paragraph(vec![Text("a"), ForceLinebreak, Text("b")])]
    );
    assert_eq!(creoles("a\\b"), vec![Paragraph(vec![Text("a\\b")])]);
}

#[test]
fn links() {
    use ICreole::*;
    assert_eq!(
        creoles("[[a]]"),
        vec![Paragraph(vec![Link("a", vec![Text("a")])])]
    );
    assert_eq!(
        creoles("[[href|name]]"),
        vec![Paragraph(vec![Link("href", vec![Text("name")])])]
    );
    assert_eq!(
        creoles("//[[a|b]]//"),
        vec![Paragraph(vec![Italic(vec![Link("a", vec![Text("b")])])])]
    );
    // single brackets are not a link
    assert_eq!(creoles("[a]"), vec![Paragraph(vec![Text("[a]")])]);
}

#[test]
fn headings() {
    use ICreole::*;
    assert_eq!(creoles("== b"), vec![Heading(2, vec![Text("b")])]);
    assert_eq!(creoles("=== c"), vec![Heading(3, vec![Text("c")])]);
}

#[test]
fn horizontal_rule() {
    use ICreole::*;
    assert_eq!(creoles("----"), vec![HorizontalLine]);
    assert_eq!(
        creoles("a\n----\nb"),
        vec![
            Paragraph(vec![Text("a")]),
            HorizontalLine,
            Paragraph(vec![Text("b")])
        ]
    );
}

#[test]
fn images() {
    use ICreole::*;
    assert_eq!(
        creoles("{{a.jpg}}"),
        vec![Paragraph(vec![Image("a.jpg", "")])]
    );
    assert_eq!(
        creoles("{{a.jpg|label}}"),
        vec![Paragraph(vec![Image("a.jpg", "label")])]
    );
}

#[test]
fn lists() {
    use ICreole::*;
    assert_eq!(
        creoles("* a\n* b"),
        vec![BulletList(vec![
            ListItem(vec![Text("a")]),
            ListItem(vec![Text("b")]),
        ])]
    );
    assert_eq!(
        creoles("# 1\n# 2"),
        vec![NumberedList(vec![
            ListItem(vec![Text("1")]),
            ListItem(vec![Text("2")]),
        ])]
    );
    assert_eq!(
        creoles("* a\n** b"),
        vec![BulletList(vec![
            ListItem(vec![Text("a")]),
            BulletList(vec![ListItem(vec![Text("b")])]),
        ])]
    );
}

#[test]
fn tables() {
    use ICreole::*;
    assert_eq!(
        creoles("|=|=a|=b|\n|0|1|2|"),
        vec![Table(vec![
            TableHeaderRow(vec![
                TableHeaderCell(vec![]),
                TableHeaderCell(vec![Text("a")]),
                TableHeaderCell(vec![Text("b")]),
            ]),
            TableRow(vec![
                TableCell(vec![Text("0")]),
                TableCell(vec![Text("1")]),
                TableCell(vec![Text("2")]),
            ]),
        ])]
    );
    // inline markup inside cells
    assert_eq!(
        creoles("|=h|\n|[[a]]|**b**|"),
        vec![Table(vec![
            TableHeaderRow(vec![TableHeaderCell(vec![Text("h")])]),
            TableRow(vec![
                TableCell(vec![Link("a", vec![Text("a")])]),
                TableCell(vec![Bold(vec![Text("b")])]),
            ]),
        ])]
    );
}

#[test]
fn nowiki() {
    use ICreole::*;
    assert_eq!(
        creoles("x {{{ //a// **b** }}} y"),
        vec![Paragraph(vec![
            Text("x "),
            DontFormat(" //a// **b** "),
            Text(" y"),
        ])]
    );
    assert_eq!(
        creoles("A paragraph.\n\n{{{\n//Some bold//\n}}}\n\nAnother."),
        vec![
            Paragraph(vec![Text("A paragraph.")]),
            DontFormat("\n//Some bold//\n"),
            Paragraph(vec![Text("\nAnother.")]),
        ]
    );
}

#[cfg(feature = "tex")]
#[test]
fn inline_tex() {
    use ICreole::*;
    assert_eq!(
        creoles("mass $$E=mc^2$$ now"),
        vec![Paragraph(vec![
            Text("mass "),
            InlineTex("E=mc^2"),
            Text(" now"),
        ])]
    );
    // without a closing `$$` the run stays text
    assert_eq!(
        creoles("$$unclosed"),
        vec![Paragraph(vec![Text("$$unclosed")])]
    );
}
