use serenity::async_trait;
use serenity::framework::StandardFramework;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::prelude::{Guild, Member, PartialGuild};
use serenity::prelude::*;
use sqlx::{Executor, PgPool};
use tokio::sync::RwLockWriteGuard;

mod app_state;
mod commands;
mod db;
pub(crate) mod error;
pub(crate) mod immut_data;
pub(crate) mod util;

use app_state::type_map_keys::{AppStateKey, PgPoolKey, ShardManagerKey};
use app_state::AppState;
use commands::Progress;
use commands::{GENERAL_GROUP, MY_HELP};
use immut_data::consts::{
    DISCORD_INTENTS, DISCORD_PREFIX, DISCORD_SERVER_ID, DISCORD_TOKEN, EXP_PER_MSG,
};
use util::members;

use crate::app_state::exp::Exp;

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
    fn print_server_members(server: &PartialGuild, members: &[Member]) {
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
        if msg.content.starts_with(DISCORD_PREFIX) {
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
            app_state::sync::add_signed_exp(&ctx.http, app_state, &self.pool, &author, EXP_PER_MSG)
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
