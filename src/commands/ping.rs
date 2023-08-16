use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::Message,
    prelude::Context,
    utils::MessageBuilder,
};

use crate::app_state::type_map_keys::BotCfgKey;

#[command]
#[description = "Check if Vampy is still around."]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    let rlock = ctx.data.read().await;
    let bot_cfg = rlock.get::<BotCfgKey>().unwrap();
    if msg.channel_id != bot_cfg.discord_bot_channel {
        let response = MessageBuilder::new()
            .mention(&msg.author)
            .push(" ")
            .push("I'm over here, lovely! 💕")
            .build();
        bot_cfg.discord_bot_channel.say(&ctx.http, &response).await?;
        msg.delete(&ctx.http).await.unwrap_or_else(|e| {
            eprintln!("Error deleting message: {e}");
        });
        return Ok(());
    }
    // TODO: Randomize response
    msg.reply(ctx, "Yes, darling? 💕").await?;

    Ok(())
}
