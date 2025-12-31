use async_trait::async_trait;
use loco_rs::{
    app::{AppContext, Hooks, Initializer},
    bgworker::{BackgroundWorker, Queue},
    boot::{create_app, BootResult, StartMode},
    config::Config,
    controller::AppRoutes,
    db::{self, truncate_table},
    environment::Environment,
    task::Tasks,
    Result,
};
use migration::Migrator;
use std::path::Path;

#[allow(unused_imports)]
use crate::{
    controllers, initializers,
    models::_entities::{users, knowledge_bases, llm_configs, prompt_templates},
    services, tasks,
    workers::downloader::DownloadWorker,
};

pub struct App;
#[async_trait]
impl Hooks for App {
    fn app_name() -> &'static str {
        env!("CARGO_CRATE_NAME")
    }

    fn app_version() -> String {
        format!(
            "{} ({})",
            env!("CARGO_PKG_VERSION"),
            option_env!("BUILD_SHA")
                .or(option_env!("GITHUB_SHA"))
                .unwrap_or("dev")
        )
    }

    async fn boot(
        mode: StartMode,
        environment: &Environment,
        config: Config,
    ) -> Result<BootResult> {
        create_app::<Self, Migrator>(mode, environment, config).await
    }

    async fn initializers(_ctx: &AppContext) -> Result<Vec<Box<dyn Initializer>>> {
        Ok(vec![Box::new(
            initializers::view_engine::ViewEngineInitializer,
        )])
    }

    async fn after_context(ctx: AppContext) -> Result<AppContext> {
        // Start the metrics collector for dashboard graphs
        services::metrics_history::start_metrics_collector();
        Ok(ctx)
    }

    fn routes(_ctx: &AppContext) -> AppRoutes {
        AppRoutes::with_default_routes() // controller routes below
            .add_route(controllers::knowledge_base::routes())
            // Landing page
            .add_route(controllers::home::routes())
            // API routes
            .add_route(controllers::generate::routes())
            .add_route(controllers::review::routes())
            .add_route(controllers::qa::routes())
            .add_route(controllers::jobs::routes())
            .add_route(controllers::llm_config::routes())
            .add_route(controllers::generation_log::routes())
            .add_route(controllers::company_rule::routes())
            .add_route(controllers::prompt_template::routes())
            .add_route(controllers::auth::routes())
            // Admin panel (HTMX views)
            .add_route(controllers::admin::routes())
    }
    async fn connect_workers(ctx: &AppContext, queue: &Queue) -> Result<()> {
        queue.register(DownloadWorker::build(ctx)).await?;
        Ok(())
    }

    #[allow(unused_variables)]
    fn register_tasks(tasks: &mut Tasks) {
        tasks.register(tasks::QueueProcessorTask);
        // tasks-inject (do not remove)
    }
    async fn truncate(ctx: &AppContext) -> Result<()> {
        truncate_table(&ctx.db, users::Entity).await?;
        truncate_table(&ctx.db, knowledge_bases::Entity).await?;
        truncate_table(&ctx.db, llm_configs::Entity).await?;
        truncate_table(&ctx.db, prompt_templates::Entity).await?;
        Ok(())
    }

    async fn seed(ctx: &AppContext, base: &Path) -> Result<()> {
        db::seed::<users::ActiveModel>(&ctx.db, &base.join("users.yaml").display().to_string())
            .await?;
        db::seed::<knowledge_bases::ActiveModel>(&ctx.db, &base.join("knowledge_bases.yaml").display().to_string())
            .await?;
        db::seed::<llm_configs::ActiveModel>(&ctx.db, &base.join("llm_configs.yaml").display().to_string())
            .await?;
        db::seed::<prompt_templates::ActiveModel>(&ctx.db, &base.join("prompt_templates.yaml").display().to_string())
            .await?;
        Ok(())
    }
}