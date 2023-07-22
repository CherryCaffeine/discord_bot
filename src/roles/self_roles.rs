use serenity::{
    model::prelude::{MessageId, Reaction, ReactionType},
    prelude::Context,
};

use crate::Bot;

trait Handle {
    fn on(&mut self, bot: &mut Bot, ctx: &mut Context, reaction: Reaction);
    fn off(&mut self, bot: &mut Bot, ctx: &mut Context, reaction: Reaction);
}

#[allow(dead_code)]
struct Handler<F1, F2>
where
    F1: FnMut(&mut Bot, &mut Context, Reaction),
    F2: FnMut(&mut Bot, &mut Context, Reaction),
{
    f1: F1,
    f2: F2,
}

#[allow(dead_code)]
pub(crate) struct SelfRoleMsg<'a> {
    // channel_id is implicit because it is a constant.
    message_id: MessageId,
    reaction_handler_pairs: Vec<(ReactionType, &'a mut (dyn Handle))>,
}
