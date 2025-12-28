//! Admin Dashboard Controller

use loco_rs::prelude::*;
use crate::models::_entities::{
    prompt_templates, company_rules, generation_logs, llm_configs
};

/// Dashboard index - renders full page with layout
#[debug_handler]
pub async fn index(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let stats = get_stats(&ctx).await?;

    format::render()
        .view(&v, "admin/dashboard/index.html", data!({
            "current_page": "dashboard",
            "stats": stats,
        }))
}

/// Dashboard main content - for HTMX partial updates
#[debug_handler]
pub async fn main(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let stats = get_stats(&ctx).await?;

    format::render()
        .view(&v, "admin/dashboard/main.html", data!({
            "current_page": "dashboard",
            "stats": stats,
        }))
}

#[derive(Debug, serde::Serialize)]
struct DashboardStats {
    prompt_templates_count: u64,
    company_rules_count: u64,
    generation_logs_count: u64,
    llm_configs_count: u64,
    recent_generations: u64,
}

async fn get_stats(ctx: &AppContext) -> Result<DashboardStats> {
    use sea_orm::{EntityTrait, PaginatorTrait};

    let prompt_templates_count = prompt_templates::Entity::find()
        .count(&ctx.db)
        .await
        .unwrap_or(0);

    let company_rules_count = company_rules::Entity::find()
        .count(&ctx.db)
        .await
        .unwrap_or(0);

    let generation_logs_count = generation_logs::Entity::find()
        .count(&ctx.db)
        .await
        .unwrap_or(0);

    let llm_configs_count = llm_configs::Entity::find()
        .count(&ctx.db)
        .await
        .unwrap_or(0);

    // Count recent generations (last 24 hours) - simplified for now
    let recent_generations = generation_logs_count.min(100);

    Ok(DashboardStats {
        prompt_templates_count,
        company_rules_count,
        generation_logs_count,
        llm_configs_count,
        recent_generations,
    })
}
