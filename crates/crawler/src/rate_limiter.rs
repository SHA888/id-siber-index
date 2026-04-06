//! Rate limiting utilities

use governor::{Quota, RateLimiter as GovernorRateLimiter};
use std::num::NonZeroU32;
use std::sync::Arc;

/// Rate limiter for crawler requests
pub struct RateLimiter {
    limiter: Arc<
        GovernorRateLimiter<
            governor::state::direct::NotKeyed,
            governor::state::InMemoryState,
            governor::clock::QuantaClock,
        >,
    >,
}

impl RateLimiter {
    pub fn new(requests_per_second: u32) -> Self {
        let quota = Quota::per_second(NonZeroU32::new(requests_per_second).unwrap());
        let limiter = GovernorRateLimiter::direct(quota);
        Self {
            limiter: Arc::new(limiter),
        }
    }

    pub async fn acquire(&self) -> Result<(), anyhow::Error> {
        self.limiter.until_ready().await;
        Ok(())
    }
}
