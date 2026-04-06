//! Authentication middleware

use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;

pub async fn auth_middleware(request: Request, next: Next) -> Result<Response, StatusCode> {
    // TODO: Implement authentication
    // For now, just pass through
    Ok(next.run(request).await)
}
