//! Closed descriptions of named target -> modal choice registrations.
//!
//! This is a source/IR contract, not an executor or an admission shortcut.
//! In particular, `registration` retains every callback and named binding:
//! replacing it with NoAction would lose payment timing and captured identity.
//! Shared runtime integration must implement the descriptor before admitting it.
use serde_json::{json, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Chooser {
    Opponent,
    SelectedOwner,
    SelectedController,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Resolution {
    DamageOrDraw {
        damage: u16,
        draw: u16,
    },
    /// CardEffectResourcePaymentSystem: Credits/adjustments apply; no cancellation.
    ExhaustOrAdjustedPayment {
        amount: u16,
    },
    OwnerDeckPlacement,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DependentChoice {
    pub trigger: String,
    pub target_name: String,
    pub choice_name: String,
    pub card_type: String,
    pub zone: String,
    pub controller: String,
    pub optional_target: bool,
    pub chooser: Chooser,
    pub resolution: Resolution,
    /// Full original registration, including prompt/highlight/capture provenance.
    pub registration: Value,
}

fn marker(value: &Value) -> bool {
    matches!(
        value["kind"].as_str(),
        Some("choiceSelection" | "resolveChoice" | "resolveTargetSelection")
    ) || match value {
        Value::Array(items) => items.iter().any(marker),
        Value::Object(fields) => fields.values().any(marker),
        _ => false,
    }
}
fn literal(value: &str) -> Value {
    json!({"kind":"const","value":value})
}
fn reference(path: &[&str]) -> Value {
    json!({"kind":"ref","path":path})
}
fn read(slot: &str, path: &[&str], role: &str) -> Value {
    json!({"kind":"namedSelectionRead","contextBinding":{
        "id":"callback","role":role,"bindAt":"callbackInvocation",
        "readAt":"expressionEvaluation","reference":"lexicalContextObject"},
        "slot":slot,"path":path,"optionalAccess":[],"identity":"physicalValueOrSelectedList"})
}
fn bounded_name(value: &Value) -> bool {
    value
        .as_str()
        .is_some_and(|text| !text.is_empty() && text.len() <= 128)
}

/// Source offsets are diagnostic names; all role/timing/identity fields remain
/// checked by whole-object equality. This never removes an unknown field.
fn normalize_callback_names(value: &mut Value) -> Result<(), String> {
    if value["kind"] == "namedSelectionRead" {
        if !bounded_name(&value["contextBinding"]["id"]) {
            return Err("named modal read needs a bounded lexical callback identity".into());
        }
        value["contextBinding"]["id"] = json!("callback");
    }
    match value {
        Value::Array(items) => {
            for item in items {
                normalize_callback_names(item)?;
            }
        }
        Value::Object(fields) => {
            for item in fields.values_mut() {
                normalize_callback_names(item)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn target(name: &str, card_type: &str, zone: &str, controller: &str, optional: bool) -> Value {
    let mut resolver = json!({"kind":"cardSelection","mode":"single","chooser":"self",
        "resultBinding":name,"defaults":"pinnedTargetResolver","contextBinding":"target",
        "choosingPlayerOverride":"contextOverrideFirst","delayOpponentBeforeCosts":true,
        "cardTypes":literal(card_type),"zones":literal(zone),"controller":literal(controller),"resolverName":name});
    if optional {
        resolver["optional"] = json!(true);
    }
    json!({"name":name,"targetResolver":resolver,"chooser":"self"})
}
fn owner_chooser() -> Value {
    json!({"kind":"conditional","condition":{"kind":"binary","op":"===",
        "left":read("unit", &["owner"], "choosingPlayer"),"right":reference(&["player"])},
        "then":literal("self"),"else":literal("opponent")})
}
fn controller_chooser() -> Value {
    json!({"kind":"selectionPlayerConversion","operation":"asRelativePlayer",
        "args":[reference(&["player"]),read("targetUnit", &["controller"], "choosingPlayer")]})
}
fn base_choice(mode: &str, slot: &str, chooser: Value) -> Value {
    json!({"kind":"choiceSelection","mode":mode,"resultBinding":slot,
        "namedResultBinding":format!("selects.{slot}.choice"),"defaults":"pinnedTargetResolver",
        "chooser":chooser,"highlightCards":read(if slot=="deck" {"unit"} else {"targetUnit"}, &[], "highlightCards"),
        "resolverName":slot,"contextSelectAssignment":"onlyWhenNameIsTarget"})
}
fn unless_choice(payment: bool) -> Value {
    let chooser = if payment {
        controller_chooser()
    } else {
        literal("opponent")
    };
    let mut resolver = base_choice("selectUnless", "opponentsChoice", chooser.clone());
    resolver["unlessLegalityMode"] = json!("mustFullyOrPartiallyResolve");
    resolver["defaultAutomaticallyWhenUnlessIllegal"] = json!(true);
    if payment {
        resolver["promptTitleParts"] = json!([
            literal("[Exhaust] "),
            read("targetUnit", &["title"], "activePromptTitle"),
            literal(" or [Pay] "),
            literal("2 resources"),
            literal("")
        ]);
        resolver["defaultEffect"] = json!({"label":"Exhaust","effect":{
            "kind":"exhaust","target":read("targetUnit", &[], "effect")}});
        resolver["unlessEffect"] = json!({"label":"Pay","effect":{
            "kind":"registrationSystem","system":"payResources","contextTarget":"self","defaults":"pinnedSystem",
            "properties":{"target":read("targetUnit", &["controller"], "effect"),"amount":{"kind":"const","value":2}}}});
    } else {
        resolver["promptTitleParts"] = json!([
            literal(""),
            read("targetUnit", &["title"], "activePromptTitle"),
            literal(" takes 7 [Damage] or opponent [Draws] 3 cards")
        ]);
        resolver["defaultEffect"] = json!({"label":"Damage","effect":{
            "kind":"damage","target":read("targetUnit", &[], "effect"),"amount":{"kind":"const","value":7}}});
        resolver["unlessEffect"] = json!({"label":"Draw","effect":{
            "kind":"draw","target":"ownPlayer","amount":{"kind":"const","value":3}}});
    }
    json!({"name":"opponentsChoice","targetResolver":resolver,"chooser":chooser,
        "dependsOn":"targetUnit","effect":{"kind":"resolveChoice","resultBinding":"opponentsChoice"}})
}
fn deck_choice() -> Value {
    let chooser = owner_chooser();
    let mut resolver = base_choice("select", "deck", chooser.clone());
    resolver["promptTitleParts"] = json!([
        literal("Move "),
        read("unit", &["title"], "activePromptTitle"),
        literal(" to [Top] or [Bottom] of your deck")
    ]);
    let entries = [("Top", "moveToTopOfDeck", "top"), ("Bottom", "moveToBottomOfDeck", "bottom")]
        .into_iter().enumerate().map(|(order,(label,kind,capture))| {
        json!({"labelParts":[literal(label)],"keyEvaluation":"beforeValueConstruction",
            "constructionCaptures":[{"name":capture,"role":"staticProperty",
                "value":reference(&["factory","targets","unit"]),"evaluateAt":"choiceFactoryInvocation","order":order}],
            "effect":{"kind":kind,"target":reference(&["systemFactoryCaptures",capture])}})
    }).collect::<Vec<_>>();
    resolver["choices"] = json!({"kind":"choiceFactoryMap","parameters":[{"root":"factory","role":"choiceFactoryContext"}],
        "captures":[],"entries":entries,"constructionOrder":"readonlyDeclarationsThenEachKeyAndValueLeftToRight",
        "contextBinding":"resolverContextByReference","captureIdentity":"valueOrPhysicalObjectReference",
        "invocationPolicy":"eachNativeChoiceGeneration","keyOrder":"javascriptPropertyOrder","keyCoercion":"javascriptPropertyKey",
        "duplicatePolicy":"lastAssignmentWins","callbacks":"deferredWithLexicalBindings","factory":true});
    json!({"name":"deck","targetResolver":resolver,"chooser":chooser,"dependsOn":"unit",
        "effect":{"kind":"resolveChoice","resultBinding":"deck"}})
}

/// Rename declared factory bindings only after proving each reference points to
/// that exact declaration. Distinct captures cannot alias the preceding branch.
fn normalize_deck_bindings(raw: &mut Value) -> Result<(), String> {
    let factory = raw
        .pointer_mut("/targetResolvers/1/targetResolver/choices")
        .filter(|value| value.is_object())
        .ok_or("deck choice factory missing or malformed")?;
    let root = factory["parameters"][0]["root"]
        .as_str()
        .filter(|_| bounded_name(&factory["parameters"][0]["root"]))
        .ok_or("deck choice needs a bounded factory parameter")?
        .to_owned();
    let entries = factory
        .get_mut("entries")
        .and_then(Value::as_array_mut)
        .ok_or("deck choice entries missing")?;
    if entries.len() != 2 {
        return Err("deck choice requires both Top and Bottom".into());
    }
    let mut names = Vec::new();
    for (index, entry) in entries.iter_mut().enumerate() {
        let captures = entry
            .get_mut("constructionCaptures")
            .and_then(Value::as_array_mut)
            .ok_or("deck movement capture missing")?;
        if captures.len() != 1
            || !bounded_name(&captures[0]["name"])
            || captures[0]["value"] != reference(&[&root, "targets", "unit"])
        {
            return Err(
                "deck movement must capture the selected physical unit at factory invocation"
                    .into(),
            );
        }
        let name = captures[0]["name"].as_str().unwrap().to_owned();
        if names.contains(&name) {
            return Err("deck branch captures require distinct identities".into());
        }
        names.push(name.clone());
        let canonical = if index == 0 { "top" } else { "bottom" };
        captures[0]["name"] = json!(canonical);
        captures[0]["value"] = reference(&["factory", "targets", "unit"]);
        if entry["effect"]["target"] != reference(&["systemFactoryCaptures", &name]) {
            return Err("deck movement reads a different construction capture".into());
        }
        entry["effect"]["target"] = reference(&["systemFactoryCaptures", canonical]);
    }
    factory["parameters"][0]["root"] = json!("factory");
    Ok(())
}

fn ordinary_source(entry: &Value, unit: bool) -> bool {
    entry["status"] == "supported"
        && crate::static_ongoing::closed_source_context(entry)
        && entry["baseClass"]
            == if unit {
                "NonLeaderUnitCard"
            } else {
                "EventCard"
            }
        && entry["card"]["types"]
            == if unit {
                json!(["unit"])
            } else {
                json!(["event"])
            }
        && entry["card"]["unique"].is_boolean()
        && entry["card"]["cost"].as_u64().is_some_and(|n| n <= 1000)
        && (!unit
            || (matches!(entry["card"]["arena"].as_str(), Some("ground" | "space"))
                && entry["card"]["hp"]
                    .as_u64()
                    .is_some_and(|n| (1..=1000).contains(&n))
                && entry["card"]["power"].as_u64().is_some_and(|n| n <= 1000)))
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
        .all(|key| entry["card"][key].is_null())
        && (entry["card"]["epicAction"].is_null() || entry["card"]["epicAction"] == "")
        && entry["card"]["keywords"] == if unit { json!(["ambush"]) } else { json!([]) }
        && entry["keywordValues"] == if unit { json!({"ambush":1}) } else { json!({}) }
}

/// Inspect the complete, bounded dependent-choice shapes. `Some` establishes a
/// source contract only. It does not produce an executable Ability or Program.
/// Unhandled modal families fail explicitly; ordinary nonmodal cards return None.
pub(crate) fn inspect(entry: &Value) -> Result<Option<DependentChoice>, String> {
    if !marker(&entry["abilities"]) {
        return Ok(None);
    }
    let fail =
        || "named modal choice requires a complete reviewed dependent registration".to_owned();
    let abilities = &entry["abilities"];
    let cards = abilities["card"].as_array().ok_or_else(fail)?;
    if abilities.as_object().is_none_or(|fields| fields.len() != 3)
        || cards.len() != 1
        || abilities["unit"] != json!([])
        || abilities["leader"] != json!([])
    {
        return Err(fail());
    }
    let raw = &cards[0];
    let trigger = raw["trigger"].as_str().ok_or_else(fail)?;
    let unit = trigger == "whenDefeated";
    if !ordinary_source(entry, unit)
        || (!unit && trigger != "event")
        || !raw["title"]
            .as_str()
            .is_some_and(|t| !t.is_empty() && t.len() <= 1024)
    {
        return Err(fail());
    }
    let is_deck = raw["targetResolvers"][1]["name"] == "deck";
    let payment = raw["targetResolvers"][1]["targetResolver"]["unlessEffect"]["effect"]["system"]
        == "payResources";
    if unit && !is_deck {
        return Err(fail());
    }
    let (
        target_name,
        choice_name,
        card_type,
        zone,
        controller,
        optional_target,
        chooser,
        resolution,
    ) = if is_deck {
        (
            "unit",
            "deck",
            "nonLeaderUnit",
            if unit { "groundArena" } else { "anyAttackable" },
            "any",
            unit,
            Chooser::SelectedOwner,
            Resolution::OwnerDeckPlacement,
        )
    } else if payment {
        (
            "targetUnit",
            "opponentsChoice",
            "unit",
            "anyAttackable",
            "any",
            false,
            Chooser::SelectedController,
            Resolution::ExhaustOrAdjustedPayment { amount: 2 },
        )
    } else {
        (
            "targetUnit",
            "opponentsChoice",
            "unit",
            "anyAttackable",
            "opponent",
            false,
            Chooser::Opponent,
            Resolution::DamageOrDraw { damage: 7, draw: 3 },
        )
    };
    let mut normalized = raw.clone();
    normalize_callback_names(&mut normalized)?;
    if is_deck {
        normalize_deck_bindings(&mut normalized)?;
    }
    let expected = json!({"trigger":trigger,"title":raw["title"],"costs":[],
        "targetResolvers":[target(target_name,card_type,zone,controller,optional_target),
            if is_deck {deck_choice()} else {unless_choice(payment)}],"effect":{"kind":"noAction"}});
    if normalized != expected {
        return Err(fail());
    }
    Ok(Some(DependentChoice {
        trigger: trigger.into(),
        target_name: target_name.into(),
        choice_name: choice_name.into(),
        card_type: card_type.into(),
        zone: zone.into(),
        controller: controller.into(),
        optional_target,
        chooser,
        resolution,
        registration: raw.clone(),
    }))
}
