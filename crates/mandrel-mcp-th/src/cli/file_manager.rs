use super::{OrganizationStrategy, ReportFormat, TemplateName, TimestampFormat};
use crate::error::Result;
use chrono::{Datelike, Utc};
use std::path::{Path, PathBuf};

pub struct FileManager {
    base_directory: PathBuf,
    organization: OrganizationStrategy,
    timestamp: TimestampFormat,
}

impl FileManager {
    pub fn new(
        base_directory: PathBuf,
        organization: OrganizationStrategy,
        timestamp: TimestampFormat,
    ) -> Result<Self> {
        // Ensure base directory exists
        std::fs::create_dir_all(&base_directory)?;

        Ok(FileManager {
            base_directory,
            organization,
            timestamp,
        })
    }

    pub fn generate_output_path(
        &self,
        format: &ReportFormat,
        template: Option<&TemplateName>,
        suite_name: &str,
    ) -> Result<PathBuf> {
        let mut path = self.base_directory.clone();

        // Apply organization strategy
        match self.organization {
            OrganizationStrategy::Flat => {
                // All files in base directory
            }
            OrganizationStrategy::ByDate => {
                let now = Utc::now();
                path.push(format!("{:04}", now.year()));
                path.push(format!("{:02}", now.month()));
                path.push(format!("{:02}", now.day()));
            }
            OrganizationStrategy::ByFormat => {
                path.push(format.to_directory_name());
            }
            OrganizationStrategy::ByTemplate => {
                if let Some(template) = template {
                    path.push(template.to_directory_name());
                }
            }
        }

        // Ensure directory exists
        std::fs::create_dir_all(&path)?;

        // Generate filename
        let timestamp = self.generate_timestamp();
        let extension = format.file_extension();
        let filename = match timestamp {
            Some(ts) => format!("{suite_name}_{ts}.{extension}"),
            None => format!("{suite_name}.{extension}"),
        };

        path.push(filename);
        Ok(path)
    }

    pub fn write_report(&self, path: &PathBuf, content: &str) -> Result<()> {
        std::fs::write(path, content)?;
        Ok(())
    }

    fn generate_timestamp(&self) -> Option<String> {
        let now = Utc::now();
        match self.timestamp {
            TimestampFormat::Iso => Some(now.format("%Y-%m-%dT%H-%M-%SZ").to_string()),
            TimestampFormat::Unix => Some(now.timestamp().to_string()),
            TimestampFormat::Human => Some(now.format("%Y-%m-%d_%H-%M-%S").to_string()),
            TimestampFormat::None => None,
        }
    }

    pub fn ensure_directory_exists(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Ok(())
    }
}
