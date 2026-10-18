//! Complete bounded All-mode Disclose registrations; unknown companion semantics fail closed.
use crate::ability_ir::{Ability, Effect, Expr, Sides, Target};
use serde_json::{json, Value};
pub(crate) fn has_disclose(a: &Ability) -> bool {
    fn effect(e: &Effect) -> bool {
        match e {
            Effect::Disclose { .. } => true,
            Effect::Simultaneous { effects }
            | Effect::Sequence { effects }
            | Effect::PhaseModifiers { effects } => effects.iter().any(effect),
            Effect::Conditional {
                on_true, on_false, ..
            } => effect(on_true) || on_false.as_deref().is_some_and(effect),
            Effect::Select { effect: e, .. } => effect(e),
            _ => false,
        }
    }
    effect(&a.effect)
        || a.then.as_deref().is_some_and(has_disclose)
        || a.if_you_do.as_deref().is_some_and(has_disclose)
        || a.target_resolvers.iter().any(|r| effect(&r.effect))
}
// Complete bounded family gate: raw registrations and catalogue source envelopes
// are both checked before instruction lowering. Continuations retain the existing
// scalar/group mutation owners; no card identity grants runtime capability.
pub(crate) fn registration(a: &Ability) -> bool {
    fn header(a: &Ability) -> bool {
        !a.attached_host
            && !a.piloting_ability
            && !a.pilot_unit_ability
            && a.if_you_do_condition.is_none()
            && a.hooks.is_empty()
            && a.gained_host.is_none()
            && !a.attacker_must_survive
            && !a.optional
            && !a.requires_confirmation
            && !a.coordinate
            && a.costs.is_empty()
            && a.context_title.is_empty()
            && a.condition.is_none()
            && a.attached_context.is_none()
            && a.target_resolvers.is_empty()
    }
    fn n(e: &Expr, value: u64) -> bool {
        matches!(e,Expr::Const{value:v} if v.as_u64()==Some(value))
    }
    fn target(t: &Target, name: &str) -> bool {
        matches!(t,Target::Name(n) if n==name)
    }
    fn spy(e: &Effect, amount: u64, ready: Option<bool>) -> bool {
        matches!(e,Effect::CreateTokenUnit{token_type,amount:n,target:t,enters_ready} if token_type=="spy"&&disclose_count(n,amount)&&target(t,"ownPlayer")&&ready.is_none_or(|r|r==*enters_ready))
    }
    fn disclose(e: &Effect) -> bool {
        matches!(e,Effect::Disclose{target:t,aspects,mode,aspect_multiplicity,selected_cards_event_field,triggered_no_match_policy,selection_context} if *aspect_multiplicity&&selected_cards_event_field=="disclosedCards"&&triggered_no_match_policy=="maskWithPauseWithoutEvent"&&selection_context=="targetPlayer"&& target(t,"ownPlayer")&&mode=="all"&&(1..=3).contains(&aspects.len())&&aspects.iter().all(|a|["vigilance","command","aggression","cunning","heroism","villainy"].contains(&a.as_str())))
    }
    fn leaf(a: &Ability) -> bool {
        if !header(a) || !a.trigger.is_empty() || a.then.is_some() || a.if_you_do.is_some() {
            return false;
        }
        let r = a.target_resolver.as_ref();
        let selected = |kind: &str, owners: &[&str]| {
            r.is_some_and(|r| {
                owners.contains(&r.owner.as_str())
                    && r.card_type == kind
                    && r.zone.as_deref() == Some("anyAttackable")
                    && !r.optional
                    && r.mode.is_none()
                    && r.count.is_none()
                    && r.arena.is_none()
                    && r.prompt_title.is_none()
                    && !r.player_selection
                    && r.native.is_none()
                    && r.prompt_title_parts.is_empty()
                    && r.predicate.is_none()
            })
        };
        match &a.effect {
            Effect::Draw{target:t,amount}=>r.is_none()&&target(t,"ownPlayer")&&n(amount,1),
            Effect::Heal{target:t,amount}=>r.is_none()&&target(t,"ownBase")&&n(amount,1),
            Effect::ReadyResources{target:t,amount}=>r.is_none()&&target(t,"ownPlayer")&&n(amount,2),
            Effect::DiscardOwnHand{target:t,amount}=>r.is_none()&&target(t,"enemyPlayer")&&n(amount,2),
            Effect::GiveExperience{target:t,amount}=>selected("unit",&["any"])&&target(t,"chosen")&&n(amount,1),
            Effect::Damage{target:t,amount}=>selected("unit",&["any"])&&target(t,"chosen")&&n(amount,5),
            Effect::GainKeyword{target:t,keyword,amount,duration}=>selected("unit",&["any","self"])&&target(t,"chosen")&&keyword=="sentinel"&&n(amount,1)&&duration=="phase",
            Effect::Defeat{target:t}=>target(t,"chosen")&&r.is_some_and(|r|r.owner=="any"&&r.card_type=="nonLeaderUnit"&&r.zone.as_deref()==Some("anyAttackable")&&!r.optional&&r.mode.is_none()&&r.count.is_none()&&r.arena.is_none()&&r.prompt_title.is_none()&&!r.player_selection&&r.native.is_none()&&r.prompt_title_parts.is_empty()&&r.predicate.as_ref().and_then(|p|serde_json::to_value(p).ok())==Some(serde_json::json!({"kind":"binary","op":"&&","left":{"kind":"query","name":"isNonLeaderUnit","receiver":{"kind":"ref","path":["card"]},"args":[]},"right":{"kind":"binary","op":">","left":{"kind":"ref","path":["card","damage"]},"right":{"kind":"const","value":0}}}))),
            e=>r.is_none()&&(spy(e,1,None)||spy(e,3,Some(false))),
        }
    }
    fn step(a: &Ability) -> bool {
        header(a)
            && a.target_resolver.is_none()
            && a.then.is_none()
            && disclose(&a.effect)
            && a.if_you_do.as_deref().is_some_and(leaf)
    }
    if !header(a) || a.target_resolver.is_some() {
        return false;
    }
    ([
        "whenPlayed",
        "whenDefeated",
        "onAttack",
        "onDefense",
        "event",
    ]
    .contains(&a.trigger.as_str())
        && step(a))
        || (a.trigger == "event"
            && a.if_you_do.is_none()
            && spy(&a.effect, 1, Some(false))
            && a.then.as_deref().is_some_and(|next| {
                next.trigger.is_empty()
                    && step(next)
                    && next
                        .if_you_do
                        .as_ref()
                        .is_some_and(|leaf| spy(&leaf.effect, 1, Some(false)))
            }))
}
fn disclose_count(e: &Expr, value: u64) -> bool {
    matches!(e,Expr::Const{value:v} if v.as_u64()==Some(value))
}

pub(crate) fn source_entry(entry: &Value, sides: &Sides) -> bool {
    let unit = entry["baseClass"] == "NonLeaderUnitCard"
        && entry["card"]["types"] == json!(["unit"])
        && (1..=2).contains(&sides.card.len())
        && sides.card.iter().all(|a| {
            ["whenPlayed", "whenDefeated", "onAttack", "onDefense"].contains(&a.trigger.as_str())
                && registration(a)
        });
    let event = entry["baseClass"] == "EventCard"
        && entry["card"]["types"] == json!(["event"])
        && sides.card.len() == 1
        && sides.card[0].trigger == "event"
        && registration(&sides.card[0]);
    let stats = if unit {
        ["cost", "hp", "power"]
            .iter()
            .all(|k| entry["card"][*k].as_u64().is_some_and(|n| n <= 1000))
            && entry["card"]["hp"].as_u64().is_some_and(|n| n > 0)
            && ["ground", "space"]
                .iter()
                .any(|a| entry["card"]["arena"] == *a)
    } else {
        entry["card"]["cost"].as_u64().is_some_and(|n| n <= 1000)
            && ["hp", "power", "arena"]
                .iter()
                .all(|k| entry["card"][*k].is_null())
    };
    (unit || event)
        && stats
        && crate::admission::SourceContextPolicy::Closed.accepts(entry)
        && sides.leader.is_empty()
        && sides.unit.is_empty()
        && entry["card"]["unique"].is_boolean()
        && entry["attachCondition"].is_null()
        && entry["piloting"].is_null()
        && [
            "pilotText",
            "pilotBox",
            "pilotingBox",
            "deployBox",
            "deployCost",
            "upgradePower",
            "upgradeHp",
        ]
        .iter()
        .all(|k| entry["card"][*k].is_null())
        && (entry["card"]["epicAction"].is_null() || entry["card"]["epicAction"] == "")
}
