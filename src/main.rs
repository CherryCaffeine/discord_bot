use serenity::async_trait;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::CommandResult;
use serenity::framework::StandardFramework;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;
use sqlx::{Executor, PgPool};

mod consts;
mod db;

use consts::*;
use db::UserDAO;

#[group]
#[commands(ping)]
struct General;

struct Bot {
    pool: PgPool,
}

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is at your service! 🌸", ready.user.name);
    }

    async fn message(&self, _: Context, msg: Message) {
        if msg.content.starts_with(DISCORD_PREFIX) {
            return;
        }
        println!("{}: {}", msg.author.name, msg.content);
        UserDAO::adjust_exp(&self.pool, &msg.author.id, EXP_PER_MSG)
            .await
            .unwrap_or_else(|e| {
                eprintln!("Sqlx error during adjusting experience: {e}");
            });
        let exp: i64 = match UserDAO::exp(&self.pool, msg.author.id).await {
            Ok(user) => user,
            Err(e) => {
                eprintln!("Sqlx error during querying the user data: {e}");
                return;
            }
        };
        println!("{}'s exp: {exp}", msg.author.name);
    }
}

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_shared_db::Postgres] pool: PgPool,
) -> shuttle_serenity::ShuttleSerenity {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix(DISCORD_PREFIX))
        .group(&GENERAL_GROUP);

    pool.execute(include_str!("../schema.sql"))
        .await
        .expect("Failed to initialize database");

    let client = Client::builder(&DISCORD_TOKEN, DISCORD_INTENTS)
        .framework(framework)
        .event_handler(Bot { pool })
        .await
        .expect("Err creating client");

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
        })
    }
    // TODO: Randomize response
    msg.reply(ctx, "Yes, darling? 💕").await?;

    Ok(())
}
