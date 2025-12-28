//! Admin Company Rules Controller
//!
//! HTMX-based CRUD for company rules

use loco_rs::prelude::*;
use sea_orm::{EntityTrait, PaginatorTrait, QueryOrder};
use serde::Deserialize;

use crate::models::_entities::company_rules::{ActiveModel, Column, Entity, Model};

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
        .view(&v, "admin/company_rule/main.html", data!({
            "current_page": "company_rules",
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
            query = query.filter(Column::CompanyId.contains(keyword));
        }
    }

    let paginator = query.paginate(&ctx.db, PAGE_SIZE);
    let total_pages = paginator.num_pages().await?;
    let items = paginator.fetch_page(page - 1).await?;

    format::render()
        .view(&v, "admin/company_rule/list.html", data!({
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
        .view(&v, "admin/company_rule/show.html", data!({
            "item": item,
        }))
}

/// New form
#[debug_handler]
pub async fn new_form(
    ViewEngine(v): ViewEngine<TeraView>,
) -> Result<Response> {
    format::render()
        .view(&v, "admin/company_rule/create.html", data!({}))
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
        .view(&v, "admin/company_rule/edit.html", data!({
            "item": item,
        }))
}

#[derive(Debug, Deserialize)]
pub struct CreateParams {
    pub company_id: String,
    pub naming_convention: Option<String>,
    pub additional_rules: Option<String>,
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

    item.company_id = Set(params.company_id);
    item.naming_convention = Set(params.naming_convention);
    item.additional_rules = Set(params.additional_rules);

    let item = item.insert(&ctx.db).await?;

    format::render()
        .view(&v, "admin/company_rule/row.html", data!({
            "item": item,
        }))
}

#[derive(Debug, Deserialize)]
pub struct UpdateParams {
    pub company_id: Option<String>,
    pub naming_convention: Option<String>,
    pub additional_rules: Option<String>,
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

    if let Some(company_id) = params.company_id {
        item.company_id = Set(company_id);
    }
    if params.naming_convention.is_some() {
        item.naming_convention = Set(params.naming_convention);
    }
    if params.additional_rules.is_some() {
        item.additional_rules = Set(params.additional_rules);
    }

    let item = item.update(&ctx.db).await?;

    format::render()
        .view(&v, "admin/company_rule/row.html", data!({
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
