//! Java language support for codeprism

mod adapter;
mod analysis;
mod ast_mapper;
mod error;
mod parser;
mod types;

pub use adapter::{parse_file, JavaLanguageParser, ParseResultConverter};
pub use analysis::{JavaAnalysisResult, JavaAnalyzer};
pub use error::{Error, Result};
pub use parser::{JavaParser, ParseContext, ParseResult};
pub use types::{Edge, EdgeKind, Language, Node, NodeId, NodeKind, Span};

// Re-export the parser for registration
pub fn create_parser() -> JavaLanguageParser {
    JavaLanguageParser::new()
}
