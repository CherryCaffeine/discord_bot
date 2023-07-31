use super::AppState;
use core::convert::identity as id;
use serenity::model::prelude::UserId;
use ux::u63;

pub(super) fn add_signed_exp(
    app_cache: &mut AppState,
    discord_id: UserId,
    delta: i64,
) -> Option<u63> {
    let server_member = app_cache
        .users
        .iter_mut()
        .find(|server_member| server_member.discord_id == discord_id)?;
    #[allow(clippy::cast_possible_wrap)]
    let old_exp: i64 = u64::from(server_member.exp) as i64;
    #[allow(clippy::cast_sign_loss)]
    let new_exp: u63 = u63::new(id::<i64>(old_exp + delta) as u64);
    server_member.exp = new_exp;
    Some(new_exp)
}
