use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[cfg(not(feature = "html"))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ICreole<'a> {
    Text(&'a str),
    Line(Vec<ICreole<'a>>),
    Bold(&'a str),
    Italic(&'a str),
    #[cfg(feature = "font-color")]
    Color(&'a str, &'a str),
    BulletList(Vec<ICreole<'a>>),
    NumberedList(Vec<ICreole<'a>>),
    ListItem(Vec<ICreole<'a>>),
    Link(&'a str, &'a str),
    Heading(u8, Vec<ICreole<'a>>),
    Silentbreak,
    ForceLinebreak,
    HorizontalLine,
    #[cfg(feature = "fold")]
    Fold(&'a str),
    Image(&'a str, &'a str),
    #[cfg(feature = "link-button")]
    LinkButton(&'a str, &'a str, &'a str),
    DontFormat(&'a str),
    Table(Vec<ICreole<'a>>),
    TableHeaderRow(Vec<ICreole<'a>>),
    TableHeaderCell(Vec<ICreole<'a>>),
    TableRow(Vec<ICreole<'a>>),
    TableCell(Vec<ICreole<'a>>),
}

impl Eq for ICreole<'_> {}
