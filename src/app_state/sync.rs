use std::convert::identity as id;

use super::in_cache;
use crate::util::macros::u63_from_as_ref_user_id;
use serenity::model::prelude::UserId;
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
    discord_id: impl AsRef<UserId>,
    delta: i64,
) -> Result<u63, sqlx::Error> {
    let discord_id: &UserId = discord_id.as_ref();
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

    Ok(db_exp)
}
