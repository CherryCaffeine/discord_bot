use serenity::framework::StandardFramework;
use serenity::framework::standard::CommandResult;
use serenity::utils::MessageBuilder;
use serenity::{async_trait, model::prelude::ChannelId};
use serenity::framework::standard::macros::{command, group};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

const DISCORD_BOT_CHANNEL: ChannelId = {
    let id: &str = env!("DISCORD_BOT_CHANNEL");
    let id: u64 = const_str::parse!(id, u64);
    ChannelId(id)
};

const DISCORD_TOKEN: &str = env!("DISCORD_TOKEN");

const DISCORD_PREFIX: &str = env!("DISCORD_PREFIX");

const DISCORD_INTENTS: GatewayIntents = {
    let fst = GatewayIntents::GUILD_MESSAGES.bits();
    let snd = GatewayIntents::MESSAGE_CONTENT.bits();
    match GatewayIntents::from_bits(fst | snd) {
        Some(intents) => intents,
        None => panic!("Invalid intents"),
    }
};

#[group]
#[commands(ping)]
struct General;

struct Bot;

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is at your service! 🌸", ready.user.name);
    }
}

#[shuttle_runtime::main]
async fn serenity() -> shuttle_serenity::ShuttleSerenity {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix(DISCORD_PREFIX))
        .group(&GENERAL_GROUP);

    let client = Client::builder(&DISCORD_TOKEN, DISCORD_INTENTS)
        .framework(framework)
        .event_handler(Bot)
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
        // TODO: handle error
        let _ = msg.delete(&ctx.http).await?;
    }
    // TODO: Randomize response
    msg.reply(ctx, "Yes, darling? 💕").await?;

    Ok(())
}
