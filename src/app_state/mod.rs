use std::convert::identity as id;

use serenity::model::prelude::Member;
use sqlx::PgPool;
use ux::u63;

use crate::db::{self, dao};

mod in_cache;
mod membership;
pub(crate) mod sync;
pub(crate) mod type_map_keys;

#[allow(dead_code)]
#[derive(Debug)]
pub struct AppState {
    users: Vec<ServerMember>,
}

/// For database operations, [`ServerMember`] is converted to [`crate::db::dao::ServerMember`].
#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct ServerMember {
    discord_id: u63,
    exp: u63,
}

impl From<dao::ServerMember> for ServerMember {
    fn from(dao: dao::ServerMember) -> Self {
        let dao::ServerMember { discord_id, exp } = dao;

        #[allow(clippy::cast_sign_loss)]
        let discord_id: u64 = id::<i64>(discord_id) as u64;
        #[allow(clippy::cast_sign_loss)]
        let exp: u64 = id::<i64>(exp) as u64;

        ServerMember {
            discord_id: u63::new(discord_id),
            exp: u63::new(exp),
        }
    }
}

impl AppState {
    pub(crate) async fn new(pool: &PgPool, fetched_members: Vec<Member>) -> Self {
        let db_members = db::server_members(pool).await.unwrap_or_else(|e| {
            panic!("Sqlx failure when querying the list of server members: {e}");
        });

        let diff = membership::Diff::new(db_members, fetched_members);
        let users: Vec<ServerMember> = diff
            .sync_and_distill(pool)
            .await
            .into_iter()
            .map(ServerMember::from)
            .collect();

        AppState { users }
    }
}
