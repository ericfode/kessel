//! Candidate inventory/type matching. A native selector retains its literal zone
//! list (including order and repeated zones); candidate predicates see the bound
//! physical card while source/player context remains that of the initiating ability.
use super::*;
use crate::ability_ir::selector_contract::{CardTypeFilter, NativeCardSelection, SelectorZone};
use crate::ability_ir::TargetResolver;
impl Engine {
    pub(super) fn selector_card_type(&self, s: &State, id: CardId) -> CardTypeFilter {
        use CardTypeFilter::*;
        let info = &self.def(s, id).info;
        if info.is("leader") {
            if s.card(id).parent != 0 {
                LeaderUpgrade
            } else if s.card(id).deployed {
                if info.is("token") {
                    TokenLeaderUnit
                } else {
                    NonTokenLeaderUnit
                }
            } else {
                Leader
            }
        } else if info.is("unit") {
            if s.card(id).parent != 0 {
                NonLeaderUnitUpgrade
            } else if info.is("token") {
                TokenUnit
            } else {
                BasicUnit
            }
        } else if info.is("upgrade") {
            if info.is("token") {
                TokenUpgrade
            } else {
                BasicUpgrade
            }
        } else if info.is("event") {
            Event
        } else if info.is("base") {
            Base
        } else {
            TokenCard
        }
    }
    pub(super) fn selector_type_matches(&self, s: &State, id: CardId, name: &str) -> bool {
        CardTypeFilter::parse(name)
            .is_ok_and(|filter| filter.matches(self.selector_card_type(s, id)))
    }
    pub(super) fn native_selector_candidates(
        &self,
        s: &State,
        source: CardId,
        r: &TargetResolver,
        native: &NativeCardSelection,
    ) -> Vec<CardId> {
        let types = native.types().expect("validated native cardTypes");
        let zones = native.zone_filters().expect("validated native zones");
        let owner = s.card(source).owner;
        let mut out = Vec::new();
        // BaseCardSelector.findPossibleCards partitions context.player before
        // opponent, independently of the seat currently choosing a target.
        for seat in [owner, owner.other()] {
            if r.owner == "self" && seat != owner || r.owner == "opponent" && seat == owner {
                continue;
            }
            let p = s.player(seat);
            for zone in &zones {
                let mut candidates = Vec::new();
                if zone.includes_ground() {
                    candidates.extend_from_slice(p.ground.as_slice());
                }
                if zone.includes_space() {
                    candidates.extend_from_slice(p.space.as_slice());
                }
                if zone.includes_base() {
                    candidates.push(p.base);
                    if p.leader != 0 && !s.card(p.leader).deployed {
                        candidates.push(p.leader);
                    }
                }
                if *zone == SelectorZone::Discard {
                    candidates.extend_from_slice(p.discard.as_slice());
                }
                for id in candidates {
                    if types
                        .iter()
                        .any(|t| t.matches(self.selector_card_type(s, id)))
                    {
                        out.push(id);
                    }
                }
            }
        }
        out
    }
    pub(super) fn exhaustible_card(&self, s: &State, id: CardId) -> bool {
        self.in_arena(s, id)
            || (id != 0
                && self.def(s, id).info.is("leader")
                && id == s.player(s.card(id).owner).leader
                && !s.card(id).deployed)
    }
}
