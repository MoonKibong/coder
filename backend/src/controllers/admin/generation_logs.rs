//! Admin Generation Logs Controller
//!
//! HTMX-based view-only for generation logs (audit trail)

use loco_rs::prelude::*;
use sea_orm::{EntityTrait, PaginatorTrait, QueryOrder};
use serde::Deserialize;

use crate::models::_entities::generation_logs::{Column, Entity, Model};

const PAGE_SIZE: u64 = 50;

/// Main page - renders full layout with list
#[debug_handler]
pub async fn main(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let items = Entity::find()
        .order_by_desc(Column::CreatedAt)
        .paginate(&ctx.db, PAGE_SIZE)
        .fetch_page(0)
        .await?;

    let total = Entity::find().count(&ctx.db).await?;

    format::render()
        .view(&v, "admin/generation_log/main.html", data!({
            "current_page": "generation_logs",
            "items": items,
            "page": 1,
            "total_pages": (total + PAGE_SIZE - 1) / PAGE_SIZE,
        }))
}

#[derive(Debug, Deserialize)]
pub struct ListParams {
    pub status: Option<String>,
    pub product: Option<String>,
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

    let mut query = Entity::find().order_by_desc(Column::CreatedAt);

    // Apply filters
    if let Some(status) = &params.status {
        if !status.is_empty() {
            query = query.filter(Column::Status.eq(status.as_str()));
        }
    }
    if let Some(product) = &params.product {
        if !product.is_empty() {
            query = query.filter(Column::Product.eq(product.as_str()));
        }
    }

    let paginator = query.paginate(&ctx.db, PAGE_SIZE);
    let total_pages = paginator.num_pages().await?;
    let items = paginator.fetch_page(page - 1).await?;

    format::render()
        .view(&v, "admin/generation_log/list.html", data!({
            "items": items,
            "page": page,
            "total_pages": total_pages,
        }))
}

/// Show single log entry
#[debug_handler]
pub async fn show(
    ViewEngine(v): ViewEngine<TeraView>,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let item = load_item(&ctx, id).await?;

    format::render()
        .view(&v, "admin/generation_log/show.html", data!({
            "item": item,
        }))
}

async fn load_item(ctx: &AppContext, id: i32) -> Result<Model> {
    let item = Entity::find_by_id(id).one(&ctx.db).await?;
    item.ok_or_else(|| Error::NotFound)
}
