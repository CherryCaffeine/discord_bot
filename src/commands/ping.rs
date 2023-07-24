use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::Message,
    prelude::Context,
    utils::MessageBuilder,
};

use crate::immut_data::consts::DISCORD_BOT_CHANNEL;

#[command]
#[description = "Check if Vampy is still around."]
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
