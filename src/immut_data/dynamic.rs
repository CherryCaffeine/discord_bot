use std::collections::HashSet;

use once_cell::sync::Lazy;
use regex::Regex;
use serenity::model::prelude::UserId;

// The method for configuration of the bot
// https://docs.rs/serenity/latest/serenity/framework/standard/struct.Configuration.html#method.owners
#[allow(clippy::unreadable_literal)]
pub(crate) fn owners() -> HashSet<UserId> {
    [UserId(286962466037170176)].into_iter().collect()
}

pub(crate) static WHITESPACE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[\s\n\r\t]+").unwrap());
