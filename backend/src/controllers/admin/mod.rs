//! Admin Panel Controllers
//!
//! HTMX-based admin panel for managing:
//! - Prompt Templates
//! - Company Rules
//! - LLM Configurations
//! - Generation Logs (view only)

pub mod dashboard;
pub mod prompt_templates;
pub mod company_rules;
pub mod generation_logs;
pub mod llm_configs;

use loco_rs::prelude::*;

/// Empty response for closing modals
#[debug_handler]
pub async fn empty() -> Result<Response> {
    format::html("")
}

/// Combine all admin routes
pub fn routes() -> Routes {
    Routes::new()
        .prefix("admin/")
        .add("/", get(dashboard::index))
        .add("empty", post(empty))
        // Dashboard
        .add("dashboard", get(dashboard::main))
        // Prompt Templates
        .add("prompt-templates", get(prompt_templates::main))
        .add("prompt-templates/list", get(prompt_templates::list))
        .add("prompt-templates/new", get(prompt_templates::new_form))
        .add("prompt-templates", post(prompt_templates::create))
        .add("prompt-templates/{id}", get(prompt_templates::show))
        .add("prompt-templates/{id}/edit", get(prompt_templates::edit_form))
        .add("prompt-templates/{id}", patch(prompt_templates::update))
        .add("prompt-templates/{id}", delete(prompt_templates::delete))
        // Company Rules
        .add("company-rules", get(company_rules::main))
        .add("company-rules/list", get(company_rules::list))
        .add("company-rules/new", get(company_rules::new_form))
        .add("company-rules", post(company_rules::create))
        .add("company-rules/{id}", get(company_rules::show))
        .add("company-rules/{id}/edit", get(company_rules::edit_form))
        .add("company-rules/{id}", patch(company_rules::update))
        .add("company-rules/{id}", delete(company_rules::delete))
        // LLM Configs
        .add("llm-configs", get(llm_configs::main))
        .add("llm-configs/list", get(llm_configs::list))
        .add("llm-configs/new", get(llm_configs::new_form))
        .add("llm-configs", post(llm_configs::create))
        .add("llm-configs/{id}/edit", get(llm_configs::edit_form))
        .add("llm-configs/{id}", patch(llm_configs::update))
        .add("llm-configs/{id}", delete(llm_configs::delete))
        // Generation Logs (read only)
        .add("generation-logs", get(generation_logs::main))
        .add("generation-logs/list", get(generation_logs::list))
        .add("generation-logs/{id}", get(generation_logs::show))
}
