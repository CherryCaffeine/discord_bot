use std::convert::identity as id;

use serenity::model::prelude::{Member, RoleId, UserId};
use sqlx::PgPool;
use ux::u63;

use crate::db::{self, dao};

use self::reqd_prompts::ReqdPrompts;

mod in_cache;
mod membership;
mod reqd_prompts;
pub(crate) mod sync;
pub(crate) mod type_map_keys;

#[allow(dead_code)]
pub(crate) struct AppState {
    pub(crate) users: Vec<ServerMember>,
    pub(crate) reqd_prompts: ReqdPrompts,
    pub(crate) sorted_earned_roles: Vec<EarnedRole>,
}

/// For database operations, [`ServerMember`] is converted to [`crate::db::dao::ServerMember`].
#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct ServerMember {
    discord_id: UserId,
    exp: u63,
}

pub(crate) struct EarnedRole {
    role_id: RoleId,
    exp_needed: u63,
}

impl From<dao::ServerMember> for ServerMember {
    fn from(dao: dao::ServerMember) -> Self {
        let dao::ServerMember { discord_id, exp } = dao;

        #[allow(clippy::cast_sign_loss)]
        let discord_id: u64 = id::<i64>(discord_id) as u64;
        #[allow(clippy::cast_sign_loss)]
        let exp: u64 = id::<i64>(exp) as u64;

        ServerMember {
            discord_id: UserId(discord_id),
            exp: u63::new(exp),
        }
    }
}

impl From<dao::EarnedRole> for EarnedRole {
    fn from(value: dao::EarnedRole) -> Self {
        let dao::EarnedRole {
            role_id,
            exp_needed,
        } = value;

        #[allow(clippy::cast_sign_loss)]
        let role_id: u64 = id::<i64>(role_id) as u64;
        #[allow(clippy::cast_sign_loss)]
        let exp: u64 = id::<i64>(exp_needed) as u64;

        EarnedRole {
            role_id: RoleId(role_id),
            exp_needed: u63::new(exp),
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
        let reqd_prompts = ReqdPrompts::default();

        let sorted_earned_roles = db::sorted_earned_roles(pool)
            .await
            .unwrap_or_else(|e| {
                panic!("Sqlx failure when querying the list of earned roles: {e}");
            })
            .into_iter()
            .map(EarnedRole::from)
            .collect::<Vec<_>>();

        AppState {
            users,
            reqd_prompts,
            sorted_earned_roles,
        }
    }
}
