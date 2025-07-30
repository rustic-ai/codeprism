//! Template engine for secure HTML report generation
//!
//! Provides safe template rendering using Tera with built-in templates
//! and support for custom branding and templates.

use crate::error::{Error, Result};
use crate::reporting::{BuiltInTemplate, TemplateContext, TemplateSource};
use std::collections::HashMap;
use std::time::Duration;
use tera::{Context, Tera};

/// Secure template renderer with sandboxing
#[derive(Debug)]
pub struct TemplateRenderer {
    engine: Tera,
    #[allow(dead_code)]
    // PLANNED(#202): Will be used for timeout controls in performance monitoring
    max_execution_time: Duration,
}

impl TemplateRenderer {
    /// Create new template renderer with built-in templates
    pub fn new() -> Result<Self> {
        let mut tera = Tera::new("templates/**/*").unwrap_or_default();

        // Add built-in templates
        Self::register_builtin_templates(&mut tera)?;

        // Configure security settings
        tera.autoescape_on(vec![".html", ".xml"]);

        // Register safe filters
        Self::register_safe_filters(&mut tera)?;

        Ok(Self {
            engine: tera,
            max_execution_time: Duration::from_secs(30),
        })
    }

    /// Render template with given context
    pub fn render_template(
        &mut self,
        source: &TemplateSource,
        context: &TemplateContext,
    ) -> Result<String> {
        match source {
            TemplateSource::BuiltIn(template) => self.render_builtin_template(template, context),
            TemplateSource::Custom { path: _ } => Err(Error::execution(
                "Custom file templates not yet implemented",
            )),
            TemplateSource::Inline { content } => self.render_inline_template(content, context),
        }
    }

    /// Render built-in template
    fn render_builtin_template(
        &self,
        template: &BuiltInTemplate,
        context: &TemplateContext,
    ) -> Result<String> {
        let template_name = template.name();
        let tera_context = self.create_tera_context(context)?;

        self.engine
            .render(template_name, &tera_context)
            .map_err(|e| Error::execution(format!("Template rendering failed: {e}")))
    }

    /// Render inline template
    fn render_inline_template(
        &mut self,
        content: &str,
        context: &TemplateContext,
    ) -> Result<String> {
        let tera_context = self.create_tera_context(context)?;

        // Validate template content for security
        self.validate_template_content(content)?;

        self.engine
            .render_str(content, &tera_context)
            .map_err(|e| Error::execution(format!("Inline template rendering failed: {e}")))
    }

    /// Create Tera context from template context
    fn create_tera_context(&self, context: &TemplateContext) -> Result<Context> {
        let mut tera_context = Context::new();

        // Add all template context fields
        tera_context.insert("report_id", &context.report_id);
        tera_context.insert("generated_at", &context.generated_at);
        tera_context.insert("version", &context.version);
        tera_context.insert("summary", &context.summary);
        tera_context.insert("test_results", &context.test_results);
        tera_context.insert("performance_metrics", &context.performance_metrics);
        tera_context.insert("environment", &context.environment);
        tera_context.insert("server_config", &context.server_config);
        tera_context.insert("branding", &context.branding);
        tera_context.insert("custom_fields", &context.custom_fields);

        Ok(tera_context)
    }

    /// Register built-in templates
    fn register_builtin_templates(tera: &mut Tera) -> Result<()> {
        // Professional template
        tera.add_raw_template("professional", include_str!("templates/professional.html"))
            .map_err(|e| {
                Error::execution(format!("Failed to register professional template: {e}"))
            })?;

        // Executive summary template
        tera.add_raw_template(
            "executive-summary",
            include_str!("templates/executive-summary.html"),
        )
        .map_err(|e| {
            Error::execution(format!(
                "Failed to register executive-summary template: {e}"
            ))
        })?;

        // Technical detailed template
        tera.add_raw_template(
            "technical-detailed",
            include_str!("templates/technical-detailed.html"),
        )
        .map_err(|e| {
            Error::execution(format!(
                "Failed to register technical-detailed template: {e}"
            ))
        })?;

        // Minimal template
        tera.add_raw_template("minimal", include_str!("templates/minimal.html"))
            .map_err(|e| Error::execution(format!("Failed to register minimal template: {e}")))?;

        Ok(())
    }

    /// Register safe template filters
    fn register_safe_filters(tera: &mut Tera) -> Result<()> {
        // Duration formatting filter
        tera.register_filter(
            "format_duration",
            |value: &tera::Value, _args: &HashMap<String, tera::Value>| {
                if let Some(duration_nanos) = value.as_u64() {
                    let duration = Duration::from_nanos(duration_nanos);
                    let ms = duration.as_millis();
                    if ms < 1000 {
                        Ok(tera::Value::String(format!("{ms}ms")))
                    } else {
                        Ok(tera::Value::String(format!(
                            "{:.2}s",
                            duration.as_secs_f64()
                        )))
                    }
                } else {
                    Ok(tera::Value::String("N/A".to_string()))
                }
            },
        );

        // Percentage formatting filter
        tera.register_filter(
            "format_percentage",
            |value: &tera::Value, _args: &HashMap<String, tera::Value>| {
                if let Some(percentage) = value.as_f64() {
                    Ok(tera::Value::String(format!("{percentage:.1}%")))
                } else {
                    Ok(tera::Value::String("N/A".to_string()))
                }
            },
        );

        // Status icon filter
        tera.register_filter(
            "status_icon",
            |value: &tera::Value, _args: &HashMap<String, tera::Value>| {
                if let Some(status) = value.as_str() {
                    let icon = match status.to_lowercase().as_str() {
                        "passed" => "✅",
                        "failed" => "❌",
                        "skipped" => "⏭️",
                        "timeout" => "⏱️",
                        "error" => "💥",
                        _ => "❓",
                    };
                    Ok(tera::Value::String(icon.to_string()))
                } else {
                    Ok(tera::Value::String("❓".to_string()))
                }
            },
        );

        Ok(())
    }

    /// Validate template content for security
    fn validate_template_content(&self, content: &str) -> Result<()> {
        // Check for unsafe patterns
        let unsafe_patterns = vec!["include", "load", "import", "{% raw %}", "{% endraw %}"];

        for pattern in unsafe_patterns {
            if content.contains(pattern) {
                return Err(Error::validation(format!(
                    "Unsafe template operation '{pattern}' not allowed"
                )));
            }
        }

        Ok(())
    }
}

impl Default for TemplateRenderer {
    fn default() -> Self {
        Self::new().expect("Failed to create default template renderer")
    }
}
