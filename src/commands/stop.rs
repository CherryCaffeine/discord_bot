use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::Message,
    prelude::Context,
};

use crate::app_state::type_map_keys::ShardManagerKey;

#[command]
#[owners_only]
#[description = "Make Vampy take a nap."]
async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
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
