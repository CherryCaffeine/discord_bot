use core::convert::identity as id;
use itertools::Itertools;
use serenity::model::prelude::{EmojiId, MessageId, RoleId};
use std::collections::HashMap;

use crate::db::dao;

#[derive(Debug)]
enum Emoji {
    Custom(EmojiId),
    BuiltIn(String),
}

// TODO: consider the structure with aggregated enum vs enum with structure variants
#[derive(Debug)]
enum SelfRoleMsgData {
    /// User can select only one role.
    ChoiceGroup(HashMap<RoleId, Emoji>),
    /// User can select arbitrary subset of the roles.
    RoleGroup(HashMap<RoleId, Emoji>),
}

#[derive(Debug)]
pub(crate) struct SelfRoleMsgs(HashMap<MessageId, SelfRoleMsgData>);

impl From<Vec<dao::SelfAssignedRole>> for SelfRoleMsgs {
    fn from(self_assigned_roles: Vec<dao::SelfAssignedRole>) -> Self {
        let mut msgs: HashMap<MessageId, SelfRoleMsgData> = HashMap::new();

        let it = self_assigned_roles.into_iter();

        let mut group_buffer = Vec::<dao::SelfAssignedRole>::with_capacity(10);
        for (msg_id, group) in &it.group_by(|a| {
            let message_id = id::<i64>(a.message_id) as u64;
            MessageId(message_id)
        }) {
            group_buffer.extend(group);
            let are_excl = group_buffer
                .iter()
                .map(|r| r.excl_role_group_id)
                .all_equal();

            let mut data: HashMap<RoleId, Emoji> = HashMap::with_capacity(10);
            let drain_it = group_buffer.drain(..).map(|r| {
                let role_id = id::<i64>(r.role_id) as u64;
                let role_id = RoleId(role_id);
                let emoji = match (r.emoji_id, r.emoji_name) {
                    (Some(emoji_id), None) => {
                        let emoji_id = id::<i64>(emoji_id) as u64;
                        let emoji_id = EmojiId(emoji_id);
                        Emoji::Custom(emoji_id)
                    }
                    (None, Some(emoji_name)) => Emoji::BuiltIn(emoji_name),
                    _ => unreachable!("Exactly one of emoji_id or emoji_name must be Some(_)"),
                };
                (role_id, emoji)
            });
            data.extend(drain_it);
            let data = if are_excl {
                SelfRoleMsgData::ChoiceGroup(data)
            } else {
                SelfRoleMsgData::RoleGroup(data)
            };

            msgs.insert(msg_id, data);
        }

        Self(msgs)
    }
}
