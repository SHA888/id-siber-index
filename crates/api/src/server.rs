//! API server implementation

use axum::Router;
use std::net::SocketAddr;

pub struct ApiServer {
    app: Router,
}

impl Default for ApiServer {
    fn default() -> Self {
        Self::new()
    }
}

impl ApiServer {
    pub fn new() -> Self {
        let app = Router::new();
        Self { app }
    }

    pub async fn run(self, host: &str, port: u16) -> anyhow::Result<()> {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));

        tracing::info!("Starting API server on http://{}:{}", host, port);

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, self.app).await?;

        Ok(())
    }
}
