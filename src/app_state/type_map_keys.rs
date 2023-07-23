use std::sync::Arc;

use serenity::{client::bridge::gateway::ShardManager, prelude::TypeMapKey};
use sqlx::PgPool;
use tokio::sync::Mutex;

use super::AppState;

pub(crate) struct ShardManagerKey;
pub(crate) struct AppStateKey;
pub(crate) struct PgPoolKey;

impl TypeMapKey for ShardManagerKey {
    type Value = Arc<Mutex<ShardManager>>;
}

impl TypeMapKey for AppStateKey {
    type Value = AppState;
}

impl TypeMapKey for PgPoolKey {
    type Value = PgPool;
}
