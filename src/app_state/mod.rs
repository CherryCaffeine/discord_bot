use std::convert::identity as id;

use serenity::model::prelude::{Member, RoleId, UserId};
use sqlx::PgPool;

use crate::db::{self, dao};

use self::{exp::Exp, reqd_prompts::ReqdPrompts};

pub(crate) mod exp;
mod in_cache;
mod membership;
mod reqd_prompts;
pub(crate) mod sync;
pub(crate) mod type_map_keys;

pub(crate) struct AppState {
    pub(crate) users: Vec<ServerMember>,
    pub(crate) reqd_prompts: ReqdPrompts,
    pub(crate) sorted_earned_roles: Vec<EarnedRole>,
}

/// For database operations, [`ServerMember`] is converted to [`crate::db::dao::ServerMember`].
#[derive(Debug)]
pub(crate) struct ServerMember {
    discord_id: UserId,
    exp: Exp,
    earned_role_idx: Option<usize>,
    nxt_exp_milestone: Option<Exp>,
}

pub(crate) struct EarnedRole {
    role_id: RoleId,
    exp_needed: Exp,
}

impl ServerMember {
    fn new(dao: dao::ServerMember, sorted_earned_roles: &[EarnedRole]) -> Self {
        let dao::ServerMember { discord_id, exp } = dao;

        #[allow(clippy::cast_sign_loss)]
        let discord_id: u64 = id::<i64>(discord_id) as u64;

        let discord_id = UserId(discord_id);
        let exp: Exp = Exp::from_i64(exp);

        let earned_role_idx = match sorted_earned_roles.binary_search_by_key(&exp, |r| r.exp_needed)
        {
            Ok(pos) => Some(pos),
            Err(pos) => {
                if pos == 0 {
                    None
                } else {
                    Some(pos - 1)
                }
            }
        };

        let nxt_exp_milestone = if let Some(earned_role_idx) = earned_role_idx {
            sorted_earned_roles
                .get(earned_role_idx + 1)
                .map(|r| r.exp_needed)
        } else {
            None
        };

        ServerMember {
            discord_id,
            exp,
            earned_role_idx,
            nxt_exp_milestone,
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
            exp_needed: Exp(exp),
        }
    }
}

impl AppState {
    pub(crate) async fn new(pool: &PgPool, fetched_members: Vec<Member>) -> Self {
        let db_members = db::server_members(pool).await.unwrap_or_else(|e| {
            panic!("Sqlx failure when querying the list of server members: {e}");
        });

        let sorted_earned_roles = db::sorted_earned_roles(pool)
            .await
            .unwrap_or_else(|e| {
                panic!("Sqlx failure when querying the list of earned roles: {e}");
            })
            .into_iter()
            .map(EarnedRole::from)
            .collect::<Vec<_>>();

        let diff = membership::Diff::new(db_members, fetched_members);
        let users: Vec<ServerMember> = diff
            .sync_and_distill(pool)
            .await
            .into_iter()
            .map(|m| ServerMember::new(m, &sorted_earned_roles))
            .collect();
        let reqd_prompts = ReqdPrompts::default();

        AppState {
            users,
            reqd_prompts,
            sorted_earned_roles,
        }
    }
}
