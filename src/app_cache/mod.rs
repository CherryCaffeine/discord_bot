use serenity::model::prelude::Member;
use sqlx::PgPool;
use ux::u63;

use crate::db;

mod membership;

/// For database operations, [`ServerMember`] is converted to [`crate::db::dao::ServerMember`].
#[allow(dead_code)]
pub(crate) struct ServerMember {
    discord_id: u63,
    exp: u63,
}

#[allow(dead_code)]
pub struct AppCache {
    users: Vec<ServerMember>,
}

impl AppCache {
    pub(crate) async fn new(pool: &PgPool, fetched_members: Vec<Member>) -> Self {
        let db_members = db::server_members(pool).await.unwrap_or_else(|e| {
            panic!("Sqlx failure when querying the list of server members: {e}");
        });
        let users: Vec<ServerMember> = Vec::with_capacity(fetched_members.len());

        let diff = membership::Diff::new(db_members, fetched_members);
        diff.update_db(pool).await;

        AppCache { users }
    }
}
