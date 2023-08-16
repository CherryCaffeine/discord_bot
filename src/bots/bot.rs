use serenity::model::prelude::{ChannelId, GuildId};

use crate::immut_data::dynamic::BotCfg;

/// A trait whose implementors are the various bots that can be run.
pub(crate) trait Bot {
    /// The ID of the Discord server that the bot is running on.
    fn discord_server_id(&self) -> GuildId;
    /// The ID of the channel that the bot should use to communicate with the user.
    fn discord_bot_channel(&self) -> ChannelId;
    /// The ID of the channel that the bot should listen to for self-roles.
    fn discord_self_role_channel(&self) -> ChannelId;
    /// The token that the bot should use to log in to Discord.
    fn discord_token(&self) -> &str;
    /// The prefix that the bot should use for commands.
    fn discord_prefix(&self) -> &str;
    /// The bot's configuration. The use of this method is reserved for situations
    /// where the user needs to mutably access [`AppState`](crate::app_state::AppState)
    /// in [`Context::data`](serenity::client::Context::data) via
    /// [`AppStateKey`](crate::app_state::type_map_keys::AppStateKey) but also somehow immutably
    /// access [`BotCfg`] from the same context.
    fn cfg(&self) -> BotCfg;
}

/// Implements [`Bot`] trait for a type.
macro_rules! impl_bot {
    ($t:ty) => {
        impl crate::bots::Bot for $t {
            fn discord_server_id(&self) -> serenity::model::prelude::GuildId {
                self.cfg.discord_server_id
            }

            fn discord_bot_channel(&self) -> serenity::model::prelude::ChannelId {
                self.cfg.discord_bot_channel
            }

            fn discord_self_role_channel(&self) -> serenity::model::prelude::ChannelId {
                self.cfg.discord_self_role_channel
            }

            fn discord_token(&self) -> &str {
                &self.cfg.discord_token
            }

            fn discord_prefix(&self) -> &str {
                &self.cfg.discord_prefix
            }

            fn cfg(&self) -> BotCfg {
                self.cfg.clone()
            }
        }
    };
}

pub(super) use impl_bot;
