use super::{exp::Exp, AppState};
use serenity::model::prelude::UserId;

pub(super) fn add_signed_exp(
    app_state: &mut AppState,
    discord_id: UserId,
    delta: i64,
) -> Option<Exp> {
    let server_member = app_state
        .users
        .iter_mut()
        .find(|server_member| server_member.discord_id == discord_id)?;
    let old_exp: i64 = server_member.exp.to_i64();
    let new_exp: Exp = Exp::from_i64(old_exp + delta);
    server_member.exp = new_exp;
    Some(new_exp)
}
