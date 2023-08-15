use serenity::prelude::GatewayIntents;

pub(crate) const SCHEMA: &str = include_str!("../../schema.pgsql");

pub(crate) const DISCORD_INTENTS: GatewayIntents = {
    let fst = GatewayIntents::GUILD_MESSAGES.bits();
    let snd = GatewayIntents::MESSAGE_CONTENT.bits();
    match GatewayIntents::from_bits(fst | snd) {
        Some(intents) => intents,
        None => panic!("Invalid intents"),
    }
};

pub(crate) const EXP_PER_MSG: i64 = 5;
