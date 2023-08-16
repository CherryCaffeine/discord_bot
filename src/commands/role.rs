use std::{collections::HashMap, ops::ControlFlow};

use crate::{
    app_state::{
        self,
        exp::Exp,
        reqd_prompts::ReqdPrompts,
        type_map_keys::{AppStateKey, BotCfgKey},
        AppState,
    },
    bots::{Bot, MainBot},
    util::say_wo_unintended_mentions,
};
use serenity::{
    async_trait,
    framework::standard::{macros::command, CommandError, CommandResult},
    http::Http,
    model::prelude::{Message, Role, RoleId, UserId},
    prelude::Context,
    utils::MessageBuilder,
};

use super::Progress;

pub(crate) enum EarnedRolePromptProgress {
    JustStarted,
    CollectedName(String),
    // After collection of exp, the prompt is done
}

#[async_trait]
impl Progress for EarnedRolePromptProgress {
    async fn advance(
        &mut self,
        bot: &MainBot,
        http: &Http,
        sorted_earned_roles: &mut Vec<app_state::EarnedRole>,
        users: &mut Vec<app_state::ServerMember>,
        msg: &Message,
    ) -> Result<Option<&mut Self>, CommandError> {
        let mut msg_builder = MessageBuilder::new();
        msg_builder.mention(&msg.author);
        msg_builder.push(" ");

        if msg.channel_id != bot.discord_bot_channel() {
            msg_builder.push("You have one or more pending prompts for adding an earned role. ");
            msg_builder.push("Please complete them in the bot channel.");
            bot.discord_bot_channel()
                .say(http, &msg_builder.build())
                .await?;
            return Ok(Some(self));
        };

        let ret = match self {
            Self::JustStarted => {
                let collected_name = msg.content.clone();
                msg_builder.push("The collected name for the role is: ");
                msg_builder.push(collected_name.as_str());
                msg_builder.push("\n\n");
                msg_builder.push(
                    "The corresponding role will be added once all necessary info is available. ",
                );
                msg_builder.push("How much exp is needed for attaining the earned role?");
                *self = Self::CollectedName(collected_name);
                Some(self)
            }
            Self::CollectedName(name) => {
                let role = bot
                    .discord_server_id()
                    .create_role(http, |r| r.name(&name))
                    .await?;
                // TODO: in case of error, repeat the prompt
                let exp_needed: Exp = match msg.content.parse::<u64>() {
                    Ok(exp_needed) => Exp(exp_needed),
                    Err(_) => {
                        return Err(CommandError::from("Failed to parse the exp_needed value"))
                    }
                };
                app_state::sync::add_earned_role(
                    http,
                    &bot.cfg,
                    sorted_earned_roles,
                    users,
                    &bot.pool,
                    role.id,
                    exp_needed,
                )
                .await?;
                msg_builder.push(&format!("The earned role {name} has been added."));
                None
            }
        };
        bot.discord_bot_channel()
            .say(http, &msg_builder.build())
            .await?;
        Ok(ret)
    }
}

impl Default for EarnedRolePromptProgress {
    fn default() -> Self {
        Self::JustStarted
    }
}

pub(crate) struct EarnedRolePromptReq {
    pub(crate) discord_id: UserId,
    pub(crate) progress: EarnedRolePromptProgress,
}

impl EarnedRolePromptReq {
    fn new(discord_id: UserId) -> Self {
        Self {
            discord_id,
            progress: EarnedRolePromptProgress::default(),
        }
    }

    /// Handles the earned role input request if it is pending.
    /// Returns `ControlFlow::Break(())` if the request was handled
    /// or `ControlFlow::Continue` if it was not pending.
    pub(crate) async fn handle_if_pending(
        bot: &MainBot,
        ctx: &Context,
        msg: &Message,
        sorted_earned_roles: &mut Vec<app_state::EarnedRole>,
        reqd_prompts: &mut ReqdPrompts,
        users: &mut Vec<app_state::ServerMember>,
    ) -> ControlFlow<()> {
        if let Some((i, req)) = reqd_prompts
            .earned_role
            .iter_mut()
            .enumerate()
            .find(|(_i, req)| req.discord_id == msg.author.id)
        {
            match req
                .progress
                .advance(bot, &ctx.http, sorted_earned_roles, users, msg)
                .await
                .unwrap()
            {
                Some(_req) => (),
                None => {
                    reqd_prompts.earned_role.remove(i);
                }
            };
            return ControlFlow::Break(());
        }
        ControlFlow::Continue(())
    }
}

#[command]
#[description = "Role command set."]
#[sub_commands(ids, add)]
async fn role(ctx: &Context, msg: &Message) -> CommandResult {
    let subcommands = ROLE_COMMAND_OPTIONS.sub_commands;
    let rlock = ctx.data.read().await;
    let bot_cfg = rlock.get::<BotCfgKey>().unwrap();

    let mut msg_builder = MessageBuilder::new();
    msg_builder.mention(&msg.author);
    msg_builder.push(" ");

    let actual_sub: Option<&str> = {
        let mut split_suffix = msg
            .content
            .trim_start_matches(&bot_cfg.discord_prefix)
            .trim_start_matches("role")
            .split_ascii_whitespace();
        split_suffix.next()
    };
    if let Some(actual_sub) = actual_sub {
        for sub in subcommands {
            if sub.options.names.contains(&actual_sub) {
                return Ok(());
            };
        }
        let actual_sub = actual_sub.replace('`', "");
        msg_builder.push(&format!("Unknown subcommand `{actual_sub}`"));
    } else {
        msg_builder.push("Try one of the following subcommands:\n");
        for sub_name in subcommands
            .iter()
            .filter_map(|sub| sub.options.names.first())
        {
            msg_builder.push("\t");
            msg_builder.push("`");
            msg_builder.push(sub_name);
            msg_builder.push("`");
        }
    };

    bot_cfg
        .discord_bot_channel
        .say(&ctx.http, &msg_builder.build())
        .await?;
    if msg.channel_id != bot_cfg.discord_bot_channel {
        msg.delete(&ctx).await?;
    }

    Ok(())
}

#[command]
async fn ids(ctx: &Context, msg: &Message) -> CommandResult {
    let rlock = ctx.data.read().await;
    let bot_cfg = rlock.get::<BotCfgKey>().unwrap();
    let roles: HashMap<RoleId, Role> = bot_cfg.discord_server_id.roles(&ctx.http).await?;

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

    say_wo_unintended_mentions(
        bot_cfg.discord_bot_channel,
        &ctx,
        Some(msg.author.id),
        &response,
    )
    .await?;
    if msg.channel_id != bot_cfg.discord_bot_channel {
        msg.delete(&ctx.http).await?;
    };
    Ok(())
}

#[command]
#[sub_commands(earned)]
async fn add(ctx: &Context, msg: &Message) -> CommandResult {
    let rlock = ctx.data.read().await;
    let bot_cfg = rlock.get::<BotCfgKey>().unwrap();
    let subcommands = ADD_COMMAND_OPTIONS.sub_commands;

    let mut msg_builder = MessageBuilder::new();
    msg_builder.mention(&msg.author);
    msg_builder.push(" ");

    let actual_sub: Option<&str> = {
        let mut split_suffix = msg
            .content
            .trim_start_matches(&bot_cfg.discord_prefix)
            .trim_start_matches("role")
            .trim_start_matches(' ')
            .trim_start_matches("add")
            .split_ascii_whitespace();
        split_suffix.next()
    };
    if let Some(actual_sub) = actual_sub {
        for sub in subcommands {
            if sub.options.names.contains(&actual_sub) {
                return Ok(());
            };
        }
        let actual_sub = actual_sub.replace('`', "");
        msg_builder.push(&format!("Unknown subcommand `{actual_sub}`"));
    } else {
        msg_builder.push("Try one of the following subcommands:\n");
        for sub_name in subcommands
            .iter()
            .filter_map(|sub| sub.options.names.first())
        {
            msg_builder.push("\t");
            msg_builder.push("`");
            msg_builder.push(sub_name);
            msg_builder.push("`");
        }
    };

    bot_cfg
        .discord_bot_channel
        .say(&ctx.http, &msg_builder.build())
        .await?;
    if msg.channel_id != bot_cfg.discord_bot_channel {
        msg.delete(&ctx).await?;
    }

    Ok(())
}

#[command]
async fn earned(ctx: &Context, msg: &Message) -> CommandResult {
    let mut msg_builder = MessageBuilder::new();
    msg_builder.mention(&msg.author);
    msg_builder.push(" ");
    msg_builder.push("What's the name of the role that you want to add?");

    {
        let mut wlock = ctx.data.write().await;
        let app_state: &mut AppState = wlock
            .get_mut::<AppStateKey>()
            .expect("Failed to get the app state from the typemap");
        let earned_role_reqs: &mut Vec<EarnedRolePromptReq> =
            &mut app_state.reqd_prompts.earned_role;
        let req: Option<&mut EarnedRolePromptReq> = earned_role_reqs
            .iter_mut()
            .find(|req| req.discord_id == msg.author.id);
        if let Some(req) = req {
            *req = EarnedRolePromptReq::new(msg.author.id);
        } else {
            earned_role_reqs.push(EarnedRolePromptReq::new(msg.author.id));
        }
    }

    let rlock = ctx.data.read().await;
    let bot_cfg = rlock.get::<BotCfgKey>().unwrap();

    bot_cfg
        .discord_bot_channel
        .say(&ctx.http, &msg_builder.build())
        .await?;
    if msg.channel_id != bot_cfg.discord_bot_channel {
        msg.delete(&ctx).await?;
    }
    Ok(())
}
