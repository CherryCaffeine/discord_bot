use std::collections::{HashMap, HashSet};

use serenity::{
    framework::standard::{
        help_commands,
        macros::{command, group, help},
        Args, CommandGroup, CommandResult, HelpOptions,
    },
    model::prelude::{Message, Role, RoleId, UserId},
    prelude::Context,
    utils::MessageBuilder,
};
use sqlx::{Column, PgPool, Row, TypeInfo, ValueRef};

use crate::{
    app_state::type_map_keys::{PgPoolKey, ShardManagerKey},
    immut_data::{
        consts::{DISCORD_BOT_CHANNEL, DISCORD_PREFIX, DISCORD_SERVER_ID},
        dynamic::WHITESPACE,
    },
    util::say_wo_unintended_mentions,
};

#[group]
#[commands(ping, role, sql, quit)]
struct General;

// The framework provides two built-in help commands for you to use.
// But you can also make your own customized help command that forwards
// to the behaviour of either of them.
#[help]
// This replaces the information that a user can pass
// a command-name as argument to gain specific information about it.
#[individual_command_tip = "Hello! こんにちは！Hola! Bonjour! 您好! 안녕하세요~\n\n\
If you want more information about a specific command, just pass the command as argument."]
// Some arguments require a `{}` in order to replace it with contextual information.
// In this case our `{}` refers to a command's name.
#[command_not_found_text = "Could not find: `{}`."]
// Define the maximum Levenshtein-distance between a searched command-name
// and commands. If the distance is lower than or equal the set distance,
// it will be displayed as a suggestion.
// Setting the distance to 0 will disable suggestions.
#[max_levenshtein_distance(3)]
// When you use sub-groups, Serenity will use the `indention_prefix` to indicate
// how deeply an item is indented.
// The default value is "-", it will be changed to "+".
#[indention_prefix = "+"]
// On another note, you can set up the help-menu-filter-behaviour.
// Here are all possible settings shown on all possible options.
// First case is if a user lacks permissions for a command, we can hide the command.
#[lacking_permissions = "Hide"]
// If the user is nothing but lacking a certain role, we just display it hence our variant is `Nothing`.
#[lacking_role = "Hide"]
// The last `enum`-variant is `Strike`, which ~~strikes~~ a command.
#[wrong_channel = "Strike"]
// Serenity will automatically analyse and generate a hint/tip explaining the possible
// cases of ~~strikethrough-commands~~, but only if
// `strikethrough_commands_tip_in_{dm, guild}` aren't specified.
// If you pass in a value, it will be displayed instead.
async fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}

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

#[command]
#[owners_only]
#[description = "Vampy will run any PostgreSQL <https://www.crunchydata.com/developers/playground/psql-basics> errands for you. Use with caution."]
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
                        format!("{value}: (INT8)")
                    } else if type_name == "BOOL" {
                        let value: bool = slice[0] == 1;
                        format!("{value:?}: (BOOL)")
                    } else {
                        format!("{slice:?}: ({type_name})")
                    }
                }
                sqlx::postgres::PgValueFormat::Text => value.as_str().unwrap().to_string(),
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
#[description = "Make Vampy take a nap."]
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
