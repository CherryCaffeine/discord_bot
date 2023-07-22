use std::collections::HashSet;

use serenity::model::prelude::UserId;

// The method for configuration of the bot
// https://docs.rs/serenity/latest/serenity/framework/standard/struct.Configuration.html#method.owners
#[allow(clippy::unreadable_literal)]
pub(crate) fn owners() -> HashSet<UserId> {
    [UserId(286962466037170176)].into_iter().collect()
}
