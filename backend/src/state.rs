use std::sync::Arc;

use sqlx::PgPool;

use crate::{
    cache::TaskCache,
    config::Config,
    email::{DevMailer, Mailer},
};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Arc<Config>,
    pub cache: Arc<dyn TaskCache>,
    pub mailer: Arc<dyn Mailer>,
}

impl AppState {
    pub fn new(db: PgPool, config: Config, cache: Arc<dyn TaskCache>) -> Self {
        Self {
            mailer: Arc::new(DevMailer::new(db.clone())),
            db,
            config: Arc::new(config),
            cache,
        }
    }
}
