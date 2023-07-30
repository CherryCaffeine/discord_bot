use crate::commands::role::EarnedRolePromptReq;

#[derive(Default)]
pub(crate) struct ReqdPrompts {
    pub(crate) earned_role: Vec<EarnedRolePromptReq>,
}
