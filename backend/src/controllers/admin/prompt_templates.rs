//! Admin Prompt Templates Controller
//!
//! HTMX-based CRUD for prompt templates.
//! Thin controller - delegates to PromptTemplateService.

use axum::extract::Multipart;
use axum::http::{header, HeaderMap, StatusCode};
use loco_rs::prelude::*;

/// Helper to check if request is from HTMX
fn is_htmx_request(headers: &HeaderMap) -> bool {
    headers.get("HX-Request").is_some()
}

/// Redirect response for non-HTMX requests to modal endpoints
fn redirect_to_main_page() -> Result<Response> {
    Ok(Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, "/admin/prompt-templates")
        .body(axum::body::Body::empty())?
        .into_response())
}

use crate::middleware::cookie_auth::AuthUser;
use crate::services::admin::prompt_template::{
    CreateParams, PromptTemplateService, QueryParams, UpdateParams,
};
use crate::services::{ImportOptions, TemplateImporter};

/// Main page - renders full layout for direct access, partial for HTMX
#[debug_handler]
pub async fn main(
    auth_user: AuthUser,
    headers: HeaderMap,
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let params = QueryParams::default();
    let response = PromptTemplateService::search(&ctx.db, &params).await?;

    // Check if this is an HTMX request
    let is_htmx = headers.get("HX-Request").is_some();
    let template = if is_htmx {
        "admin/prompt_template/main.html"
    } else {
        "admin/prompt_template/index.html"
    };

    format::render().view(
        &v,
        template,
        data!({
            "current_page": "prompt_templates",
            "user": auth_user,
            "items": response.items,
            "page": response.page,
            "page_size": response.page_size,
            "total_pages": response.total_pages,
            "total_items": response.total_items,
        }),
    )
}

/// List view - for HTMX partial updates
#[debug_handler]
pub async fn list(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
    Query(params): Query<QueryParams>,
) -> Result<Response> {
    let response = PromptTemplateService::search(&ctx.db, &params).await?;

    format::render().view(
        &v,
        "admin/prompt_template/list.html",
        data!({
            "items": response.items,
            "page": response.page,
            "page_size": response.page_size,
            "total_pages": response.total_pages,
            "total_items": response.total_items,
        }),
    )
}

/// Show single item
#[debug_handler]
pub async fn show(
    headers: HeaderMap,
    ViewEngine(v): ViewEngine<TeraView>,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    // Redirect to main page if not an HTMX request (direct URL access)
    if !is_htmx_request(&headers) {
        return redirect_to_main_page();
    }

    let item = PromptTemplateService::find_by_id(&ctx.db, id).await?;

    format::render().view(
        &v,
        "admin/prompt_template/show.html",
        data!({
            "item": item,
        }),
    )
}

/// New form
#[debug_handler]
pub async fn new_form(
    headers: HeaderMap,
    ViewEngine(v): ViewEngine<TeraView>,
) -> Result<Response> {
    // Redirect to main page if not an HTMX request (direct URL access)
    if !is_htmx_request(&headers) {
        return redirect_to_main_page();
    }

    format::render().view(&v, "admin/prompt_template/create.html", data!({}))
}

/// Edit form
#[debug_handler]
pub async fn edit_form(
    headers: HeaderMap,
    ViewEngine(v): ViewEngine<TeraView>,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    // Redirect to main page if not an HTMX request (direct URL access)
    if !is_htmx_request(&headers) {
        return redirect_to_main_page();
    }

    let item = PromptTemplateService::find_by_id(&ctx.db, id).await?;

    format::render().view(
        &v,
        "admin/prompt_template/edit.html",
        data!({
            "item": item,
        }),
    )
}

/// Create new item
#[debug_handler]
pub async fn create(
    ViewEngine(v): ViewEngine<TeraView>,
    State(ctx): State<AppContext>,
    Json(params): Json<CreateParams>,
) -> Result<Response> {
    PromptTemplateService::create(&ctx.db, params).await?;

    // Return the full list to replace #search-result
    let query_params = QueryParams::default();
    let response = PromptTemplateService::search(&ctx.db, &query_params).await?;

    format::render().view(
        &v,
        "admin/prompt_template/list.html",
        data!({
            "items": response.items,
            "page": response.page,
            "page_size": response.page_size,
            "total_pages": response.total_pages,
            "total_items": response.total_items,
        }),
    )
}

/// Update existing item
#[debug_handler]
pub async fn update(
    ViewEngine(v): ViewEngine<TeraView>,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
    Json(params): Json<UpdateParams>,
) -> Result<Response> {
    PromptTemplateService::update(&ctx.db, id, params).await?;

    // Return the full list to replace #search-result
    let query_params = QueryParams::default();
    let response = PromptTemplateService::search(&ctx.db, &query_params).await?;

    format::render().view(
        &v,
        "admin/prompt_template/list.html",
        data!({
            "items": response.items,
            "page": response.page,
            "page_size": response.page_size,
            "total_pages": response.total_pages,
            "total_items": response.total_items,
        }),
    )
}

/// Delete item
#[debug_handler]
pub async fn delete(Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    PromptTemplateService::delete(&ctx.db, id).await?;
    format::html("")
}

/// Import form
#[debug_handler]
pub async fn import_form(
    headers: HeaderMap,
    ViewEngine(v): ViewEngine<TeraView>,
    State(_ctx): State<AppContext>,
    _auth_user: AuthUser,
) -> Result<Response> {
    // Redirect to main page if not an HTMX request (direct URL access)
    if !is_htmx_request(&headers) {
        return redirect_to_main_page();
    }

    format::render().view(&v, "admin/prompt_template/import.html", data!({}))
}

/// Import template from file
#[debug_handler]
pub async fn import(
    State(ctx): State<AppContext>,
    _auth_user: AuthUser,
    mut multipart: Multipart,
) -> Result<Response> {
    let mut file_content: Option<String> = None;
    let mut file_type: Option<String> = None;
    let mut deactivate_old = true;
    let mut set_active = true;

    // Parse multipart form data
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        Error::string(&format!("Failed to read multipart field: {}", e))
    })? {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "file" => {
                let filename = field.file_name().unwrap_or("").to_string();
                let data = field.bytes().await.map_err(|e| {
                    Error::string(&format!("Failed to read file data: {}", e))
                })?;

                file_content = Some(String::from_utf8(data.to_vec()).map_err(|e| {
                    Error::string(&format!("File is not valid UTF-8: {}", e))
                })?);

                // Detect file type from extension
                if filename.ends_with(".yaml") || filename.ends_with(".yml") {
                    file_type = Some("yaml".to_string());
                } else if filename.ends_with(".json") {
                    file_type = Some("json".to_string());
                } else {
                    return Err(Error::string("Unsupported file type. Please upload .yaml, .yml, or .json"));
                }
            }
            "deactivate_old" => {
                let value = field.text().await.map_err(|e| {
                    Error::string(&format!("Failed to read field: {}", e))
                })?;
                deactivate_old = value == "true" || value == "on";
            }
            "set_active" => {
                let value = field.text().await.map_err(|e| {
                    Error::string(&format!("Failed to read field: {}", e))
                })?;
                set_active = value == "true" || value == "on";
            }
            _ => {}
        }
    }

    // Validate we have a file
    let content = file_content.ok_or_else(|| Error::string("No file uploaded"))?;
    let ftype = file_type.ok_or_else(|| Error::string("Could not determine file type"))?;

    // Import options
    let options = ImportOptions {
        deactivate_old,
        force_version: None,
        set_active,
    };

    // Import template
    let result = if ftype == "yaml" {
        TemplateImporter::import_from_yaml(&ctx.db, &content, options).await
    } else {
        TemplateImporter::import_from_json(&ctx.db, &content, options).await
    };

    match result {
        Ok(import_result) => {
            // Return success message
            format::html(&format!(
                r#"<div class="p-4 rounded-lg bg-green-500/10 border border-green-500/20">
                    <p class="text-sm text-green-700 font-medium">{}</p>
                    <p class="text-xs text-green-600 mt-1">Template ID: {} | Version: {}</p>
                </div>
                <script>
                    setTimeout(() => {{
                        document.getElementById('modal-container').innerHTML = '';
                        htmx.trigger('#search-form', 'submit');
                    }}, 2000);
                </script>"#,
                import_result.message,
                import_result.template_id.unwrap_or(0),
                import_result.version
            ))
        }
        Err(e) => {
            // Return error message
            format::html(&format!(
                r#"<div class="p-4 rounded-lg bg-red-500/10 border border-red-500/20">
                    <p class="text-sm text-red-700 font-medium">Import Failed</p>
                    <p class="text-xs text-red-600 mt-1">{}</p>
                </div>"#,
                e.to_string()
            ))
        }
    }
}

/// Export template to YAML
#[debug_handler]
pub async fn export(
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
    _auth_user: AuthUser,
) -> Result<Response> {
    let yaml_content = TemplateImporter::export_to_yaml(&ctx.db, id).await
        .map_err(|e| Error::string(&format!("Export failed: {}", e)))?;

    // Get template info for filename
    let template = PromptTemplateService::find_by_id(&ctx.db, id).await?;
    let filename = format!("{}-v{}.yaml", template.name, template.version);

    // Return as downloadable file
    let response = Response::builder()
        .header("Content-Type", "application/x-yaml")
        .header("Content-Disposition", format!("attachment; filename=\"{}\"", filename))
        .body(yaml_content.into())
        .map_err(|e| Error::string(&format!("Failed to build response: {}", e)))?;

    Ok(response)
}
