use super::{exp::Exp, in_cache, EarnedRole, ServerMember};
use serenity::{
    http::Http,
    model::prelude::{Member, RoleId, UserId},
};
use sqlx::PgPool;

use super::AppState;
use crate::immut_data::dynamic::BotConfig;
use crate::db;

/// "Synchronized" way of adding experience points to a user.
///
/// "Synchronized" means that it updates both the database and the cache.
pub(crate) async fn add_signed_exp(
    http: &Http,
    bot_config: &BotConfig,
    app_state: &mut AppState,
    pool: &PgPool,
    member: &Member,
    delta: i64,
) -> crate::util::Result<Exp> {
    let discord_id: UserId = member.user.id;
    let db_exp: Exp = db::add_signed_exp(pool, discord_id, delta).await?;
    let in_cache_exp = if let Some(exp) = in_cache::add_signed_exp(app_state, discord_id, delta) {
        exp
    } else {
        eprintln!("Couldn't find the user in the cache");
        Exp::from_i64(delta)
    };

    if db_exp != in_cache_exp {
        eprintln!("The database and the cache are out of sync");
        eprintln!("db_exp: {db_exp:?}");
        eprintln!("in_cache_exp: {in_cache_exp:?}");
    }

    let Some(user) = app_state
        .users
        .iter_mut()
        .find(|server_member| server_member.discord_id == discord_id)
    else {
        panic!("Couldn't find the user in the cache");
    };

    let Some(exp_milestone) = user.nxt_exp_milestone else {
        return Ok(db_exp);
    };

    if user.exp >= exp_milestone {
        let next_earned_role_idx = match user.earned_role_idx {
            Some(idx) => {
                let old_role = match app_state.sorted_earned_roles.get(idx) {
                    Some(r) => r.role_id,
                    None => unreachable!("The user has an invalid earned_role_idx"),
                };
                http.remove_member_role(bot_config.discord_server_id.0, user.discord_id.0, old_role.0, None)
                    .await?;
                idx + 1
            }
            None => 0,
        };
        let next_earned_role_id = match app_state.sorted_earned_roles.get(next_earned_role_idx) {
            Some(r) => r.role_id,
            None => unreachable!("The user has an invalid earned_role_idx"),
        };
        user.earned_role_idx = Some(next_earned_role_idx);
        user.nxt_exp_milestone = app_state
            .sorted_earned_roles
            .get(next_earned_role_idx)
            .map(|r| r.exp_needed);
        http.add_member_role(
            bot_config.discord_server_id.0,
            user.discord_id.0,
            next_earned_role_id.0,
            None,
        )
        .await?;
    }

    Ok(db_exp)
}

pub(crate) async fn add_earned_role(
    http: &Http,
    bot_config: &BotConfig,
    sorted_earned_roles: &mut Vec<EarnedRole>,
    users: &mut [ServerMember],
    pool: &PgPool,
    role_id: RoleId,
    exp_needed: Exp,
) -> crate::util::Result<()> {
    db::add_earned_role(pool, role_id, exp_needed).await?;
    let pos = sorted_earned_roles
        .binary_search_by_key(&exp_needed, |r| r.exp_needed)
        .expect_err("The role with the same exp_needed already exists");
    sorted_earned_roles.insert(
        pos,
        EarnedRole {
            role_id,
            exp_needed,
        },
    );
    let sm_iter = users.iter_mut();
    if sorted_earned_roles.len() == 1 {
        for sm in sm_iter {
            if sm.exp > exp_needed {
                sm.earned_role_idx = Some(0);
                sm.nxt_exp_milestone = None;
                http.add_member_role(bot_config.discord_server_id.0, sm.discord_id.0, role_id.0, None)
                    .await?;
            } else {
                sm.earned_role_idx = None;
                sm.nxt_exp_milestone = Some(exp_needed);
            }
        }
    } else if sorted_earned_roles.len() - 1 == pos {
        let old_last_idx = pos - 1;
        for sm in sm_iter {
            let Some(old_earned_role_idx) = sm.earned_role_idx else {
                continue;
            };
            if old_earned_role_idx != old_last_idx {
                continue;
            }
            if sm.exp < exp_needed {
                sm.nxt_exp_milestone = Some(exp_needed);
            } else {
                sm.earned_role_idx = Some(pos - 1);
                sm.nxt_exp_milestone = None;
                let old_role_id = match sorted_earned_roles.get(old_earned_role_idx) {
                    Some(r) => r.role_id,
                    None => unreachable!("The user has an invalid earned_role_idx"),
                };
                http.remove_member_role(bot_config.discord_server_id.0, sm.discord_id.0, old_role_id.0, None)
                    .await?;
                http.add_member_role(bot_config.discord_server_id.0, sm.discord_id.0, role_id.0, None)
                    .await?;
            }
        }
    } else {
        for ServerMember {
            ref discord_id,
            earned_role_idx,
            nxt_exp_milestone,
            ref exp,
            ..
        } in sm_iter
        {
            match (earned_role_idx, nxt_exp_milestone) {
                (Some(idx), _m) if *idx > pos => {
                    *idx += 1;
                    // the milestone should be the same
                }
                (Some(idx), m) if *idx == pos && *exp < exp_needed => {
                    *m = Some(exp_needed);
                }
                (Some(idx), _m) if *idx == pos && *exp >= exp_needed => {
                    let old_role: RoleId = sorted_earned_roles[*idx].role_id;
                    *idx += 1;
                    let new_role: RoleId = sorted_earned_roles[*idx].role_id;
                    http.add_member_role(bot_config.discord_server_id.0, discord_id.0, new_role.0, None).await
                        .unwrap_or_else(|e| panic!("Failed to give a role to a server member with discord_id={discord_id}: {e}"));
                    http.remove_member_role(bot_config.discord_server_id.0, discord_id.0, old_role.0, None).await
                        .unwrap_or_else(|e| panic!("Failed to remove a role from a server member with discord_id: {discord_id}: {e}"));
                }
                _ => (),
            }
        }
    }
    Ok(())
}
