use rand::seq::SliceRandom;
use serenity::{
    framework::StandardFramework,
    http::{CacheHttp, Http},
    model::prelude::{ChannelId, GuildId, Member, UserId},
    prelude::{EventHandler, Mentionable, TypeMap},
    utils::MessageBuilder,
    Client,
};
use tokio::sync::RwLockWriteGuard;

use crate::{
    app_state::type_map_keys::{BotCfgKey, ShardManagerKey},
    bots::Bot,
    commands::{GENERAL_GROUP, MY_HELP},
    immut_data::{self, consts::DISCORD_INTENTS},
};

pub(crate) mod macros;

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("Sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("Serenity error: {0}")]
    Serenity(#[from] serenity::Error),
}

pub(crate) type Result<T> = core::result::Result<T, Error>;

pub(super) async fn members(http: impl AsRef<Http>, discord_server_id: GuildId) -> Vec<Member> {
    const DEFAULT_LIMIT: usize = 1000;
    const USE_DEFAULT_LIMIT: Option<u64> = None;
    const NO_USER_ID_OFFSET: Option<UserId> = None;

    let members = discord_server_id
        .members(http, USE_DEFAULT_LIMIT, NO_USER_ID_OFFSET)
        .await
        .unwrap_or_else(|e| {
            panic!("Failed to get the list of server members: {e}");
        });

    if members.len() == DEFAULT_LIMIT {
        let err = concat!(
            "Default limit for GuildId::members(...) reached.\n",
            "Chunkwise member list retrieval is required."
        );
        panic!("{err}");
    }

    members
}

pub(super) async fn say_wo_unintended_mentions(
    chan: ChannelId,
    cache_http: impl CacheHttp,
    author_mention: Option<impl Mentionable>,
    content: impl std::fmt::Display,
) -> serenity::Result<()> {
    // The function works by sending a message with a random emote, then editing

    const SHORT_LIVED_MESSAGES: &[&str] = &[
        "Hi there :wink:",
        "You're lovely :kissing_heart:",
        "You're cute :smiling_face_with_3_hearts:",
        "I'm totally sane :zany_face:",
        "Silly goose :stuck_out_tongue_winking_eye:",
    ];

    let short_lived_msg_wo_mention = SHORT_LIVED_MESSAGES
        .choose(&mut rand::thread_rng())
        .unwrap_or_else(|| unreachable!());

    let short_lived_msg = {
        let mut msg_builder = MessageBuilder::new();
        if let Some(author_mention) = author_mention {
            msg_builder.mention(&author_mention).push(" ");
        }
        msg_builder.push(short_lived_msg_wo_mention).build()
    };

    let mut bots_response = chan.say(cache_http.http(), &short_lived_msg).await?;
    let long_lived_msg = {
        let mut msg_builder = MessageBuilder::new();
        msg_builder
            .push(content)
            .push("\n\n")
            .push("Did you see that? :flushed: ")
            .push("I edited the message in order to avoid unintended pings. ")
            .push(r#"That's why, I swear :sweat_smile: I have nothing to hide :skull:"#);
        msg_builder.build()
    };
    bots_response
        .edit(cache_http, |m| m.content(&long_lived_msg))
        .await?;

    Ok(())
}

pub(super) async fn build_client<B: EventHandler + Bot + 'static>(bot: B) -> Client {
    let framework = StandardFramework::new()
        .configure(|c| {
            c.prefix(bot.discord_prefix());
            c.owners(immut_data::dynamic::owners());
            c
        })
        .help(&MY_HELP)
        .group(&GENERAL_GROUP);

    let bot_cfg = bot.cfg();

    let client = Client::builder(bot.discord_token(), DISCORD_INTENTS)
        .framework(framework)
        .event_handler(bot)
        .await
        .expect("Err creating client");

    {
        let mut wlock: RwLockWriteGuard<TypeMap> = client.data.write().await;
        wlock.insert::<ShardManagerKey>(client.shard_manager.clone());
        wlock.insert::<BotCfgKey>(bot_cfg);
    }

    client
}
