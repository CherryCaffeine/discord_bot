//! Module for Data Acess Objects

use sqlx::FromRow;

#[derive(FromRow)]
#[allow(dead_code)]
pub(crate) struct User {
    discord_id: i64,
    exp: i64,
    on_server: bool,
}

/// Data Access Object for [`crate::app_cache::ServerMember`].
#[derive(FromRow, Debug)]
#[allow(dead_code)]
pub(crate) struct ServerMember {
    pub(crate) discord_id: i64,
    exp: i64,
}
