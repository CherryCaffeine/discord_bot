use serenity::async_trait;
use serenity::client::bridge::gateway::ShardManager;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::CommandResult;
use serenity::framework::StandardFramework;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::prelude::{Guild, Member, PartialGuild, Role, RoleId};
use serenity::prelude::*;
use serenity::utils::MessageBuilder;
use sqlx::{Column, Executor, PgPool, Row, TypeInfo, ValueRef};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLockWriteGuard;
use ux::u63;

mod app_cache;
mod db;
pub(crate) mod immut_data;
mod roles;
pub(crate) mod util;

use crate::immut_data::dynamic::WHITESPACE;
use crate::util::say_wo_unintended_mentions;
use app_cache::AppCache;
use immut_data::consts::{
    DISCORD_BOT_CHANNEL, DISCORD_INTENTS, DISCORD_PREFIX, DISCORD_SERVER_ID, DISCORD_TOKEN,
    EXP_PER_MSG,
};
use util::members;

struct ShardManagerKey;

impl TypeMapKey for ShardManagerKey {
    type Value = Arc<Mutex<ShardManager>>;
}

struct AppCacheKey;

impl TypeMapKey for AppCacheKey {
    type Value = AppCache;
}

struct PgPoolKey;

impl TypeMapKey for PgPoolKey {
    type Value = PgPool;
}

#[group]
#[commands(ping, role_ids, sql, quit)]
struct General;

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

        let app_cache = AppCache::new(&self.pool, members).await;
        let mut wlock: RwLockWriteGuard<TypeMap> = ctx.data.write().await;
        wlock.insert::<AppCacheKey>(app_cache);
        wlock.insert::<PgPoolKey>(self.pool.clone());

        println!("{} is at your service! 🌸", ready.user.name);
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content.starts_with(DISCORD_PREFIX) {
            return;
        }
        if msg.author.bot {
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
    pool.execute(include_str!("../schema.pgsql"))
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
        return Ok(());
    }
    // TODO: Randomize response
    msg.reply(ctx, "Yes, darling? 💕").await?;

    Ok(())
}

#[command]
async fn role_ids(ctx: &Context, msg: &Message) -> CommandResult {
    let roles: HashMap<RoleId, Role> = DISCORD_SERVER_ID.roles(&ctx.http).await?;

    let response: String = {
        let mut msg_builder = MessageBuilder::new();
        msg_builder
            .mention(&msg.author)
            .push("\n\n")
            .push("Roles' IDs:\n");
        for (role_id, role) in &roles {
            msg_builder
                .push("\t")
                .push(role.name.as_str())
                .push(": ")
                .push(role_id.0.to_string())
                .push("\n");
        }
        msg_builder.build()
    };

    if msg.channel_id != DISCORD_BOT_CHANNEL {
        say_wo_unintended_mentions(DISCORD_BOT_CHANNEL, &ctx, Some(msg.author.id), &response)
            .await?;
        msg.delete(&ctx.http).await.unwrap_or_else(|e| {
            eprintln!("Error deleting message: {e}");
        });
        return Ok(());
    };
    say_wo_unintended_mentions(msg.channel_id, &ctx, Some(msg.author.id), &response).await?;
    Ok(())
}

#[command]
#[owners_only]
async fn sql(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let query = {
        let q = msg
            .content
            .trim_start_matches(DISCORD_PREFIX)
            .trim_start_matches("sql ");
        WHITESPACE.replace_all(q, " ")
    };
    println!("Executing query: \"{query}\"");
    let pool: &PgPool = data
        .get::<PgPoolKey>()
        .expect("Failed to get the database pool from the typemap");
    // Result of the query is a vector of rows
    let res: Vec<sqlx::postgres::PgRow> = sqlx::query(&query).fetch_all(pool).await?;
    let mut simplified = Vec::<HashMap<String, String>>::with_capacity(res.len());
    for row in res {
        let columns = row.columns();
        let mut hm = HashMap::<String, String>::with_capacity(columns.len());
        for col in row.columns() {
            let value = row.try_get_raw(col.ordinal()).unwrap();
            let value = match value.format() {
                sqlx::postgres::PgValueFormat::Binary => {
                    let type_info = value.type_info();
                    let type_name = type_info.name();
                    let slice = value.as_bytes().unwrap();
                    if type_name == "INT8" {
                        let value = i64::from_be_bytes(slice.try_into().unwrap());
                        format!("{}: (INT8)", value)
                    } else if type_name == "BOOL" {
                        let value: bool = slice[0] == 1;
                        format!("{value:?}: (BOOL)")
                    } else {
                        format!("{slice:?}: ({type_name})")
                    }
                }
                sqlx::postgres::PgValueFormat::Text => {
                    format!("{}", value.as_str().unwrap())
                }
            };
            hm.insert(col.name().to_string(), value);
        }
        simplified.push(hm);
    }

    let response = {
        let db_response = serde_json::to_string_pretty(&simplified)?;
        let mut msg_builder = MessageBuilder::new();
        msg_builder
            .mention(&msg.author)
            .push("\n\n")
            .push("Result:\n")
            .push("```json\n")
            .push(&db_response)
            .push("```");
        msg_builder.build()
    };

    msg.reply(&ctx.http, &response).await?;

    Ok(())
}

#[command]
#[owners_only]
async fn quit(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;

    if let Some(sm) = data.get::<ShardManagerKey>() {
        msg.reply(ctx, "Shutting down!").await?;
        let mut wlock = sm.lock().await;
        wlock.shutdown_all().await;
        // TODO: This doesn't work withouth the following line. Why?
        std::process::exit(0);
    } else {
        msg.reply(ctx, "There was a problem getting the shard manager")
            .await?;
    }
    Ok(())
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
