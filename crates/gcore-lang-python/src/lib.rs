//! Python language support for gcore

mod adapter;
mod ast_mapper;
mod error;
mod parser;
mod types;

pub use adapter::{parse_file, PythonLanguageParser, ParseResultConverter};
pub use error::{Error, Result};
pub use parser::{PythonParser, ParseContext, ParseResult};
pub use types::{Edge, EdgeKind, Language, Node, NodeId, NodeKind, Span};

// Re-export the parser for registration
pub fn create_parser() -> PythonLanguageParser {
    PythonLanguageParser::new()
}
