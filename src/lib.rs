//! `creole-nom` parses [WikiCreole 1.0] wiki markup into a zero-copy AST using
//! [nom] 8: every node borrows `&'a str` slices of the input, nothing is allocated
//! except the `Vec` spine of each node's children.
//!
//! Two entry points, exposed through [`prelude`]:
//!
//! - [`creoles`](parser::creoles) — infallible convenience wrapper; swallows
//!   parse errors and returns `Vec::new()` on failure, one element per
//!   `\n`-separated block.
//! - [`try_creoles`](parser::try_creoles) — the real `nom::IResult` twin, for when
//!   you want the remainder or the error.
//!
//! ```
//! use creole_nom::prelude::{creoles, ICreole};
//! use ICreole::*;
//!
//! let nodes = creoles("= Title\n**bold** and //italic//");
//! assert_eq!(
//!     nodes,
//!     vec![
//!         Heading(1, vec![Text("Title")]),
//!         Paragraph(vec![
//!             Bold(vec![Text("bold")]),
//!             Text(" and "),
//!             Italic(vec![Text("italic")]),
//!         ]),
//!     ],
//! );
//! ```
//!
//! ```
//! use creole_nom::prelude::{try_creoles, ICreole};
//! use ICreole::*;
//!
//! let (rest, nodes) = try_creoles("plain text").unwrap();
//! assert_eq!(rest, "");
//! assert_eq!(nodes, vec![Paragraph(vec![Text("plain text")])]);
//! ```
//!
//! Markup that is optional in WikiCreole or nonstandard is gated behind Cargo
//! features (`extend`, `font-color`, `fold`, `link-button`; umbrella `extended`):
//! the corresponding [`ICreole`](creole::ICreole) variants exist only when built
//! with them, so match exhaustively against the feature set you compile with.
//!
//! [WikiCreole 1.0]: http://www.wikicreole.org/wiki/Creole1.0TestCases
//! [nom]: https://docs.rs/nom

pub mod creole;
pub mod lexer;
pub mod parser;

/// Everything needed for the common case: the two entry points and the AST type.
pub mod prelude {
    pub use crate::parser::{creoles, try_creoles};
    // pub use crate::creole::{Creole, Creoles, };
    pub use crate::creole::ICreole;
}
