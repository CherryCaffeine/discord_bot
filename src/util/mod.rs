use rand::seq::SliceRandom;
use serenity::{
    http::{CacheHttp, Http},
    model::prelude::{ChannelId, Member, UserId},
    prelude::Mentionable,
    utils::MessageBuilder,
};

use crate::immut_data::consts::DISCORD_SERVER_ID;

pub(crate) mod macros;

pub(super) async fn members(http: impl AsRef<Http>) -> Vec<Member> {
    const DEFAULT_LIMIT: usize = 1000;
    const USE_DEFAULT_LIMIT: Option<u64> = None;
    const NO_USER_ID_OFFSET: Option<UserId> = None;

    let members = DISCORD_SERVER_ID
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
