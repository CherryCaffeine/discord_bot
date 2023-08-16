use std::collections::HashSet;

use once_cell::sync::Lazy;
use regex::Regex;
use serenity::model::prelude::{ChannelId, GuildId, UserId};

// The method for configuration of the bot
// https://docs.rs/serenity/latest/serenity/framework/standard/struct.Configuration.html#method.owners
#[allow(clippy::unreadable_literal)]
pub(crate) fn owners() -> HashSet<UserId> {
    [UserId(286962466037170176)].into_iter().collect()
}

pub(crate) static WHITESPACE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[\s\n\r\t]+").unwrap());

use shuttle_secrets::SecretStore;

#[derive(Clone)]
pub(crate) struct BotCfg {
    pub(crate) discord_server_id: GuildId,
    pub(crate) discord_bot_channel: ChannelId,
    pub(crate) discord_self_role_channel: ChannelId,
    pub(crate) discord_token: String,
    pub(crate) discord_prefix: String,
}

impl BotCfg {
    pub(crate) fn new(secret_store: SecretStore) -> Self {
        let discord_server_id = secret_store.get("DISCORD_SERVER_ID").unwrap();
        let discord_server_id = GuildId(discord_server_id.parse::<u64>().unwrap());

        let discord_bot_channel = secret_store.get("DISCORD_BOT_CHANNEL").unwrap();
        let discord_bot_channel = ChannelId(discord_bot_channel.parse::<u64>().unwrap());

        let discord_self_role_channel = secret_store.get("DISCORD_SELF_ROLE_CHANNEL").unwrap();
        let discord_self_role_channel =
            ChannelId(discord_self_role_channel.parse::<u64>().unwrap());

        let discord_token = secret_store.get("DISCORD_TOKEN").unwrap();
        let discord_prefix = secret_store.get("DISCORD_PREFIX").unwrap();

        Self {
            discord_server_id,
            discord_bot_channel,
            discord_self_role_channel,
            discord_token,
            discord_prefix,
        }
    }
}
