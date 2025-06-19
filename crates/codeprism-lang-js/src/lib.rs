//! JavaScript/TypeScript language support for codeprism

mod adapter;
mod ast_mapper;
mod error;
mod parser;
mod types;

pub use adapter::{parse_file, JavaScriptLanguageParser, ParseResultConverter};
pub use error::{Error, Result};
pub use parser::{JavaScriptParser, ParseContext, ParseResult};
pub use types::{Edge, EdgeKind, Language, Node, NodeId, NodeKind, Span};

// Re-export the parser for registration
pub fn create_parser() -> JavaScriptLanguageParser {
    JavaScriptLanguageParser::new()
}
