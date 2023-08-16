use std::ops::ControlFlow;

use serenity::prelude::Context;

use crate::app_state;
use crate::{bots::MainBot, commands::role::EarnedRolePromptReq};
use serenity::model::prelude::Message;

#[derive(Default)]
pub(crate) struct ReqdPrompts {
    pub(crate) earned_role: Vec<EarnedRolePromptReq>,
}

impl ReqdPrompts {
    /// Handles the input request if it is pending.
    /// Returns `ControlFlow::Break(())` if the request was handled
    /// or `ControlFlow::Continue` if it was not pending.
    pub(crate) async fn handle_if_pending(
        &mut self,
        bot: &MainBot,
        ctx: &Context,
        msg: &Message,
        sorted_earned_roles: &mut Vec<app_state::EarnedRole>,
        users: &mut Vec<app_state::ServerMember>,
    ) -> ControlFlow<()> {
        EarnedRolePromptReq::handle_if_pending(bot, ctx, msg, sorted_earned_roles, self, users)
            .await?;
        ControlFlow::Continue(())
    }
}
