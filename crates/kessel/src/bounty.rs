//! The Bounty keyword family: a keyword-owned triggered ability, not an ordinary
//! death trigger.
//!
//! Pinned engine contract (`abilities/keyword/BountyAbility.ts`,
//! `core/card/propertyMixins/UnitProperties.ts`):
//! - `UnitProperties.addBountyAbility` binds one printed `Bounty` keyword instance to
//!   the card implementation's ability properties. A printed instance without a
//!   definition, or a second definition for the same printed instance, is a
//!   `Contract.fail`: the card source itself is invalid. The runtime therefore
//!   requires exactly one compiled registration iff the printed text has exactly one
//!   own-line `Bounty` keyword, and rejects both halves of a disagreement.
//! - The registration listens for `onCardDefeated`/`onCardCaptured` of the source,
//!   is controlled by `RelativePlayer.Opponent` (the opponent of the source's
//!   controller collects it), is optional by default (7.5.13.E) unless the card
//!   overrides that, and sets `canResolveWithoutLegalTargets` with
//!   `hasAnyLegalEffects() === true`, so the trigger window always offers it and
//!   resolving it always collects the bounty.
//!
//! The metadata object below is emitted by `compiler/bounty-registration.js`; it is
//! validated exactly so a future compiler change is an explicit rejection instead of
//! a silently different rule. `keywordValues` never contains `bounty`: the compiler's
//! printed-keyword parse (`compiler/build.js`) omits ability-definition keywords, so
//! the printed-text rule mirrors the engine's own
//! `/(?:^|(?:\n))Bounty/` parser (`core/ability/KeywordHelpers.ts`).
use crate::ability_ir::{Ability, Effect, Sides, Target, TargetResolver};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Both events the engine registers. The Kessel runtime subset has no card-capture
/// mechanic (`Effect` has no capture variant), so the capture half is unreachable
/// there; the metadata still requires both so a future capture implementation cannot
/// reuse a half registration.
pub const EVENTS: [&str; 2] = ["onCardDefeated", "onCardCaptured"];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Metadata {
    pub events: Vec<String>,
    pub event_card: String,
    pub can_be_triggered_by: String,
    pub context_player: String,
    pub zone: String,
    pub can_resolve_without_legal_targets: bool,
    pub emits_collected_event: String,
}

impl Metadata {
    /// The single reviewed registration shape emitted by the compiler.
    pub fn reviewed(&self) -> bool {
        self.events.iter().map(String::as_str).eq(EVENTS.iter().copied())
            && self.event_card == "source"
            && self.can_be_triggered_by == "opponent"
            && self.context_player == "collector"
            && self.zone == "anyArena"
            && self.can_resolve_without_legal_targets
            && self.emits_collected_event == "onBountyCollected"
    }
}

/// Own-line printed `Bounty` instances, exactly the engine's keyword parser rule.
/// Leaders print keywords on the deploy side, so both sides count there.
pub(crate) fn printed_count(entry: &Value) -> usize {
    let card = &entry["card"];
    let mut texts = vec![card["text"].as_str().unwrap_or("")];
    if card["types"].as_array().is_some_and(|types| types.iter().any(|kind| kind == "leader")) {
        texts.push(card["deployBox"].as_str().unwrap_or(""));
    }
    texts
        .into_iter()
        .flat_map(|text| text.split('\n'))
        .filter(|line| line.starts_with("Bounty"))
        .count()
}

fn any_marker(entry: &Value) -> bool {
    crate::ability_ir::contains(&entry["abilities"], |value| {
        value["trigger"] == "bounty" || value.get("bounty").is_some()
    }) || printed_count(entry) > 0
}

/// Whether the entry belongs to the Bounty family gate. Mention-only providers
/// (`card.keywords` lists `bounty` with no own-line text and no registration) are
/// metadata, not family members.
pub(crate) fn marker(entry: &Value) -> bool {
    any_marker(entry)
}

fn unit_resolver(resolver: &TargetResolver) -> bool {
    resolver.owner == "any"
        && resolver.card_type == "unit"
        && resolver.zone.as_deref() == Some("anyAttackable")
        && resolver.predicate.is_none()
        && !resolver.optional
        && resolver.mode.is_none()
        && resolver.count.is_none()
        && resolver.arena.is_none()
        && resolver.prompt_title.is_none()
        && resolver.native.is_none()
}

/// Bounded effect shapes with an established witness in the admitted corpus. The
/// collector is the ability's context player, so "own"/"enemy" targets and resolver
/// ownership are player-relative, never relative to the defeated card's controller.
fn effect_shape(a: &Ability) -> bool {
    let selected = |target: &Target, resolver: Option<&TargetResolver>| {
        matches!(target, Target::Name(name) if name == "chosen") && resolver.is_some_and(unit_resolver)
    };
    let unselected = |target: &Target| a.target_resolver.is_none() && matches!(target, Target::Name(name) if name == "ownPlayer" || name == "ownTopDeck");
    match &a.effect {
        Effect::Draw { target, .. } => unselected(target),
        Effect::ReadyResources { target, .. } => unselected(target),
        Effect::ResourceCard { target } => unselected(target),
        Effect::Damage { target, .. } | Effect::Exhaust { target } => selected(target, a.target_resolver.as_ref()),
        _ => false,
    }
}

/// One reviewed Bounty registration. No card-ID exceptions: every printed Bounty
/// card must fit this bounded contract or stay an explicit rejection.
pub(crate) fn registration(a: &Ability) -> bool {
    a.trigger == "bounty"
        && a.bounty.as_ref().is_some_and(Metadata::reviewed)
        && a.hooks.is_empty()
        && a.gained_host.is_none()
        && a.attached_context.is_none()
        && !a.attached_host
        && !a.attacker_must_survive
        && !a.coordinate
        && !a.requires_confirmation
        && a.costs.is_empty()
        && a.condition.is_none()
        && a.context_title.is_empty()
        && a.target_resolvers.is_empty()
        && a.then.is_none()
        && a.if_you_do.is_none()
        && a
            .title
            .strip_prefix("Collect Bounty: ")
            .is_some_and(|title| !title.trim().is_empty())
        && effect_shape(a)
}

/// Whole-entry invariants. `compile` enforces the ability header; this gate binds it
/// to the printed source and rejects both metadata/program disagreements.
pub(crate) fn entry(entry: &Value, sides: &Sides) -> bool {
    if !any_marker(entry) {
        return true;
    }
    let bounties = sides
        .leader
        .iter()
        .chain(&sides.unit)
        .chain(&sides.card)
        .filter(|ability| ability.trigger == "bounty" || ability.bounty.is_some())
        .count();
    let printed = printed_count(entry);
    if printed > 1 || bounties != printed {
        return false;
    }
    if printed == 0 {
        return bounties == 0;
    }
    // A printed instance requires exactly one registration on an ordinary unit's
    // card side (other ordinary abilities, e.g. Val's When Defeated, share that side),
    // with the provider's keyword list agreeing that Bounty is intrinsic.
    let bounty = sides.card.iter().find(|ability| ability.trigger == "bounty" || ability.bounty.is_some());
    bounties == 1
        && sides.leader.is_empty()
        && sides.unit.is_empty()
        && bounty.is_some_and(registration)
        && entry["baseClass"] == "NonLeaderUnitCard"
        && entry["card"]["types"] == json!(["unit"])
        && entry["card"]["unique"].is_boolean()
        && entry["keywordValues"].get("bounty").is_none()
        && entry["card"]["keywords"]
            .as_array()
            .is_some_and(|keywords| keywords.iter().any(|keyword| keyword.as_str() == Some("bounty")))
        && crate::admission::SourceContextPolicy::Closed.accepts(entry)
        && entry["piloting"].is_null()
        && entry["attachCondition"].is_null()
        && [
            "deployBox",
            "deployCost",
            "upgradePower",
            "upgradeHp",
        ]
        .iter()
        .all(|key| entry["card"][*key].is_null())
        && (entry["card"]["epicAction"].is_null() || entry["card"]["epicAction"] == "")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Set a nested JSON path, creating missing objects, so mutants can add fields
    /// that the reviewed shape must reject.
    fn set(raw: &mut Value, path: &str, value: Value) {
        let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
        let mut cursor = raw;
        for part in &parts[..parts.len() - 1] {
            if cursor.get(*part).is_none() {
                cursor.as_object_mut().unwrap().insert((*part).into(), json!({}));
            }
            cursor = &mut cursor[*part];
        }
        cursor.as_object_mut().unwrap().insert(parts[parts.len() - 1].into(), value);
    }

    fn metadata() -> Value {
        json!({
            "events":["onCardDefeated","onCardCaptured"],
            "eventCard":"source",
            "canBeTriggeredBy":"opponent",
            "contextPlayer":"collector",
            "zone":"anyArena",
            "canResolveWithoutLegalTargets":true,
            "emitsCollectedEvent":"onBountyCollected"
        })
    }

    fn draw_ability() -> Value {
        json!({
            "trigger":"bounty",
            "title":"Collect Bounty: Draw a card",
            "costs":[],
            "effect":{"kind":"draw","target":"ownPlayer","amount":{"kind":"const","value":1}},
            "optional":true,
            "bounty":metadata()
        })
    }

    #[test]
    fn metadata_is_exact_and_unknown_fields_reject() {
        assert!(serde_json::from_value::<Metadata>(metadata()).unwrap().reviewed());
        for (path, value) in [
            ("/events", json!(["onCardDefeated"])),
            ("/events", json!(["onCardDefeated", "onCardCaptured", "onCardDefeated"])),
            ("/eventCard", json!("collector")),
            ("/canBeTriggeredBy", json!("self")),
            ("/contextPlayer", json!("source")),
            ("/zone", json!("arena")),
            ("/canResolveWithoutLegalTargets", json!(false)),
            ("/emitsCollectedEvent", json!("onBountyCollectedAgain")),
        ] {
            let mut raw = metadata();
            set(&mut raw, path, value.clone());
            let parsed: Metadata = serde_json::from_value(raw.clone()).unwrap();
            assert!(!parsed.reviewed(), "{path}={value}");
        }
        let mut raw = metadata();
        raw["extra"] = json!(true);
        assert!(serde_json::from_value::<Metadata>(raw).is_err());
    }

    #[test]
    fn printed_count_mirrors_the_engine_own_line_rule() {
        let mut entry = json!({"card":{"types":["unit"],"text":"Grit\nBounty — Draw a card. (When this unit is defeated, your opponent collects its bounty.)"}});
        assert_eq!(printed_count(&entry), 1);
        entry["card"]["text"] = json!("If an enemy unit has a Bounty, you may ready this unit.");
        assert_eq!(printed_count(&entry), 0);
        entry["card"]["text"] = json!("Bounty — Draw a card.\nBounty — Draw a card.");
        assert_eq!(printed_count(&entry), 2);
        entry["card"]["types"] = json!(["leader"]);
        entry["card"]["text"] = json!("Deploy this leader.");
        entry["card"]["deployBox"] = json!("Bounty — Draw a card.");
        assert_eq!(printed_count(&entry), 1);
    }

    #[test]
    fn registration_binds_metadata_header_and_bounded_effect() {
        let ability: Ability = serde_json::from_value(draw_ability()).unwrap();
        assert!(registration(&ability));
        for (path, value) in [
            ("/trigger", json!("whenDefeated")),
            ("/title", json!("Draw a card")),
            ("/title", json!("Collect Bounty: ")),
            ("/costs", json!([{"kind":"exhaustSelf"}])),
            ("/condition", json!({"kind":"const","value":true})),
            ("/contextTitle", json!([{"kind":"const","value":"title"}])),
            ("/then", json!({"title":"then","effect":{"kind":"noAction"}})),
            ("/ifYouDo", json!({"title":"then","effect":{"kind":"noAction"}})),
            ("/attachedContext", json!("host")),
            ("/coordinate", json!(true)),
            ("/requiresConfirmation", json!(true)),
            ("/bounty/zone", json!("arena")),
        ] {
            let mut raw = draw_ability();
            set(&mut raw, path, value.clone());
            let changed: Ability = serde_json::from_value(raw).unwrap();
            assert!(!registration(&changed), "{path}={value}");
        }
        let mut raw = draw_ability();
        raw.as_object_mut().unwrap().remove("bounty");
        assert!(!registration(&serde_json::from_value(raw).unwrap()));
        // The context player is the collector, so a source-relative selector cannot
        // express a bounty effect.
        let mut raw = draw_ability();
        raw["effect"] = json!({"kind":"damage","target":"chosen","amount":{"kind":"const","value":2}});
        raw["targetResolver"] = json!({"owner":"self","type":"unit","zone":"anyAttackable"});
        assert!(!registration(&serde_json::from_value(raw).unwrap()));
        let mut raw = draw_ability();
        raw["effect"] = json!({"kind":"damage","target":"chosen","amount":{"kind":"const","value":2}});
        raw["targetResolver"] = json!({"owner":"any","type":"unit","zone":"anyAttackable"});
        assert!(registration(&serde_json::from_value(raw).unwrap()));
    }

    fn source() -> Value {
        json!({
            "baseClass":"NonLeaderUnitCard",
            "card":{"types":["unit"],"unique":false,"keywords":["bounty"],"text":"Bounty — Draw a card.","epicAction":null,"deployBox":null,"deployCost":null,"upgradePower":null,"upgradeHp":null},
            "keywordValues":{},
            "piloting":null,
            "attachCondition":null,
            "abilities":{"leader":[],"unit":[],"card":[draw_ability()]}
        })
    }

    fn accepted(source: &Value) -> bool {
        let sides = match crate::attached_registration::sides(source) {
            Ok(sides) => sides,
            Err(_) => return false,
        };
        entry(source, &sides)
    }

    #[test]
    fn entry_requires_one_registration_per_printed_instance_on_an_ordinary_unit() {
        assert!(accepted(&source()));
        for (path, value) in [
            ("/baseClass", json!("LeaderUnitCard")),
            ("/card/types", json!(["leader"])),
            ("/card/keywords", json!([])),
            ("/keywordValues", json!({"bounty":1})),
            ("/card/text", json!("Grit")),
            ("/card/text", json!("Bounty — Draw.\nBounty — Draw.")),
            ("/abilities/card", json!([])),
            ("/abilities/card", json!([draw_ability(), draw_ability()])),
            ("/abilities/leader", json!([draw_ability()])),
            ("/piloting", json!({})),
            ("/attachCondition", json!({"kind":"const","value":true})),
            ("/card/deployBox", json!("Deploy")),
            ("/card/upgradePower", json!(1)),
            ("/card/epicAction", json!("Epic Action: do something")),
        ] {
            let mut changed = source();
            set(&mut changed, path, value.clone());
            assert!(!accepted(&changed), "{path}={value}");
        }
    }

    #[test]
    fn metadata_alone_without_a_printed_instance_rejects_and_mentions_stay_outside_the_family() {
        let mut unprinted = source();
        unprinted["card"]["text"] = json!("Grit (This unit gets +1/+0 for each damage on it.)");
        assert!(!accepted(&unprinted), "registration without a printed instance");
        let mut mention = source();
        mention["card"]["text"] = json!("When Played: If an enemy unit has a Bounty, you may ready this unit.");
        mention["card"]["keywords"] = json!(["bounty"]);
        mention["keywordValues"] = json!({});
        mention["abilities"]["card"] = json!([{
            "trigger":"whenPlayed","title":"Ready this unit","costs":[],"optional":true,
            "effect":{"kind":"ready","target":"self"}
        }]);
        assert!(!marker(&mention), "a provider mention is not a family member");
        assert!(accepted(&mention));
    }
}
