//! Live arena access shares the ordinary constant evaluator, including Support's
//! snapshotted programs. Grants never depend on the donor remaining in play.
use super::*;
use crate::ability_ir::Effect;
use super::generic_runtime::ConstantScope;
impl Engine {
    pub(super) fn cross_arena_permission(&self,s:&State,id:CardId,permission:&str)->bool {
        self.constant_effects_matching(s,id,ConstantScope::AttackPermission).iter().any(|(_,e)|
            matches!(e,Effect::AttackPermission{permission:p,duration,..} if p==permission && duration=="persistent"))
    }
}
