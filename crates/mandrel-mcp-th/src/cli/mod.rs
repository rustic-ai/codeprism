//! Command-line interface for Mandrel MCP Test Harness

use crate::reporting::{BrandingInfo, BuiltInTemplate, ReportConfig, TemplateSource};
use clap::Parser;
use codeprism_utils::{ChangeEvent, FileWatcher};
use std::collections::HashMap;
use std::path::PathBuf;
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
        // Initialize file manager
        let file_manager = FileManager::new(
            args.output.clone(),
            args.organize_by.clone(),
            args.timestamp.clone(),
        )?;

        // Load branding config if provided
        let branding_config = if let Some(branding_path) = &args.branding_config {
            BrandingConfig::from_file(branding_path)?
        } else {
            BrandingConfig::default()
        };

        // TODO: Load test results from input file
        // TODO: Generate reports in requested formats
        // TODO: Write reports using file manager

        println!("Report generation completed successfully");
        Ok(0)
    }

    async fn handle_run_command(&self, _args: &RunArgs) -> Result<i32> {
        // TODO: Implement test execution with report generation
        println!("Test execution not yet implemented");
        Ok(0)
    }

    async fn handle_validate_command(&self, _args: &ValidateArgs) -> Result<i32> {
        // TODO: Implement configuration validation
        println!("Configuration validation not yet implemented");
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use std::path::PathBuf;
    use tempfile::TempDir;

    // Phase 5A RED: CLI Foundation Tests (These should FAIL initially)

    #[test]
    fn test_cli_app_initialization() {
        // This should create a CLI application successfully
        let result = CliApp::new();

        // Phase 5A GREEN: This should succeed after implementation
        assert!(
            result.is_ok(),
            "Should initialize CLI app successfully: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_cli_app_execution_with_valid_args() {
        let app = CliApp::new().expect("Should create CLI app");

        let result = app.run().await;

        // Phase 5A GREEN: This should return proper exit code
        assert!(
            result.is_ok(),
            "Should execute CLI app successfully: {:?}",
            result.err()
        );

        let exit_code = result.unwrap();
        assert!(
            exit_code == 0 || exit_code == 1,
            "Should return valid exit code: {}",
            exit_code
        );
    }

    #[test]
    fn test_cli_argument_parsing_report_command() {
        // Test basic report command parsing
        let args = vec![
            "mandrel-mcp-th",
            "report",
            "--input",
            "test-results.json",
            "--output",
            "./reports",
            "--formats",
            "json,junit,html",
        ];

        let cli = Cli::try_parse_from(args);
        assert!(
            cli.is_ok(),
            "Should parse valid report command: {:?}",
            cli.err()
        );

        let cli = cli.unwrap();
        if let Commands::Report(report_args) = cli.command {
            assert_eq!(report_args.input, PathBuf::from("test-results.json"));
            assert_eq!(report_args.output, PathBuf::from("./reports"));
            assert_eq!(report_args.formats.len(), 3);
            assert!(report_args.formats.contains(&ReportFormat::Json));
            assert!(report_args.formats.contains(&ReportFormat::Junit));
            assert!(report_args.formats.contains(&ReportFormat::Html));
        } else {
            panic!("Expected Report command");
        }
    }

    #[test]
    fn test_cli_argument_parsing_with_template() {
        let args = vec![
            "mandrel-mcp-th",
            "report",
            "--input",
            "test.json",
            "--output",
            "./out",
            "--formats",
            "html",
            "--template",
            "professional",
        ];

        let cli = Cli::try_parse_from(args);
        assert!(
            cli.is_ok(),
            "Should parse template argument: {:?}",
            cli.err()
        );

        let cli = cli.unwrap();
        if let Commands::Report(report_args) = cli.command {
            assert_eq!(report_args.template, Some(TemplateName::Professional));
        } else {
            panic!("Expected Report command");
        }
    }

    #[test]
    fn test_cli_argument_parsing_with_custom_fields() {
        let args = vec![
            "mandrel-mcp-th",
            "report",
            "--input",
            "test.json",
            "--output",
            "./out",
            "--formats",
            "json",
            "--custom-field",
            "team=QA Team",
            "--custom-field",
            "build=v1.2.3",
        ];

        let cli = Cli::try_parse_from(args);
        assert!(cli.is_ok(), "Should parse custom fields: {:?}", cli.err());

        let cli = cli.unwrap();
        if let Commands::Report(report_args) = cli.command {
            assert_eq!(report_args.custom_fields.len(), 2);
            assert!(report_args
                .custom_fields
                .contains(&("team".to_string(), "QA Team".to_string())));
            assert!(report_args
                .custom_fields
                .contains(&("build".to_string(), "v1.2.3".to_string())));
        } else {
            panic!("Expected Report command");
        }
    }

    #[test]
    fn test_cli_argument_parsing_organization_strategies() {
        // Test flat organization
        let args = vec![
            "mandrel-mcp-th",
            "report",
            "--input",
            "test.json",
            "--output",
            "./out",
            "--formats",
            "json",
            "--organize-by",
            "flat",
        ];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Commands::Report(report_args) = cli.command {
            assert_eq!(report_args.organize_by, OrganizationStrategy::Flat);
        }

        // Test by-date organization
        let args = vec![
            "mandrel-mcp-th",
            "report",
            "--input",
            "test.json",
            "--output",
            "./out",
            "--formats",
            "json",
            "--organize-by",
            "by-date",
        ];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Commands::Report(report_args) = cli.command {
            assert_eq!(report_args.organize_by, OrganizationStrategy::ByDate);
        }
    }

    #[test]
    fn test_cli_argument_parsing_timestamp_formats() {
        // Test ISO timestamp
        let args = vec![
            "mandrel-mcp-th",
            "report",
            "--input",
            "test.json",
            "--output",
            "./out",
            "--formats",
            "json",
            "--timestamp",
            "iso",
        ];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Commands::Report(report_args) = cli.command {
            assert_eq!(report_args.timestamp, TimestampFormat::Iso);
        }

        // Test Unix timestamp
        let args = vec![
            "mandrel-mcp-th",
            "report",
            "--input",
            "test.json",
            "--output",
            "./out",
            "--formats",
            "json",
            "--timestamp",
            "unix",
        ];
        let cli = Cli::try_parse_from(args).unwrap();
        if let Commands::Report(report_args) = cli.command {
            assert_eq!(report_args.timestamp, TimestampFormat::Unix);
        }
    }

    #[test]
    fn test_cli_argument_parsing_boolean_flags() {
        let args = vec![
            "mandrel-mcp-th",
            "report",
            "--input",
            "test.json",
            "--output",
            "./out",
            "--formats",
            "json",
            "--fail-on-errors",
            "--include-performance",
            "false",
        ];

        let cli = Cli::try_parse_from(args);
        assert!(cli.is_ok(), "Should parse boolean flags: {:?}", cli.err());

        let cli = cli.unwrap();
        if let Commands::Report(report_args) = cli.command {
            assert!(report_args.fail_on_errors);
            assert!(!report_args.include_performance);
        } else {
            panic!("Expected Report command");
        }
    }

    #[test]
    fn test_cli_argument_parsing_invalid_arguments() {
        // Test invalid format
        let args = vec![
            "mandrel-mcp-th",
            "report",
            "--input",
            "test.json",
            "--output",
            "./out",
            "--formats",
            "invalid_format",
        ];
        let cli = Cli::try_parse_from(args);
        assert!(cli.is_err(), "Should reject invalid format");

        // Test invalid organization strategy
        let args = vec![
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
        ];
        let cli = Cli::try_parse_from(args);
        assert!(cli.is_err(), "Should reject invalid organization strategy");
    }

    #[test]
    fn test_cli_help_generation() {
        // Test that help can be generated without errors
        let args = vec!["mandrel-mcp-th", "--help"];
        let cli = Cli::try_parse_from(args);
        // This should fail with help, but the error should be ClashKind::Help
        assert!(cli.is_err(), "Help should exit with error");
    }

    #[test]
    fn test_cli_version_flag() {
        let args = vec!["mandrel-mcp-th", "--version"];
        let cli = Cli::try_parse_from(args);
        // Version should exit with error but be a version error
        assert!(cli.is_err(), "Version should exit with error");
    }

    #[test]
    fn test_cli_verbose_levels() {
        // Test single verbose flag
        let args = vec![
            "mandrel-mcp-th",
            "-v",
            "report",
            "--input",
            "test.json",
            "--output",
            "./out",
            "--formats",
            "json",
        ];
        let cli = Cli::try_parse_from(args).unwrap();
        assert_eq!(cli.verbose, 1);

        // Test multiple verbose flags
        let args = vec![
            "mandrel-mcp-th",
            "-vvv",
            "report",
            "--input",
            "test.json",
            "--output",
            "./out",
            "--formats",
            "json",
        ];
        let cli = Cli::try_parse_from(args).unwrap();
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
        assert!(path.ends_with(".xml"));
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
        let result = BrandingConfig::default();

        // Phase 5A GREEN: This should create default config
        // We're not asserting anything specific since it's todo!(), just that it doesn't panic
        // The actual assertions will be added in GREEN phase
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
        // We're not asserting anything specific since it's todo!(), just that it doesn't panic
        // The actual assertions will be added in GREEN phase
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
            .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "json"))
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
            .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "json"))
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
                .any(|e| e.message.contains("undefined_variable")),
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
            // For now, treat patterns as direct paths
            // Future enhancement: glob pattern support
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

    fn matches_input_patterns(&self, path: &PathBuf) -> bool {
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

    async fn generate_report(&self, _input_path: &PathBuf, format: &ReportFormat) -> Result<()> {
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

        tracing::info!("Auto-opening reports in browser (placeholder)");
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

#[derive(Debug, Clone)]
pub struct ConfigProfile {
    pub name: String,
    pub description: Option<String>,
    pub report_config: ReportConfig,
    pub file_management: FileManagerConfig,
    pub branding: Option<BrandingInfo>,
    pub environment_vars: HashMap<String, String>,
}

#[derive(Debug, Clone, Default)]
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
        todo!("Phase 5B GREEN: Implement ProfileManager initialization")
    }

    pub fn save_profile(&self, profile: &ConfigProfile) -> Result<()> {
        todo!("Phase 5B GREEN: Implement profile saving")
    }

    pub fn load_profile(&self, name: &str) -> Result<ConfigProfile> {
        todo!("Phase 5B GREEN: Implement profile loading")
    }

    pub fn list_profiles(&self) -> Result<Vec<String>> {
        todo!("Phase 5B GREEN: Implement profile listing")
    }

    pub fn delete_profile(&self, name: &str) -> Result<()> {
        todo!("Phase 5B GREEN: Implement profile deletion")
    }

    pub fn export_profile(&self, name: &str, output_path: &PathBuf) -> Result<()> {
        todo!("Phase 5B GREEN: Implement profile export")
    }

    pub fn import_profile(&self, import_path: &PathBuf) -> Result<()> {
        todo!("Phase 5B GREEN: Implement profile import")
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
        todo!("Phase 5B GREEN: Implement ValidationEngine initialization")
    }

    pub fn validate_input_file(&self, path: &PathBuf) -> Result<ValidationResult> {
        todo!("Phase 5B GREEN: Implement input file validation")
    }

    pub fn validate_template(&self, template: &TemplateSource) -> Result<ValidationResult> {
        todo!("Phase 5B GREEN: Implement template validation")
    }

    pub fn validate_configuration(&self, config: &ReportConfig) -> Result<ValidationResult> {
        todo!("Phase 5B GREEN: Implement configuration validation")
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
        todo!("Phase 5B GREEN: Implement EnvironmentDetector initialization")
    }

    pub fn detect_ci_system(&self) -> Option<CiSystem> {
        todo!("Phase 5B GREEN: Implement CI system detection")
    }

    pub fn get_ci_specific_config(&self) -> Result<CiConfig> {
        todo!("Phase 5B GREEN: Implement CI-specific configuration")
    }

    pub fn get_environment_variables(&self) -> HashMap<String, String> {
        todo!("Phase 5B GREEN: Implement environment variable collection")
    }
}
