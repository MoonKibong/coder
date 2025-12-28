use crate::domain::{ScreenType, UiIntent};
use crate::models::_entities::{company_rules, prompt_templates};
use crate::services::template::DefaultTemplates;
use anyhow::Result;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

/// Compiled prompt ready to be sent to LLM
#[derive(Debug, Clone)]
pub struct CompiledPrompt {
    /// System prompt (role/rules)
    pub system: String,

    /// User prompt (specific request)
    pub user: String,
}

impl CompiledPrompt {
    /// Combine system and user prompts into a single prompt string
    pub fn full(&self) -> String {
        format!("{}\n\n{}", self.system, self.user)
    }
}

/// Service for compiling UiIntent into LLM prompts
pub struct PromptCompiler;

impl PromptCompiler {
    /// Compile a UiIntent into a prompt using templates from database
    pub async fn compile(
        db: &DatabaseConnection,
        intent: &UiIntent,
        product: &str,
        company_id: Option<&str>,
    ) -> Result<CompiledPrompt> {
        // 1. Load template from DB (or use defaults)
        let template = Self::load_template(db, product, intent.screen_type.as_str()).await;

        // 2. Load company rules if provided
        let rules = if let Some(cid) = company_id {
            Self::load_company_rules(db, cid).await.ok()
        } else {
            None
        };

        // 3. Build system prompt
        let system = Self::build_system_prompt(&template, &rules);

        // 4. Build user prompt from intent
        let user = Self::build_user_prompt(&template, intent, &rules);

        Ok(CompiledPrompt { system, user })
    }

    /// Compile using default templates (no database)
    pub fn compile_with_defaults(intent: &UiIntent, company_rules: Option<&str>) -> CompiledPrompt {
        let system = Self::get_default_system_prompt(intent.screen_type);
        let user = Self::build_user_prompt_from_intent(intent, company_rules);

        CompiledPrompt { system, user }
    }

    /// Load template from database or return None for defaults
    async fn load_template(
        db: &DatabaseConnection,
        product: &str,
        screen_type: &str,
    ) -> Option<prompt_templates::Model> {
        prompt_templates::Entity::find()
            .filter(prompt_templates::Column::Product.eq(product))
            .filter(prompt_templates::Column::ScreenType.eq(Some(screen_type.to_string())))
            .filter(prompt_templates::Column::IsActive.eq(Some(true)))
            .one(db)
            .await
            .ok()
            .flatten()
    }

    /// Load company rules from database
    async fn load_company_rules(
        db: &DatabaseConnection,
        company_id: &str,
    ) -> Result<company_rules::Model> {
        company_rules::Entity::find()
            .filter(company_rules::Column::CompanyId.eq(company_id))
            .one(db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Company rules not found for: {}", company_id))
    }

    /// Build system prompt from template and rules
    fn build_system_prompt(
        template: &Option<prompt_templates::Model>,
        rules: &Option<company_rules::Model>,
    ) -> String {
        let base_prompt = template
            .as_ref()
            .map(|t| t.system_prompt.clone())
            .unwrap_or_else(|| DefaultTemplates::xframe5_list_system_prompt().to_string());

        // Append company rules if available
        if let Some(r) = rules {
            if let Some(ref additional) = r.additional_rules {
                if !additional.is_empty() {
                    return format!("{}\n\nCOMPANY-SPECIFIC RULES:\n{}", base_prompt, additional);
                }
            }
        }

        base_prompt
    }

    /// Build user prompt from template and intent
    fn build_user_prompt(
        template: &Option<prompt_templates::Model>,
        intent: &UiIntent,
        rules: &Option<company_rules::Model>,
    ) -> String {
        let company_rules_str = rules
            .as_ref()
            .and_then(|r| r.additional_rules.clone())
            .unwrap_or_default();

        if let Some(t) = template {
            Self::render_template(&t.user_prompt_template, intent, &company_rules_str)
        } else {
            let rules_ref = if company_rules_str.is_empty() {
                None
            } else {
                Some(company_rules_str.as_str())
            };
            Self::build_user_prompt_from_intent(intent, rules_ref)
        }
    }

    /// Get default system prompt for screen type
    fn get_default_system_prompt(screen_type: ScreenType) -> String {
        match screen_type {
            ScreenType::List | ScreenType::ListWithPopup => {
                DefaultTemplates::xframe5_list_system_prompt().to_string()
            }
            ScreenType::Detail | ScreenType::Popup => {
                DefaultTemplates::xframe5_detail_system_prompt().to_string()
            }
        }
    }

    /// Render a template with intent data
    fn render_template(template: &str, intent: &UiIntent, company_rules: &str) -> String {
        let dsl_description = Self::describe_intent(intent);
        let datasets = Self::describe_datasets(&intent.datasets);
        let grid_columns = Self::describe_grids(&intent.grids);
        let actions = Self::describe_actions(&intent.actions);

        template
            .replace("{{dsl_description}}", &dsl_description)
            .replace("{{screen_type}}", intent.screen_type.as_str())
            .replace("{{screen_name}}", &intent.screen_name)
            .replace("{{datasets}}", &datasets)
            .replace("{{grid_columns}}", &grid_columns)
            .replace("{{form_fields}}", &grid_columns) // Same format for now
            .replace("{{actions}}", &actions)
            .replace("{{notes}}", intent.notes.as_deref().unwrap_or(""))
            .replace("{{company_rules}}", company_rules)
            // Handle conditional blocks (simple version)
            .lines()
            .filter(|line| !line.contains("{{#if") && !line.contains("{{/if}}"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Build user prompt directly from intent
    fn build_user_prompt_from_intent(intent: &UiIntent, company_rules: Option<&str>) -> String {
        let mut prompt = format!(
            "Generate an xFrame5 {} screen based on the following specification:\n\n",
            intent.screen_type.as_str()
        );

        prompt.push_str(&Self::describe_intent(intent));

        prompt.push_str("\n\nRequirements:\n");
        prompt.push_str(&format!("- Screen type: {}\n", intent.screen_type.as_str()));
        prompt.push_str(&format!("- Screen name: {}\n", intent.screen_name));

        if !intent.datasets.is_empty() {
            prompt.push_str(&format!("- Datasets: {}\n", Self::describe_datasets(&intent.datasets)));
        }

        if !intent.grids.is_empty() {
            prompt.push_str(&format!("- Grid columns: {}\n", Self::describe_grids(&intent.grids)));
        }

        if !intent.actions.is_empty() {
            prompt.push_str(&format!("- Actions: {}\n", Self::describe_actions(&intent.actions)));
        }

        if let Some(notes) = &intent.notes {
            prompt.push_str(&format!("\nAdditional notes:\n{}\n", notes));
        }

        if let Some(rules) = company_rules {
            if !rules.is_empty() {
                prompt.push_str(&format!("\nCompany-specific rules:\n{}\n", rules));
            }
        }

        prompt.push_str("\nGenerate the XML and JavaScript code following xFrame5 patterns.");

        prompt
    }

    /// Generate a human-readable description of the intent
    fn describe_intent(intent: &UiIntent) -> String {
        let mut desc = format!(
            "Create a {} screen named '{}'.\n",
            intent.screen_type.as_str(),
            intent.screen_name
        );

        if !intent.datasets.is_empty() {
            desc.push_str("\nDatasets:\n");
            for ds in &intent.datasets {
                desc.push_str(&format!("- {} (table: {})\n", ds.id, ds.table_name.as_deref().unwrap_or("unknown")));
                if !ds.columns.is_empty() {
                    desc.push_str("  Columns:\n");
                    for col in &ds.columns {
                        desc.push_str(&format!(
                            "    - {} ({}, {}, {}{})\n",
                            col.name,
                            col.label,
                            col.ui_type.as_str(),
                            col.data_type.as_str(),
                            if col.required { ", required" } else { "" }
                        ));
                    }
                }
            }
        }

        if !intent.grids.is_empty() {
            desc.push_str("\nGrids:\n");
            for grid in &intent.grids {
                desc.push_str(&format!("- {} (bound to {})\n", grid.id, grid.dataset_id));
                if !grid.columns.is_empty() {
                    desc.push_str("  Columns: ");
                    let col_names: Vec<_> = grid.columns.iter().map(|c| c.header.as_str()).collect();
                    desc.push_str(&col_names.join(", "));
                    desc.push('\n');
                }
            }
        }

        if !intent.actions.is_empty() {
            desc.push_str("\nActions:\n");
            for action in &intent.actions {
                desc.push_str(&format!(
                    "- {} ({}): {}\n",
                    action.id,
                    action.label,
                    action.function_name
                ));
            }
        }

        desc
    }

    /// Describe datasets for template
    fn describe_datasets(datasets: &[crate::domain::DatasetIntent]) -> String {
        datasets
            .iter()
            .map(|ds| {
                let cols: Vec<_> = ds.columns.iter().map(|c| c.name.as_str()).collect();
                format!("{} [{}]", ds.id, cols.join(", "))
            })
            .collect::<Vec<_>>()
            .join("; ")
    }

    /// Describe grids for template
    fn describe_grids(grids: &[crate::domain::GridIntent]) -> String {
        grids
            .iter()
            .map(|g| {
                let cols: Vec<_> = g.columns.iter().map(|c| c.header.as_str()).collect();
                format!("{}: {}", g.id, cols.join(", "))
            })
            .collect::<Vec<_>>()
            .join("; ")
    }

    /// Describe actions for template
    fn describe_actions(actions: &[crate::domain::ActionIntent]) -> String {
        actions
            .iter()
            .map(|a| format!("{} ({})", a.label, a.function_name))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{ColumnIntent, DatasetIntent, GridColumnIntent, GridIntent, UiType};

    fn create_test_intent() -> UiIntent {
        let columns = vec![
            ColumnIntent::new("id", "ID").primary_key(),
            ColumnIntent::new("name", "이름").with_ui_type(UiType::Input).required(),
            ColumnIntent::new("email", "이메일").with_ui_type(UiType::Input),
        ];

        let grid_columns = vec![
            GridColumnIntent::new("name", "이름"),
            GridColumnIntent::new("email", "이메일"),
        ];

        let dataset = DatasetIntent::new("ds_member")
            .with_table("member")
            .with_columns(columns);

        let grid = GridIntent::new("grid_member", "ds_member")
            .with_columns(grid_columns);

        UiIntent::new("member_list", ScreenType::List)
            .with_dataset(dataset)
            .with_grid(grid)
    }

    #[test]
    fn test_compile_with_defaults() {
        let intent = create_test_intent();
        let prompt = PromptCompiler::compile_with_defaults(&intent, None);

        assert!(!prompt.system.is_empty());
        assert!(prompt.user.contains("member_list"));
        assert!(prompt.user.contains("ds_member"));
        assert!(prompt.user.contains("이름"));
    }

    #[test]
    fn test_describe_intent() {
        let intent = create_test_intent();
        let desc = PromptCompiler::describe_intent(&intent);

        assert!(desc.contains("member_list"));
        assert!(desc.contains("ds_member"));
        assert!(desc.contains("member"));
    }

    #[test]
    fn test_full_prompt() {
        let intent = create_test_intent();
        let prompt = PromptCompiler::compile_with_defaults(&intent, None);
        let full = prompt.full();

        assert!(full.contains("xFrame5"));
        assert!(full.contains("member_list"));
    }
}
