use crate::web::security::RateLimiter;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub rate_limiter: Arc<RateLimiter>,
}
