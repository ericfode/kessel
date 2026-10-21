//! Typed predicate projection only. Producers must supply the engine event's
//! captured facts and resolved/cancelled outcome; this layer never reconstructs
//! event.willDefeat from later wounds, HP, phase or active attack context.
use crate::round_limit_contract::Registration;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Event {
    PlayedUpgrade {
        player: u8,
        card_is_upgrade: bool,
        attach_target: u16,
    },
    UnitDamage {
        controller: u8,
        card_is_unit: bool,
        will_defeat: bool,
    },
    AttackDeclared {
        attacker_controller: u8,
        ground_attacker: bool,
        any_base_target: bool,
    },
    Defeated {
        card: u16,
        controller: u8,
        was_unit: bool,
        unique: bool,
        had_weakness: bool,
    },
}
/// All five predicates are evaluated in the captured registration's player and
/// physical source context. Event cancellation is a separate window filter.
pub(crate) fn qualifies(
    registration: Registration,
    source: u16,
    player: u8,
    event: Event,
    resolved: bool,
) -> Result<bool, String> {
    if source == 0 || source > 255 || player > 1 {
        return Err("invalid round-limit registration event context".into());
    }
    let actor = match event {
        Event::PlayedUpgrade { player, .. } => player,
        Event::UnitDamage { controller, .. } | Event::Defeated { controller, .. } => controller,
        Event::AttackDeclared {
            attacker_controller,
            ..
        } => attacker_controller,
    };
    if actor > 1 {
        return Err("invalid captured round-limit event player".into());
    }
    if !resolved {
        return Ok(false);
    }
    Ok(match (registration, event) {
        (
            Registration::PlayedUpgradeOnSelf,
            Event::PlayedUpgrade {
                player: played_by,
                card_is_upgrade,
                attach_target,
            },
        ) => played_by == player && card_is_upgrade && attach_target == source,
        (
            Registration::FriendlyUnitNonlethalDamage,
            Event::UnitDamage {
                controller,
                card_is_unit,
                will_defeat,
            },
        ) => !will_defeat && card_is_unit && controller == player,
        (
            Registration::EnemyGroundAttackAtBase,
            Event::AttackDeclared {
                attacker_controller,
                ground_attacker,
                any_base_target,
            },
        ) => attacker_controller != player && ground_attacker && any_base_target,
        (
            Registration::EnemyWeaknessUnitDefeated,
            Event::Defeated {
                controller,
                was_unit,
                had_weakness,
                ..
            },
        ) => was_unit && controller != player && had_weakness,
        (
            Registration::OtherUniqueUnitDefeated,
            Event::Defeated {
                card,
                was_unit,
                unique,
                ..
            },
        ) => was_unit && unique && card != source,
        _ => false,
    })
}

/// Every persistent context that can resume a resolver contributes a root.
/// Locals must publish their handles into these slots before a collection call.
/// Full State snapshots are facts-only: they never restore an older ledger.
pub(crate) fn roots(s:&crate::model::State)->Vec<crate::round_limit_state::Handle> {
    use crate::round_limit_state::Handle;
    let mut out=Vec::new();let mut push=|h:Handle|{if h!=Handle::NONE {out.push(h);}};
    push(s.round_limit);push(s.support.context.round_limit);
    for w in [&s.death_pending,&s.death_window].into_iter().chain(s.death_frames.iter().map(|f|&f.window)) {for e in &w.entries {push(e.round_limit);}}
    for f in &s.death_frames {push(f.context.round_limit);}
    out
}
pub(crate) fn collect(s:&mut crate::model::State)->Result<(),String> {
    s.round_limits.collect(&roots(s))
}
pub(crate) fn native(s:&mut crate::model::State,source:crate::model::CardId,registration:Registration)->Result<crate::round_limit_state::Handle,String> {
    collect(s)?;
    s.round_limits.native(source,registration.ordinal())
}
