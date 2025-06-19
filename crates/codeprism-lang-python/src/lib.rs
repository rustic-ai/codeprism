//! Python language support for codeprism

mod adapter;
mod analysis;
mod ast_mapper;
mod error;
mod parser;
mod types;

pub use adapter::{parse_file, ParseResultConverter, PythonLanguageParser};
pub use analysis::PythonAnalyzer;
pub use error::{Error, Result};
pub use parser::{ParseContext, ParseResult, PythonParser};
pub use types::{Edge, EdgeKind, Language, Node, NodeId, NodeKind, Span};

// Re-export the parser for registration
pub fn create_parser() -> PythonLanguageParser {
    PythonLanguageParser::new()
}
