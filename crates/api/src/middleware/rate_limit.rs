//! Rate limiting middleware

use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct RateLimiter {
    #[allow(dead_code)]
    requests: Arc<RwLock<std::collections::HashMap<String, u32>>>,
    #[allow(dead_code)]
    max_requests: u32,
}

impl RateLimiter {
    pub fn new(max_requests: u32) -> Self {
        Self {
            requests: Arc::new(RwLock::new(std::collections::HashMap::new())),
            max_requests,
        }
    }
}

pub async fn rate_limit_middleware(
    _rate_limiter: Arc<RateLimiter>,
    request: Request,
    next: Next,
) -> Result<Response, axum::http::StatusCode> {
    // TODO: Implement proper rate limiting
    // For now, just pass through
    Ok(next.run(request).await)
}
