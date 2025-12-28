use crate::models::_entities::prompt_templates;
use anyhow::{anyhow, Result};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};

/// Service for loading prompt templates from database
pub struct TemplateService;

impl TemplateService {
    /// Get the active template for a product and screen type
    pub async fn get_active(
        db: &DatabaseConnection,
        product: &str,
        screen_type: Option<&str>,
    ) -> Result<prompt_templates::Model> {
        let mut query = prompt_templates::Entity::find()
            .filter(prompt_templates::Column::Product.eq(product))
            .filter(prompt_templates::Column::IsActive.eq(Some(true)));

        if let Some(st) = screen_type {
            query = query.filter(prompt_templates::Column::ScreenType.eq(Some(st.to_string())));
        }

        // Get the highest version
        let template = query
            .order_by_desc(prompt_templates::Column::Version)
            .one(db)
            .await?;

        template.ok_or_else(|| {
            anyhow!(
                "No active template found for product '{}' and screen_type '{:?}'",
                product,
                screen_type
            )
        })
    }

    /// Get template by ID
    pub async fn get_by_id(db: &DatabaseConnection, id: i32) -> Result<prompt_templates::Model> {
        prompt_templates::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| anyhow!("Template not found: {}", id))
    }

    /// Get all templates for a product
    pub async fn get_all_for_product(
        db: &DatabaseConnection,
        product: &str,
    ) -> Result<Vec<prompt_templates::Model>> {
        let templates = prompt_templates::Entity::find()
            .filter(prompt_templates::Column::Product.eq(product))
            .order_by_desc(prompt_templates::Column::Version)
            .all(db)
            .await?;

        Ok(templates)
    }

    /// Get the latest version number for a product/name combination
    pub async fn get_latest_version(
        db: &DatabaseConnection,
        product: &str,
        name: &str,
    ) -> Result<i32> {
        let template = prompt_templates::Entity::find()
            .filter(prompt_templates::Column::Product.eq(product))
            .filter(prompt_templates::Column::Name.eq(name))
            .order_by_desc(prompt_templates::Column::Version)
            .one(db)
            .await?;

        Ok(template.map(|t| t.version).unwrap_or(0))
    }
}

/// Default template content for xFrame5 UI generation
pub struct DefaultTemplates;

#[allow(dead_code)]
impl DefaultTemplates {
    /// System prompt for xFrame5 list screen generation
    pub fn xframe5_list_system_prompt() -> &'static str {
        r#"You are an expert xFrame5 frontend code generator. Your task is to generate XML view files and JavaScript event handlers for xFrame5 applications.

RULES:
1. Generate valid xFrame5 XML with proper Dataset and Grid definitions
2. Use proper column bindings between Dataset and Grid
3. Generate JavaScript with standard transaction functions (fn_search, fn_save, fn_delete, fn_add)
4. Follow xFrame5 naming conventions
5. Add TODO comments for any information you need but don't have
6. NEVER make up API endpoints - use TODO placeholders instead

OUTPUT FORMAT:
Respond with exactly two sections:

--- XML ---
<your XML content here>

--- JS ---
<your JavaScript content here>

Do not include any explanation outside these sections."#
    }

    /// User prompt template for list screen
    pub fn xframe5_list_user_template() -> &'static str {
        r#"Generate an xFrame5 list screen based on the following specification:

{{dsl_description}}

Requirements:
- Screen type: {{screen_type}}
- Screen name: {{screen_name}}
- Datasets: {{datasets}}
- Grid columns: {{grid_columns}}
- Actions: {{actions}}

{{#if notes}}
Additional notes:
{{notes}}
{{/if}}

{{#if company_rules}}
Company-specific rules:
{{company_rules}}
{{/if}}

Generate the XML and JavaScript code following xFrame5 patterns."#
    }

    /// System prompt for detail screen
    pub fn xframe5_detail_system_prompt() -> &'static str {
        r#"You are an expert xFrame5 frontend code generator. Your task is to generate XML view files and JavaScript event handlers for xFrame5 detail/form screens.

RULES:
1. Generate valid xFrame5 XML with proper Dataset and form control definitions
2. Use proper field bindings with Attribute Map properties
3. Generate JavaScript with standard functions (fn_init, fn_save, fn_delete, fn_validate)
4. Handle form validation properly
5. Add TODO comments for any information you need but don't have
6. NEVER make up API endpoints - use TODO placeholders instead

OUTPUT FORMAT:
Respond with exactly two sections:

--- XML ---
<your XML content here>

--- JS ---
<your JavaScript content here>

Do not include any explanation outside these sections."#
    }

    /// User prompt template for detail screen
    pub fn xframe5_detail_user_template() -> &'static str {
        r#"Generate an xFrame5 detail/form screen based on the following specification:

{{dsl_description}}

Requirements:
- Screen type: {{screen_type}}
- Screen name: {{screen_name}}
- Datasets: {{datasets}}
- Form fields: {{form_fields}}
- Actions: {{actions}}

{{#if notes}}
Additional notes:
{{notes}}
{{/if}}

{{#if company_rules}}
Company-specific rules:
{{company_rules}}
{{/if}}

Generate the XML and JavaScript code following xFrame5 patterns."#
    }
}
