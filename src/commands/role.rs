use std::collections::HashMap;

use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::{Message, Role, RoleId},
    prelude::Context,
    utils::MessageBuilder,
};

use crate::{
    immut_data::consts::{DISCORD_BOT_CHANNEL, DISCORD_PREFIX, DISCORD_SERVER_ID},
    util::say_wo_unintended_mentions,
};

#[command]
#[description = "Role command set."]
async fn role(ctx: &Context, msg: &Message) -> CommandResult {
    if msg
        .content
        .trim_start_matches(DISCORD_PREFIX)
        .trim_start_matches("role")
        != "ids"
    {
        return Ok(());
    }
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
