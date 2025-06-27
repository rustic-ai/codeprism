//! Interactive development REPL for parser development

use crate::{AstVisualizer, GraphVizExporter, ParserValidator, PerformanceProfiler};
use anyhow::Result;
use colored::Colorize;
use std::io::{self, Write};

/// Interactive development REPL
#[derive(Debug)]
pub struct DevRepl {
    language: Option<String>,
    history: Vec<String>,
    current_source: Option<String>,
    visualizer: Option<AstVisualizer>,
    validator: Option<ParserValidator>,
    profiler: Option<PerformanceProfiler>,
    exporter: Option<GraphVizExporter>,
    prompt: String,
}

/// REPL command types
#[derive(Debug, Clone)]
pub enum ReplCommand {
    Parse {
        source: String,
    },
    Load {
        file_path: String,
    },
    Show {
        what: ShowTarget,
    },
    Set {
        option: String,
        value: String,
    },
    Export {
        format: ExportFormat,
        output: Option<String>,
    },
    Compare {
        old_source: String,
        new_source: String,
    },
    Profile {
        command: String,
    },
    Help,
    Clear,
    History,
    Exit,
    Unknown {
        input: String,
    },
}

/// What to show in the REPL
#[derive(Debug, Clone)]
pub enum ShowTarget {
    Ast,
    Nodes,
    Edges,
    Stats,
    Tree,
    Validation,
    Performance,
    Config,
}

/// Export format options
#[derive(Debug, Clone)]
pub enum ExportFormat {
    GraphViz,
    Json,
    Csv,
    Tree,
}

/// Result of executing a REPL command
#[derive(Debug)]
pub struct ReplResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

impl DevRepl {
    /// Create a new development REPL
    pub fn new(language: Option<&str>) -> Result<Self> {
        Ok(Self {
            language: language.map(|s| s.to_string()),
            history: Vec::new(),
            current_source: None,
            visualizer: None,
            validator: None,
            profiler: None,
            exporter: None,
            prompt: "codeprism> ".to_string(),
        })
    }

    /// Set the AST visualizer
    pub fn set_visualizer(&mut self, visualizer: AstVisualizer) {
        self.visualizer = Some(visualizer);
    }

    /// Set the parser validator
    pub fn set_validator(&mut self, validator: ParserValidator) {
        self.validator = Some(validator);
    }

    /// Set the performance profiler
    pub fn set_profiler(&mut self, profiler: PerformanceProfiler) {
        self.profiler = Some(profiler);
    }

    /// Set the GraphViz exporter
    pub fn set_exporter(&mut self, exporter: GraphVizExporter) {
        self.exporter = Some(exporter);
    }

    /// Run the interactive REPL
    pub async fn run(&mut self) -> Result<()> {
        self.print_welcome();
        self.print_help();

        loop {
            match self.read_command().await {
                Ok(command) => {
                    if matches!(command, ReplCommand::Exit) {
                        break;
                    }

                    let result = self.execute_command(command).await;
                    self.print_result(&result);
                }
                Err(e) => {
                    eprintln!("Error reading command: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Print welcome message
    fn print_welcome(&self) {
        println!("{}", "CodePrism Parser Development REPL".bold().blue());
        println!("{}", "====================================".blue());
        if let Some(ref lang) = self.language {
            println!("Language: {}", lang.green());
        }
        println!("Type 'help' for available commands or 'exit' to quit.\n");
    }

    /// Print help information
    fn print_help(&self) {
        println!("{}", "Available Commands:".bold());
        println!("  {} <code>           - Parse source code", "parse".cyan());
        println!(
            "  {} <file>           - Load source from file",
            "load".cyan()
        );
        println!(
            "  {} <target>         - Show AST, nodes, edges, stats, etc.",
            "show".cyan()
        );
        println!(
            "  {} <opt> <val>      - Set configuration option",
            "set".cyan()
        );
        println!(
            "  {} <fmt> [file]     - Export to GraphViz, JSON, etc.",
            "export".cyan()
        );
        println!(
            "  {} <old> <new>      - Compare two code snippets",
            "compare".cyan()
        );
        println!(
            "  {} <cmd>            - Profile parsing performance",
            "profile".cyan()
        );
        println!(
            "  {}                  - Show command history",
            "history".cyan()
        );
        println!("  {}                  - Clear screen", "clear".cyan());
        println!("  {}                  - Show this help", "help".cyan());
        println!("  {}                  - Exit REPL", "exit".cyan());
        println!();
    }

    /// Read a command from the user
    async fn read_command(&mut self) -> Result<ReplCommand> {
        print!("{}", self.prompt);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_string();

        if !input.is_empty() {
            self.history.push(input.clone());
        }

        Ok(self.parse_command(&input))
    }

    /// Parse a command string
    fn parse_command(&self, input: &str) -> ReplCommand {
        let parts: Vec<&str> = input.split_whitespace().collect();

        if parts.is_empty() {
            return ReplCommand::Unknown {
                input: input.to_string(),
            };
        }

        match parts[0].to_lowercase().as_str() {
            "parse" => {
                if parts.len() > 1 {
                    let source = parts[1..].join(" ");
                    ReplCommand::Parse { source }
                } else {
                    ReplCommand::Unknown {
                        input: input.to_string(),
                    }
                }
            }
            "load" => {
                if parts.len() > 1 {
                    ReplCommand::Load {
                        file_path: parts[1].to_string(),
                    }
                } else {
                    ReplCommand::Unknown {
                        input: input.to_string(),
                    }
                }
            }
            "show" => {
                if parts.len() > 1 {
                    let target = match parts[1].to_lowercase().as_str() {
                        "ast" => ShowTarget::Ast,
                        "nodes" => ShowTarget::Nodes,
                        "edges" => ShowTarget::Edges,
                        "stats" => ShowTarget::Stats,
                        "tree" => ShowTarget::Tree,
                        "validation" => ShowTarget::Validation,
                        "performance" => ShowTarget::Performance,
                        "config" => ShowTarget::Config,
                        _ => {
                            return ReplCommand::Unknown {
                                input: input.to_string(),
                            }
                        }
                    };
                    ReplCommand::Show { what: target }
                } else {
                    ReplCommand::Unknown {
                        input: input.to_string(),
                    }
                }
            }
            "set" => {
                if parts.len() > 2 {
                    ReplCommand::Set {
                        option: parts[1].to_string(),
                        value: parts[2..].join(" "),
                    }
                } else {
                    ReplCommand::Unknown {
                        input: input.to_string(),
                    }
                }
            }
            "export" => {
                if parts.len() > 1 {
                    let format = match parts[1].to_lowercase().as_str() {
                        "graphviz" | "dot" => ExportFormat::GraphViz,
                        "json" => ExportFormat::Json,
                        "csv" => ExportFormat::Csv,
                        "tree" => ExportFormat::Tree,
                        _ => {
                            return ReplCommand::Unknown {
                                input: input.to_string(),
                            }
                        }
                    };
                    let output = if parts.len() > 2 {
                        Some(parts[2].to_string())
                    } else {
                        None
                    };
                    ReplCommand::Export { format, output }
                } else {
                    ReplCommand::Unknown {
                        input: input.to_string(),
                    }
                }
            }
            "compare" => {
                if parts.len() > 2 {
                    ReplCommand::Compare {
                        old_source: parts[1].to_string(),
                        new_source: parts[2..].join(" "),
                    }
                } else {
                    ReplCommand::Unknown {
                        input: input.to_string(),
                    }
                }
            }
            "profile" => {
                if parts.len() > 1 {
                    ReplCommand::Profile {
                        command: parts[1..].join(" "),
                    }
                } else {
                    ReplCommand::Unknown {
                        input: input.to_string(),
                    }
                }
            }
            "help" => ReplCommand::Help,
            "clear" => ReplCommand::Clear,
            "history" => ReplCommand::History,
            "exit" | "quit" | "q" => ReplCommand::Exit,
            _ => ReplCommand::Unknown {
                input: input.to_string(),
            },
        }
    }

    /// Execute a REPL command
    async fn execute_command(&mut self, command: ReplCommand) -> ReplResult {
        match command {
            ReplCommand::Parse { source } => self.handle_parse(&source).await,
            ReplCommand::Load { file_path } => self.handle_load(&file_path).await,
            ReplCommand::Show { what } => self.handle_show(what).await,
            ReplCommand::Set { option, value } => self.handle_set(&option, &value).await,
            ReplCommand::Export { format, output } => self.handle_export(format, output).await,
            ReplCommand::Compare {
                old_source,
                new_source,
            } => self.handle_compare(&old_source, &new_source).await,
            ReplCommand::Profile { command } => self.handle_profile(&command).await,
            ReplCommand::Help => self.handle_help().await,
            ReplCommand::Clear => self.handle_clear().await,
            ReplCommand::History => self.handle_history().await,
            ReplCommand::Exit => ReplResult {
                success: true,
                output: "Goodbye!".to_string(),
                error: None,
            },
            ReplCommand::Unknown { input } => ReplResult {
                success: false,
                output: String::new(),
                error: Some(format!(
                    "Unknown command: '{}'. Type 'help' for available commands.",
                    input
                )),
            },
        }
    }

    /// Handle parse command
    async fn handle_parse(&mut self, source: &str) -> ReplResult {
        // This is a simplified implementation
        // In a real REPL, this would use the actual parser
        self.current_source = Some(source.to_string());

        ReplResult {
            success: true,
            output: format!("Parsed source: '{}' (mock implementation)", source),
            error: None,
        }
    }

    /// Handle load command
    async fn handle_load(&mut self, file_path: &str) -> ReplResult {
        match std::fs::read_to_string(file_path) {
            Ok(content) => {
                self.current_source = Some(content.clone());
                ReplResult {
                    success: true,
                    output: format!("Loaded {} bytes from '{}'", content.len(), file_path),
                    error: None,
                }
            }
            Err(e) => ReplResult {
                success: false,
                output: String::new(),
                error: Some(format!("Failed to load file '{}': {}", file_path, e)),
            },
        }
    }

    /// Handle show command
    async fn handle_show(&self, what: ShowTarget) -> ReplResult {
        match what {
            ShowTarget::Ast => {
                if let Some(ref source) = self.current_source {
                    ReplResult {
                        success: true,
                        output: format!("AST for source (simplified):\n{}", source),
                        error: None,
                    }
                } else {
                    ReplResult {
                        success: false,
                        output: String::new(),
                        error: Some("No source loaded. Use 'parse' or 'load' first.".to_string()),
                    }
                }
            }
            ShowTarget::Config => {
                let config_info = format!(
                    "REPL Configuration:\n- Language: {:?}\n- Prompt: {}\n- History size: {}",
                    self.language,
                    self.prompt,
                    self.history.len()
                );
                ReplResult {
                    success: true,
                    output: config_info,
                    error: None,
                }
            }
            _ => ReplResult {
                success: true,
                output: format!("Show {:?} - not yet implemented", what),
                error: None,
            },
        }
    }

    /// Handle set command
    async fn handle_set(&mut self, option: &str, value: &str) -> ReplResult {
        match option.to_lowercase().as_str() {
            "prompt" => {
                self.prompt = value.to_string();
                ReplResult {
                    success: true,
                    output: format!("Prompt set to '{}'", value),
                    error: None,
                }
            }
            "language" => {
                self.language = Some(value.to_string());
                ReplResult {
                    success: true,
                    output: format!("Language set to '{}'", value),
                    error: None,
                }
            }
            _ => ReplResult {
                success: false,
                output: String::new(),
                error: Some(format!("Unknown option: '{}'", option)),
            },
        }
    }

    /// Handle export command
    async fn handle_export(&self, format: ExportFormat, output: Option<String>) -> ReplResult {
        let output_desc = output.as_deref().unwrap_or("stdout");
        ReplResult {
            success: true,
            output: format!(
                "Export to {:?} format -> {} (not yet implemented)",
                format, output_desc
            ),
            error: None,
        }
    }

    /// Handle compare command
    async fn handle_compare(&self, old_source: &str, new_source: &str) -> ReplResult {
        ReplResult {
            success: true,
            output: format!(
                "Compare '{}' vs '{}' (not yet implemented)",
                old_source, new_source
            ),
            error: None,
        }
    }

    /// Handle profile command
    async fn handle_profile(&mut self, command: &str) -> ReplResult {
        ReplResult {
            success: true,
            output: format!("Profile command '{}' (not yet implemented)", command),
            error: None,
        }
    }

    /// Handle help command
    async fn handle_help(&self) -> ReplResult {
        self.print_help();
        ReplResult {
            success: true,
            output: String::new(),
            error: None,
        }
    }

    /// Handle clear command
    async fn handle_clear(&self) -> ReplResult {
        // Clear screen
        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush().unwrap_or(());

        ReplResult {
            success: true,
            output: String::new(),
            error: None,
        }
    }

    /// Handle history command
    async fn handle_history(&self) -> ReplResult {
        let mut output = String::new();
        output.push_str("Command History:\n");

        for (i, cmd) in self.history.iter().enumerate() {
            output.push_str(&format!("  {}: {}\n", i + 1, cmd));
        }

        if self.history.is_empty() {
            output.push_str("  (no commands in history)\n");
        }

        ReplResult {
            success: true,
            output,
            error: None,
        }
    }

    /// Print command result
    fn print_result(&self, result: &ReplResult) {
        if let Some(ref error) = result.error {
            eprintln!("{}", error.red());
        }

        if !result.output.is_empty() {
            println!("{}", result.output);
        }

        if !result.success && result.error.is_none() {
            eprintln!("{}", "Command failed".red());
        }

        println!(); // Add spacing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repl_creation() {
        let repl = DevRepl::new(Some("rust")).unwrap();
        assert_eq!(repl.language, Some("rust".to_string()));
        assert_eq!(repl.prompt, "codeprism> ");
        assert!(repl.history.is_empty());
    }

    #[test]
    fn test_parse_command() {
        let repl = DevRepl::new(None).unwrap();

        let cmd = repl.parse_command("parse fn main() {}");
        match cmd {
            ReplCommand::Parse { source } => assert_eq!(source, "fn main() {}"),
            _ => panic!("Expected parse command"),
        }
    }

    #[test]
    fn test_parse_load_command() {
        let repl = DevRepl::new(None).unwrap();

        let cmd = repl.parse_command("load test.rs");
        match cmd {
            ReplCommand::Load { file_path } => assert_eq!(file_path, "test.rs"),
            _ => panic!("Expected load command"),
        }
    }

    #[test]
    fn test_parse_show_command() {
        let repl = DevRepl::new(None).unwrap();

        let cmd = repl.parse_command("show ast");
        match cmd {
            ReplCommand::Show { what } => assert!(matches!(what, ShowTarget::Ast)),
            _ => panic!("Expected show command"),
        }
    }

    #[test]
    fn test_parse_unknown_command() {
        let repl = DevRepl::new(None).unwrap();

        let cmd = repl.parse_command("unknown_command");
        match cmd {
            ReplCommand::Unknown { input } => assert_eq!(input, "unknown_command"),
            _ => panic!("Expected unknown command"),
        }
    }

    #[test]
    fn test_parse_exit_command() {
        let repl = DevRepl::new(None).unwrap();

        let cmd = repl.parse_command("exit");
        assert!(matches!(cmd, ReplCommand::Exit));

        let cmd = repl.parse_command("quit");
        assert!(matches!(cmd, ReplCommand::Exit));

        let cmd = repl.parse_command("q");
        assert!(matches!(cmd, ReplCommand::Exit));
    }
}
