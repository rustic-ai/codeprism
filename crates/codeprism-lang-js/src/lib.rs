//! JavaScript/TypeScript language support for codeprism

mod adapter;
mod analysis;
mod ast_mapper;
mod error;
mod parser;
mod types;

pub use adapter::{parse_file, JavaScriptLanguageParser, ParseResultConverter};
pub use analysis::{
    ComponentType, HookInfo, JavaScriptAnalyzer, ModernFeatureType, ModernJsFeatureInfo,
    NodeJsPatternInfo, NodePatternType, PropsInfo, ReactComponentInfo, RouteInfo,
};
pub use error::{Error, Result};
pub use parser::{JavaScriptParser, ParseContext, ParseResult};
pub use types::{Edge, EdgeKind, Language, Node, NodeId, NodeKind, Span};

// Re-export the parser for registration
pub fn create_parser() -> JavaScriptLanguageParser {
    JavaScriptLanguageParser::new()
}

// Re-export the analyzer for registration
pub fn create_analyzer() -> JavaScriptAnalyzer {
    JavaScriptAnalyzer::new()
}
