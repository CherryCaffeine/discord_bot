use immut_data::dynamic::BotConfig;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::prelude::{Guild, Member, PartialGuild, GuildId, ChannelId};
use serenity::prelude::*;
use shuttle_secrets::SecretStore;
use sqlx::{Executor, PgPool};
use tokio::sync::RwLockWriteGuard;

mod app_state;
mod commands;
mod db;
pub(crate) mod error;
pub(crate) mod immut_data;
pub(crate) mod util;
mod config_ext;

use app_state::type_map_keys::{AppStateKey, PgPoolKey};
use app_state::AppState;
use commands::Progress;
use immut_data::consts::EXP_PER_MSG;
use util::{members, build_client};
use config_ext::{ConfigExt, impl_config_ext};
use crate::app_state::exp::Exp;

struct Bot {
    pool: PgPool,
    pub(crate) bot_config: BotConfig,
}

impl Bot {
    async fn new(pool: PgPool, secret_store: SecretStore) -> Self {
        let bot_config = BotConfig::new(secret_store);
        pool.execute(include_str!("../schema.pgsql"))
            .await
            .expect("Failed to initialize database");
        Self { pool, bot_config }
    }

    fn print_server_members(server: &PartialGuild, members: &[Member]) {
        println!("Members of {} ({} total):", server.name, members.len());

        for m in members.iter() {
            let id = m.user.id;
            let name = m.display_name();
            println!("{id:>20} {name}");
        }
    }
}

impl_config_ext!(Bot);

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, ctx: Context, ready: Ready) {
        let members = members(&ctx.http, self.discord_server_id()).await;

        let guild: PartialGuild = Guild::get(&ctx.http, self.discord_server_id()).await
            .unwrap_or_else(|e| panic!("Encountered a Serenity error when getting partial guild information about the discord server: {e:?}"));

        Self::print_server_members(&guild, &members);

        let app_state = AppState::new(&self.pool, members).await;
        {
            let mut wlock: RwLockWriteGuard<TypeMap> = ctx.data.write().await;
            wlock.insert::<AppStateKey>(app_state);
            wlock.insert::<PgPoolKey>(self.pool.clone());
        }

        let bot_name: &str = &ready.user.name;
        println!("{bot_name} is at your service! 🌸");
    }

    async fn message(&self, ctx: Context, msg: Message) {
        let mut wlock = ctx.data.write().await;
        let app_state: &mut AppState = wlock
            .get_mut::<AppStateKey>()
            .expect("Failed to get the app cache from the typemap");
        let AppState {
            users,
            reqd_prompts,
            sorted_earned_roles,
            self_role_msgs: _self_role_msgs,
        } = app_state;
        if let Some((i, req)) = reqd_prompts
            .earned_role
            .iter_mut()
            .enumerate()
            .find(|(_i, req)| req.discord_id == msg.author.id)
        {
            match req
                .progress
                .advance(self, &ctx.http, sorted_earned_roles, users, &msg)
                .await
                .unwrap()
            {
                Some(_req) => (),
                None => {
                    app_state.reqd_prompts.earned_role.remove(i);
                }
            };
            return;
        }
        // we retain wlock because the checks are quick
        if msg.content.starts_with(self.discord_prefix()) {
            return;
        }
        if msg.author.bot {
            return;
        }
        println!("{}: {}", msg.author.name, msg.content);

        let res: error::Result<Exp> = {
            let author: Member = msg.member(&ctx).await.unwrap_or_else(|e| {
                panic!("Failed to get member info for the message author: {e}")
            });
            app_state::sync::add_signed_exp(&ctx.http, &self.bot_config, app_state, &self.pool, &author, EXP_PER_MSG)
                .await
        };

        match res {
            Ok(exp) => {
                println!("{}'s exp: {exp:?}", msg.author.name);
            }
            Err(e) => {
                eprintln!("Sqlx error during adjusting experience: {e}");
            }
        };
    }
}

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_shared_db::Postgres] pool: PgPool,
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
    let bot = Bot::new(pool, secret_store).await;
    let client = build_client(bot).await;
    Ok(client.into())
}

#[cfg(test)]
mod tests {
    use crate::app_state::type_map_keys::ShardManagerKey;

    use super::*;
    use serenity::client::bridge::gateway::ShardManager;
    use std::sync::Arc;

    struct TestEventHandler {
        #[allow(dead_code)]
        pool: PgPool,
        bot_config: BotConfig,
    }

    impl TestEventHandler {
        async fn new(pool: PgPool, secret_store: SecretStore) -> Self {
            let bot_config = BotConfig::new(secret_store);
            pool.execute(include_str!("../schema.pgsql"))
                .await
                .expect("Failed to initialize database");
            Self { pool, bot_config }
        }
    }

    impl_config_ext!(TestEventHandler);

    #[async_trait]
    impl EventHandler for TestEventHandler {
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

    // We had to desugar the #[tokio::test] macro because
    // we need to access the secret storage
    #[::core::prelude::v1::test]
    fn test_props() {
        async fn __shuttle_test_props(
            pool: PgPool,
            secret_store: SecretStore,
        ) -> shuttle_serenity::ShuttleSerenity {
            let test_bot = TestEventHandler::new(pool, secret_store).await;
            let client = build_client(test_bot).await;
            Ok(client.into())
        }

        async fn loader(
            mut factory: shuttle_runtime::ProvisionerFactory,
            mut resource_tracker: shuttle_runtime::ResourceTracker,
            logger: shuttle_runtime::Logger,
        ) -> shuttle_serenity::ShuttleSerenity {
            use shuttle_runtime::tracing_subscriber::prelude::*;
            use shuttle_runtime::Context;
            use shuttle_runtime::ResourceBuilder;
            let filter_layer = shuttle_runtime::tracing_subscriber::EnvFilter::try_from_default_env()
                .or_else(|_| shuttle_runtime::tracing_subscriber::EnvFilter::try_new("INFO"))
                .unwrap();
            shuttle_runtime::tracing_subscriber::registry()
                .with(filter_layer)
                .with(logger)
                .init();
            let pool = shuttle_runtime::get_resource(
                shuttle_shared_db::Postgres::new(),
                &mut factory,
                &mut resource_tracker,
            )
            .await
            .context(format!(
                "failed to provision {}",
                stringify!(shuttle_shared_db::Postgres)
            ))?;
            let secret_store = shuttle_runtime::get_resource(
                shuttle_secrets::Secrets::new(),
                &mut factory,
                &mut resource_tracker,
            )
            .await
            .context(format!(
                "failed to provision {}",
                stringify!(shuttle_secrets::Secrets)
            ))?;
            __shuttle_test_props(pool, secret_store).await
        }

        let body = async {
            shuttle_runtime::start(loader).await;
        };
        tokio::pin!(body);
        let body: ::std::pin::Pin<&mut dyn ::std::future::Future<Output = ()>> = body;
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed building the Runtime")
            .block_on(body);
    }
}
