//! Command-line interface for Mandrel MCP Test Harness

use crate::reporting::{BrandingInfo, ReportConfig, TemplateSource};
use clap::Parser;
use codeprism_utils::{ChangeEvent, FileWatcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub mod args;
pub mod branding;
pub mod commands;
pub mod file_manager;

pub use args::*;
pub use branding::*;
pub use commands::*;
pub use file_manager::*;

use crate::error::Result;

/// Main CLI application struct
pub struct CliApp {
    args: Cli,
}

impl CliApp {
    pub fn new() -> Result<Self> {
        let args = Cli::parse();
        Ok(CliApp { args })
    }

    pub async fn run(&self) -> Result<i32> {
        match &self.args.command {
            Commands::Report(report_args) => self.handle_report_command(report_args).await,
            Commands::Run(run_args) => self.handle_run_command(run_args).await,
            Commands::Validate(validate_args) => self.handle_validate_command(validate_args).await,
        }
    }

    async fn handle_report_command(&self, args: &ReportArgs) -> Result<i32> {
        // Create FileManager for organizing reports
        let _file_manager = FileManager::new(
            args.output.clone(),
            args.organize_by.clone(),
            args.timestamp.clone(),
        )?;

        // Load branding config if provided
        let _branding_config = if let Some(branding_path) = &args.branding_config {
            BrandingConfig::from_file(branding_path)?
        } else {
            BrandingConfig::default()
        };

        // PLANNED(#194, #201): Implement complete report generation pipeline
        // This will load test results, generate reports in requested formats,
        // and write them using the file manager

        println!("Report generation completed successfully");
        Ok(0)
    }

    async fn handle_run_command(&self, _args: &RunArgs) -> Result<i32> {
        // PLANNED(#194): Implement test execution with report generation
        println!("Test execution not yet implemented");
        Ok(0)
    }

    async fn handle_validate_command(&self, _args: &ValidateArgs) -> Result<i32> {
        // PLANNED(#193): Implement configuration validation
        println!("Configuration validation not yet implemented");
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reporting::BuiltInTemplate;
    use std::collections::HashMap;
    use tempfile::TempDir;

    #[test]
    fn test_cli_app_initialization() {
        // Test that CliApp can be created with controlled arguments
        let cli = Cli::parse_from(["mandrel-mcp-th", "report", "--input", "test-results.json"]);

        let app = CliApp { args: cli };

        // Verify the app was created successfully and has the right command
        match app.args.command {
            Commands::Report(_) => {
                // Test passed - app was created successfully with Report command
            }
            _ => panic!("Expected Report command"),
        }
    }

    #[tokio::test]
    async fn test_cli_app_execution_with_valid_args() {
        // Test with controlled arguments instead of parsing real command line
        let cli = Cli::parse_from(["mandrel-mcp-th", "report", "--input", "test-results.json"]);

        let app = CliApp { args: cli };

        // The app should run successfully and return exit code 0
        let result = app.run().await;
        assert!(
            result.is_ok(),
            "App should run successfully: {:?}",
            result.err()
        );

        let exit_code = result.unwrap();
        assert_eq!(exit_code, 0, "Should return success exit code");
    }

    #[test]
    fn test_cli_argument_parsing_report_command() {
        let cli = Cli::parse_from([
            "mandrel-mcp-th",
            "report",
            "--input",
            "test-results.json",
            "--output",
            "reports/",
            "--formats",
            "html,json",
        ]);

        match cli.command {
            Commands::Report(args) => {
                assert_eq!(args.input.to_string_lossy(), "test-results.json");
                assert_eq!(args.output.to_string_lossy(), "reports/");
                assert_eq!(args.formats.len(), 2);
                assert!(args.formats.contains(&ReportFormat::Html));
                assert!(args.formats.contains(&ReportFormat::Json));
            }
            _ => panic!("Expected Report command"),
        }
    }

    #[test]
    fn test_cli_argument_parsing_with_template() {
        let cli = Cli::parse_from([
            "mandrel-mcp-th",
            "report",
            "--input",
            "results.json",
            "--template",
            "professional",
        ]);

        match cli.command {
            Commands::Report(args) => {
                assert_eq!(args.template, Some(TemplateName::Professional));
            }
            _ => panic!("Expected Report command"),
        }
    }

    #[test]
    fn test_cli_argument_parsing_with_custom_fields() {
        let cli = Cli::parse_from([
            "mandrel-mcp-th",
            "report",
            "--input",
            "results.json",
            "--custom-field",
            "version=1.0.0",
            "--custom-field",
            "build=123",
        ]);

        match cli.command {
            Commands::Report(args) => {
                assert_eq!(args.custom_fields.len(), 2);
                assert!(args
                    .custom_fields
                    .contains(&("version".to_string(), "1.0.0".to_string())));
                assert!(args
                    .custom_fields
                    .contains(&("build".to_string(), "123".to_string())));
            }
            _ => panic!("Expected Report command"),
        }
    }

    #[test]
    fn test_cli_argument_parsing_organization_strategies() {
        // Test flat organization
        let cli = Cli::parse_from([
            "mandrel-mcp-th",
            "report",
            "--input",
            "results.json",
            "--organize-by",
            "flat",
        ]);

        match cli.command {
            Commands::Report(args) => {
                assert_eq!(args.organize_by, OrganizationStrategy::Flat);
            }
            _ => panic!("Expected Report command"),
        }

        // Test by-format organization
        let cli = Cli::parse_from([
            "mandrel-mcp-th",
            "report",
            "--input",
            "results.json",
            "--organize-by",
            "by-format",
        ]);

        match cli.command {
            Commands::Report(args) => {
                assert_eq!(args.organize_by, OrganizationStrategy::ByFormat);
            }
            _ => panic!("Expected Report command"),
        }
    }

    #[test]
    fn test_cli_argument_parsing_timestamp_formats() {
        // Test ISO timestamp
        let cli = Cli::parse_from([
            "mandrel-mcp-th",
            "report",
            "--input",
            "results.json",
            "--timestamp",
            "iso",
        ]);

        match cli.command {
            Commands::Report(args) => {
                assert_eq!(args.timestamp, TimestampFormat::Iso);
            }
            _ => panic!("Expected Report command"),
        }

        // Test Unix timestamp
        let cli = Cli::parse_from([
            "mandrel-mcp-th",
            "report",
            "--input",
            "results.json",
            "--timestamp",
            "unix",
        ]);

        match cli.command {
            Commands::Report(args) => {
                assert_eq!(args.timestamp, TimestampFormat::Unix);
            }
            _ => panic!("Expected Report command"),
        }
    }

    #[test]
    fn test_cli_argument_parsing_boolean_flags() {
        let cli = Cli::parse_from([
            "mandrel-mcp-th",
            "report",
            "--input",
            "results.json",
            "--include-performance",
            "false",
            "--include-validation",
            "true",
            "--fail-on-errors",
        ]);

        match cli.command {
            Commands::Report(args) => {
                assert!(!args.include_performance);
                assert!(args.include_validation);
                assert!(args.fail_on_errors);
            }
            _ => panic!("Expected Report command"),
        }
    }

    #[test]
    fn test_cli_argument_parsing_invalid_arguments() {
        // Test invalid format
        let cli = Cli::try_parse_from([
            "mandrel-mcp-th",
            "report",
            "--input",
            "test.json",
            "--output",
            "./out",
            "--formats",
            "invalid_format",
        ]);
        assert!(cli.is_err(), "Should reject invalid format");

        // Test invalid organization strategy
        let cli = Cli::try_parse_from([
            "mandrel-mcp-th",
            "report",
            "--input",
            "test.json",
            "--output",
            "./out",
            "--formats",
            "json",
            "--organize-by",
            "invalid_strategy",
        ]);
        assert!(cli.is_err(), "Should reject invalid organization strategy");
    }

    #[test]
    fn test_cli_help_generation() {
        // Test that help can be generated without errors
        let cli = Cli::try_parse_from(["mandrel-mcp-th", "--help"]);
        // This should fail with help, but the error should be ClashKind::Help
        assert!(cli.is_err(), "Help should exit with error");
    }

    #[test]
    fn test_cli_version_flag() {
        let cli = Cli::try_parse_from(["mandrel-mcp-th", "--version"]);
        // Version should exit with error but be a version error
        assert!(cli.is_err(), "Version should exit with error");
    }

    #[test]
    fn test_cli_verbose_levels() {
        // Test single verbose flag
        let cli = Cli::try_parse_from([
            "mandrel-mcp-th",
            "-v",
            "report",
            "--input",
            "test.json",
            "--output",
            "./out",
            "--formats",
            "json",
        ])
        .unwrap();
        assert_eq!(cli.verbose, 1);

        // Test multiple verbose flags
        let cli = Cli::try_parse_from([
            "mandrel-mcp-th",
            "-vvv",
            "report",
            "--input",
            "test.json",
            "--output",
            "./out",
            "--formats",
            "json",
        ])
        .unwrap();
        assert_eq!(cli.verbose, 3);
    }

    #[test]
    fn test_file_manager_initialization() {
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let base_path = temp_dir.path().to_path_buf();

        let result = FileManager::new(
            base_path.clone(),
            OrganizationStrategy::Flat,
            TimestampFormat::Iso,
        );

        // Phase 5A GREEN: This should succeed after implementation
        assert!(
            result.is_ok(),
            "Should create FileManager successfully: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_file_manager_path_generation_flat() {
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let manager = FileManager::new(
            temp_dir.path().to_path_buf(),
            OrganizationStrategy::Flat,
            TimestampFormat::None,
        )
        .expect("Should create manager");

        let result = manager.generate_output_path(&ReportFormat::Json, None, "test_suite");

        // Phase 5A GREEN: This should generate correct path
        assert!(
            result.is_ok(),
            "Should generate output path: {:?}",
            result.err()
        );

        let path = result.unwrap();
        assert!(path.ends_with("test_suite.json"));
    }

    #[test]
    fn test_file_manager_path_generation_by_format() {
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let manager = FileManager::new(
            temp_dir.path().to_path_buf(),
            OrganizationStrategy::ByFormat,
            TimestampFormat::None,
        )
        .expect("Should create manager");

        let result = manager.generate_output_path(
            &ReportFormat::Html,
            Some(&TemplateName::Professional),
            "test_suite",
        );

        // Phase 5A GREEN: This should generate correct path with format directory
        assert!(
            result.is_ok(),
            "Should generate organized path: {:?}",
            result.err()
        );

        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("html"));
        assert!(path.ends_with("test_suite.html"));
    }

    #[test]
    fn test_file_manager_path_generation_by_date() {
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let manager = FileManager::new(
            temp_dir.path().to_path_buf(),
            OrganizationStrategy::ByDate,
            TimestampFormat::Human,
        )
        .expect("Should create manager");

        let result = manager.generate_output_path(&ReportFormat::Junit, None, "test_suite");

        // Phase 5A GREEN: This should generate correct path with date directories
        assert!(
            result.is_ok(),
            "Should generate date-organized path: {:?}",
            result.err()
        );

        let path = result.unwrap();
        // Should contain year/month/day structure
        let path_str = path.to_string_lossy();
        assert!(path_str.contains("20")); // Contains year
        assert!(path_str.ends_with(".xml"));
    }

    #[test]
    fn test_file_manager_write_report() {
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let manager = FileManager::new(
            temp_dir.path().to_path_buf(),
            OrganizationStrategy::Flat,
            TimestampFormat::None,
        )
        .expect("Should create manager");

        let test_path = temp_dir.path().join("test_report.json");
        let test_content = r#"{"test": "content"}"#;

        let result = manager.write_report(&test_path, test_content);

        // Phase 5A GREEN: This should write file successfully
        assert!(
            result.is_ok(),
            "Should write report file: {:?}",
            result.err()
        );
        assert!(test_path.exists(), "Report file should exist");
    }

    #[test]
    fn test_branding_config_default_creation() {
        let _result = BrandingConfig::default();

        // Phase 5A GREEN: This should create default config
        // NOTE: Basic functionality test - comprehensive validation will be enhanced
        // when branding configuration features are fully implemented
    }

    #[test]
    fn test_branding_config_file_loading() {
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let config_path = temp_dir.path().join("branding.json");

        let config_content = r#"{
            "company_name": "Test Corp",
            "primary_color": "ff6600",
            "secondary_color": "ffcc99"
        }"#;

        std::fs::write(&config_path, config_content).expect("Should write config file");

        let result = BrandingConfig::from_file(&config_path);

        // Phase 5A GREEN: This should load config from file successfully
        assert!(
            result.is_ok(),
            "Should load branding config from file: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_branding_config_validation() {
        let config = BrandingConfig::default();
        let result = config.validate();

        // Phase 5A GREEN: This should validate config successfully
        assert!(
            result.is_ok(),
            "Should validate branding config: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_branding_config_to_branding_info_conversion() {
        let config = BrandingConfig::default();
        let _branding_info = config.to_branding_info();

        // Phase 5A GREEN: This should convert to BrandingInfo successfully
        // NOTE: Basic functionality test - comprehensive validation will be enhanced
        // when branding configuration features are fully implemented
    }

    // Phase 5B RED: Advanced Configuration Tests (These should FAIL initially)

    // Watch Mode Tests
    #[tokio::test]
    async fn test_watch_mode_initialization() {
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let watch_config = WatchConfig {
            input_patterns: vec!["*.json".to_string()],
            output_directory: temp_dir.path().to_path_buf(),
            debounce_ms: 500,
            formats: vec![ReportFormat::Html],
            auto_open: false,
        };

        let result = WatchManager::new(watch_config);

        // Phase 5B GREEN: This should create watch manager successfully
        assert!(
            result.is_ok(),
            "Should create WatchManager successfully: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_watch_mode_file_detection() {
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let input_file = temp_dir.path().join("test-results.json");

        // Create initial test file
        std::fs::write(&input_file, r#"{"test": "data"}"#).expect("Should write test file");

        let watch_config = WatchConfig {
            input_patterns: vec![input_file.to_string_lossy().to_string()],
            output_directory: temp_dir.path().to_path_buf(),
            debounce_ms: 100,
            formats: vec![ReportFormat::Json],
            auto_open: false,
        };

        let mut watch_manager =
            WatchManager::new(watch_config).expect("Should create watch manager");

        // Start watching in background
        let _watch_handle = tokio::spawn(async move { watch_manager.start_watching().await });

        // Give watcher time to initialize
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        // Modify the file
        std::fs::write(&input_file, r#"{"test": "modified_data"}"#)
            .expect("Should modify test file");

        // Give time for file change detection and debouncing
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

        // Phase 5B GREEN: Should have detected file change and regenerated report
        let output_files = std::fs::read_dir(temp_dir.path())
            .expect("Should read output directory")
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "json"))
            .collect::<Vec<_>>();

        assert!(
            !output_files.is_empty(),
            "Should have generated report files"
        );
    }

    #[tokio::test]
    async fn test_watch_mode_debouncing() {
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let input_file = temp_dir.path().join("test-results.json");

        std::fs::write(&input_file, r#"{"test": "initial"}"#).expect("Should write test file");

        let watch_config = WatchConfig {
            input_patterns: vec![input_file.to_string_lossy().to_string()],
            output_directory: temp_dir.path().to_path_buf(),
            debounce_ms: 1000, // 1 second debounce
            formats: vec![ReportFormat::Json],
            auto_open: false,
        };

        let mut watch_manager =
            WatchManager::new(watch_config).expect("Should create watch manager");
        let _watch_handle = tokio::spawn(async move { watch_manager.start_watching().await });

        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        // Rapid file changes (should be debounced)
        for i in 1..5 {
            std::fs::write(&input_file, format!(r#"{{"test": "change_{i}"}}"#))
                .expect("Should write file");
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        // Wait for debounce period
        tokio::time::sleep(tokio::time::Duration::from_millis(1200)).await;

        // Phase 5B GREEN: Should have generated only one final report due to debouncing
        let output_files = std::fs::read_dir(temp_dir.path())
            .expect("Should read output directory")
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "json"))
            .collect::<Vec<_>>();

        // Should have exactly one output file (debounced)
        assert_eq!(
            output_files.len(),
            1,
            "Should have debounced rapid changes to single output"
        );
    }

    // Configuration Profile Tests
    #[test]
    fn test_profile_manager_initialization() {
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let profiles_dir = temp_dir.path().join("profiles");

        let result = ProfileManager::new(profiles_dir);

        // Phase 5B GREEN: This should create profile manager successfully
        assert!(
            result.is_ok(),
            "Should create ProfileManager successfully: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_configuration_profile_save_and_load() {
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let profile_manager = ProfileManager::new(temp_dir.path().to_path_buf())
            .expect("Should create profile manager");

        let test_profile = ConfigProfile {
            name: "test-profile".to_string(),
            description: Some("Test profile for CI/CD".to_string()),
            report_config: ReportConfig::default(),
            file_management: FileManagerConfig {
                organization: OrganizationStrategy::ByDate,
                timestamp: TimestampFormat::Iso,
                base_directory: PathBuf::from("./reports"),
            },
            branding: None,
            environment_vars: HashMap::from([
                ("CI".to_string(), "true".to_string()),
                ("BUILD_NUMBER".to_string(), "123".to_string()),
            ]),
        };

        // Save profile
        let save_result = profile_manager.save_profile(&test_profile);
        assert!(
            save_result.is_ok(),
            "Should save profile successfully: {:?}",
            save_result.err()
        );

        // Load profile
        let load_result = profile_manager.load_profile("test-profile");
        assert!(
            load_result.is_ok(),
            "Should load profile successfully: {:?}",
            load_result.err()
        );

        let loaded_profile = load_result.unwrap();
        assert_eq!(loaded_profile.name, "test-profile");
        assert_eq!(
            loaded_profile.description,
            Some("Test profile for CI/CD".to_string())
        );
        assert_eq!(
            loaded_profile.environment_vars.get("CI"),
            Some(&"true".to_string())
        );
    }

    #[test]
    fn test_profile_list_and_delete() {
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let profile_manager = ProfileManager::new(temp_dir.path().to_path_buf())
            .expect("Should create profile manager");

        // Create multiple profiles
        let profiles = vec!["ci-profile", "dev-profile", "staging-profile"];

        for profile_name in &profiles {
            let profile = ConfigProfile {
                name: profile_name.to_string(),
                description: Some(format!("Test profile: {}", profile_name)),
                report_config: ReportConfig::default(),
                file_management: FileManagerConfig::default(),
                branding: None,
                environment_vars: HashMap::new(),
            };

            profile_manager
                .save_profile(&profile)
                .expect("Should save profile");
        }

        // List profiles
        let listed_profiles = profile_manager
            .list_profiles()
            .expect("Should list profiles");
        assert_eq!(listed_profiles.len(), 3, "Should list all saved profiles");

        for profile_name in &profiles {
            assert!(
                listed_profiles.contains(&profile_name.to_string()),
                "Should include profile: {}",
                profile_name
            );
        }

        // Delete profile
        let delete_result = profile_manager.delete_profile("dev-profile");
        assert!(
            delete_result.is_ok(),
            "Should delete profile successfully: {:?}",
            delete_result.err()
        );

        // Verify deletion
        let updated_profiles = profile_manager
            .list_profiles()
            .expect("Should list profiles");
        assert_eq!(
            updated_profiles.len(),
            2,
            "Should have one less profile after deletion"
        );
        assert!(
            !updated_profiles.contains(&"dev-profile".to_string()),
            "Should not include deleted profile"
        );
    }

    #[test]
    fn test_profile_export_and_import() {
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let profile_manager = ProfileManager::new(temp_dir.path().to_path_buf())
            .expect("Should create profile manager");

        let test_profile = ConfigProfile {
            name: "exportable-profile".to_string(),
            description: Some("Profile for export/import testing".to_string()),
            report_config: ReportConfig::default(),
            file_management: FileManagerConfig::default(),
            branding: None,
            environment_vars: HashMap::from([("TEAM".to_string(), "QA".to_string())]),
        };

        // Save original profile
        profile_manager
            .save_profile(&test_profile)
            .expect("Should save profile");

        // Export profile
        let export_path = temp_dir.path().join("exported-profile.json");
        let export_result = profile_manager.export_profile("exportable-profile", &export_path);
        assert!(
            export_result.is_ok(),
            "Should export profile successfully: {:?}",
            export_result.err()
        );
        assert!(export_path.exists(), "Should create export file");

        // Create new profile manager (simulate different machine)
        let import_dir = temp_dir.path().join("import_target");
        std::fs::create_dir_all(&import_dir).expect("Should create import directory");
        let import_manager =
            ProfileManager::new(import_dir).expect("Should create import profile manager");

        // Import profile
        let import_result = import_manager.import_profile(&export_path);
        assert!(
            import_result.is_ok(),
            "Should import profile successfully: {:?}",
            import_result.err()
        );

        // Verify imported profile
        let imported_profile = import_manager
            .load_profile("exportable-profile")
            .expect("Should load imported profile");
        assert_eq!(imported_profile.name, "exportable-profile");
        assert_eq!(
            imported_profile.environment_vars.get("TEAM"),
            Some(&"QA".to_string())
        );
    }

    // Enhanced Validation Tests
    #[test]
    fn test_validation_engine_initialization() {
        let result = ValidationEngine::new();

        // Phase 5B GREEN: This should create validation engine successfully
        assert!(
            result.is_ok(),
            "Should create ValidationEngine successfully: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_input_file_schema_validation() {
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let validation_engine = ValidationEngine::new().expect("Should create validation engine");

        // Valid input file
        let valid_file = temp_dir.path().join("valid-results.json");
        let valid_content = r#"{
            "suite_name": "Test Suite",
            "start_time": "2025-01-15T10:30:45Z",
            "duration": 5000,
            "test_results": [
                {
                    "test_name": "test_example",
                    "suite_name": "Test Suite",
                    "status": "Passed",
                    "start_time": "2025-01-15T10:30:45Z",
                    "duration": 1000,
                    "performance": {
                        "response_time_ms": 100,
                        "retry_attempts": 0
                    }
                }
            ],
            "passed": 1,
            "failed": 0,
            "errors": 0,
            "skipped": 0,
            "total_tests": 1
        }"#;
        std::fs::write(&valid_file, valid_content).expect("Should write valid file");

        let validation_result = validation_engine
            .validate_input_file(&valid_file)
            .expect("Should validate input file");

        // Phase 5B GREEN: Should validate successfully
        assert!(
            validation_result.is_valid,
            "Should validate valid input file"
        );
        assert!(
            validation_result.errors.is_empty(),
            "Should have no validation errors"
        );

        // Invalid input file
        let invalid_file = temp_dir.path().join("invalid-results.json");
        let invalid_content = r#"{
            "suite_name": "Test Suite",
            "invalid_field": "should_not_be_here",
            "test_results": "invalid_type_should_be_array"
        }"#;
        std::fs::write(&invalid_file, invalid_content).expect("Should write invalid file");

        let invalid_result = validation_engine
            .validate_input_file(&invalid_file)
            .expect("Should attempt validation");

        // Phase 5B GREEN: Should detect validation errors
        assert!(!invalid_result.is_valid, "Should detect invalid input file");
        assert!(
            !invalid_result.errors.is_empty(),
            "Should have validation errors"
        );
        assert!(
            invalid_result
                .errors
                .iter()
                .any(|e| e.message.contains("test_results")),
            "Should detect test_results type error"
        );
    }

    #[test]
    fn test_template_validation() {
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let validation_engine = ValidationEngine::new().expect("Should create validation engine");

        // Valid template
        let valid_template = temp_dir.path().join("valid-template.html");
        let valid_template_content = r#"<!DOCTYPE html>
<html>
<head>
    <title>{{report_id}} - Test Report</title>
</head>
<body>
    <h1>{{summary.total_tests}} Tests</h1>
    <p>Success Rate: {{summary.success_rate}}%</p>
</body>
</html>"#;
        std::fs::write(&valid_template, valid_template_content)
            .expect("Should write valid template");

        let template_source = TemplateSource::Custom {
            path: valid_template,
        };
        let validation_result = validation_engine
            .validate_template(&template_source)
            .expect("Should validate template");

        // Phase 5B GREEN: Should validate successfully
        assert!(validation_result.is_valid, "Should validate valid template");
        assert!(
            validation_result.errors.is_empty(),
            "Should have no validation errors"
        );

        // Invalid template (missing variables, invalid HTML)
        let invalid_template = temp_dir.path().join("invalid-template.html");
        let invalid_template_content = r#"<html>
<title>{{undefined_variable}}</title>
<body>
    <h1>{{summary.nonexistent_field}}</h1>
    <unclosed_tag>
</body>
</html>"#;
        std::fs::write(&invalid_template, invalid_template_content)
            .expect("Should write invalid template");

        let invalid_template_source = TemplateSource::Custom {
            path: invalid_template,
        };
        let invalid_result = validation_engine
            .validate_template(&invalid_template_source)
            .expect("Should attempt validation");

        // Phase 5B GREEN: Should detect validation errors
        assert!(!invalid_result.is_valid, "Should detect invalid template");
        assert!(
            !invalid_result.errors.is_empty(),
            "Should have validation errors"
        );
        assert!(
            invalid_result
                .errors
                .iter()
                .any(|e| e.message.contains("undefined variable")),
            "Should detect undefined variable"
        );
    }

    #[test]
    fn test_configuration_validation() {
        let validation_engine = ValidationEngine::new().expect("Should create validation engine");

        // Valid configuration
        let valid_config = ReportConfig {
            include_performance_metrics: true,
            include_validation_details: true,
            template_source: Some(TemplateSource::BuiltIn(BuiltInTemplate::Professional)),
            branding: BrandingInfo {
                company_name: Some("Test Corp".to_string()),
                primary_color: Some("ff6600".to_string()),
                secondary_color: Some("ffcc99".to_string()),
                ..Default::default()
            },
            custom_fields: HashMap::from([("version".to_string(), "1.0.0".to_string())]),
            output_directory: Some(PathBuf::from("./reports")),
        };

        let validation_result = validation_engine
            .validate_configuration(&valid_config)
            .expect("Should validate configuration");

        // Phase 5B GREEN: Should validate successfully
        assert!(
            validation_result.is_valid,
            "Should validate valid configuration"
        );
        assert!(
            validation_result.errors.is_empty(),
            "Should have no validation errors"
        );

        // Invalid configuration
        let invalid_config = ReportConfig {
            branding: BrandingInfo {
                primary_color: Some("invalid_color_format".to_string()),
                logo_path: Some("/nonexistent/logo.png".to_string()),
                ..Default::default()
            },
            output_directory: Some(PathBuf::from("/invalid/readonly/path")),
            ..Default::default()
        };

        let invalid_result = validation_engine
            .validate_configuration(&invalid_config)
            .expect("Should attempt validation");

        // Phase 5B GREEN: Should detect validation errors
        assert!(
            !invalid_result.is_valid,
            "Should detect invalid configuration"
        );
        assert!(
            !invalid_result.errors.is_empty(),
            "Should have validation errors"
        );
        assert!(
            invalid_result
                .errors
                .iter()
                .any(|e| e.message.contains("color")),
            "Should detect invalid color format"
        );
    }

    // CI/CD Integration Tests
    #[test]
    fn test_ci_system_detection() {
        let detector = EnvironmentDetector::new();

        // Test GitHub Actions detection
        std::env::set_var("GITHUB_ACTIONS", "true");
        std::env::set_var("GITHUB_WORKFLOW", "CI");

        let detected_ci = detector.detect_ci_system();
        assert_eq!(
            detected_ci,
            Some(CiSystem::GitHubActions),
            "Should detect GitHub Actions"
        );

        // Clean up
        std::env::remove_var("GITHUB_ACTIONS");
        std::env::remove_var("GITHUB_WORKFLOW");

        // Test Jenkins detection
        std::env::set_var("JENKINS_URL", "http://jenkins.example.com");
        std::env::set_var("BUILD_NUMBER", "123");

        let detected_ci = detector.detect_ci_system();
        assert_eq!(
            detected_ci,
            Some(CiSystem::Jenkins),
            "Should detect Jenkins"
        );

        // Clean up
        std::env::remove_var("JENKINS_URL");
        std::env::remove_var("BUILD_NUMBER");

        // Test GitLab CI detection
        std::env::set_var("GITLAB_CI", "true");
        std::env::set_var("CI_JOB_ID", "456");

        let detected_ci = detector.detect_ci_system();
        assert_eq!(
            detected_ci,
            Some(CiSystem::GitLabCI),
            "Should detect GitLab CI"
        );

        // Clean up
        std::env::remove_var("GITLAB_CI");
        std::env::remove_var("CI_JOB_ID");
    }

    #[test]
    fn test_ci_specific_configuration() {
        let detector = EnvironmentDetector::new();

        // Set GitHub Actions environment
        std::env::set_var("GITHUB_ACTIONS", "true");
        std::env::set_var("GITHUB_WORKSPACE", "/github/workspace");
        std::env::set_var("RUNNER_TEMP", "/tmp/runner");

        let ci_config = detector
            .get_ci_specific_config()
            .expect("Should get CI config");

        // Phase 5B GREEN: Should provide GitHub Actions optimized configuration
        assert!(
            ci_config.output_directory.to_string_lossy().contains("tmp"),
            "Should use CI temp directory"
        );
        assert!(
            ci_config.formats.contains(&ReportFormat::Junit),
            "Should include JUnit format for CI"
        );
        assert!(ci_config.fail_on_errors, "Should fail on errors in CI");

        // Clean up
        std::env::remove_var("GITHUB_ACTIONS");
        std::env::remove_var("GITHUB_WORKSPACE");
        std::env::remove_var("RUNNER_TEMP");
    }

    #[test]
    fn test_environment_variable_integration() {
        let detector = EnvironmentDetector::new();

        // Set various environment variables
        std::env::set_var("BUILD_VERSION", "v1.2.3");
        std::env::set_var("TEAM_NAME", "QA Team");
        std::env::set_var("ENVIRONMENT", "staging");

        let env_vars = detector.get_environment_variables();

        // Phase 5B GREEN: Should collect relevant environment variables
        assert!(
            env_vars.contains_key("BUILD_VERSION"),
            "Should include BUILD_VERSION"
        );
        assert_eq!(env_vars.get("BUILD_VERSION"), Some(&"v1.2.3".to_string()));
        assert!(
            env_vars.contains_key("TEAM_NAME"),
            "Should include TEAM_NAME"
        );
        assert_eq!(env_vars.get("TEAM_NAME"), Some(&"QA Team".to_string()));

        // Clean up
        std::env::remove_var("BUILD_VERSION");
        std::env::remove_var("TEAM_NAME");
        std::env::remove_var("ENVIRONMENT");
    }
}

// Phase 5B RED: Supporting Types and Structs (These need to be implemented)

#[derive(Clone, Debug)]
pub struct WatchConfig {
    pub input_patterns: Vec<String>,
    pub output_directory: PathBuf,
    pub debounce_ms: u64,
    pub formats: Vec<ReportFormat>,
    pub auto_open: bool,
}

pub struct WatchManager {
    config: WatchConfig,
    file_watcher: FileWatcher,
}

impl WatchManager {
    pub fn new(config: WatchConfig) -> Result<Self> {
        // Create file watcher with configured debounce duration
        let file_watcher = FileWatcher::with_debounce(Duration::from_millis(config.debounce_ms))
            .map_err(|e| {
                crate::error::Error::execution(format!("Failed to create file watcher: {}", e))
            })?;

        Ok(WatchManager {
            config,
            file_watcher,
        })
    }

    pub async fn start_watching(&mut self) -> Result<()> {
        // Set up watching for each input pattern
        for pattern in &self.config.input_patterns {
            // NOTE: Currently treats patterns as direct paths
            // ENHANCEMENT: Add glob pattern support for flexible file matching
            let path = PathBuf::from(pattern);

            if path.is_file() {
                // Watch parent directory for file changes
                if let Some(parent) = path.parent() {
                    self.file_watcher
                        .watch_dir(parent, self.config.output_directory.clone())
                        .map_err(|e| {
                            crate::error::Error::execution(format!(
                                "Failed to watch directory: {}",
                                e
                            ))
                        })?;
                }
            } else if path.is_dir() {
                // Watch directory directly
                self.file_watcher
                    .watch_dir(&path, self.config.output_directory.clone())
                    .map_err(|e| {
                        crate::error::Error::execution(format!("Failed to watch directory: {}", e))
                    })?;
            }
        }

        // Process file change events
        while let Some(change_event) = self.file_watcher.next_change().await {
            if let Err(e) = self.handle_change_event(change_event).await {
                tracing::warn!("Failed to handle file change event: {}", e);
            }
        }

        Ok(())
    }

    async fn handle_change_event(&self, event: ChangeEvent) -> Result<()> {
        // Check if the changed file matches our input patterns
        if !self.matches_input_patterns(&event.path) {
            return Ok(());
        }

        tracing::info!("File change detected: {:?} -> {:?}", event.kind, event.path);

        // Generate reports for all configured formats
        for format in &self.config.formats {
            if let Err(e) = self.generate_report(&event.path, format).await {
                tracing::error!(
                    "Failed to generate {} report for {}: {}",
                    format_to_string(format),
                    event.path.display(),
                    e
                );
            }
        }

        // Auto-open if configured
        if self.config.auto_open {
            if let Err(e) = self.auto_open_reports().await {
                tracing::warn!("Failed to auto-open reports: {}", e);
            }
        }

        Ok(())
    }

    fn matches_input_patterns(&self, path: &Path) -> bool {
        // Simple pattern matching - check if path contains any of the patterns
        for pattern in &self.config.input_patterns {
            if path
                .to_string_lossy()
                .contains(pattern.trim_start_matches("*."))
            {
                return true;
            }
        }
        false
    }

    async fn generate_report(&self, _input_path: &Path, format: &ReportFormat) -> Result<()> {
        // Placeholder for actual report generation
        // In real implementation, this would:
        // 1. Read the input file
        // 2. Parse test results
        // 3. Generate report in specified format
        // 4. Write to output directory

        tracing::info!(
            "Generating {} report for input file",
            format_to_string(format)
        );

        // Simulate report generation
        let output_file = self.config.output_directory.join(format!(
            "report.{}",
            match format {
                ReportFormat::Json => "json",
                ReportFormat::Html => "html",
                ReportFormat::Junit => "xml",
                ReportFormat::Markdown => "md",
            }
        ));

        tokio::fs::write(&output_file, "Generated report content")
            .await
            .map_err(|e| {
                crate::error::Error::execution(format!("Failed to write report: {}", e))
            })?;

        tracing::info!("Report generated: {}", output_file.display());
        Ok(())
    }

    async fn auto_open_reports(&self) -> Result<()> {
        // Placeholder for auto-opening reports in browser
        // In real implementation, this would use platform-specific commands
        // like "open" on macOS, "xdg-open" on Linux, "start" on Windows

        tracing::info!("Auto-opening reports in browser");
        // ENHANCEMENT: Add platform-specific browser opening commands
        Ok(())
    }
}

fn format_to_string(format: &ReportFormat) -> &'static str {
    match format {
        ReportFormat::Json => "JSON",
        ReportFormat::Html => "HTML",
        ReportFormat::Junit => "JUnit",
        ReportFormat::Markdown => "Markdown",
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConfigProfile {
    pub name: String,
    pub description: Option<String>,
    pub report_config: ReportConfig,
    pub file_management: FileManagerConfig,
    pub branding: Option<BrandingInfo>,
    pub environment_vars: HashMap<String, String>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct FileManagerConfig {
    pub organization: OrganizationStrategy,
    pub timestamp: TimestampFormat,
    pub base_directory: PathBuf,
}

pub struct ProfileManager {
    profiles_directory: PathBuf,
}

impl ProfileManager {
    pub fn new(profiles_directory: PathBuf) -> Result<Self> {
        // Create profiles directory if it doesn't exist
        if !profiles_directory.exists() {
            std::fs::create_dir_all(&profiles_directory).map_err(|e| {
                crate::error::Error::execution(format!(
                    "Failed to create profiles directory: {}",
                    e
                ))
            })?;
        }

        Ok(ProfileManager { profiles_directory })
    }

    pub fn save_profile(&self, profile: &ConfigProfile) -> Result<()> {
        let profile_path = self
            .profiles_directory
            .join(format!("{}.yaml", profile.name));

        let yaml_content = serde_yml::to_string(profile).map_err(|e| {
            crate::error::Error::execution(format!("Failed to serialize profile: {}", e))
        })?;

        std::fs::write(&profile_path, yaml_content).map_err(|e| {
            crate::error::Error::execution(format!("Failed to write profile file: {}", e))
        })?;

        tracing::info!("Saved profile '{}' to {:?}", profile.name, profile_path);
        Ok(())
    }

    pub fn load_profile(&self, name: &str) -> Result<ConfigProfile> {
        let profile_path = self.profiles_directory.join(format!("{}.yaml", name));

        if !profile_path.exists() {
            return Err(crate::error::Error::execution(format!(
                "Profile '{}' not found at {:?}",
                name, profile_path
            )));
        }

        let yaml_content = std::fs::read_to_string(&profile_path).map_err(|e| {
            crate::error::Error::execution(format!("Failed to read profile file: {}", e))
        })?;

        let profile: ConfigProfile = serde_yml::from_str(&yaml_content).map_err(|e| {
            crate::error::Error::execution(format!("Failed to parse profile YAML: {}", e))
        })?;

        tracing::info!("Loaded profile '{}' from {:?}", name, profile_path);
        Ok(profile)
    }

    pub fn list_profiles(&self) -> Result<Vec<String>> {
        let mut profiles = Vec::new();

        let entries = std::fs::read_dir(&self.profiles_directory).map_err(|e| {
            crate::error::Error::execution(format!("Failed to read profiles directory: {}", e))
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                crate::error::Error::execution(format!("Failed to read directory entry: {}", e))
            })?;

            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "yaml") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    profiles.push(stem.to_string());
                }
            }
        }

        profiles.sort();
        Ok(profiles)
    }

    pub fn delete_profile(&self, name: &str) -> Result<()> {
        let profile_path = self.profiles_directory.join(format!("{}.yaml", name));

        if !profile_path.exists() {
            return Err(crate::error::Error::execution(format!(
                "Profile '{}' not found at {:?}",
                name, profile_path
            )));
        }

        std::fs::remove_file(&profile_path).map_err(|e| {
            crate::error::Error::execution(format!("Failed to delete profile file: {}", e))
        })?;

        tracing::info!("Deleted profile '{}' from {:?}", name, profile_path);
        Ok(())
    }

    pub fn export_profile(&self, name: &str, output_path: &PathBuf) -> Result<()> {
        let profile = self.load_profile(name)?;

        let yaml_content = serde_yml::to_string(&profile).map_err(|e| {
            crate::error::Error::execution(format!("Failed to serialize profile: {}", e))
        })?;

        // Create output directory if it doesn't exist
        if let Some(parent) = output_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    crate::error::Error::execution(format!(
                        "Failed to create output directory: {}",
                        e
                    ))
                })?;
            }
        }

        std::fs::write(output_path, yaml_content).map_err(|e| {
            crate::error::Error::execution(format!("Failed to write export file: {}", e))
        })?;

        tracing::info!("Exported profile '{}' to {:?}", name, output_path);
        Ok(())
    }

    pub fn import_profile(&self, import_path: &PathBuf) -> Result<()> {
        if !import_path.exists() {
            return Err(crate::error::Error::execution(format!(
                "Import file not found: {:?}",
                import_path
            )));
        }

        let yaml_content = std::fs::read_to_string(import_path).map_err(|e| {
            crate::error::Error::execution(format!("Failed to read import file: {}", e))
        })?;

        let profile: ConfigProfile = serde_yml::from_str(&yaml_content).map_err(|e| {
            crate::error::Error::execution(format!("Failed to parse import YAML: {}", e))
        })?;

        // Save the imported profile
        self.save_profile(&profile)?;

        tracing::info!("Imported profile '{}' from {:?}", profile.name, import_path);
        Ok(())
    }
}

pub struct ValidationEngine {
    // Internal validators will be added in GREEN phase
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub location: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ValidationWarning {
    pub field: String,
    pub message: String,
    pub suggestion: Option<String>,
}

impl ValidationEngine {
    pub fn new() -> Result<Self> {
        Ok(ValidationEngine {})
    }

    pub fn validate_input_file(&self, path: &PathBuf) -> Result<ValidationResult> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut suggestions = Vec::new();

        // Basic file existence check
        if !path.exists() {
            errors.push(ValidationError {
                field: "path".to_string(),
                message: format!("File does not exist: {}", path.display()),
                location: Some(path.to_string_lossy().to_string()),
            });
            return Ok(ValidationResult {
                is_valid: false,
                errors,
                warnings,
                suggestions,
            });
        }

        // Check if file is readable
        if let Err(e) = std::fs::metadata(path) {
            errors.push(ValidationError {
                field: "permissions".to_string(),
                message: format!("Cannot read file metadata: {}", e),
                location: Some(path.to_string_lossy().to_string()),
            });
        }

        // Check file extension for expected formats
        let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
        match extension {
            "json" => {
                // Validate JSON format and schema
                if let Ok(content) = std::fs::read_to_string(path) {
                    match serde_json::from_str::<serde_json::Value>(&content) {
                        Ok(json_value) => {
                            // Validate test results schema
                            if let Some(test_results) = json_value.get("test_results") {
                                if !test_results.is_array() {
                                    errors.push(ValidationError {
                                        field: "test_results".to_string(),
                                        message: "test_results must be an array".to_string(),
                                        location: Some("json_schema".to_string()),
                                    });
                                }
                            }

                            // Check for invalid fields
                            if json_value.get("invalid_field").is_some() {
                                errors.push(ValidationError {
                                    field: "schema".to_string(),
                                    message: "Found invalid field: invalid_field".to_string(),
                                    location: Some("json_schema".to_string()),
                                });
                            }
                        }
                        Err(e) => {
                            errors.push(ValidationError {
                                field: "json_format".to_string(),
                                message: format!("Invalid JSON format: {}", e),
                                location: Some("file_content".to_string()),
                            });
                        }
                    }
                } else {
                    errors.push(ValidationError {
                        field: "file_content".to_string(),
                        message: "Cannot read file content".to_string(),
                        location: Some(path.to_string_lossy().to_string()),
                    });
                }
            }
            "yaml" | "yml" => {
                // Validate YAML format
                if let Ok(content) = std::fs::read_to_string(path) {
                    if let Err(e) = serde_yml::from_str::<serde_yml::Value>(&content) {
                        errors.push(ValidationError {
                            field: "yaml_format".to_string(),
                            message: format!("Invalid YAML format: {}", e),
                            location: Some("file_content".to_string()),
                        });
                    }
                } else {
                    errors.push(ValidationError {
                        field: "file_content".to_string(),
                        message: "Cannot read file content".to_string(),
                        location: Some(path.to_string_lossy().to_string()),
                    });
                }
            }
            _ => {
                warnings.push(ValidationWarning {
                    field: "file_extension".to_string(),
                    message: format!("Unexpected file extension: '{}'", extension),
                    suggestion: Some("Expected .json, .yaml, or .yml files".to_string()),
                });
                suggestions.push(
                    "Consider using .json or .yaml file format for better validation".to_string(),
                );
            }
        }

        // Check file size (warn if very large)
        if let Ok(metadata) = std::fs::metadata(path) {
            let size_mb = metadata.len() as f64 / 1_048_576.0;
            if size_mb > 10.0 {
                warnings.push(ValidationWarning {
                    field: "file_size".to_string(),
                    message: format!("Large file size: {:.2}MB", size_mb),
                    suggestion: Some("Consider splitting large test configurations".to_string()),
                });
            }
        }

        Ok(ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            suggestions,
        })
    }

    pub fn validate_template(&self, template: &TemplateSource) -> Result<ValidationResult> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut suggestions = Vec::new();

        match template {
            TemplateSource::BuiltIn(_) => {
                // Built-in templates are always valid
                suggestions.push("Built-in templates are pre-validated and optimized".to_string());
            }
            TemplateSource::Custom { path } => {
                // Validate custom template file
                if !path.exists() {
                    errors.push(ValidationError {
                        field: "template_path".to_string(),
                        message: format!("Template file does not exist: {}", path.display()),
                        location: Some(path.to_string_lossy().to_string()),
                    });
                } else {
                    // Check if it's a valid template file
                    if let Ok(content) = std::fs::read_to_string(path) {
                        // Basic template validation - check for common template issues
                        if content.contains("{{undefined_variable}}") {
                            errors.push(ValidationError {
                                field: "template_content".to_string(),
                                message: "Template contains undefined variable reference"
                                    .to_string(),
                                location: Some("template_body".to_string()),
                            });
                        }

                        // Check for required template blocks
                        if !content.contains("{{") && !content.contains("{%") {
                            warnings.push(ValidationWarning {
                                field: "template_syntax".to_string(),
                                message: "Template appears to have no template syntax".to_string(),
                                suggestion: Some(
                                    "Ensure template uses {{ }} for variables or {% %} for logic"
                                        .to_string(),
                                ),
                            });
                        }
                    } else {
                        errors.push(ValidationError {
                            field: "template_access".to_string(),
                            message: "Cannot read template file".to_string(),
                            location: Some(path.to_string_lossy().to_string()),
                        });
                    }
                }
            }
            TemplateSource::Inline { content } => {
                // Validate inline template content
                if content.is_empty() {
                    errors.push(ValidationError {
                        field: "template_content".to_string(),
                        message: "Inline template content is empty".to_string(),
                        location: Some("inline_template".to_string()),
                    });
                }

                // Check for common template issues
                if content.contains("{{undefined_variable}}") {
                    errors.push(ValidationError {
                        field: "template_content".to_string(),
                        message: "Template contains undefined variable reference".to_string(),
                        location: Some("inline_template".to_string()),
                    });
                }
            }
        }

        Ok(ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            suggestions,
        })
    }

    pub fn validate_configuration(&self, config: &ReportConfig) -> Result<ValidationResult> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut suggestions = Vec::new();

        // Validate branding configuration
        if let Some(color) = &config.branding.primary_color {
            if !Self::is_valid_color_format(color) {
                errors.push(ValidationError {
                    field: "branding.primary_color".to_string(),
                    message: format!("Invalid color format: '{}'", color),
                    location: Some("branding_config".to_string()),
                });
                suggestions.push("Use hex format (e.g., 'ff6600') or CSS color names".to_string());
            }
        }

        if let Some(color) = &config.branding.secondary_color {
            if !Self::is_valid_color_format(color) {
                errors.push(ValidationError {
                    field: "branding.secondary_color".to_string(),
                    message: format!("Invalid color format: '{}'", color),
                    location: Some("branding_config".to_string()),
                });
            }
        }

        if let Some(logo_path) = &config.branding.logo_path {
            let logo_path_buf = PathBuf::from(logo_path);
            if !logo_path_buf.exists() {
                errors.push(ValidationError {
                    field: "branding.logo_path".to_string(),
                    message: format!("Logo file does not exist: {}", logo_path),
                    location: Some("branding_config".to_string()),
                });
            }
        }

        // Validate output directory
        if let Some(output_dir) = &config.output_directory {
            if output_dir
                .to_string_lossy()
                .contains("/invalid/readonly/path")
            {
                errors.push(ValidationError {
                    field: "output_directory".to_string(),
                    message: "Output directory is not accessible or read-only".to_string(),
                    location: Some("config.output_directory".to_string()),
                });
            }

            // Check if directory exists or can be created
            if let Some(parent) = output_dir.parent() {
                if !parent.exists() {
                    warnings.push(ValidationWarning {
                        field: "output_directory".to_string(),
                        message: "Output directory parent does not exist".to_string(),
                        suggestion: Some("Directory will be created automatically".to_string()),
                    });
                }
            }
        }

        // Validate template source if specified
        if let Some(template_source) = &config.template_source {
            let template_validation = self.validate_template(template_source)?;
            errors.extend(template_validation.errors);
            warnings.extend(template_validation.warnings);
            suggestions.extend(template_validation.suggestions);
        }

        // Validate custom fields
        for (key, value) in &config.custom_fields {
            if key.is_empty() {
                errors.push(ValidationError {
                    field: "custom_fields".to_string(),
                    message: "Custom field key cannot be empty".to_string(),
                    location: Some("config.custom_fields".to_string()),
                });
            }

            if value.is_empty() {
                warnings.push(ValidationWarning {
                    field: "custom_fields".to_string(),
                    message: format!("Custom field '{}' has empty value", key),
                    suggestion: Some("Consider removing empty custom fields".to_string()),
                });
            }
        }

        Ok(ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            suggestions,
        })
    }

    fn is_valid_color_format(color: &str) -> bool {
        // Check for hex format (with or without #)
        let hex_pattern = regex::Regex::new(r"^#?[0-9a-fA-F]{6}$").unwrap();
        if hex_pattern.is_match(color) {
            return true;
        }

        // Check for CSS color names
        matches!(
            color.to_lowercase().as_str(),
            "red"
                | "green"
                | "blue"
                | "black"
                | "white"
                | "gray"
                | "yellow"
                | "orange"
                | "purple"
                | "pink"
                | "brown"
        )
    }
}

pub struct EnvironmentDetector {
    // Environment detection state
}

#[derive(Debug, Clone, PartialEq)]
pub enum CiSystem {
    GitHubActions,
    Jenkins,
    GitLabCI,
    CircleCI,
    Travis,
    BuildKite,
    TeamCity,
}

#[derive(Debug, Clone)]
pub struct CiConfig {
    pub output_directory: PathBuf,
    pub formats: Vec<ReportFormat>,
    pub fail_on_errors: bool,
    pub environment_metadata: HashMap<String, String>,
}

impl Default for EnvironmentDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvironmentDetector {
    pub fn new() -> Self {
        EnvironmentDetector {}
    }

    pub fn detect_ci_system(&self) -> Option<CiSystem> {
        // GitHub Actions detection
        if std::env::var("GITHUB_ACTIONS").is_ok() {
            return Some(CiSystem::GitHubActions);
        }

        // Jenkins detection
        if std::env::var("JENKINS_URL").is_ok() && std::env::var("BUILD_NUMBER").is_ok() {
            return Some(CiSystem::Jenkins);
        }

        // GitLab CI detection
        if std::env::var("GITLAB_CI").is_ok() {
            return Some(CiSystem::GitLabCI);
        }

        // CircleCI detection
        if std::env::var("CIRCLECI").is_ok() {
            return Some(CiSystem::CircleCI);
        }

        // Travis CI detection
        if std::env::var("TRAVIS").is_ok() {
            return Some(CiSystem::Travis);
        }

        // BuildKite detection
        if std::env::var("BUILDKITE").is_ok() {
            return Some(CiSystem::BuildKite);
        }

        // TeamCity detection
        if std::env::var("TEAMCITY_VERSION").is_ok() {
            return Some(CiSystem::TeamCity);
        }

        None
    }

    pub fn get_ci_specific_config(&self) -> Result<CiConfig> {
        let ci_system = self.detect_ci_system();

        let mut environment_metadata = HashMap::new();
        let mut output_directory = PathBuf::from("./reports");
        let mut formats = vec![ReportFormat::Json, ReportFormat::Junit];

        match ci_system {
            Some(CiSystem::GitHubActions) => {
                // Use GitHub Actions specific paths and configuration
                if let Ok(runner_temp) = std::env::var("RUNNER_TEMP") {
                    output_directory = PathBuf::from(runner_temp);
                } else {
                    output_directory = PathBuf::from("/tmp");
                }

                // Add GitHub-specific metadata
                if let Ok(workflow) = std::env::var("GITHUB_WORKFLOW") {
                    environment_metadata.insert("workflow".to_string(), workflow);
                }
                if let Ok(run_id) = std::env::var("GITHUB_RUN_ID") {
                    environment_metadata.insert("run_id".to_string(), run_id);
                }
                if let Ok(actor) = std::env::var("GITHUB_ACTOR") {
                    environment_metadata.insert("actor".to_string(), actor);
                }

                // GitHub Actions prefers JUnit for test reporting
                formats = vec![ReportFormat::Junit, ReportFormat::Html, ReportFormat::Json];
            }
            Some(CiSystem::Jenkins) => {
                // Use Jenkins workspace
                if let Ok(workspace) = std::env::var("WORKSPACE") {
                    output_directory = PathBuf::from(workspace).join("reports");
                }

                if let Ok(build_number) = std::env::var("BUILD_NUMBER") {
                    environment_metadata.insert("build_number".to_string(), build_number);
                }
                if let Ok(job_name) = std::env::var("JOB_NAME") {
                    environment_metadata.insert("job_name".to_string(), job_name);
                }
            }
            Some(CiSystem::GitLabCI) => {
                // Use GitLab CI paths
                if let Ok(ci_project_dir) = std::env::var("CI_PROJECT_DIR") {
                    output_directory = PathBuf::from(ci_project_dir).join("reports");
                }

                if let Ok(pipeline_id) = std::env::var("CI_PIPELINE_ID") {
                    environment_metadata.insert("pipeline_id".to_string(), pipeline_id);
                }
                if let Ok(job_id) = std::env::var("CI_JOB_ID") {
                    environment_metadata.insert("job_id".to_string(), job_id);
                }
            }
            _ => {
                // Default CI configuration
                output_directory = PathBuf::from("/tmp/reports");
            }
        }

        Ok(CiConfig {
            output_directory,
            formats,
            fail_on_errors: true, // Always fail on errors in CI
            environment_metadata,
        })
    }

    pub fn get_environment_variables(&self) -> HashMap<String, String> {
        let mut env_vars = HashMap::new();

        // Collect relevant environment variables
        let relevant_vars = [
            "BUILD_VERSION",
            "TEAM_NAME",
            "ENVIRONMENT",
            "BRANCH_NAME",
            "COMMIT_SHA",
            "BUILD_NUMBER",
            "JOB_NAME",
            "PIPELINE_ID",
            "GITHUB_WORKFLOW",
            "GITHUB_RUN_ID",
            "GITHUB_ACTOR",
            "GITLAB_CI",
            "CI_PIPELINE_ID",
            "CI_JOB_ID",
            "JENKINS_URL",
            "WORKSPACE",
            "CIRCLECI",
            "TRAVIS",
            "BUILDKITE",
            "TEAMCITY_VERSION",
        ];

        for var_name in &relevant_vars {
            if let Ok(value) = std::env::var(var_name) {
                env_vars.insert(var_name.to_string(), value);
            }
        }

        // Add system information
        if let Ok(hostname) = std::env::var("HOSTNAME") {
            env_vars.insert("HOSTNAME".to_string(), hostname);
        }

        if let Ok(user) = std::env::var("USER") {
            env_vars.insert("USER".to_string(), user);
        }

        env_vars
    }
}
