//! Admin Dashboard Controller
//!
//! Provides system monitoring and analytics for the admin dashboard.

use loco_rs::prelude::*;
use sea_orm::{EntityTrait, PaginatorTrait};

use crate::middleware::cookie_auth::AuthUser;
use crate::models::_entities::{company_rules, llm_configs, prompt_templates, users};
use crate::services::analytics::AnalyticsService;
use crate::services::metrics_history::get_metrics_store;
use crate::services::system_monitor::{format_bytes, format_uptime, SystemMonitor};

/// Dashboard index - renders full page with layout
#[debug_handler]
pub async fn index(
    auth_user: AuthUser,
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let data = get_dashboard_data(&ctx).await?;

    format::render().view(
        &v,
        "admin/dashboard/index.html",
        data!({
            "current_page": "dashboard",
            "user": auth_user,
            "config_stats": data.config_stats,
            "system_metrics": data.system_metrics,
            "analytics": data.analytics,
        }),
    )
}

/// Dashboard main content - for HTMX partial updates
#[debug_handler]
pub async fn main(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let data = get_dashboard_data(&ctx).await?;

    format::render().view(
        &v,
        "admin/dashboard/main.html",
        data!({
            "current_page": "dashboard",
            "config_stats": data.config_stats,
            "system_metrics": data.system_metrics,
            "analytics": data.analytics,
        }),
    )
}

/// System metrics endpoint for real-time updates
#[debug_handler]
pub async fn system_metrics(ViewEngine(v): ViewEngine<TeraView>) -> Result<Response> {
    let metrics = tokio::task::spawn_blocking(SystemMonitor::get_metrics)
        .await
        .map_err(|e| Error::string(&format!("Failed to get system metrics: {}", e)))?;
    let formatted = format_system_metrics(&metrics);

    format::render().view(
        &v,
        "admin/dashboard/partials/system_metrics.html",
        data!({
            "system_metrics": formatted,
        }),
    )
}

/// Analytics endpoint for real-time updates
#[debug_handler]
pub async fn analytics(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let analytics = AnalyticsService::get_dashboard_analytics(&ctx.db).await?;

    format::render().view(
        &v,
        "admin/dashboard/partials/analytics.html",
        data!({
            "analytics": analytics,
        }),
    )
}

/// Historical metrics endpoint for graphs (JSON)
#[debug_handler]
pub async fn metrics_history() -> Result<Response> {
    let history = get_metrics_store().get_history();
    format::json(history)
}

#[derive(Debug, serde::Serialize)]
struct DashboardData {
    config_stats: ConfigStats,
    system_metrics: FormattedSystemMetrics,
    analytics: crate::services::analytics::DashboardAnalytics,
}

#[derive(Debug, serde::Serialize)]
struct ConfigStats {
    prompt_templates_count: u64,
    company_rules_count: u64,
    llm_configs_count: u64,
    users_count: u64,
}

#[derive(Debug, serde::Serialize)]
struct FormattedSystemMetrics {
    cpu_usage: f32,
    cpu_cores: usize,
    cpu_brand: String,
    memory_usage: f32,
    memory_total: String,
    memory_used: String,
    memory_available: String,
    disks: Vec<FormattedDisk>,
    network_received: String,
    network_transmitted: String,
    os_name: String,
    os_version: String,
    hostname: String,
    uptime: String,
}

#[derive(Debug, serde::Serialize)]
struct FormattedDisk {
    name: String,
    usage_percent: f32,
    total: String,
    used: String,
    available: String,
    fs_type: String,
}

async fn get_dashboard_data(ctx: &AppContext) -> Result<DashboardData> {
    // Get configuration counts
    let prompt_templates_count = prompt_templates::Entity::find()
        .count(&ctx.db)
        .await
        .unwrap_or(0);

    let company_rules_count = company_rules::Entity::find()
        .count(&ctx.db)
        .await
        .unwrap_or(0);

    let llm_configs_count = llm_configs::Entity::find()
        .count(&ctx.db)
        .await
        .unwrap_or(0);

    let users_count = users::Entity::find().count(&ctx.db).await.unwrap_or(0);

    let config_stats = ConfigStats {
        prompt_templates_count,
        company_rules_count,
        llm_configs_count,
        users_count,
    };

    // Get system metrics in a blocking task to avoid blocking the async runtime
    let metrics = tokio::task::spawn_blocking(SystemMonitor::get_metrics)
        .await
        .map_err(|e| Error::string(&format!("Failed to get system metrics: {}", e)))?;
    let system_metrics = format_system_metrics(&metrics);

    // Get analytics
    let analytics = AnalyticsService::get_dashboard_analytics(&ctx.db).await?;

    Ok(DashboardData {
        config_stats,
        system_metrics,
        analytics,
    })
}

fn format_system_metrics(
    metrics: &crate::services::system_monitor::SystemMetrics,
) -> FormattedSystemMetrics {
    let disks: Vec<FormattedDisk> = metrics
        .disks
        .iter()
        .map(|d| FormattedDisk {
            name: d.name.clone(),
            usage_percent: d.usage_percent,
            total: format_bytes(d.total_bytes),
            used: format_bytes(d.used_bytes),
            available: format_bytes(d.available_bytes),
            fs_type: d.fs_type.clone(),
        })
        .collect();

    FormattedSystemMetrics {
        cpu_usage: metrics.cpu.usage_percent,
        cpu_cores: metrics.cpu.logical_cores,
        cpu_brand: metrics.cpu.brand.clone(),
        memory_usage: metrics.memory.usage_percent,
        memory_total: format_bytes(metrics.memory.total_bytes),
        memory_used: format_bytes(metrics.memory.used_bytes),
        memory_available: format_bytes(metrics.memory.available_bytes),
        disks,
        network_received: format_bytes(metrics.network.received_bytes),
        network_transmitted: format_bytes(metrics.network.transmitted_bytes),
        os_name: metrics.system_info.os_name.clone(),
        os_version: metrics.system_info.os_version.clone(),
        hostname: metrics.system_info.hostname.clone(),
        uptime: format_uptime(metrics.system_info.uptime_seconds),
    }
}
