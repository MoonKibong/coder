//! Admin Prompt Templates Controller
//!
//! HTMX-based CRUD for prompt templates

use loco_rs::prelude::*;
use sea_orm::{EntityTrait, PaginatorTrait, QueryOrder};
use serde::Deserialize;

use crate::models::_entities::prompt_templates::{ActiveModel, Column, Entity, Model};

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
        .view(&v, "admin/prompt_template/main.html", data!({
            "current_page": "prompt_templates",
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

    // Apply keyword filter if provided
    if let Some(keyword) = &params.keyword {
        if !keyword.is_empty() {
            use sea_orm::Condition;
            query = query.filter(
                Condition::any()
                    .add(Column::Name.contains(keyword))
                    .add(Column::Product.contains(keyword))
            );
        }
    }

    let paginator = query.paginate(&ctx.db, PAGE_SIZE);
    let total_pages = paginator.num_pages().await?;
    let items = paginator.fetch_page(page - 1).await?;

    format::render()
        .view(&v, "admin/prompt_template/list.html", data!({
            "items": items,
            "page": page,
            "total_pages": total_pages,
        }))
}

/// Show single item
#[debug_handler]
pub async fn show(
    ViewEngine(v): ViewEngine<TeraView>,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let item = load_item(&ctx, id).await?;

    format::render()
        .view(&v, "admin/prompt_template/show.html", data!({
            "item": item,
        }))
}

/// New form
#[debug_handler]
pub async fn new_form(
    ViewEngine(v): ViewEngine<TeraView>,
) -> Result<Response> {
    format::render()
        .view(&v, "admin/prompt_template/create.html", data!({}))
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
        .view(&v, "admin/prompt_template/edit.html", data!({
            "item": item,
        }))
}

#[derive(Debug, Deserialize)]
pub struct CreateParams {
    pub name: String,
    pub product: String,
    pub screen_type: Option<String>,
    pub system_prompt: String,
    pub user_prompt_template: String,
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
    item.product = Set(params.product);
    item.screen_type = Set(params.screen_type);
    item.system_prompt = Set(params.system_prompt);
    item.user_prompt_template = Set(params.user_prompt_template);
    item.version = Set(1);
    item.is_active = Set(params.is_active);

    let item = item.insert(&ctx.db).await?;

    // Return the new row for HTMX to prepend
    format::render()
        .view(&v, "admin/prompt_template/row.html", data!({
            "item": item,
        }))
}

#[derive(Debug, Deserialize)]
pub struct UpdateParams {
    pub name: Option<String>,
    pub product: Option<String>,
    pub screen_type: Option<String>,
    pub system_prompt: Option<String>,
    pub user_prompt_template: Option<String>,
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
    if let Some(product) = params.product {
        item.product = Set(product);
    }
    if params.screen_type.is_some() {
        item.screen_type = Set(params.screen_type);
    }
    if let Some(system_prompt) = params.system_prompt {
        item.system_prompt = Set(system_prompt);
    }
    if let Some(user_prompt_template) = params.user_prompt_template {
        item.user_prompt_template = Set(user_prompt_template);
    }
    if params.is_active.is_some() {
        item.is_active = Set(params.is_active);
    }

    // Increment version on update
    let current_version = item.version.clone().unwrap();
    item.version = Set(current_version + 1);

    let item = item.update(&ctx.db).await?;

    // Return updated row for HTMX to swap
    format::render()
        .view(&v, "admin/prompt_template/row.html", data!({
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

    // Return empty response - HTMX will remove the row
    format::html("")
}

async fn load_item(ctx: &AppContext, id: i32) -> Result<Model> {
    let item = Entity::find_by_id(id).one(&ctx.db).await?;
    item.ok_or_else(|| Error::NotFound)
}
