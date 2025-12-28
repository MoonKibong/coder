//! Home Controller
//!
//! Landing page for xFrame5 Code Generator

use loco_rs::prelude::*;

/// Landing page
#[debug_handler]
pub async fn index(
    ViewEngine(v): ViewEngine<TeraView>,
) -> Result<Response> {
    format::render()
        .view(&v, "home/index.html", data!({}))
}

pub fn routes() -> Routes {
    Routes::new()
        .add("/", get(index))
}
