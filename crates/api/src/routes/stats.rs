//! Statistics routes

use axum::{Router, routing::get};

use crate::handlers::stats::*;

pub fn routes() -> Router {
    Router::new().route("/", get(get_stats))
}
