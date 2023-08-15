use std::sync::Arc;

use serenity::{async_trait, prelude::{Context, EventHandler, TypeMap}, model::prelude::Ready, client::bridge::gateway::ShardManager};
use shuttle_secrets::SecretStore;
use sqlx::{PgPool, Executor};
use tokio::sync::Mutex;

use crate::{immut_data::dynamic::BotConfig, util::members, app_state::type_map_keys::ShardManagerKey};

use super::{config_ext::impl_config_ext, ConfigExt};

pub(crate) struct TestBot {
    #[allow(dead_code)]
    pool: PgPool,
    bot_config: BotConfig,
}

impl TestBot {
    pub(crate) async fn new(pool: PgPool, secret_store: SecretStore) -> Self {
        let bot_config = BotConfig::new(secret_store);
        pool.execute(crate::immut_data::consts::SCHEMA)
            .await
            .expect("Failed to initialize database");
        Self { pool, bot_config }
    }
}

impl_config_ext!(TestBot);

#[async_trait]
impl EventHandler for TestBot {
    async fn ready(&self, ctx: Context, _: Ready) {
        let members = members(&ctx.http, self.discord_server_id()).await;

        // check if the members are sorted by id
        for w in members.windows(2) {
            let [left, right] = w else { unreachable!() };
            assert!(left.user.id <= right.user.id);
        }

        let mut owlock: tokio::sync::OwnedRwLockWriteGuard<TypeMap> =
            ctx.data.write_owned().await;
        let sm: Arc<Mutex<ShardManager>> = owlock
            .remove::<ShardManagerKey>()
            .expect("The typemap was expected to contain a shard manager");
        let mut sm: tokio::sync::MutexGuard<'_, ShardManager> = sm.lock().await;
        sm.shutdown_all().await;
    }
}
