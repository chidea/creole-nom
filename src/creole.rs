//! The AST: [`ICreole`], a zero-copy enum whose variants borrow `&'a str`
//! slices straight out of the input.
//!
//! Nodes carrying inline content hold `Vec<ICreole<'a>>` children (a paragraph,
//! a list item, the inside of `**bold**`…); leaf/markup nodes hold raw slices.
//! `Serialize`/`Deserialize` round-trip the tree (useful for shipping it to a
//! renderer, e.g. the companion `wasm-creole-live-editor`); serde field names
//! are the variant names.

use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// One parsed node: block-level (`Paragraph`, `Heading`, `Table`, lists, …) or
/// inline (`Text`, `Bold`, `Link`, …). Feature-gated variants are produced only
/// when their Cargo feature is enabled — see `Cargo.toml`.
// #[cfg(not(feature = "html"))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ICreole<'a> {
    /// Literal text (everything between markup constructs).
    Text(&'a str),
    /// `<<IconName(text)>>` — extension macro name + inner text. `extend`.
    #[cfg(feature = "extend")]
    Extend(&'a str, &'a str),
    // #[cfg(feature = "extend")]
    // Icon(&'a str),
    /// A run of inline nodes; paragraphs are separated by blank lines.
    Paragraph(Vec<ICreole<'a>>),
    /// `**strong**`.
    Bold(Vec<ICreole<'a>>),
    /// `//emphasis//`.
    Italic(Vec<ICreole<'a>>),
    /// `[[color:red:text]]` — color + text. `font-color`. Constructible only:
    /// the parser is currently disabled (see `src/parser/inline.rs`).
    #[cfg(feature = "font-color")]
    Color(&'a str, &'a str),
    /// `* item` run; each child is a `ListItem`. Nesting by deeper `*`.
    BulletList(Vec<ICreole<'a>>),
    /// `# item` run; each child is a `ListItem`. Nesting by deeper `#`.
    NumberedList(Vec<ICreole<'a>>),
    /// One line inside a list; children are the item's inline nodes.
    ListItem(Vec<ICreole<'a>>),
    /// `[[href]]` / `[[href|text]]` — href + inline children for the label.
    Link(&'a str, Vec<ICreole<'a>>),
    /// `= h1 =` … level + inline children.
    Heading(u8, Vec<ICreole<'a>>),
    /// `~~` — renderers should drop it (soft break).
    Silentbreak,
    /// `\\` at end of line — hard line break.
    ForceLinebreak,
    /// `---` on its own line.
    HorizontalLine,
    /// `><<text>>` collapsed block content. `fold`.
    #[cfg(feature = "fold")]
    Fold(&'a str),
    /// `[[href|img:text]]` — href + alt/title text.
    Image(&'a str, &'a str),
    /// `[{href|text}]` button link — href + inline children. `link-button`.
    #[cfg(feature = "link-button")]
    LinkButton(&'a str, Vec<ICreole<'a>>),
    /// `{{{raw}}}` — preformatted, unparsed content.
    DontFormat(&'a str),
    /// A table; children are `TableHeaderRow`/`TableRow` nodes.
    Table(Vec<ICreole<'a>>),
    /// `|= h` row; children are `TableHeaderCell`.
    TableHeaderRow(Vec<ICreole<'a>>),
    /// One `|= h` cell; children are inline nodes.
    TableHeaderCell(Vec<ICreole<'a>>),
    /// `| body` row; children are `TableCell`.
    TableRow(Vec<ICreole<'a>>),
    /// One `| body` cell; children are inline nodes.
    TableCell(Vec<ICreole<'a>>),
    // from MoinMoin
    // BoldItalic(Vec<ICreole<'a>>),
    // Unterline(Vec<ICreole<'a>>),
    // Monospace(Vec<ICreole<'a>>),
}

/// Total equality; safe because the tree only ever holds `str` slices, integers
/// and child vectors — no floats.
impl Eq for ICreole<'_> {}
