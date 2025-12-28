use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        // Insert xFrame5 list template
        let insert_list = Query::insert()
            .into_table(Alias::new("prompt_templates"))
            .columns([
                Alias::new("name"),
                Alias::new("product"),
                Alias::new("screen_type"),
                Alias::new("system_prompt"),
                Alias::new("user_prompt_template"),
                Alias::new("version"),
                Alias::new("is_active"),
            ])
            .values_panic([
                "xframe5-list".into(),
                "xframe5-ui".into(),
                "list".into(),
                XFRAME5_LIST_SYSTEM_PROMPT.into(),
                XFRAME5_LIST_USER_TEMPLATE.into(),
                1.into(),
                true.into(),
            ])
            .to_owned();

        m.exec_stmt(insert_list).await?;

        // Insert xFrame5 detail template
        let insert_detail = Query::insert()
            .into_table(Alias::new("prompt_templates"))
            .columns([
                Alias::new("name"),
                Alias::new("product"),
                Alias::new("screen_type"),
                Alias::new("system_prompt"),
                Alias::new("user_prompt_template"),
                Alias::new("version"),
                Alias::new("is_active"),
            ])
            .values_panic([
                "xframe5-detail".into(),
                "xframe5-ui".into(),
                "detail".into(),
                XFRAME5_DETAIL_SYSTEM_PROMPT.into(),
                XFRAME5_DETAIL_USER_TEMPLATE.into(),
                1.into(),
                true.into(),
            ])
            .to_owned();

        m.exec_stmt(insert_detail).await?;

        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        // Delete xFrame5 templates
        let delete = Query::delete()
            .from_table(Alias::new("prompt_templates"))
            .and_where(Expr::col(Alias::new("product")).eq("xframe5-ui"))
            .to_owned();

        m.exec_stmt(delete).await?;

        Ok(())
    }
}

const XFRAME5_LIST_SYSTEM_PROMPT: &str = r#"You are an expert xFrame5 frontend code generator. Your task is to generate XML view files and JavaScript event handlers for xFrame5 applications.

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

Do not include any explanation outside these sections.

{{company_rules}}"#;

const XFRAME5_LIST_USER_TEMPLATE: &str = r#"Generate an xFrame5 list screen based on the following specification:

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

Generate the XML and JavaScript code following xFrame5 patterns."#;

const XFRAME5_DETAIL_SYSTEM_PROMPT: &str = r#"You are an expert xFrame5 frontend code generator. Your task is to generate XML view files and JavaScript event handlers for xFrame5 detail/form screens.

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

Do not include any explanation outside these sections.

{{company_rules}}"#;

const XFRAME5_DETAIL_USER_TEMPLATE: &str = r#"Generate an xFrame5 detail/form screen based on the following specification:

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

Generate the XML and JavaScript code following xFrame5 patterns."#;

