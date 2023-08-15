use serenity::model::prelude::{GuildId, ChannelId};

use crate::immut_data::dynamic::BotConfig;

pub(crate) trait ConfigExt {
    fn discord_server_id(&self) -> GuildId;
    fn discord_bot_channel(&self) -> ChannelId;
    fn discord_self_role_channel(&self) -> ChannelId;
    fn discord_token(&self) -> &str;
    fn discord_prefix(&self) -> &str;
    fn bot_config(&self) -> BotConfig;
}

macro_rules! impl_config_ext {
    ($t:ty) => {
        impl crate::bots::ConfigExt for $t {
            fn discord_server_id(&self) -> serenity::model::prelude::GuildId {
                self.bot_config.discord_server_id
            }
        
            fn discord_bot_channel(&self) -> serenity::model::prelude::ChannelId {
                self.bot_config.discord_bot_channel
            }
        
            fn discord_self_role_channel(&self) -> serenity::model::prelude::ChannelId {
                self.bot_config.discord_self_role_channel
            }
        
            fn discord_token(&self) -> &str {
                &self.bot_config.discord_token
            }
        
            fn discord_prefix(&self) -> &str {
                &self.bot_config.discord_prefix
            }
        
            fn bot_config(&self) -> BotConfig {
                self.bot_config.clone()
            }
        }
    };
}

pub(super) use impl_config_ext;
