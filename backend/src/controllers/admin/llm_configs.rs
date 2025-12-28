//! Admin LLM Configs Controller
//!
//! HTMX-based CRUD for LLM configurations

use loco_rs::prelude::*;
use sea_orm::{EntityTrait, PaginatorTrait, QueryOrder};
use serde::Deserialize;

use crate::models::_entities::llm_configs::{ActiveModel, Column, Entity, Model};

const PAGE_SIZE: u64 = 20;

/// Main page - renders full layout with list
#[debug_handler]
pub async fn main(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let items = Entity::find()
        .order_by_desc(Column::UpdatedAt)
        .all(&ctx.db)
        .await?;

    format::render()
        .view(&v, "admin/llm_config/main.html", data!({
            "current_page": "llm_configs",
            "items": items,
        }))
}

#[derive(Debug, Deserialize)]
pub struct ListParams {
    pub keyword: Option<String>,
    pub page: Option<u64>,
}

/// List view - for HTMX partial updates
#[debug_handler]
pub async fn list(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
    Query(params): Query<ListParams>,
) -> Result<Response> {
    let page = params.page.unwrap_or(1).max(1);

    let mut query = Entity::find().order_by_desc(Column::UpdatedAt);

    if let Some(keyword) = &params.keyword {
        if !keyword.is_empty() {
            use sea_orm::Condition;
            query = query.filter(
                Condition::any()
                    .add(Column::Name.contains(keyword))
                    .add(Column::Provider.contains(keyword))
            );
        }
    }

    let paginator = query.paginate(&ctx.db, PAGE_SIZE);
    let total_pages = paginator.num_pages().await?;
    let items = paginator.fetch_page(page - 1).await?;

    format::render()
        .view(&v, "admin/llm_config/list.html", data!({
            "items": items,
            "page": page,
            "total_pages": total_pages,
        }))
}

/// New form
#[debug_handler]
pub async fn new_form(
    ViewEngine(v): ViewEngine<TeraView>,
) -> Result<Response> {
    format::render()
        .view(&v, "admin/llm_config/create.html", data!({}))
}

/// Edit form
#[debug_handler]
pub async fn edit_form(
    ViewEngine(v): ViewEngine<TeraView>,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let item = load_item(&ctx, id).await?;

    format::render()
        .view(&v, "admin/llm_config/edit.html", data!({
            "item": item,
        }))
}

#[derive(Debug, Deserialize)]
pub struct CreateParams {
    pub name: String,
    pub provider: String,
    pub model_name: String,
    pub endpoint_url: String,
    pub api_key: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<i32>,
    pub is_active: Option<bool>,
}

/// Create new item
#[debug_handler]
pub async fn create(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
    Json(params): Json<CreateParams>,
) -> Result<Response> {
    let mut item = ActiveModel {
        ..Default::default()
    };

    item.name = Set(params.name);
    item.provider = Set(params.provider);
    item.model_name = Set(params.model_name);
    item.endpoint_url = Set(params.endpoint_url);
    item.api_key = Set(params.api_key);
    item.temperature = Set(params.temperature);
    item.max_tokens = Set(params.max_tokens);
    item.is_active = Set(params.is_active);

    let item = item.insert(&ctx.db).await?;

    format::render()
        .view(&v, "admin/llm_config/row.html", data!({
            "item": item,
        }))
}

#[derive(Debug, Deserialize)]
pub struct UpdateParams {
    pub name: Option<String>,
    pub provider: Option<String>,
    pub model_name: Option<String>,
    pub endpoint_url: Option<String>,
    pub api_key: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<i32>,
    pub is_active: Option<bool>,
}

/// Update existing item
#[debug_handler]
pub async fn update(
    ViewEngine(v): ViewEngine<TeraView>,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
    Json(params): Json<UpdateParams>,
) -> Result<Response> {
    let item = load_item(&ctx, id).await?;
    let mut item = item.into_active_model();

    if let Some(name) = params.name {
        item.name = Set(name);
    }
    if let Some(provider) = params.provider {
        item.provider = Set(provider);
    }
    if let Some(model_name) = params.model_name {
        item.model_name = Set(model_name);
    }
    if let Some(endpoint_url) = params.endpoint_url {
        item.endpoint_url = Set(endpoint_url);
    }
    if params.api_key.is_some() {
        item.api_key = Set(params.api_key);
    }
    if params.temperature.is_some() {
        item.temperature = Set(params.temperature);
    }
    if params.max_tokens.is_some() {
        item.max_tokens = Set(params.max_tokens);
    }
    if params.is_active.is_some() {
        item.is_active = Set(params.is_active);
    }

    let item = item.update(&ctx.db).await?;

    format::render()
        .view(&v, "admin/llm_config/row.html", data!({
            "item": item,
        }))
}

/// Delete item
#[debug_handler]
pub async fn delete(
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    load_item(&ctx, id).await?.delete(&ctx.db).await?;
    format::html("")
}

async fn load_item(ctx: &AppContext, id: i32) -> Result<Model> {
    let item = Entity::find_by_id(id).one(&ctx.db).await?;
    item.ok_or_else(|| Error::NotFound)
}
