//! Template Import Service
//!
//! Handles importing prompt templates from YAML/JSON files.
//! Supports version management and automatic version incrementing.

use anyhow::{anyhow, Result};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};

use crate::models::_entities::prompt_templates;
use crate::services::template::TemplateService;

/// Template file format for import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateFile {
    /// Metadata about the template
    pub metadata: TemplateMetadata,

    /// System prompt content
    pub system_prompt: String,

    /// User prompt template with placeholders
    pub user_prompt_template: String,

    /// Optional validation rules
    #[serde(default)]
    pub validation_rules: Vec<String>,
}

/// Template metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMetadata {
    /// Template name (e.g., "xframe5-list")
    pub name: String,

    /// Product identifier (e.g., "xframe5-ui")
    pub product: String,

    /// Screen type (e.g., "list", "detail")
    pub screen_type: Option<String>,

    /// Version number (will be auto-incremented if exists)
    #[serde(default)]
    pub version: Option<i32>,

    /// Author/vendor name
    #[serde(default)]
    pub author: Option<String>,

    /// Release date
    #[serde(default)]
    pub release_date: Option<String>,

    /// Changelog description
    #[serde(default)]
    pub changelog: Option<String>,
}

/// Import options
#[derive(Debug, Clone)]
pub struct ImportOptions {
    /// Deactivate previous versions
    pub deactivate_old: bool,

    /// Force version number (override auto-increment)
    pub force_version: Option<i32>,

    /// Make this version active
    pub set_active: bool,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            deactivate_old: true,
            force_version: None,
            set_active: true,
        }
    }
}

/// Import result
#[derive(Debug, Clone, Serialize)]
pub struct ImportResult {
    pub success: bool,
    pub template_id: Option<i32>,
    pub version: i32,
    pub message: String,
    pub previous_version: Option<i32>,
}

/// Service for importing templates from files
pub struct TemplateImporter;

impl TemplateImporter {
    /// Import a template from YAML content
    pub async fn import_from_yaml(
        db: &DatabaseConnection,
        yaml_content: &str,
        options: ImportOptions,
    ) -> Result<ImportResult> {
        // 1. Parse YAML
        let template_file: TemplateFile = serde_yaml::from_str(yaml_content)
            .map_err(|e| anyhow!("Failed to parse YAML: {}", e))?;

        // 2. Validate
        Self::validate_template(&template_file)?;

        // 3. Import
        Self::import_template(db, template_file, options).await
    }

    /// Import a template from JSON content
    pub async fn import_from_json(
        db: &DatabaseConnection,
        json_content: &str,
        options: ImportOptions,
    ) -> Result<ImportResult> {
        // 1. Parse JSON
        let template_file: TemplateFile = serde_json::from_str(json_content)
            .map_err(|e| anyhow!("Failed to parse JSON: {}", e))?;

        // 2. Validate
        Self::validate_template(&template_file)?;

        // 3. Import
        Self::import_template(db, template_file, options).await
    }

    /// Validate template structure
    fn validate_template(template: &TemplateFile) -> Result<()> {
        if template.metadata.name.is_empty() {
            return Err(anyhow!("Template name is required"));
        }
        if template.metadata.product.is_empty() {
            return Err(anyhow!("Product is required"));
        }
        if template.system_prompt.is_empty() {
            return Err(anyhow!("System prompt is required"));
        }
        if template.user_prompt_template.is_empty() {
            return Err(anyhow!("User prompt template is required"));
        }

        Ok(())
    }

    /// Import template into database
    async fn import_template(
        db: &DatabaseConnection,
        template: TemplateFile,
        options: ImportOptions,
    ) -> Result<ImportResult> {
        let meta = &template.metadata;

        // 1. Determine version number
        let latest_version = TemplateService::get_latest_version(
            db,
            &meta.product,
            &meta.name,
        )
        .await
        .unwrap_or(0);

        let new_version = if let Some(forced) = options.force_version {
            forced
        } else if let Some(requested) = meta.version {
            // Use requested version if higher than latest
            requested.max(latest_version + 1)
        } else {
            // Auto-increment
            latest_version + 1
        };

        // 2. Deactivate old versions if requested
        if options.deactivate_old && latest_version > 0 {
            Self::deactivate_versions(db, &meta.product, &meta.name).await?;
        }

        // 3. Insert new template
        let new_template = prompt_templates::ActiveModel {
            name: Set(meta.name.clone()),
            product: Set(meta.product.clone()),
            screen_type: Set(meta.screen_type.clone()),
            system_prompt: Set(template.system_prompt.clone()),
            user_prompt_template: Set(template.user_prompt_template.clone()),
            version: Set(new_version),
            is_active: Set(Some(options.set_active)),
            ..Default::default()
        };

        let inserted = new_template.insert(db).await?;

        Ok(ImportResult {
            success: true,
            template_id: Some(inserted.id),
            version: new_version,
            message: format!(
                "Successfully imported {} v{} (previous: v{})",
                meta.name,
                new_version,
                if latest_version > 0 { latest_version.to_string() } else { "none".to_string() }
            ),
            previous_version: if latest_version > 0 { Some(latest_version) } else { None },
        })
    }

    /// Deactivate all versions of a template
    async fn deactivate_versions(
        db: &DatabaseConnection,
        product: &str,
        name: &str,
    ) -> Result<()> {
        // Find all templates with this product/name
        let templates = prompt_templates::Entity::find()
            .filter(prompt_templates::Column::Product.eq(product))
            .filter(prompt_templates::Column::Name.eq(name))
            .all(db)
            .await?;

        // Deactivate each one
        for template in templates {
            let mut active_model: prompt_templates::ActiveModel = template.into();
            active_model.is_active = Set(Some(false));
            active_model.update(db).await?;
        }

        Ok(())
    }

    /// Export a template to YAML format
    pub async fn export_to_yaml(
        db: &DatabaseConnection,
        template_id: i32,
    ) -> Result<String> {
        let template = TemplateService::get_by_id(db, template_id).await?;

        let template_file = TemplateFile {
            metadata: TemplateMetadata {
                name: template.name,
                product: template.product,
                screen_type: template.screen_type,
                version: Some(template.version),
                author: None,
                release_date: None,
                changelog: None,
            },
            system_prompt: template.system_prompt,
            user_prompt_template: template.user_prompt_template,
            validation_rules: vec![],
        };

        serde_yaml::to_string(&template_file)
            .map_err(|e| anyhow!("Failed to serialize to YAML: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_yaml() {
        let yaml = r#"
metadata:
  name: test-template
  product: test-product
  screen_type: list
  version: 1
  author: Test Author
  changelog: Initial version

system_prompt: |
  You are a test generator.

user_prompt_template: |
  Generate {{entity_name}}.

validation_rules:
  - must_have_entity_name
"#;

        let result = serde_yaml::from_str::<TemplateFile>(yaml);
        assert!(result.is_ok());

        let template = result.unwrap();
        assert_eq!(template.metadata.name, "test-template");
        assert_eq!(template.metadata.product, "test-product");
        assert!(template.system_prompt.contains("test generator"));
    }

    #[test]
    fn test_validate_template() {
        let mut template = TemplateFile {
            metadata: TemplateMetadata {
                name: "test".to_string(),
                product: "test-product".to_string(),
                screen_type: None,
                version: None,
                author: None,
                release_date: None,
                changelog: None,
            },
            system_prompt: "System prompt".to_string(),
            user_prompt_template: "User prompt".to_string(),
            validation_rules: vec![],
        };

        assert!(TemplateImporter::validate_template(&template).is_ok());

        // Test empty name
        template.metadata.name = String::new();
        assert!(TemplateImporter::validate_template(&template).is_err());
    }
}
