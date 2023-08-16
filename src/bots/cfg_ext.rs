use serenity::model::prelude::{GuildId, ChannelId};

use crate::immut_data::dynamic::BotCfg;

pub(crate) trait CfgExt {
    fn discord_server_id(&self) -> GuildId;
    fn discord_bot_channel(&self) -> ChannelId;
    fn discord_self_role_channel(&self) -> ChannelId;
    fn discord_token(&self) -> &str;
    fn discord_prefix(&self) -> &str;
    fn cfg(&self) -> BotCfg;
}

macro_rules! impl_cfg_ext {
    ($t:ty) => {
        impl crate::bots::CfgExt for $t {
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

pub(super) use impl_cfg_ext;
