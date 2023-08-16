use std::sync::Arc;

use serenity::{
    async_trait,
    client::bridge::gateway::ShardManager,
    model::prelude::Ready,
    prelude::{Context, EventHandler, TypeMap},
};
use shuttle_secrets::SecretStore;
use sqlx::{Executor, PgPool};
use tokio::sync::Mutex;

use crate::{
    app_state::type_map_keys::ShardManagerKey, immut_data::dynamic::BotCfg, util::members,
};

use super::{bot::impl_bot, Bot};

/// The bot structure that is used to execute tests during [EventHandler::ready] event.
///
/// The actual bot is [`MainBot`](crate::bots::MainBot).
pub(crate) struct TestBot {
    /// Database connection pool for PostgreSQL database.
    /// It is used to persist data between restarts.
    #[allow(dead_code)]
    pub(crate) pool: PgPool,
    /// The configuration of the bot.
    pub(crate) cfg: BotCfg,
}

impl TestBot {
    /// Creates a new instance of the bot.
    pub(crate) async fn new(pool: PgPool, secret_store: SecretStore) -> Self {
        let cfg = BotCfg::new(secret_store);
        pool.execute(crate::immut_data::consts::SCHEMA)
            .await
            .expect("Failed to initialize database");
        Self { pool, cfg }
    }
}

impl_bot!(TestBot);

#[async_trait]
impl EventHandler for TestBot {
    async fn ready(&self, ctx: Context, _: Ready) {
        let members = members(&ctx.http, self.discord_server_id()).await;

        // check if the members are sorted by id
        for w in members.windows(2) {
            let [left, right] = w else { unreachable!() };
            assert!(left.user.id <= right.user.id);
        }

        let mut owlock: tokio::sync::OwnedRwLockWriteGuard<TypeMap> = ctx.data.write_owned().await;
        let sm: Arc<Mutex<ShardManager>> = owlock
            .remove::<ShardManagerKey>()
            .expect("The typemap was expected to contain a shard manager");
        let mut sm: tokio::sync::MutexGuard<'_, ShardManager> = sm.lock().await;
        sm.shutdown_all().await;
    }
}
