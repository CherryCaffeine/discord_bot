//! Module for Data Acess Objects

use sqlx::FromRow;

#[derive(FromRow)]
#[allow(dead_code)]
pub(crate) struct User {
    discord_id: i64,
    exp: i64,
    on_server: bool,
}

/// Data Access Object for [`crate::app_state::ServerMember`].
// The struct is Copy coincidentally, but it's not a requirement.
#[derive(FromRow, Debug, Clone, Copy)]
pub(crate) struct ServerMember {
    pub(crate) discord_id: i64,
    pub(crate) exp: i64,
}

#[derive(FromRow)]
pub(crate) struct EarnedRole {
    pub(crate) role_id: i64,
    pub(crate) exp_needed: i64,
}

#[derive(FromRow)]
pub(crate) struct SelfAssignedRole {
    pub(crate) excl_role_group_id: i64,
    pub(crate) role_id: i64,
    pub(crate) message_id: i64,
    pub(crate) emoji_id: Option<i64>,
    pub(crate) emoji_name: Option<String>,
}
