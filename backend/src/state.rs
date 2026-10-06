use std::sync::Arc;

use sqlx::PgPool;

use crate::{
    config::Config,
    email::{DevMailer, Mailer},
};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Arc<Config>,
    pub mailer: Arc<dyn Mailer>,
}

impl AppState {
    pub fn new(db: PgPool, config: Config) -> Self {
        Self {
            mailer: Arc::new(DevMailer::new(db.clone())),
            db,
            config: Arc::new(config),
        }
    }
}
