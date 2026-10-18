//! Closed source contracts for defeat-history projections. This module contains
//! no history storage or runtime evaluator. Controller and type are event-time
//! facts; repeated defeats are events, not distinct current discard identities.
use serde_json::{json, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelativePlayer {
    Own,
    Opponent,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Projection {
    SomeControlledUnit(RelativePlayer),
    ControlledUnitCount(RelativePlayer),
    SomeLeaderUnit,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Registration {
    FriendlyToken,
    FriendlyHeal,
    FriendlyOptionalStats,
    FriendlyExperience,
    FriendlyReplacementStats,
    LeaderRaid,
    EnemyStatsAndSaboteur,
    EnemyCountExperienceAndCapturedReturn,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Contract {
    pub registration: Registration,
    pub projection: Projection,
    /// Physical token dependencies still require selected-pool provenance checks.
    pub token: Option<&'static str>,
    /// Sion's separate captured-power/discard-return boundary is not implemented here.
    pub captured_power_return: bool,
}
fn invalid() -> String {
    "defeat history requires a complete reviewed ordinary source, watcher and projection".into()
}
fn binding(v: &Value) -> Option<&str> {
    v.as_str().filter(|s| !s.is_empty() && s.len() <= 128)
}
fn reference(name: &str) -> Value {
    json!({"kind":"ref","path":[name]})
}
fn raw_query(binding: &str, operation: &str, args: Value) -> Value {
    json!({"kind":"historyQuery","watcher":"cardsDefeatedThisPhase","binding":binding,
        "operation":operation,"args":args,"readAt":"expressionEvaluation"})
}
fn leader_filter(binding: &str) -> Value {
    json!({"kind":"dataLambda","bindings":[binding],"body":{
        "kind":"enumTypeTest","test":"isLeaderUnit","args":[{
            "kind":"dataPath","root":[binding],"steps":[
                {"key":"lastKnownInformation","optional":false},{"key":"type","optional":false}]}]}})
}
/// Validate a complete raw query, including alpha-bound lambda identity and read
/// time. The count projection may execute only through nonoptional `.length` in
/// the complete registration below; it is never a boolean/list fallback.
pub fn projection(v: &Value) -> Result<Projection, String> {
    let b = binding(&v["binding"]).ok_or_else(invalid)?;
    for (name, player) in [
        ("player", RelativePlayer::Own),
        ("opponent", RelativePlayer::Opponent),
    ] {
        if *v
            == raw_query(
                b,
                "someDefeatedUnitControlledByPlayer",
                json!([reference(name)]),
            )
        {
            return Ok(Projection::SomeControlledUnit(player));
        }
    }
    if *v
        == raw_query(
            b,
            "getDefeatedUnitsControlledByPlayer",
            json!([reference("opponent")]),
        )
    {
        return Ok(Projection::ControlledUnitCount(RelativePlayer::Opponent));
    }
    let lambda = binding(&v["args"][0]["bindings"][0]);
    if let Some(lambda) = lambda {
        if *v
            == raw_query(
                b,
                "someUnitDefeatedThisPhase",
                json!([leader_filter(lambda)]),
            )
        {
            return Ok(Projection::SomeLeaderUnit);
        }
    }
    Err(invalid())
}
pub fn marker(v: &Value) -> bool {
    let mut pending = vec![(v, 0usize)];
    let mut remaining = 2048usize;
    while let Some((v, depth)) = pending.pop() {
        // Conservatively mark oversized input so the closed contract rejects it.
        if remaining == 0 || depth > 24 {
            return true;
        }
        remaining -= 1;
        if v["kind"] == "historyQuery" && v["watcher"] == "cardsDefeatedThisPhase" {
            return true;
        }
        match v {
            Value::Object(o) => pending.extend(o.values().map(|v| (v, depth + 1))),
            Value::Array(a) => pending.extend(a.iter().map(|v| (v, depth + 1))),
            _ => {}
        }
        if pending.len() > 2048 {
            return true;
        }
    }
    false
}
fn projected(p: Projection) -> Value {
    // Private comparison shape, never returned as executable IR.
    json!({"defeatProjection": match p {
        Projection::SomeControlledUnit(RelativePlayer::Own) => "friendly",
        Projection::SomeControlledUnit(RelativePlayer::Opponent) => "enemy",
        Projection::ControlledUnitCount(RelativePlayer::Opponent) => "enemyCount",
        Projection::ControlledUnitCount(RelativePlayer::Own) => "friendlyCount",
        Projection::SomeLeaderUnit => "leader",
    }})
}
fn normalize(
    v: &Value,
    used: &mut Vec<(String, Projection)>,
    depth: usize,
    budget: &mut usize,
) -> Result<Value, String> {
    if depth > 20 || *budget == 0 {
        return Err(invalid());
    }
    *budget -= 1;
    if v["kind"] == "historyQuery" {
        let p = projection(v)?;
        used.push((v["binding"].as_str().ok_or_else(invalid)?.into(), p));
        return Ok(projected(p));
    }
    // An input cannot forge this private comparison marker.
    if v.get("defeatProjection").is_some() {
        return Err(invalid());
    }
    match v {
        Value::Object(o) => o
            .iter()
            .map(|(k, v)| Ok((k.clone(), normalize(v, used, depth + 1, budget)?)))
            .collect(),
        Value::Array(a) => a
            .iter()
            .map(|v| normalize(v, used, depth + 1, budget))
            .collect(),
        _ => Ok(v.clone()),
    }
}
fn literal(n: i64) -> Value {
    json!({"kind":"const","value":n})
}
fn stats(n: i64, duration: &str) -> Value {
    let amount = || {
        if n < 0 {
            json!({"kind":"unary","op":"-","value":literal(-n)})
        } else {
            literal(n)
        }
    };
    json!({"kind":"modifyStats","target":"chosen","duration":duration,"power":amount(),"hp":amount()})
}
fn conditional(condition: Value, on_true: Value, on_false: Value) -> Value {
    json!({"kind":"conditional","condition":condition,"onTrue":on_true,"onFalse":on_false})
}
fn ability(trigger: &str, effect: Value) -> Value {
    json!({"trigger":trigger,"costs":[],"effect":effect})
}
fn chosen(mut a: Value, owner: &str) -> Value {
    a["targetResolver"] = json!({"owner":owner,"type":"unit","zone":"anyAttackable"});
    a
}
fn keyword(keyword: &str, amount: i64) -> Value {
    json!({"kind":"gainKeyword","target":"self","duration":"persistent","keyword":keyword,"amount":literal(amount)})
}
fn watcher(binding: &str) -> Value {
    json!({"kind":"phaseHistory","binding":binding,"watcher":"cardsDefeatedThisPhase",
        "sharedScope":"game","registerAt":"setupStateWatchers","resetEvent":"onPhaseEnded",
        "implementation":"CardsDefeatedThisPhaseWatcher","queryContract":"pinnedWatcherApi"})
}
fn captured_return(a: &mut Value) -> bool {
    let path = &a["effect"]["condition"]["left"];
    let Some(id) = binding(&path["contextBinding"]["id"]) else {
        return false;
    };
    let parts = id.split(':').collect::<Vec<_>>();
    if parts.len() != 3
        || parts[0] != "callback"
        || parts[1].parse::<u32>().is_err()
        || parts[2] != "0"
    {
        return false;
    }
    let expected_path = json!({"kind":"dataPath","root":["context"],"contextBinding":{
        "id":id,"role":"condition","bindAt":"callbackInvocation","readAt":"expressionEvaluation","reference":"lexicalContextObject"},
        "steps":[{"key":"event","optional":false},{"key":"lastKnownInformation","optional":false},{"key":"power","optional":false}]});
    if *path != expected_path {
        return false;
    }
    a["effect"]["condition"]["left"] = json!({"capturedDefeatPower":true});
    true
}
/// Recognize the source contract without admitting its runtime. The caller must
/// still validate ordinary effects, native tokens, exact history and Sion's
/// independent captured-return companion before executing a complete card.
pub fn contract(e: &Value) -> Result<Option<Contract>, String> {
    if !marker(&e["abilities"]) {
        return Ok(None);
    }
    if e["status"] != "supported"
        || !crate::admission::closed_source_helpers(e)
        || !e["piloting"].is_null()
        || !e["attachCondition"].is_null()
        || ![
            "pilotText",
            "pilotBox",
            "pilotingBox",
            "deployBox",
            "deployCost",
            "upgradePower",
            "upgradeHp",
        ]
        .iter()
        .all(|k| e["card"][*k].is_null())
        || !(e["card"]["epicAction"].is_null() || e["card"]["epicAction"] == "")
        || !e["card"]["unique"].is_boolean()
        || !e["card"]["cost"].as_u64().is_some_and(|n| n <= 1000)
        || e["abilities"]["leader"] != json!([])
        || e["abilities"]["unit"] != json!([])
        || !e["abilities"].as_object().is_some_and(|o| o.len() == 3)
    {
        return Err(invalid());
    }
    let event = e["baseClass"] == "EventCard" && e["card"]["types"] == json!(["event"]);
    let unit = e["baseClass"] == "NonLeaderUnitCard" && e["card"]["types"] == json!(["unit"]);
    if !(event || unit)
        || (unit
            && (!["hp", "power"]
                .iter()
                .all(|k| e["card"][*k].as_u64().is_some_and(|n| n <= 1000))
                || e["card"]["hp"] == 0
                || !["ground", "space"].iter().any(|a| e["card"]["arena"] == *a)))
        || (event
            && !["hp", "power", "arena"]
                .iter()
                .all(|k| e["card"][*k].is_null()))
    {
        return Err(invalid());
    }
    let mut used = vec![];
    let mut budget = 384;
    let mut abilities = normalize(&e["abilities"]["card"], &mut used, 0, &mut budget)?;
    if used.len() != 1 || e["stateWatchers"] != json!([watcher(&used[0].0)]) {
        return Err(invalid());
    }
    let Some(list) = abilities.as_array_mut() else {
        return Err(invalid());
    };
    if list.is_empty() || list.len() > 2 {
        return Err(invalid());
    }
    for a in list.iter_mut() {
        let title = a
            .as_object_mut()
            .ok_or_else(invalid)?
            .remove("title")
            .ok_or_else(invalid)?;
        if !title
            .as_str()
            .is_some_and(|s| !s.is_empty() && s.len() <= 512)
        {
            return Err(invalid());
        }
    }
    let friendly = projected(Projection::SomeControlledUnit(RelativePlayer::Own));
    let enemy = projected(Projection::SomeControlledUnit(RelativePlayer::Opponent));
    let noop = json!({"kind":"noAction"});
    let mut variants = vec![];
    let mut add = |registration, shape, token, keywords| {
        variants.push((registration, shape, token, keywords))
    };
    for token in ["spy", "mandalorian"] {
        let trigger = ability(
            "whenPlayed",
            conditional(
                friendly.clone(),
                json!({"kind":"createTokenUnit","tokenType":token,"amount":literal(1),"target":"ownPlayer","entersReady":false}),
                noop.clone(),
            ),
        );
        let shape = if token == "spy" {
            json!([trigger])
        } else {
            let mut constant = ability("constant", keyword("sentinel", 1));
            constant["condition"] = json!({"kind":"query","name":"hasSomeArenaUnit","receiver":reference("player"),"args":[{
                "kind":"binary","op":"&&","left":{"kind":"const","value":true},"right":{
                    "kind":"query","name":"isTokenUnit","receiver":reference("card"),"args":[]}}]});
            json!([constant, trigger])
        };
        add(Registration::FriendlyToken, shape, Some(token), json!({}));
    }
    add(
        Registration::FriendlyHeal,
        json!([ability(
            "whenPlayed",
            conditional(
                friendly.clone(),
                json!({"kind":"heal","target":"ownBase","amount":literal(4)}),
                noop.clone()
            )
        )]),
        None,
        json!({"restore":2}),
    );
    let mut optional = chosen(
        ability(
            "whenPlayed",
            conditional(friendly.clone(), stats(-1, "phase"), noop.clone()),
        ),
        "any",
    );
    optional["optional"] = json!(true);
    add(
        Registration::FriendlyOptionalStats,
        json!([optional]),
        None,
        json!({}),
    );
    add(
        Registration::FriendlyExperience,
        json!([chosen(
            ability(
                "event",
                conditional(
                    friendly.clone(),
                    json!({"kind":"giveExperience","target":"chosen","amount":literal(2)}),
                    noop.clone()
                )
            ),
            "any"
        )]),
        Some("experience"),
        json!({}),
    );
    add(
        Registration::FriendlyReplacementStats,
        json!([chosen(
            ability(
                "whenPlayed",
                conditional(friendly, stats(-6, "phase"), stats(-3, "phase"))
            ),
            "opponent"
        )]),
        None,
        json!({"restore":3}),
    );
    let mut raid = ability("constant", keyword("raid", 3));
    raid["condition"] = projected(Projection::SomeLeaderUnit);
    add(Registration::LeaderRaid, json!([raid]), None, json!({}));
    let mut jyn = ability(
        "constant",
        json!({"kind":"simultaneous","effects":[
        {"kind":"modifyStats","target":"self","duration":"persistent","power":literal(1),"hp":literal(0)},keyword("saboteur",1)]}),
    );
    jyn["condition"] = enemy;
    add(
        Registration::EnemyStatsAndSaboteur,
        json!([jyn]),
        None,
        json!({}),
    );
    // The alpha-normalized callback path is compared only in this exact companion.
    if list.len() == 2 && captured_return(&mut list[1]) {
        let count = json!({"kind":"dataMember","receiver":projected(Projection::ControlledUnitCount(RelativePlayer::Opponent)),"key":"length","optional":false});
        let condition = json!({"kind":"binary","op":">=","left":{"capturedDefeatPower":true},"right":literal(7)});
        add(
            Registration::EnemyCountExperienceAndCapturedReturn,
            json!([
                ability(
                    "whenPlayed",
                    json!({"kind":"giveExperience","target":"self","amount":count})
                ),
                ability(
                    "whenDefeated",
                    conditional(
                        condition,
                        json!({"kind":"returnToHand","target":"self"}),
                        noop
                    )
                )
            ]),
            Some("experience"),
            json!({}),
        );
    }
    for (registration, shape, token, keywords) in variants {
        let printed_keywords = match registration {
            Registration::FriendlyHeal | Registration::FriendlyReplacementStats => {
                json!(["restore"])
            }
            Registration::LeaderRaid => json!(["raid"]),
            Registration::EnemyStatsAndSaboteur => json!(["saboteur"]),
            Registration::FriendlyToken if token == Some("mandalorian") => json!(["sentinel"]),
            _ => json!([]),
        };
        if abilities == shape
            && e["keywordValues"] == keywords
            && e["card"]["keywords"] == printed_keywords
            && (event == (registration == Registration::FriendlyExperience))
        {
            return Ok(Some(Contract {
                registration,
                projection: used[0].1,
                token,
                captured_power_return: registration
                    == Registration::EnemyCountExperienceAndCapturedReturn,
            }));
        }
    }
    Err(invalid())
}
pub fn entry(e: &Value) -> bool {
    matches!(contract(e), Ok(Some(_)))
}
