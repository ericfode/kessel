//! SWU policy at the graph-search boundary.
//!
//! Search owns proof budgets, cycles and transpositions. This adapter owns SWU
//! action ordering, uncommitted UI cancellation and pessimistic regroup ordering.
//! Regroup commits the prover's resource choice first, allowing the defender to
//! observe it before choosing a reply. Legal transitions remain owned by Engine.
use crate::{search::Game, Action, Engine, Seat, State};

impl Game for Engine {
    type Position = State;
    type Move = Action;
    fn actor(&self, s: &State) -> Option<Seat> {
        s.actor()
    }
    fn terminal(&self, s: &State) -> bool {
        s.is_terminal()
    }
    fn winner(&self, s: &State) -> Option<Seat> {
        s.winner()
    }
    fn window(&self, s: &State) -> bool {
        s.is_action_window()
    }
    fn hash(&self, s: &State) -> u64 {
        s.zobrist()
    }
    fn order_regroup(&self, s: &mut State, p: Seat) {
        Engine::order_regroup(self, s, p);
    }
    fn actions(&self, s: &State) -> Vec<Action> {
        let mut actions = self.legal_actions(s);
        // Reordering only: no replies or targets are capped. Base attacks and
        // existing attackers are typically the shortest endgame proofs.
        actions.sort_by_key(|action| {
            if let Some(id) = action.id() {
                if s.players.iter().any(|p| p.base == id) {
                    return 0;
                }
                if s.players
                    .iter()
                    .any(|p| p.ground.contains(id) || p.space.contains(id))
                {
                    return 1;
                }
                if s.players.iter().any(|p| p.leader == id) {
                    return 2;
                }
                return 3;
            }
            if action.label == "Pass" {
                9
            } else {
                8
            }
        });
        actions
    }
    fn cancels_uncommitted(&self, s: &State, a: &Action) -> bool {
        use crate::model::Decision;
        a.kind == "button"
            && a.label == "Cancel"
            && match s.decision {
                Decision::LeaderMenu | Decision::NoEffect | Decision::AbilityMenu => true,
                Decision::AbilityNoEffect | Decision::AbilityTarget => {
                    !s.ability_committed && s.ability_return == 0
                }
                Decision::AttackTarget => !s.ambush,
                Decision::VaderBase => s.trigger_context == 0,
                _ => false,
            }
    }
    fn apply(&self, s: &State, a: &Action) -> Result<State, String> {
        Engine::apply(self, s, a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiled_ability_cancellation_is_skipped_only_before_commitment() {
        use crate::model::Decision;
        let pool: serde_json::Value =
            serde_json::from_str(include_str!("../data/pool.json")).unwrap();
        let engine = Engine::from_pool_value(&pool).unwrap();
        let cancel = Action::button("Cancel", "cancel");
        let mut state = State::default();
        for decision in [
            Decision::AbilityMenu,
            Decision::AbilityTarget,
            Decision::AbilityNoEffect,
        ] {
            state.decision = decision;
            assert!(engine.cancels_uncommitted(&state, &cancel));
        }
        state.decision = Decision::AbilityTarget;
        state.ability_committed = true;
        assert!(!engine.cancels_uncommitted(&state, &cancel));
        state.ability_committed = false;
        state.ability_return = 1;
        assert!(!engine.cancels_uncommitted(&state, &cancel));
        state.decision = Decision::AbilityOptional;
        assert!(!engine.cancels_uncommitted(&state, &Action::button("Pass", "pass")));
    }

    #[test]
    fn attack_cancellation_respects_ambush_and_trigger_commitment() {
        use crate::model::Decision;
        let engine = Engine::from_pool_value(
            &serde_json::from_str(include_str!("../data/pool.json")).unwrap(),
        )
        .unwrap();
        let cancel = Action::button("Cancel", "cancel");
        for (decision, ambush, trigger_context, expected) in [
            (Decision::AttackTarget, false, 0, true),
            (Decision::AttackTarget, true, 0, false),
            (Decision::VaderBase, false, 0, true),
            (Decision::VaderBase, false, 1, false),
            (Decision::Window, false, 0, false),
        ] {
            let state = State {
                decision,
                ambush,
                trigger_context,
                ..State::default()
            };
            assert_eq!(engine.cancels_uncommitted(&state, &cancel), expected);
        }
    }
}
