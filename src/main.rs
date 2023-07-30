use serenity::async_trait;
use serenity::framework::StandardFramework;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::prelude::{Guild, Member, PartialGuild};
use serenity::prelude::*;
use sqlx::{Executor, PgPool};
use tokio::sync::RwLockWriteGuard;
use ux::u63;

mod app_state;
mod commands;
mod db;
pub(crate) mod immut_data;
mod roles;
pub(crate) mod util;

use app_state::type_map_keys::{AppStateKey, PgPoolKey, ShardManagerKey};
use app_state::AppState;
use commands::Progress;
use commands::{GENERAL_GROUP, MY_HELP};
use immut_data::consts::{
    DISCORD_INTENTS, DISCORD_PREFIX, DISCORD_SERVER_ID, DISCORD_TOKEN, EXP_PER_MSG,
};
use util::members;

struct Bot {
    pool: PgPool,
}

async fn build_client<H: EventHandler + 'static>(event_handler: H) -> Client {
    let framework = StandardFramework::new()
        .configure(|c| {
            c.prefix(DISCORD_PREFIX);
            c.owners(immut_data::dynamic::owners());
            c
        })
        .help(&MY_HELP)
        .group(&GENERAL_GROUP);

    let client = Client::builder(DISCORD_TOKEN, DISCORD_INTENTS)
        .framework(framework)
        .event_handler(event_handler)
        .await
        .expect("Err creating client");

    {
        let mut wlock: RwLockWriteGuard<TypeMap> = client.data.write().await;
        wlock.insert::<ShardManagerKey>(client.shard_manager.clone());
    }

    client
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

        let app_cache = AppState::new(&self.pool, members).await;
        {
            let mut wlock: RwLockWriteGuard<TypeMap> = ctx.data.write().await;
            wlock.insert::<AppStateKey>(app_cache);
            wlock.insert::<PgPoolKey>(self.pool.clone());
        }

        println!("{} is at your service! 🌸", ready.user.name);
    }

    async fn message(&self, ctx: Context, msg: Message) {
        let mut wlock = ctx.data.write().await;
        let app_cache: &mut AppState = wlock
            .get_mut::<AppStateKey>()
            .expect("Failed to get the app cache from the typemap");
        if let Some((i, req)) = app_cache
            .reqd_prompts
            .earned_role
            .iter_mut()
            .enumerate()
            .find(|(_i, req)| req.discord_id == msg.author.id)
        {
            match req.progress.advance(self, &ctx, &msg).await.unwrap() {
                Some(_req) => (),
                None => {
                    app_cache.reqd_prompts.earned_role.remove(i);
                }
            };
            return;
        }
        if msg.content.starts_with(DISCORD_PREFIX) {
            return;
        }
        if msg.author.bot {
            return;
        }
        println!("{}: {}", msg.author.name, msg.content);

        let res: Result<u63, sqlx::Error> = {
            let mut wlock: RwLockWriteGuard<TypeMap> = ctx.data.write().await;
            let app_cache: &mut AppState = wlock
                .get_mut::<AppStateKey>()
                .expect("Failed to get the app cache from the typemap");
            app_state::sync::add_signed_exp(app_cache, &self.pool, &msg.author.id, EXP_PER_MSG)
                .await
        };

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
    pool.execute(include_str!("../schema.pgsql"))
        .await
        .expect("Failed to initialize database");

    let client = build_client(Bot { pool }).await;

    Ok(client.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serenity::client::bridge::gateway::ShardManager;
    use std::sync::Arc;

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

        if let Err(why) = client.start().await {
            println!("Client error: {:?}", why);
        }
    }
}
