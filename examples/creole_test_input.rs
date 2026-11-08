//! Parses the official WikiCreole 1.0 test corpus (`CreoleTestInput.txt`,
//! blank-line-separated markup blocks) with the real public API and prints
//! one AST summary per block. Serves as the upstream-driven smoke runner:
//! it must exit 0 with no panics and no unexpected empty parses.
//!
//! Run: `cargo run --release --example creole_test_input` (optionally pass a
//! corpus path as the first argument).

use creole_nom::prelude::*;

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "examples/CreoleTestInput.txt".to_string());
    let input =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {path}: {e}"));

    let mut blocks = 0usize;
    let mut empty = 0usize;
    for (i, block) in input.split("\n\n").enumerate() {
        if block.trim().is_empty() {
            continue;
        }
        blocks += 1;
        let ast = creoles(block);
        if ast.is_empty() {
            empty += 1;
            eprintln!("block {i}: EMPTY parse for {:?}", first_line(block));
        }
        println!(
            "block {i:>2} {:?} -> {}",
            first_line(block),
            ast_summary(&ast)
        );
    }

    // whole corpus in one shot: infallible entry point must not panic either
    let whole = creoles(&input);
    println!(
        "corpus: {blocks} blocks, {} top-level nodes, {empty} empty parses",
        whole.len()
    );
    if empty > 0 {
        std::process::exit(1);
    }
}

fn first_line(block: &str) -> String {
    let line = block.lines().next().unwrap_or("").trim();
    let cut: String = line.chars().take(40).collect();
    if line.chars().count() > 40 {
        format!("{cut}…")
    } else {
        cut
    }
}

/// Compact node-name tree so regressions are visible in the diff of output.
fn ast_summary(ast: &[ICreole]) -> String {
    let mut out = String::new();
    for node in ast {
        if !out.is_empty() {
            out.push('|');
        }
        match node {
            ICreole::Text(t) => {
                out.push_str(&format!("Text({})", t.chars().take(20).collect::<String>()))
            }
            ICreole::Bold(t) => out.push_str(&format!("Bold({})", ast_summary(t))),
            ICreole::Italic(t) => out.push_str(&format!("Italic({})", ast_summary(t))),
            ICreole::Link(h, t) => out.push_str(&format!("Link({h},{})", ast_summary(t))),
            ICreole::Image(h, title) => out.push_str(&format!("Image({h},{title})")),
            ICreole::Heading(l, t) => out.push_str(&format!("H{l}({})", ast_summary(t))),
            ICreole::BulletList(t) => out.push_str(&format!("UL({})", ast_summary(t))),
            ICreole::NumberedList(t) => out.push_str(&format!("OL({})", ast_summary(t))),
            ICreole::ListItem(t) => out.push_str(&format!("Li({})", ast_summary(t))),
            ICreole::Table(rows) => {
                out.push_str("Table(");
                for row in rows {
                    match row {
                        ICreole::TableHeaderRow(c) => {
                            out.push_str(&format!("h[{}]", ast_summary(c)))
                        }
                        ICreole::TableRow(c) => out.push_str(&format!("r[{}]", ast_summary(c))),
                        _ => out.push('?'),
                    }
                }
                out.push(')');
            }
            ICreole::TableCell(t) | ICreole::TableHeaderCell(t) => {
                out.push_str(&format!("C({})", ast_summary(t)));
            }
            ICreole::Paragraph(t) => out.push_str(&format!("P({})", ast_summary(t))),
            other => out.push_str(&format!("{other:?}")),
        }
    }
    out
}
