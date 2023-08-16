use std::sync::Arc;

use serenity::{client::bridge::gateway::ShardManager, prelude::TypeMapKey};
use sqlx::PgPool;
use tokio::sync::Mutex;

use crate::immut_data::dynamic::BotCfg;

use super::AppState;

pub(crate) struct ShardManagerKey;
pub(crate) struct AppStateKey;
pub(crate) struct PgPoolKey;
pub(crate) struct BotCfgKey;

impl TypeMapKey for ShardManagerKey {
    type Value = Arc<Mutex<ShardManager>>;
}

impl TypeMapKey for AppStateKey {
    type Value = AppState;
}

impl TypeMapKey for PgPoolKey {
    type Value = PgPool;
}

impl TypeMapKey for BotCfgKey {
    type Value = BotCfg;
}
