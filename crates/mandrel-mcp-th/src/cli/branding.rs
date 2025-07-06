use crate::error::Result;
use crate::reporting::BrandingInfo;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Debug, Clone)]
pub struct BrandingConfig {
    pub company_name: Option<String>,
    pub logo_path: Option<PathBuf>,
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub css_overrides: Option<String>,
    pub custom_css_file: Option<PathBuf>,
}

impl BrandingConfig {
    pub fn from_file(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: BrandingConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn default() -> Self {
        BrandingConfig {
            company_name: None,
            logo_path: None,
            primary_color: None,
            secondary_color: None,
            css_overrides: None,
            custom_css_file: None,
        }
    }

    pub fn to_branding_info(&self) -> BrandingInfo {
        BrandingInfo {
            company_name: self.company_name.clone(),
            logo_path: self
                .logo_path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string()),
            primary_color: self.primary_color.clone(),
            secondary_color: self.secondary_color.clone(),
            css_overrides: self.load_css_overrides(),
        }
    }

    fn load_css_overrides(&self) -> Option<String> {
        // Priority: direct css_overrides, then css_file
        if let Some(css) = &self.css_overrides {
            return Some(css.clone());
        }

        if let Some(css_file) = &self.custom_css_file {
            if let Ok(content) = std::fs::read_to_string(css_file) {
                return Some(content);
            }
        }

        None
    }

    pub fn validate(&self) -> Result<()> {
        // Validate color formats if provided
        if let Some(color) = &self.primary_color {
            if !color.is_empty() && !color.chars().all(|c| c.is_ascii_alphanumeric()) {
                return Err(crate::error::Error::config("Invalid primary color format"));
            }
        }

        if let Some(color) = &self.secondary_color {
            if !color.is_empty() && !color.chars().all(|c| c.is_ascii_alphanumeric()) {
                return Err(crate::error::Error::config(
                    "Invalid secondary color format",
                ));
            }
        }

        // Validate file paths if provided
        if let Some(logo_path) = &self.logo_path {
            if !logo_path.exists() {
                return Err(crate::error::Error::config("Logo file does not exist"));
            }
        }

        if let Some(css_file) = &self.custom_css_file {
            if !css_file.exists() {
                return Err(crate::error::Error::config("CSS file does not exist"));
            }
        }

        Ok(())
    }
}

impl Default for BrandingConfig {
    fn default() -> Self {
        Self::default()
    }
}
