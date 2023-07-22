use serenity::async_trait;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::CommandResult;
use serenity::framework::StandardFramework;
use serenity::http::Http;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::prelude::{Guild, Member, PartialGuild, UserId};
use serenity::prelude::*;
use serenity::utils::MessageBuilder;
use sqlx::{Executor, PgPool};
use tokio::sync::RwLockWriteGuard;

mod app_cache;
mod consts;
mod db;
pub(crate) mod macros;
mod self_roles;

use app_cache::AppCache;
use consts::{
    DISCORD_BOT_CHANNEL, DISCORD_INTENTS, DISCORD_PREFIX, DISCORD_SERVER_ID, DISCORD_TOKEN,
    EXP_PER_MSG,
};
use ux::u63;

struct AppCacheKey;

impl TypeMapKey for AppCacheKey {
    type Value = AppCache;
}

#[group]
#[commands(ping)]
struct General;

struct Bot {
    pool: PgPool,
}

async fn build_client<H: EventHandler + 'static>(event_handler: H) -> Client {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix(DISCORD_PREFIX))
        .group(&GENERAL_GROUP);

    Client::builder(DISCORD_TOKEN, DISCORD_INTENTS)
        .framework(framework)
        .event_handler(event_handler)
        .await
        .expect("Err creating client")
}

async fn members(http: impl AsRef<Http>) -> Vec<Member> {
    const DEFAULT_LIMIT: usize = 1000;
    const USE_DEFAULT_LIMIT: Option<u64> = None;
    const NO_USER_ID_OFFSET: Option<UserId> = None;

    let members = DISCORD_SERVER_ID
        .members(http, USE_DEFAULT_LIMIT, NO_USER_ID_OFFSET)
        .await
        .unwrap_or_else(|e| {
            panic!("Failed to get the list of server members: {e}");
        });

    if members.len() == DEFAULT_LIMIT {
        let err = concat!(
            "Default limit for GuildId::members(...) reached.\n",
            "Chunkwise member list retrieval is required."
        );
        panic!("{err}");
    }

    members
}

impl Bot {
    fn print_server_members(server: &PartialGuild, members: &Vec<Member>) {
        println!("Members of {} ({} total):", server.name, members.len());

        for m in members.iter() {
            let id = m.user.id;
            let name = m.display_name();
            println!("{id:>20} {name}");
        }
    }
}

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, ctx: Context, ready: Ready) {
        let members = members(&ctx.http).await;

        let guild: PartialGuild = Guild::get(&ctx.http, DISCORD_SERVER_ID).await
            .expect("Encountered a Serenity error when getting partial guild information about the discord server");

        Self::print_server_members(&guild, &members);

        let mut wlock: RwLockWriteGuard<TypeMap> = ctx.data.write().await;

        let app_cache = AppCache::new(&self.pool, members).await;

        wlock.insert::<AppCacheKey>(app_cache);

        println!("{} is at your service! 🌸", ready.user.name);
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content.starts_with(DISCORD_PREFIX) {
            return;
        }
        println!("{}: {}", msg.author.name, msg.content);
        let mut wlock: RwLockWriteGuard<TypeMap> = ctx.data.write().await;
        let app_cache: &mut AppCache = wlock
            .get_mut::<AppCacheKey>()
            .expect("Failed to get the app cache from the typemap");

        let res: Result<u63, sqlx::Error> =
            app_cache::sync::add_signed_exp(app_cache, &self.pool, &msg.author.id, EXP_PER_MSG)
                .await;

        match res {
            Ok(exp) => {
                println!("{}'s exp: {exp}", msg.author.name);
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
) -> shuttle_serenity::ShuttleSerenity {
    pool.execute(include_str!("../schema.sql"))
        .await
        .expect("Failed to initialize database");

    let client = build_client(Bot { pool }).await;

    Ok(client.into())
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    if msg.channel_id != DISCORD_BOT_CHANNEL {
        let response = MessageBuilder::new()
            .mention(&msg.author)
            .push(" ")
            .push("I'm over here, lovely! 💕")
            .build();
        DISCORD_BOT_CHANNEL.say(&ctx.http, &response).await?;
        msg.delete(&ctx.http).await.unwrap_or_else(|e| {
            eprintln!("Error deleting message: {e}");
        });
    }
    // TODO: Randomize response
    msg.reply(ctx, "Yes, darling? 💕").await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serenity::client::bridge::gateway::ShardManager;
    use std::sync::Arc;

    struct ShardManagerKey;

    impl TypeMapKey for ShardManagerKey {
        type Value = Arc<Mutex<ShardManager>>;
    }

    struct TestEventHandler;

    #[async_trait]
    impl EventHandler for TestEventHandler {
        async fn ready(&self, ctx: Context, _: Ready) {
            let members = members(&ctx.http).await;

            // check if the members are sorted by id
            for w in members.windows(2) {
                assert!(w[0].user.id <= w[1].user.id);
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

    #[tokio::test]
    async fn test_props() {
        let mut client = build_client(TestEventHandler).await;

        {
            let mut wlock = client.data.write().await;
            wlock.insert::<ShardManagerKey>(client.shard_manager.clone());
        }

        if let Err(why) = client.start().await {
            println!("Client error: {:?}", why);
        }
    }
}
