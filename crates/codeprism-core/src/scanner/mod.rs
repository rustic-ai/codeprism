//! Repository scanner for discovering and filtering source files

use crate::ast::Language;
use crate::error::{Error, Result};
use rayon::prelude::*;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use walkdir::WalkDir;

/// File discovery result
#[derive(Debug, Clone)]
pub struct DiscoveredFile {
    /// File path
    pub path: PathBuf,
    /// Detected language
    pub language: Language,
    /// File size in bytes
    pub size: usize,
}

/// Repository scan result
#[derive(Debug)]
pub struct ScanResult {
    /// Total files discovered
    pub total_files: usize,
    /// Files by language
    pub files_by_language: std::collections::HashMap<Language, Vec<DiscoveredFile>>,
    /// Scan duration in milliseconds
    pub duration_ms: u64,
    /// Errors encountered during scan
    pub errors: Vec<Error>,
}

impl ScanResult {
    /// Create a new empty scan result
    pub fn new() -> Self {
        Self {
            total_files: 0,
            files_by_language: std::collections::HashMap::new(),
            duration_ms: 0,
            errors: Vec::new(),
        }
    }

    /// Get total number of files discovered
    pub fn file_count(&self) -> usize {
        self.total_files
    }

    /// Get files for a specific language
    pub fn files_for_language(&self, language: Language) -> Vec<&DiscoveredFile> {
        self.files_by_language
            .get(&language)
            .map(|files| files.iter().collect())
            .unwrap_or_default()
    }

    /// Get all discovered files
    pub fn all_files(&self) -> Vec<&DiscoveredFile> {
        self.files_by_language.values().flatten().collect()
    }
}

impl Default for ScanResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Progress reporter for scan operations
pub trait ProgressReporter: Send + Sync {
    /// Report progress with current file count and estimated total
    fn report_progress(&self, current: usize, total: Option<usize>);

    /// Report completion
    fn report_complete(&self, result: &ScanResult);

    /// Report an error
    fn report_error(&self, error: &Error);
}

/// No-op progress reporter
#[derive(Debug, Default)]
pub struct NoOpProgressReporter;

impl ProgressReporter for NoOpProgressReporter {
    fn report_progress(&self, _current: usize, _total: Option<usize>) {}
    fn report_complete(&self, _result: &ScanResult) {}
    fn report_error(&self, _error: &Error) {}
}

/// How to handle dependency directories
#[derive(Debug, Clone, PartialEq)]
pub enum DependencyMode {
    /// Exclude all dependency directories
    Exclude,
    /// Include dependency directories with smart filtering
    Smart,
    /// Include all dependency directories
    IncludeAll,
}

/// Repository scanner for discovering source files
pub struct RepositoryScanner {
    supported_extensions: std::collections::HashSet<String>,
    exclude_dirs: HashSet<String>,
    dependency_mode: DependencyMode,
}

impl RepositoryScanner {
    /// Create a new repository scanner
    pub fn new() -> Self {
        let mut supported_extensions = std::collections::HashSet::new();
        supported_extensions.extend(
            [
                "js", "mjs", "cjs", "jsx", // JavaScript
                "ts", "tsx", // TypeScript
                "py", "pyw",  // Python
                "java", // Java
                "go",   // Go
                "rs",   // Rust
                "c", "h", // C
                "cpp", "cc", "cxx", "hpp", "hxx", // C++
            ]
            .iter()
            .map(|s| s.to_string()),
        );

        let mut exclude_dirs = HashSet::new();
        // Default exclusions - basic set that most projects will want
        exclude_dirs.insert(".git".to_string());
        exclude_dirs.insert("node_modules".to_string());
        exclude_dirs.insert("target".to_string());
        exclude_dirs.insert("build".to_string());
        exclude_dirs.insert("dist".to_string());
        exclude_dirs.insert(".vscode".to_string());
        exclude_dirs.insert(".idea".to_string());

        Self {
            supported_extensions,
            exclude_dirs,
            dependency_mode: DependencyMode::Exclude,
        }
    }

    /// Create a repository scanner with custom exclude directories
    pub fn with_exclude_dirs(exclude_dirs: Vec<String>) -> Self {
        let mut scanner = Self::new();
        scanner.exclude_dirs.clear();
        scanner.exclude_dirs.extend(exclude_dirs);
        scanner
    }

    /// Set dependency scanning mode
    pub fn with_dependency_mode(mut self, mode: DependencyMode) -> Self {
        self.dependency_mode = mode;
        self
    }

    /// Add additional directories to exclude
    pub fn add_exclude_dirs(&mut self, dirs: Vec<String>) {
        self.exclude_dirs.extend(dirs);
    }

    /// Set supported file extensions
    pub fn with_extensions(mut self, extensions: Vec<String>) -> Self {
        self.supported_extensions.clear();
        self.supported_extensions.extend(extensions);
        self
    }

    /// Scan a repository directory and discover source files
    pub async fn scan_repository<P: AsRef<Path>>(
        &self,
        repo_path: P,
        progress_reporter: Arc<dyn ProgressReporter>,
    ) -> Result<ScanResult> {
        let repo_path = repo_path.as_ref();
        let start_time = std::time::Instant::now();

        // Discover files
        let discovered_paths = self.discover_files(repo_path)?;
        progress_reporter.report_progress(discovered_paths.len(), Some(discovered_paths.len()));

        // Process files in parallel
        let processed_counter = Arc::new(AtomicUsize::new(0));
        let progress_clone = Arc::clone(&progress_reporter);
        let counter_clone = Arc::clone(&processed_counter);

        let mut result = ScanResult::new();

        // Process files in parallel batches
        let batch_size = 100;
        for chunk in discovered_paths.chunks(batch_size) {
            let discovered_files: Vec<_> = chunk
                .par_iter()
                .filter_map(|path| {
                    let processed = counter_clone.fetch_add(1, Ordering::Relaxed) + 1;
                    if processed % 50 == 0 {
                        progress_clone.report_progress(processed, Some(discovered_paths.len()));
                    }

                    match self.process_file(path) {
                        Ok(Some(file)) => Some(file),
                        Ok(None) => None, // Filtered out
                        Err(e) => {
                            progress_clone.report_error(&e);
                            None
                        }
                    }
                })
                .collect();

            // Group by language
            for file in discovered_files {
                result
                    .files_by_language
                    .entry(file.language)
                    .or_default()
                    .push(file);
                result.total_files += 1;
            }
        }

        result.duration_ms = start_time.elapsed().as_millis() as u64;
        progress_reporter.report_complete(&result);
        Ok(result)
    }

    /// Discover all potential files in the repository
    pub fn discover_files<P: AsRef<Path>>(&self, repo_path: P) -> Result<Vec<PathBuf>> {
        let repo_path = repo_path.as_ref();

        if !repo_path.exists() {
            return Err(Error::io(format!(
                "Repository path does not exist: {}",
                repo_path.display()
            )));
        }

        if !repo_path.is_dir() {
            return Err(Error::io(format!(
                "Repository path is not a directory: {}",
                repo_path.display()
            )));
        }

        let mut files = Vec::new();
        let walker = WalkDir::new(repo_path)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| {
                // Filter out excluded directories during walking for efficiency
                if e.path().is_dir() {
                    !self.should_exclude_directory(e.path(), repo_path)
                } else {
                    true
                }
            });

        for entry in walker {
            match entry {
                Ok(entry) => {
                    let path = entry.path();

                    // Skip directories - we only want files
                    if path.is_dir() {
                        continue;
                    }

                    // Check if it's a file we might be interested in
                    if self.should_include_file(path) {
                        files.push(path.to_path_buf());
                    }
                }
                Err(e) => {
                    // Log error but continue scanning
                    tracing::warn!("Error accessing file during scan: {}", e);
                }
            }
        }

        Ok(files)
    }

    /// Check if a directory should be excluded from scanning
    fn should_exclude_directory(&self, dir_path: &Path, repo_root: &Path) -> bool {
        // Get the relative path from repo root
        if let Ok(rel_path) = dir_path.strip_prefix(repo_root) {
            let path_components: Vec<&str> = rel_path
                .components()
                .filter_map(|c| c.as_os_str().to_str())
                .collect();

            // Check for dependency directories
            let is_in_dependency = self.is_in_dependency_directory(&path_components);

            match self.dependency_mode {
                DependencyMode::Exclude => {
                    // Fixed: Only exclude if the current directory name is in exclude list
                    // Don't exclude parent directories that contain excluded subdirectories
                    if let Some(current_dir_name) = path_components.last() {
                        if self.exclude_dirs.contains(*current_dir_name) {
                            return true;
                        }
                    }
                }
                DependencyMode::Smart => {
                    // Smart mode - exclude non-essential parts of dependencies
                    if is_in_dependency {
                        return self.should_exclude_dependency_directory(&path_components);
                    } else {
                        // For non-dependency directories, only exclude the specific directory
                        if let Some(current_dir_name) = path_components.last() {
                            if self.exclude_dirs.contains(*current_dir_name) {
                                return true;
                            }
                        }
                    }
                }
                DependencyMode::IncludeAll => {
                    // Only exclude basic directories (git, build artifacts, etc.)
                    let basic_excludes =
                        [".git", "build", "dist", ".vscode", ".idea", "__pycache__"];
                    if let Some(current_dir_name) = path_components.last() {
                        if basic_excludes.contains(current_dir_name) {
                            return true;
                        }
                    }
                }
            }
        }

        // Also check the immediate directory name (fallback)
        if let Some(dir_name) = dir_path.file_name().and_then(|n| n.to_str()) {
            match self.dependency_mode {
                DependencyMode::Exclude => self.exclude_dirs.contains(dir_name),
                DependencyMode::Smart => {
                    // In smart mode, only exclude if it's not a dependency dir or if it's a cache
                    let is_dependency =
                        ["node_modules", "venv", ".venv", ".tox", "vendor"].contains(&dir_name);
                    if is_dependency {
                        false // Don't exclude main dependency directories
                    } else {
                        self.exclude_dirs.contains(dir_name)
                    }
                }
                DependencyMode::IncludeAll => {
                    let basic_excludes =
                        [".git", "build", "dist", ".vscode", ".idea", "__pycache__"];
                    basic_excludes.contains(&dir_name)
                }
            }
        } else {
            false
        }
    }

    /// Check if we're inside a dependency directory
    fn is_in_dependency_directory(&self, path_components: &[&str]) -> bool {
        let dependency_dirs = ["node_modules", "venv", ".venv", ".tox", "vendor", "target"];
        path_components
            .iter()
            .any(|&component| dependency_dirs.contains(&component))
    }

    /// Smart filtering for dependency directories
    fn should_exclude_dependency_directory(&self, path_components: &[&str]) -> bool {
        // Find the dependency directory index
        let dependency_dirs = ["node_modules", "venv", ".venv", ".tox", "vendor", "target"];
        if let Some(dep_index) = path_components
            .iter()
            .position(|&c| dependency_dirs.contains(&c))
        {
            let depth_in_dependency = path_components.len() - dep_index - 1;

            // Exclude deep nested directories in dependencies (more than 3 levels deep)
            if depth_in_dependency > 3 {
                return true;
            }

            // Exclude certain patterns in dependencies
            let exclude_patterns = [
                "test",
                "tests",
                "__pycache__",
                ".pytest_cache",
                "docs",
                "examples",
                "benchmarks",
                "node_modules",
                "build",
                "dist",
                ".git",
                "coverage",
            ];

            for &component in &path_components[dep_index + 1..] {
                if exclude_patterns.contains(&component) {
                    return true;
                }
            }
        }

        false
    }

    /// Process a single file and create a DiscoveredFile if it should be included
    fn process_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Option<DiscoveredFile>> {
        let file_path = file_path.as_ref();

        // Get file metadata
        let metadata = std::fs::metadata(file_path).map_err(|e| {
            Error::io(format!(
                "Failed to read metadata for {}: {}",
                file_path.display(),
                e
            ))
        })?;

        let file_size = metadata.len() as usize;

        // Check file size limit - be more lenient for dependency files in smart mode
        let size_limit = match self.dependency_mode {
            DependencyMode::Smart => 20 * 1024 * 1024, // 20MB for dependencies
            _ => 10 * 1024 * 1024,                     // 10MB for regular files
        };

        if file_size > size_limit {
            return Ok(None); // Skip large files
        }

        // Detect language
        let language = self.detect_language(file_path);

        // Skip unknown languages
        if language == Language::Unknown {
            return Ok(None);
        }

        // Smart filtering for dependency files
        if self.dependency_mode == DependencyMode::Smart {
            if let Some(repo_root) = file_path.ancestors().nth(10) {
                // Approximate repo root
                if let Ok(rel_path) = file_path.strip_prefix(repo_root) {
                    let path_components: Vec<&str> = rel_path
                        .components()
                        .filter_map(|c| c.as_os_str().to_str())
                        .collect();

                    if self.is_in_dependency_directory(&path_components) {
                        // Only include important files from dependencies
                        if !self.is_important_dependency_file(file_path) {
                            return Ok(None);
                        }
                    }
                }
            }
        }

        Ok(Some(DiscoveredFile {
            path: file_path.to_path_buf(),
            language,
            size: file_size,
        }))
    }

    /// Check if a dependency file is important enough to include
    fn is_important_dependency_file(&self, file_path: &Path) -> bool {
        if let Some(file_name) = file_path.file_name().and_then(|n| n.to_str()) {
            // Always include main entry points and public APIs
            let important_files = [
                "__init__.py",
                "index.js",
                "index.ts",
                "lib.rs",
                "main.rs",
                "package.json",
                "setup.py",
                "Cargo.toml",
                "requirements.txt",
            ];

            if important_files.contains(&file_name) {
                return true;
            }

            // Include files without common internal indicators
            let internal_indicators = [
                "_internal",
                "_private",
                "internal",
                "private",
                ".test.",
                ".spec.",
                "_test",
                "_spec",
            ];

            let path_str = file_path.to_string_lossy().to_lowercase();
            if internal_indicators
                .iter()
                .any(|&indicator| path_str.contains(indicator))
            {
                return false;
            }

            // Include if it's in a top-level directory of the dependency
            if let Some(parent) = file_path.parent() {
                if let Some(parent_name) = parent.file_name().and_then(|n| n.to_str()) {
                    // If the parent is a dependency directory, this is likely a top-level file
                    let dependency_dirs = ["node_modules", "venv", ".venv", ".tox", "vendor"];
                    if dependency_dirs.contains(&parent_name) {
                        return true;
                    }
                }
            }
        }

        // Default to excluding to be conservative
        false
    }

    /// Check if a file should be included in the scan
    fn should_include_file<P: AsRef<Path>>(&self, file_path: P) -> bool {
        let file_path = file_path.as_ref();

        // Check file extension
        if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
            let ext_lower = ext.to_lowercase();

            // Check if it's a supported extension
            if self.supported_extensions.contains(&ext_lower) {
                return true;
            }
        }

        false
    }

    /// Detect programming language from file path
    pub fn detect_language<P: AsRef<Path>>(&self, file_path: P) -> Language {
        let file_path = file_path.as_ref();

        if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
            Language::from_extension(ext)
        } else {
            Language::Unknown
        }
    }
}

impl Default for RepositoryScanner {
    fn default() -> Self {
        Self::new()
    }
}
