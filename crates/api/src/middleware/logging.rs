//! Logging middleware

use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use tracing::info;

pub async fn logging_middleware(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();

    let start_time = std::time::Instant::now();
    let response = next.run(request).await;
    let duration = start_time.elapsed();

    info!(
        method = %method,
        uri = %uri,
        status = %response.status(),
        duration_ms = duration.as_millis(),
        "Request completed"
    );

    response
}
