//! Crawler scheduler

use std::time::Duration;
use tokio::time::interval;

pub struct CrawlerScheduler {
    interval: Duration,
}

impl CrawlerScheduler {
    pub fn new(interval: Duration) -> Self {
        Self { interval }
    }

    pub async fn run<F>(&self, task: F) -> anyhow::Result<()>
    where
        F: Fn() -> anyhow::Result<()>,
    {
        let mut ticker = interval(self.interval);
        ticker.tick().await; // Skip first tick

        loop {
            ticker.tick().await;
            if let Err(e) = task() {
                tracing::error!("Scheduled task failed: {}", e);
            }
        }
    }
}
