//! CORS middleware

use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use tower_http::cors::CorsLayer;

pub fn cors_layer() -> CorsLayer {
    // TODO: Configure CORS properly
    CorsLayer::permissive()
}
