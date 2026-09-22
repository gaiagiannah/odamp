use std::sync::Arc;
use sqlx::PgPool;

pub struct AppState {
    pub pool: PgPool,
    pub jwt_secret: String,
    pub compliance: Arc<std::sync::Mutex<odamp_compliance::ComplianceEngine>>,
}   