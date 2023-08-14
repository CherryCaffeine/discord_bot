use serenity::prelude::GatewayIntents;

// pub(crate) const DISCORD_SERVER_ID: GuildId = {
//     let id: &str = env!("DISCORD_SERVER_ID");
//     let id: u64 = const_str::parse!(id, u64);
//     GuildId(id)
// };

// pub(crate) const DISCORD_BOT_CHANNEL: ChannelId = {
//     let id: &str = env!("DISCORD_BOT_CHANNEL");
//     let id: u64 = const_str::parse!(id, u64);
//     ChannelId(id)
// };

// #[allow(dead_code)]
// pub(crate) const DISCORD_SELF_ROLE_CHANNEL: ChannelId = {
//     let id: &str = env!("DISCORD_SELF_ROLE_CHANNEL");
//     let id: u64 = const_str::parse!(id, u64);
//     ChannelId(id)
// };

// pub(crate) const DISCORD_TOKEN: &str = env!("DISCORD_TOKEN");

// pub(crate) const DISCORD_PREFIX: &str = env!("DISCORD_PREFIX");

pub(crate) const DISCORD_INTENTS: GatewayIntents = {
    let fst = GatewayIntents::GUILD_MESSAGES.bits();
    let snd = GatewayIntents::MESSAGE_CONTENT.bits();
    match GatewayIntents::from_bits(fst | snd) {
        Some(intents) => intents,
        None => panic!("Invalid intents"),
    }
};

pub(crate) const EXP_PER_MSG: i64 = 5;
