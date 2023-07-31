use std::convert::identity as id;

use super::{in_cache, type_map_keys::AppStateKey, EarnedRole};
use crate::util::macros::u63_from_as_ref_user_id;
use serenity::{
    model::prelude::{Member, RoleId, UserId},
    prelude::Context,
};
use sqlx::PgPool;
use ux::u63;

use crate::db;

use super::AppState;

/// "Synchronized" way of adding experience points to a user.
///
/// "Synchronized" means that it updates both the database and the cache.
pub(crate) async fn add_signed_exp(
    app_cache: &mut AppState,
    pool: &PgPool,
    member: &Member,
    delta: i64,
) -> Result<u63, sqlx::Error> {
    let discord_id: UserId = member.user.id;
    let db_exp: i64 = db::add_signed_exp(pool, discord_id, delta).await?;
    #[allow(clippy::cast_sign_loss)]
    let db_exp: u63 = u63::new(id::<i64>(db_exp) as u64);
    let discord_id = u63_from_as_ref_user_id!(discord_id);
    let in_cache_exp = if let Some(exp) = in_cache::add_signed_exp(app_cache, discord_id, delta) {
        exp
    } else {
        eprintln!("Couldn't find the user in the cache");
        #[allow(clippy::cast_sign_loss)]
        u63::new(id::<i64>(delta) as u64)
    };

    if db_exp != in_cache_exp {
        eprintln!("The database and the cache are out of sync");
        eprintln!("db_exp: {db_exp}");
        eprintln!("in_cache_exp: {in_cache_exp}");
    }

    // TODO: find a way to grant roles efficiently.

    Ok(db_exp)
}

pub(crate) async fn add_earned_role(
    ctx: &Context,
    pool: &PgPool,
    role_id: RoleId,
    exp_needed: u63,
) -> Result<(), sqlx::Error> {
    {
        let role_id = id::<u64>(role_id.0) as i64;
        let exp_needed = u64::from(exp_needed) as i64;
        db::add_earned_role(pool, role_id, exp_needed).await?;
    }
    let mut wlock = ctx.data.write().await;
    let app_state = wlock
        .get_mut::<AppStateKey>()
        .expect("Failed to get the app cache from the typemap");
    let pos = app_state
        .sorted_earned_roles
        .binary_search_by_key(&exp_needed, |r| r.exp_needed)
        .expect_err("The role with the same exp_needed already exists");
    app_state.sorted_earned_roles.insert(
        pos,
        EarnedRole {
            role_id,
            exp_needed,
        },
    );
    Ok(())
}
