//! Content parsers for documentation and configuration files
//!
//! This module provides parsers for various non-code file formats including
//! markdown, configuration files, and plain text documents.

use super::{ConfigFormat, ContentChunk, ContentNode, ContentType, DocumentFormat};
use crate::ast::Span;
use anyhow::{anyhow, Result};
use regex::Regex;
use serde_json::Value;
use std::path::Path;

/// Document parser for various file formats
pub struct DocumentParser {
    /// Markdown parser
    markdown_parser: MarkdownParser,
    /// Configuration file parser
    config_parser: ConfigParser,
    /// Plain text parser
    text_parser: TextParser,
}

impl DocumentParser {
    /// Create a new document parser
    pub fn new() -> Self {
        Self {
            markdown_parser: MarkdownParser::new(),
            config_parser: ConfigParser::new(),
            text_parser: TextParser::new(),
        }
    }

    /// Parse a file based on its extension
    pub fn parse_file(&self, file_path: &Path, content: &str) -> Result<ContentNode> {
        let content_type = self.detect_content_type(file_path)?;
        let mut node = ContentNode::new(file_path.to_path_buf(), content_type.clone());

        let chunks = match content_type {
            ContentType::Documentation { format } => match format {
                DocumentFormat::Markdown => self.markdown_parser.parse(file_path, content)?,
                DocumentFormat::PlainText
                | DocumentFormat::RestructuredText
                | DocumentFormat::AsciiDoc
                | DocumentFormat::Html => self.text_parser.parse(file_path, content, format)?,
            },
            ContentType::Configuration { format } => {
                self.config_parser.parse(file_path, content, format)?
            }
            ContentType::PlainText => {
                self.text_parser
                    .parse(file_path, content, DocumentFormat::PlainText)?
            }
            _ => return Err(anyhow!("Unsupported content type for document parser")),
        };

        for chunk in chunks {
            node.add_chunk(chunk);
        }
        node.file_size = content.len();

        Ok(node)
    }

    /// Detect content type from file extension
    fn detect_content_type(&self, file_path: &Path) -> Result<ContentType> {
        // Handle special files without extensions first
        if let Some(file_name) = file_path.file_name().and_then(|n| n.to_str()) {
            if file_name == ".env" {
                return Ok(ContentType::Configuration {
                    format: ConfigFormat::Env,
                });
            }
        }

        let extension = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        match extension.as_str() {
            "md" | "markdown" => Ok(ContentType::Documentation {
                format: DocumentFormat::Markdown,
            }),
            "rst" => Ok(ContentType::Documentation {
                format: DocumentFormat::RestructuredText,
            }),
            "adoc" | "asciidoc" => Ok(ContentType::Documentation {
                format: DocumentFormat::AsciiDoc,
            }),
            "html" | "htm" => Ok(ContentType::Documentation {
                format: DocumentFormat::Html,
            }),
            "txt" | "text" => Ok(ContentType::Documentation {
                format: DocumentFormat::PlainText,
            }),
            "json" => Ok(ContentType::Configuration {
                format: ConfigFormat::Json,
            }),
            "yaml" | "yml" => Ok(ContentType::Configuration {
                format: ConfigFormat::Yaml,
            }),
            "toml" => Ok(ContentType::Configuration {
                format: ConfigFormat::Toml,
            }),
            "ini" => Ok(ContentType::Configuration {
                format: ConfigFormat::Ini,
            }),
            "properties" => Ok(ContentType::Configuration {
                format: ConfigFormat::Properties,
            }),
            "env" => Ok(ContentType::Configuration {
                format: ConfigFormat::Env,
            }),
            "xml" => Ok(ContentType::Configuration {
                format: ConfigFormat::Xml,
            }),
            _ => Ok(ContentType::PlainText),
        }
    }
}

impl Default for DocumentParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Markdown document parser
pub struct MarkdownParser {
    /// Regex for headers
    header_regex: Regex,
    /// Regex for code blocks
    code_block_regex: Regex,
    /// Regex for inline code
    #[allow(dead_code)] // Will be used for inline code extraction
    inline_code_regex: Regex,
    /// Regex for links
    #[allow(dead_code)] // Will be used for link extraction
    link_regex: Regex,
    /// Regex for lists
    #[allow(dead_code)] // Will be used for list extraction
    list_regex: Regex,
}

impl MarkdownParser {
    /// Create a new markdown parser
    pub fn new() -> Self {
        Self {
            header_regex: Regex::new(r"(?m)^(#{1,6})\s+(.+)$").unwrap(),
            code_block_regex: Regex::new(r"```(\w+)?\n([\s\S]*?)\n```").unwrap(),
            inline_code_regex: Regex::new(r"`([^`]+)`").unwrap(),
            link_regex: Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap(),
            list_regex: Regex::new(r"(?m)^[\s]*[-*+]\s+(.+)$").unwrap(),
        }
    }

    /// Parse markdown content into chunks
    pub fn parse(&self, file_path: &Path, content: &str) -> Result<Vec<ContentChunk>> {
        let mut chunks = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut _current_line = 0;
        let mut chunk_index = 0;

        // Parse headers
        for (line_idx, line) in lines.iter().enumerate() {
            if let Some(captures) = self.header_regex.captures(line) {
                let level = captures.get(1).unwrap().as_str().len();
                let header_text = captures.get(2).unwrap().as_str();

                let span = self.calculate_line_span(line_idx, line, content);
                let chunk = ContentChunk::new(
                    file_path.to_path_buf(),
                    ContentType::Documentation {
                        format: DocumentFormat::Markdown,
                    },
                    header_text.to_string(),
                    span,
                    chunk_index,
                )
                .with_metadata(serde_json::json!({
                    "header_level": level,
                    "element_type": "header"
                }));

                chunks.push(chunk);
                chunk_index += 1;
            }
        }

        // Parse code blocks
        for captures in self.code_block_regex.captures_iter(content) {
            let language = captures.get(1).map(|m| m.as_str()).unwrap_or("text");
            let code_content = captures.get(2).unwrap().as_str();
            let full_match = captures.get(0).unwrap();

            let span = self.calculate_match_span(&full_match, content);
            let chunk = ContentChunk::new(
                file_path.to_path_buf(),
                ContentType::Documentation {
                    format: DocumentFormat::Markdown,
                },
                code_content.to_string(),
                span,
                chunk_index,
            )
            .with_metadata(serde_json::json!({
                "language": language,
                "element_type": "code_block"
            }));

            chunks.push(chunk);
            chunk_index += 1;
        }

        // Parse regular paragraphs (non-header, non-code block content)
        let mut paragraph_start = 0;
        let mut in_paragraph = false;
        let mut paragraph_lines = Vec::new();

        for (line_idx, line) in lines.iter().enumerate() {
            let line_trimmed = line.trim();

            // Skip headers and lines that are part of code blocks
            if self.header_regex.is_match(line)
                || line_trimmed.starts_with("```")
                || line_trimmed.is_empty()
            {
                // End current paragraph if we have one
                if in_paragraph && !paragraph_lines.is_empty() {
                    let paragraph_text = paragraph_lines.join("\n");
                    let span =
                        self.calculate_paragraph_span(paragraph_start, line_idx - 1, content);

                    let chunk = ContentChunk::new(
                        file_path.to_path_buf(),
                        ContentType::Documentation {
                            format: DocumentFormat::Markdown,
                        },
                        paragraph_text,
                        span,
                        chunk_index,
                    )
                    .with_metadata(serde_json::json!({
                        "element_type": "paragraph"
                    }));

                    chunks.push(chunk);
                    chunk_index += 1;
                }

                in_paragraph = false;
                paragraph_lines.clear();
                continue;
            }

            // Start or continue paragraph
            if !in_paragraph {
                in_paragraph = true;
                paragraph_start = line_idx;
            }
            paragraph_lines.push(line_trimmed);
        }

        // Handle final paragraph
        if in_paragraph && !paragraph_lines.is_empty() {
            let paragraph_text = paragraph_lines.join("\n");
            let span = self.calculate_paragraph_span(paragraph_start, lines.len() - 1, content);

            let chunk = ContentChunk::new(
                file_path.to_path_buf(),
                ContentType::Documentation {
                    format: DocumentFormat::Markdown,
                },
                paragraph_text,
                span,
                chunk_index,
            )
            .with_metadata(serde_json::json!({
                "element_type": "paragraph"
            }));

            chunks.push(chunk);
        }

        Ok(chunks)
    }

    /// Calculate span for a single line
    fn calculate_line_span(&self, line_idx: usize, line: &str, content: &str) -> Span {
        let lines_before: usize = content.lines().take(line_idx).map(|l| l.len() + 1).sum();
        let start_byte = lines_before;
        let end_byte = start_byte + line.len();

        Span::new(
            start_byte,
            end_byte,
            line_idx + 1,
            line_idx + 1,
            1,
            line.len() + 1,
        )
    }

    /// Calculate span for a regex match
    fn calculate_match_span(&self, match_obj: &regex::Match, content: &str) -> Span {
        let start_byte = match_obj.start();
        let end_byte = match_obj.end();

        // Count lines up to start
        let content_before = &content[..start_byte];
        let start_line = content_before.lines().count();
        let start_column = content_before.lines().last().map(|l| l.len()).unwrap_or(0) + 1;

        // Count lines in match
        let match_content = match_obj.as_str();
        let lines_in_match = match_content.lines().count();
        let end_line = start_line + lines_in_match.saturating_sub(1);
        let end_column = if lines_in_match > 1 {
            match_content.lines().last().map(|l| l.len()).unwrap_or(0) + 1
        } else {
            start_column + match_content.len()
        };

        Span::new(
            start_byte,
            end_byte,
            start_line.max(1),
            end_line.max(1),
            start_column,
            end_column,
        )
    }

    /// Calculate span for a paragraph
    fn calculate_paragraph_span(&self, start_line: usize, end_line: usize, content: &str) -> Span {
        let lines: Vec<&str> = content.lines().collect();
        let start_byte: usize = lines
            .iter()
            .take(start_line)
            .map(|l| l.len() + 1)
            .sum::<usize>();
        let end_byte: usize = lines
            .iter()
            .take(end_line + 1)
            .map(|l| l.len() + 1)
            .sum::<usize>()
            - 1;

        Span::new(
            start_byte,
            end_byte,
            start_line + 1,
            end_line + 1,
            1,
            lines.get(end_line).map(|l| l.len()).unwrap_or(0) + 1,
        )
    }
}

impl Default for MarkdownParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration file parser
pub struct ConfigParser;

impl ConfigParser {
    /// Create a new configuration parser
    pub fn new() -> Self {
        Self
    }

    /// Parse configuration file content
    pub fn parse(
        &self,
        file_path: &Path,
        content: &str,
        format: ConfigFormat,
    ) -> Result<Vec<ContentChunk>> {
        match format {
            ConfigFormat::Json => self.parse_json(file_path, content),
            ConfigFormat::Yaml => self.parse_yaml(file_path, content),
            ConfigFormat::Toml => self.parse_toml(file_path, content),
            ConfigFormat::Ini => self.parse_ini(file_path, content),
            ConfigFormat::Properties => self.parse_properties(file_path, content),
            ConfigFormat::Env => self.parse_env(file_path, content),
            ConfigFormat::Xml => self.parse_xml(file_path, content),
        }
    }

    /// Parse JSON configuration
    fn parse_json(&self, file_path: &Path, content: &str) -> Result<Vec<ContentChunk>> {
        let mut chunks = Vec::new();

        // Try to parse as JSON to validate structure
        match serde_json::from_str::<Value>(content) {
            Ok(value) => {
                // Extract key-value pairs and create chunks
                self.extract_json_values(&value, file_path, content, &mut chunks, 0, "");
            }
            Err(_) => {
                // If JSON is invalid, treat as plain text
                chunks.push(
                    ContentChunk::new(
                        file_path.to_path_buf(),
                        ContentType::Configuration {
                            format: ConfigFormat::Json,
                        },
                        content.to_string(),
                        Span::new(
                            0,
                            content.len(),
                            1,
                            content.lines().count(),
                            1,
                            content.lines().last().map(|l| l.len()).unwrap_or(0),
                        ),
                        0,
                    )
                    .with_metadata(serde_json::json!({
                        "parse_error": true,
                        "config_type": "json"
                    })),
                );
            }
        }

        Ok(chunks)
    }

    /// Extract values from JSON recursively
    #[allow(clippy::only_used_in_recursion)] // Method is used recursively by design
    fn extract_json_values(
        &self,
        value: &Value,
        file_path: &Path,
        content: &str,
        chunks: &mut Vec<ContentChunk>,
        chunk_index: usize,
        key_path: &str,
    ) {
        match value {
            Value::Object(map) => {
                for (key, val) in map {
                    let new_path = if key_path.is_empty() {
                        key.clone()
                    } else {
                        format!("{key_path}.{key}")
                    };
                    self.extract_json_values(
                        val,
                        file_path,
                        content,
                        chunks,
                        chunks.len(),
                        &new_path,
                    );
                }
            }
            Value::Array(arr) => {
                for (index, val) in arr.iter().enumerate() {
                    let new_path = format!("{key_path}[{index}]");
                    self.extract_json_values(
                        val,
                        file_path,
                        content,
                        chunks,
                        chunks.len(),
                        &new_path,
                    );
                }
            }
            Value::String(_) | Value::Number(_) | Value::Bool(_) => {
                // Create a chunk for this key-value pair
                let value_str = match value {
                    Value::String(s) => s.clone(),
                    _ => value.to_string(),
                };

                // Include key in the searchable content
                let searchable_content = if key_path.is_empty() {
                    value_str.clone()
                } else {
                    format!("{key_path}: {value_str}")
                };

                // Try to find the approximate location in the original content
                if let Some(position) = content.find(&value_str) {
                    let lines_before = content[..position].lines().count();
                    let line_start = content[..position].rfind('\n').map(|i| i + 1).unwrap_or(0);
                    let column = position - line_start + 1;

                    let span = Span::new(
                        position,
                        position + value_str.len(),
                        lines_before.max(1),
                        lines_before.max(1),
                        column,
                        column + value_str.len(),
                    );

                    let chunk = ContentChunk::new(
                        file_path.to_path_buf(),
                        ContentType::Configuration {
                            format: ConfigFormat::Json,
                        },
                        searchable_content,
                        span,
                        chunk_index,
                    )
                    .with_metadata(serde_json::json!({
                        "key_path": key_path,
                        "value": value_str,
                        "value_type": match value {
                            Value::String(_) => "string",
                            Value::Number(_) => "number",
                            Value::Bool(_) => "boolean",
                            _ => "unknown"
                        },
                        "config_type": "json"
                    }));

                    chunks.push(chunk);
                }
            }
            Value::Null => {} // Skip null values
        }
    }

    /// Parse YAML configuration (simplified)
    fn parse_yaml(&self, file_path: &Path, content: &str) -> Result<Vec<ContentChunk>> {
        // Simple line-by-line parsing for YAML
        let mut chunks = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (line_idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Look for key-value pairs
            if let Some(colon_pos) = trimmed.find(':') {
                let key = trimmed[..colon_pos].trim();
                let value = trimmed[colon_pos + 1..].trim();

                if !value.is_empty() {
                    let span = self.calculate_line_span(line_idx, line, content);
                    let chunk = ContentChunk::new(
                        file_path.to_path_buf(),
                        ContentType::Configuration {
                            format: ConfigFormat::Yaml,
                        },
                        format!("{key}: {value}"),
                        span,
                        chunks.len(),
                    )
                    .with_metadata(serde_json::json!({
                        "key": key,
                        "value": value,
                        "config_type": "yaml"
                    }));

                    chunks.push(chunk);
                }
            }
        }

        Ok(chunks)
    }

    /// Parse TOML configuration (simplified)
    fn parse_toml(&self, file_path: &Path, content: &str) -> Result<Vec<ContentChunk>> {
        // Similar to YAML but with different syntax
        let mut chunks = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (line_idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Handle sections
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                let section = &trimmed[1..trimmed.len() - 1];
                let span = self.calculate_line_span(line_idx, line, content);
                let chunk = ContentChunk::new(
                    file_path.to_path_buf(),
                    ContentType::Configuration {
                        format: ConfigFormat::Toml,
                    },
                    section.to_string(),
                    span,
                    chunks.len(),
                )
                .with_metadata(serde_json::json!({
                    "element_type": "section",
                    "section_name": section,
                    "config_type": "toml"
                }));

                chunks.push(chunk);
                continue;
            }

            // Handle key-value pairs
            if let Some(eq_pos) = trimmed.find('=') {
                let key = trimmed[..eq_pos].trim();
                let value = trimmed[eq_pos + 1..].trim();

                let span = self.calculate_line_span(line_idx, line, content);
                let chunk = ContentChunk::new(
                    file_path.to_path_buf(),
                    ContentType::Configuration {
                        format: ConfigFormat::Toml,
                    },
                    format!("{key} = {value}"),
                    span,
                    chunks.len(),
                )
                .with_metadata(serde_json::json!({
                    "key": key,
                    "value": value,
                    "config_type": "toml"
                }));

                chunks.push(chunk);
            }
        }

        Ok(chunks)
    }

    /// Parse INI configuration
    fn parse_ini(&self, file_path: &Path, content: &str) -> Result<Vec<ContentChunk>> {
        // Similar pattern to TOML
        self.parse_key_value_format(file_path, content, ConfigFormat::Ini, "ini")
    }

    /// Parse properties configuration
    fn parse_properties(&self, file_path: &Path, content: &str) -> Result<Vec<ContentChunk>> {
        self.parse_key_value_format(file_path, content, ConfigFormat::Properties, "properties")
    }

    /// Parse environment file
    fn parse_env(&self, file_path: &Path, content: &str) -> Result<Vec<ContentChunk>> {
        self.parse_key_value_format(file_path, content, ConfigFormat::Env, "env")
    }

    /// Parse XML configuration (simplified)
    fn parse_xml(&self, file_path: &Path, content: &str) -> Result<Vec<ContentChunk>> {
        // Simple XML tag extraction without backreferences
        let tag_regex = Regex::new(r"<([^/>]+)>([^<]+)</[^>]+>").unwrap();
        let mut chunks = Vec::new();

        for (idx, captures) in tag_regex.captures_iter(content).enumerate() {
            let tag_name = captures.get(1).unwrap().as_str();
            let tag_content = captures.get(2).unwrap().as_str().trim();

            if !tag_content.is_empty() {
                let full_match = captures.get(0).unwrap();
                let span = self.calculate_match_span(&full_match, content);

                let chunk = ContentChunk::new(
                    file_path.to_path_buf(),
                    ContentType::Configuration {
                        format: ConfigFormat::Xml,
                    },
                    tag_content.to_string(),
                    span,
                    idx,
                )
                .with_metadata(serde_json::json!({
                    "tag_name": tag_name,
                    "config_type": "xml"
                }));

                chunks.push(chunk);
            }
        }

        Ok(chunks)
    }

    /// Generic key-value format parser
    fn parse_key_value_format(
        &self,
        file_path: &Path,
        content: &str,
        format: ConfigFormat,
        format_name: &str,
    ) -> Result<Vec<ContentChunk>> {
        let mut chunks = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (line_idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') {
                continue;
            }

            // Look for key=value pattern
            if let Some(eq_pos) = trimmed.find('=') {
                let key = trimmed[..eq_pos].trim();
                let value = trimmed[eq_pos + 1..].trim();

                let span = self.calculate_line_span(line_idx, line, content);
                let chunk = ContentChunk::new(
                    file_path.to_path_buf(),
                    ContentType::Configuration {
                        format: format.clone(),
                    },
                    format!("{key}={value}"),
                    span,
                    chunks.len(),
                )
                .with_metadata(serde_json::json!({
                    "key": key,
                    "value": value,
                    "config_type": format_name
                }));

                chunks.push(chunk);
            }
        }

        Ok(chunks)
    }

    /// Calculate span for a line
    fn calculate_line_span(&self, line_idx: usize, line: &str, content: &str) -> Span {
        let lines_before: usize = content.lines().take(line_idx).map(|l| l.len() + 1).sum();
        let start_byte = lines_before;
        let end_byte = start_byte + line.len();

        Span::new(
            start_byte,
            end_byte,
            line_idx + 1,
            line_idx + 1,
            1,
            line.len() + 1,
        )
    }

    /// Calculate span for a regex match
    fn calculate_match_span(&self, match_obj: &regex::Match, content: &str) -> Span {
        let start_byte = match_obj.start();
        let end_byte = match_obj.end();

        let content_before = &content[..start_byte];
        let start_line = content_before.lines().count();
        let start_column = content_before.lines().last().map(|l| l.len()).unwrap_or(0) + 1;

        let match_content = match_obj.as_str();
        let lines_in_match = match_content.lines().count();
        let end_line = start_line + lines_in_match.saturating_sub(1);
        let end_column = if lines_in_match > 1 {
            match_content.lines().last().map(|l| l.len()).unwrap_or(0) + 1
        } else {
            start_column + match_content.len()
        };

        Span::new(
            start_byte,
            end_byte,
            start_line.max(1),
            end_line.max(1),
            start_column,
            end_column,
        )
    }
}

impl Default for ConfigParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Plain text parser
pub struct TextParser;

impl TextParser {
    /// Create a new text parser
    pub fn new() -> Self {
        Self
    }

    /// Parse plain text into chunks (paragraph-based)
    pub fn parse(
        &self,
        file_path: &Path,
        content: &str,
        format: DocumentFormat,
    ) -> Result<Vec<ContentChunk>> {
        let mut chunks = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        let mut paragraph_start = 0;
        let mut paragraph_lines = Vec::new();
        let mut chunk_index = 0;

        for (line_idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            if trimmed.is_empty() {
                // End current paragraph
                if !paragraph_lines.is_empty() {
                    let paragraph_text = paragraph_lines.join("\n");
                    let span = self.calculate_paragraph_span(paragraph_start, line_idx - 1, &lines);

                    let chunk = ContentChunk::new(
                        file_path.to_path_buf(),
                        ContentType::Documentation {
                            format: format.clone(),
                        },
                        paragraph_text,
                        span,
                        chunk_index,
                    )
                    .with_metadata(serde_json::json!({
                        "element_type": "paragraph",
                        "line_count": paragraph_lines.len()
                    }));

                    chunks.push(chunk);
                    chunk_index += 1;
                    paragraph_lines.clear();
                }
                continue;
            }

            // Start new paragraph or continue existing one
            if paragraph_lines.is_empty() {
                paragraph_start = line_idx;
            }
            paragraph_lines.push(trimmed);
        }

        // Handle final paragraph
        if !paragraph_lines.is_empty() {
            let paragraph_text = paragraph_lines.join("\n");
            let span = self.calculate_paragraph_span(paragraph_start, lines.len() - 1, &lines);

            let chunk = ContentChunk::new(
                file_path.to_path_buf(),
                ContentType::Documentation { format },
                paragraph_text,
                span,
                chunk_index,
            )
            .with_metadata(serde_json::json!({
                "element_type": "paragraph",
                "line_count": paragraph_lines.len()
            }));

            chunks.push(chunk);
        }

        Ok(chunks)
    }

    /// Calculate span for a paragraph
    fn calculate_paragraph_span(&self, start_line: usize, end_line: usize, lines: &[&str]) -> Span {
        let start_byte: usize = lines
            .iter()
            .take(start_line)
            .map(|l| l.len() + 1)
            .sum::<usize>();
        let end_byte: usize = lines
            .iter()
            .take(end_line + 1)
            .map(|l| l.len() + 1)
            .sum::<usize>()
            - 1;

        Span::new(
            start_byte,
            end_byte,
            start_line + 1,
            end_line + 1,
            1,
            lines.get(end_line).map(|l| l.len()).unwrap_or(0) + 1,
        )
    }
}

impl Default for TextParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_parser_creation() {
        let _parser = DocumentParser::new();
        // Just test that creation doesn't panic
        // Test passes - parser handles empty files correctly
    }

    #[test]
    fn test_content_type_detection() {
        let parser = DocumentParser::new();

        let test_cases = vec![
            (
                "test.md",
                ContentType::Documentation {
                    format: DocumentFormat::Markdown,
                },
            ),
            (
                "README.markdown",
                ContentType::Documentation {
                    format: DocumentFormat::Markdown,
                },
            ),
            (
                "doc.rst",
                ContentType::Documentation {
                    format: DocumentFormat::RestructuredText,
                },
            ),
            (
                "manual.adoc",
                ContentType::Documentation {
                    format: DocumentFormat::AsciiDoc,
                },
            ),
            (
                "page.html",
                ContentType::Documentation {
                    format: DocumentFormat::Html,
                },
            ),
            (
                "notes.txt",
                ContentType::Documentation {
                    format: DocumentFormat::PlainText,
                },
            ),
            (
                "config.json",
                ContentType::Configuration {
                    format: ConfigFormat::Json,
                },
            ),
            (
                "config.yaml",
                ContentType::Configuration {
                    format: ConfigFormat::Yaml,
                },
            ),
            (
                "config.yml",
                ContentType::Configuration {
                    format: ConfigFormat::Yaml,
                },
            ),
            (
                "Cargo.toml",
                ContentType::Configuration {
                    format: ConfigFormat::Toml,
                },
            ),
            (
                "settings.ini",
                ContentType::Configuration {
                    format: ConfigFormat::Ini,
                },
            ),
            (
                "app.properties",
                ContentType::Configuration {
                    format: ConfigFormat::Properties,
                },
            ),
            (
                ".env",
                ContentType::Configuration {
                    format: ConfigFormat::Env,
                },
            ),
            (
                "config.xml",
                ContentType::Configuration {
                    format: ConfigFormat::Xml,
                },
            ),
            ("unknown.xyz", ContentType::PlainText),
        ];

        for (filename, expected_type) in test_cases {
            let path = Path::new(filename);
            let detected_type = parser.detect_content_type(path).unwrap();
            assert_eq!(
                std::mem::discriminant(&detected_type),
                std::mem::discriminant(&expected_type),
                "Failed for file: {filename}"
            );
        }
    }

    #[test]
    fn test_markdown_parser_headers() {
        let parser = MarkdownParser::new();
        let content = r#"# Main Title
Some content here.

## Secondary Title
More content.

### Subsection
Even more content.

#### Level 4
Content at level 4.

##### Level 5
Content at level 5.

###### Level 6
Content at level 6."#;

        let chunks = parser.parse(Path::new("test.md"), content).unwrap();

        // Should extract all headers
        let headers: Vec<_> = chunks
            .iter()
            .filter(|chunk| {
                if let Some(metadata) = chunk.metadata.as_object() {
                    metadata.get("element_type").and_then(|v| v.as_str()) == Some("header")
                } else {
                    false
                }
            })
            .collect();

        assert_eq!(headers.len(), 6, "Should find 6 headers");

        // Test header levels
        let header_levels: Vec<_> = headers
            .iter()
            .filter_map(|chunk| {
                chunk
                    .metadata
                    .as_object()
                    .and_then(|m| m.get("header_level"))
                    .and_then(|v| v.as_u64())
            })
            .collect();

        assert_eq!(header_levels, vec![1, 2, 3, 4, 5, 6]);
        assert_eq!(headers[0].content, "Main Title");
        assert_eq!(headers[1].content, "Secondary Title");
        assert_eq!(headers[2].content, "Subsection");
    }

    #[test]
    fn test_markdown_parser_code_blocks() {
        let parser = MarkdownParser::new();
        let content = r#"Here is some Python code:

```python
def hello_world():
    print("Hello, World!")
    return "success"
```

And here is some JavaScript:

```javascript
function greet(name) {
    console.log(`Hello, ${name}!`);
}
```

And a generic code block:

```
generic code here
no language specified
```"#;

        let chunks = parser.parse(Path::new("test.md"), content).unwrap();

        let code_blocks: Vec<_> = chunks
            .iter()
            .filter(|chunk| {
                if let Some(metadata) = chunk.metadata.as_object() {
                    metadata.get("element_type").and_then(|v| v.as_str()) == Some("code_block")
                } else {
                    false
                }
            })
            .collect();

        assert_eq!(code_blocks.len(), 3, "Should find 3 code blocks");

        // Test Python code block
        assert!(code_blocks[0].content.contains("def hello_world"));
        assert!(code_blocks[0].content.contains("print(\"Hello, World!\")"));
        let python_lang = code_blocks[0]
            .metadata
            .as_object()
            .unwrap()
            .get("language")
            .unwrap()
            .as_str()
            .unwrap();
        assert_eq!(python_lang, "python");

        // Test JavaScript code block
        assert!(code_blocks[1].content.contains("function greet"));
        let js_lang = code_blocks[1]
            .metadata
            .as_object()
            .unwrap()
            .get("language")
            .unwrap()
            .as_str()
            .unwrap();
        assert_eq!(js_lang, "javascript");

        // Test generic code block
        assert!(code_blocks[2].content.contains("generic code here"));
        let generic_lang = code_blocks[2]
            .metadata
            .as_object()
            .unwrap()
            .get("language")
            .unwrap()
            .as_str()
            .unwrap();
        assert_eq!(generic_lang, "text");
    }

    #[test]
    fn test_markdown_parser_paragraphs() {
        let parser = MarkdownParser::new();
        let content = r#"This is the first paragraph with some content.
It spans multiple lines.

This is the second paragraph.

# A Header

This is a paragraph after a header.

Another paragraph here."#;

        let chunks = parser.parse(Path::new("test.md"), content).unwrap();

        let paragraphs: Vec<_> = chunks
            .iter()
            .filter(|chunk| {
                if let Some(metadata) = chunk.metadata.as_object() {
                    metadata.get("element_type").and_then(|v| v.as_str()) == Some("paragraph")
                } else {
                    false
                }
            })
            .collect();

        assert!(paragraphs.len() >= 3, "Should find at least 3 paragraphs");
        assert!(paragraphs[0].content.contains("first paragraph"));
        assert!(paragraphs[1].content.contains("second paragraph"));
    }

    #[test]
    fn test_json_config_parser() {
        let parser = ConfigParser::new();
        let content = r#"{
  "database": {
    "host": "localhost",
    "port": 5432,
    "name": "myapp"
  },
  "features": ["auth", "logging", "metrics"],
  "debug": true,
  "version": "1.0.0"
}"#;

        let chunks = parser
            .parse(Path::new("config.json"), content, ConfigFormat::Json)
            .unwrap();

        assert!(!chunks.is_empty(), "Should extract chunks from JSON");

        // Should find various value types
        let string_chunks: Vec<_> = chunks
            .iter()
            .filter(|chunk| {
                if let Some(metadata) = chunk.metadata.as_object() {
                    metadata.get("value_type").and_then(|v| v.as_str()) == Some("string")
                } else {
                    false
                }
            })
            .collect();

        let boolean_chunks: Vec<_> = chunks
            .iter()
            .filter(|chunk| {
                if let Some(metadata) = chunk.metadata.as_object() {
                    metadata.get("value_type").and_then(|v| v.as_str()) == Some("boolean")
                } else {
                    false
                }
            })
            .collect();

        assert!(!string_chunks.is_empty(), "Should find string values");
        assert!(!boolean_chunks.is_empty(), "Should find boolean values");
    }

    #[test]
    fn test_yaml_config_parser() {
        let parser = ConfigParser::new();
        let content = r#"database:
  host: localhost
  port: 5432
  name: myapp

features:
  - auth
  - logging
  - metrics

debug: true
version: "1.0.0"
"#;

        let chunks = parser
            .parse(Path::new("config.yaml"), content, ConfigFormat::Yaml)
            .unwrap();

        assert!(!chunks.is_empty(), "Should extract chunks from YAML");

        // Should find key-value pairs
        let has_database = chunks
            .iter()
            .any(|chunk| chunk.content.contains("host: localhost"));
        let has_debug = chunks
            .iter()
            .any(|chunk| chunk.content.contains("debug: true"));

        assert!(has_database, "Should find database configuration");
        assert!(has_debug, "Should find debug setting");
    }

    #[test]
    fn test_toml_config_parser() {
        let parser = ConfigParser::new();
        let content = r#"[database]
host = "localhost"
port = 5432
name = "myapp"

[features]
auth = true
logging = true
metrics = false

debug = true
version = "1.0.0"
"#;

        let chunks = parser
            .parse(Path::new("Cargo.toml"), content, ConfigFormat::Toml)
            .unwrap();

        assert!(!chunks.is_empty(), "Should extract chunks from TOML");

        // Should find sections and key-value pairs
        let sections: Vec<_> = chunks
            .iter()
            .filter(|chunk| {
                if let Some(metadata) = chunk.metadata.as_object() {
                    metadata.get("element_type").and_then(|v| v.as_str()) == Some("section")
                } else {
                    false
                }
            })
            .collect();

        assert!(sections.len() >= 2, "Should find at least 2 sections");
        assert!(sections.iter().any(|s| s.content == "database"));
        assert!(sections.iter().any(|s| s.content == "features"));

        let key_values: Vec<_> = chunks
            .iter()
            .filter(|chunk| chunk.content.contains(" = "))
            .collect();

        assert!(!key_values.is_empty(), "Should find key-value pairs");
    }

    #[test]
    fn test_ini_config_parser() {
        let parser = ConfigParser::new();
        let content = r#"[database]
host=localhost
port=5432
name=myapp

[logging]
level=info
file=/var/log/app.log

debug=true
"#;

        let chunks = parser
            .parse(Path::new("config.ini"), content, ConfigFormat::Ini)
            .unwrap();

        assert!(!chunks.is_empty(), "Should extract chunks from INI");

        let key_values: Vec<_> = chunks
            .iter()
            .filter(|chunk| chunk.content.contains("="))
            .collect();

        assert!(
            key_values.len() >= 5,
            "Should find multiple key-value pairs"
        );
        assert!(key_values
            .iter()
            .any(|kv| kv.content.contains("host=localhost")));
        assert!(key_values
            .iter()
            .any(|kv| kv.content.contains("level=info")));
    }

    #[test]
    fn test_properties_config_parser() {
        let parser = ConfigParser::new();
        let content = r#"# Application configuration
database.host=localhost
database.port=5432
database.name=myapp

# Logging configuration  
logging.level=info
logging.file=/var/log/app.log

debug=true
"#;

        let chunks = parser
            .parse(
                Path::new("app.properties"),
                content,
                ConfigFormat::Properties,
            )
            .unwrap();

        assert!(!chunks.is_empty(), "Should extract chunks from properties");

        let properties: Vec<_> = chunks
            .iter()
            .filter(|chunk| chunk.content.contains("="))
            .collect();

        assert!(properties.len() >= 5, "Should find multiple properties");
        assert!(properties
            .iter()
            .any(|p| p.content.contains("database.host=localhost")));
        assert!(properties
            .iter()
            .any(|p| p.content.contains("logging.level=info")));
    }

    #[test]
    fn test_env_config_parser() {
        let parser = ConfigParser::new();
        let content = r#"DATABASE_HOST=localhost
DATABASE_PORT=5432
DATABASE_NAME=myapp
DEBUG=true
SECRET_KEY=abc123xyz
"#;

        let chunks = parser
            .parse(Path::new(".env"), content, ConfigFormat::Env)
            .unwrap();

        assert!(!chunks.is_empty(), "Should extract chunks from env file");

        let env_vars: Vec<_> = chunks
            .iter()
            .filter(|chunk| chunk.content.contains("="))
            .collect();

        assert_eq!(env_vars.len(), 5, "Should find 5 environment variables");
        assert!(env_vars
            .iter()
            .any(|var| var.content.contains("DATABASE_HOST=localhost")));
        assert!(env_vars
            .iter()
            .any(|var| var.content.contains("DEBUG=true")));
    }

    #[test]
    fn test_xml_config_parser() {
        let parser = ConfigParser::new();
        let content = r#"<configuration>
  <database>
    <host>localhost</host>
    <port>5432</port>
    <name>myapp</name>
  </database>
  <features>
    <auth>true</auth>
    <logging>true</logging>
  </features>
  <debug>true</debug>
</configuration>"#;

        let chunks = parser
            .parse(Path::new("config.xml"), content, ConfigFormat::Xml)
            .unwrap();

        assert!(!chunks.is_empty(), "Should extract chunks from XML");

        // Should find tag contents
        let tag_contents: Vec<_> = chunks
            .iter()
            .filter(|chunk| !chunk.content.trim().is_empty())
            .collect();

        assert!(!tag_contents.is_empty(), "Should find tag contents");
        assert!(tag_contents.iter().any(|tag| tag.content == "localhost"));
        assert!(tag_contents.iter().any(|tag| tag.content == "5432"));
        assert!(tag_contents.iter().any(|tag| tag.content == "true"));
    }

    #[test]
    fn test_text_parser_paragraphs() {
        let parser = TextParser::new();
        let content = r#"This is the first paragraph.
It has multiple lines.

This is the second paragraph.

This is the third paragraph.
It also has multiple lines.
And even more lines."#;

        let chunks = parser
            .parse(
                Path::new("document.txt"),
                content,
                DocumentFormat::PlainText,
            )
            .unwrap();

        assert_eq!(chunks.len(), 3, "Should find 3 paragraphs");

        assert!(chunks[0].content.contains("first paragraph"));
        assert!(chunks[1].content.contains("second paragraph"));
        assert!(chunks[2].content.contains("third paragraph"));

        // Check metadata
        for chunk in &chunks {
            let metadata = chunk.metadata.as_object().unwrap();
            assert_eq!(
                metadata.get("element_type").unwrap().as_str().unwrap(),
                "paragraph"
            );
            assert!(metadata.get("line_count").unwrap().as_u64().unwrap() >= 1);
        }
    }

    #[test]
    fn test_invalid_json_handling() {
        let parser = ConfigParser::new();
        let invalid_json = r#"{ invalid json content here"#;

        let chunks = parser
            .parse(Path::new("bad.json"), invalid_json, ConfigFormat::Json)
            .unwrap();

        assert_eq!(
            chunks.len(),
            1,
            "Should create a single chunk for invalid JSON"
        );
        assert_eq!(chunks[0].content, invalid_json);

        let metadata = chunks[0].metadata.as_object().unwrap();
        assert!(metadata.get("parse_error").unwrap().as_bool().unwrap());
        assert_eq!(
            metadata.get("config_type").unwrap().as_str().unwrap(),
            "json"
        );
    }

    #[test]
    fn test_empty_content_handling() {
        let parser = DocumentParser::new();

        let empty_md = "";
        let node = parser.parse_file(Path::new("empty.md"), empty_md).unwrap();

        assert_eq!(
            node.chunks.len(),
            0,
            "Empty content should produce no chunks"
        );
        assert_eq!(node.file_size, 0);
    }

    #[test]
    fn test_large_content_handling() {
        let parser = DocumentParser::new();

        // Create a large markdown document
        let mut content = String::new();
        for i in 0..100 {
            content.push_str(&format!(
                "# Header {i}\n\nThis is paragraph {i} with some content.\n\n"
            ));
        }

        let node = parser.parse_file(Path::new("large.md"), &content).unwrap();

        assert!(node.chunks.len() >= 100, "Should handle large content");
        assert_eq!(node.file_size, content.len());

        // Should find headers and paragraphs
        let headers = node
            .chunks
            .iter()
            .filter(|chunk| {
                if let Some(metadata) = chunk.metadata.as_object() {
                    metadata.get("element_type").and_then(|v| v.as_str()) == Some("header")
                } else {
                    false
                }
            })
            .count();

        assert!(headers >= 100, "Should find many headers");
    }

    #[test]
    fn test_content_span_calculation() {
        let parser = MarkdownParser::new();
        let content = "# Title\nSome content.";

        let chunks = parser.parse(Path::new("test.md"), content).unwrap();

        for chunk in chunks {
            assert!(
                chunk.span.start_byte < chunk.span.end_byte,
                "Start should be before end"
            );
            assert!(
                chunk.span.start_line <= chunk.span.end_line,
                "Start line should be <= end line"
            );
            assert!(chunk.span.start_column >= 1, "Column should be 1-indexed");
            assert!(
                chunk.span.end_byte <= content.len(),
                "End should not exceed content length"
            );
        }
    }
}
