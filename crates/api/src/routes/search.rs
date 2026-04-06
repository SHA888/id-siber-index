//! Search routes

use axum::{Router, routing::get};

use crate::handlers::search::*;

pub fn routes() -> Router {
    Router::new().route("/", get(search_incidents))
}
