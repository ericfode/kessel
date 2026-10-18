//! Closed source contract for compulsory card naming and an opponent Play lock.
//! This module proves source shape only. It never erases the continuation into
//! NoAction and never admits a registration without a complete runtime adapter.
use serde_json::{json, Value};

pub(crate) const REGISTRY_COUNT: usize = 1931;
pub(crate) const REGISTRY_SHA256: &str = "3c793f2120a9b07d451e4166fa3841832775f8371a102b60ea0329f79852b717";
// SHA256 is of pinned forceteki/test/json/_playableCardTitles.json bytes.
// The default registry is the entire file in its original order; optional format
// legal-title filtering preserves that order (Game.filterToLegalTitles).
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub(crate) enum Registry { PlayableCardTitles }
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub(crate) enum Recipient { Opponent }
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub(crate) enum Lifetime { WhileSourceInPlay }
#[derive(Debug,Clone,PartialEq,Eq)]
pub(crate) struct Descriptor {
    pub trigger: String,
    pub choice_binding: String,
    pub registry: Registry,
    pub recipient: Recipient,
    pub lifetime: Lifetime,
}
#[derive(Debug,Clone,PartialEq)]
pub(crate) struct Prepared {
    pub registration: Value,
    pub descriptor: Descriptor,
}
const EXPECTED: &str = r###"{"trigger":"whenPlayed","title":"Name a card","costs":[],"targetResolver":{"kind":"choiceSelection","mode":"dropdownList","resultBinding":"select","namedResultBinding":"selects.target.choice","defaults":"pinnedTargetResolver","chooser":{"kind":"const","value":"self"},"options":{"kind":"choiceRegistry","registry":"playableCardTitles","order":"engineRegistry","capture":"setupRegistryReference"},"logSelection":true},"effect":{"kind":"resolveTargetSelection","resultBinding":"select"},"then":{"title":"While this unit is in play, opponents can't play [value]","costs":[],"titleParts":[{"kind":"const","value":"While this unit is in play, opponents can't play "},{"kind":"ref","path":["select"]},{"kind":"const","value":""}],"effect":{"kind":"lastingPropertiesFactory","parameters":[{"root":"attackBinding1_0","role":"systemContext"}],"body":{"kind":"lastingEffectContract","scope":"player","target":{"kind":"ref","path":["attackBinding1_0","player","opponent"]},"duration":"whileSourceInPlay","effects":[{"kind":"playerRestriction","target":"effectPlayer","duration":"phase","restriction":"play","condition":{"kind":"attackCallback","parameters":[{"root":"attackBinding2_0","role":"attemptedActionContext"}],"body":{"kind":"binary","op":"===","left":{"kind":"ref","path":["attackBinding2_0","ability","card","title"]},"right":{"kind":"ref","path":["select"]}}}}],"applicationContext":"retainSourceAbility","effectContext":"pinnedOngoingFrameworkContext","sourceZoneFilter":"anyArena","isLastingEffect":true,"cardApplicability":"unfiltered","requiresSourceInPlayAtApplication":false,"sourceLifetime":"expireWhenSourceOutOfPlayDuringEffectResolution"}}}}"###;
fn bounded_binding(value:&Value)->Option<&str> {
    value.as_str().filter(|s| !s.is_empty() && s.len()<=128
        && s.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'_'||b==b'$'))
}
fn source(entry:&Value)->bool {
    entry["status"]=="supported" && crate::admission::SourceContextPolicy::Closed.accepts(entry)
        && entry["baseClass"]=="NonLeaderUnitCard" && entry["card"]["types"]==json!(["unit"])
        && entry["card"]["unique"].is_boolean()
        && entry["card"]["cost"].as_u64().is_some_and(|n|n<=1000)
        && entry["card"]["hp"].as_u64().is_some_and(|n|(1..=1000).contains(&n))
        && entry["card"]["power"].as_u64().is_some_and(|n|n<=1000)
        && matches!(entry["card"]["arena"].as_str(),Some("ground"|"space"))
        && entry["piloting"].is_null() && entry["attachCondition"].is_null()
        && ["pilotText","pilotBox","pilotingBox","deployBox","deployCost","upgradePower","upgradeHp"]
            .iter().all(|key|entry["card"][key].is_null())
        && (entry["card"]["epicAction"].is_null()||entry["card"]["epicAction"]=="")
        && ((entry["card"]["keywords"]==json!([])&&entry["keywordValues"]==json!({}))
            ||(entry["card"]["keywords"]==json!(["restore"])&&entry["keywordValues"]==json!({"restore":1})))
}
/// Only a root naming resolver enters this family. Nested naming (Qi'ra), phase
/// locks and other dropdown effects retain their separate explicit rejection.
pub(crate) fn prepare(entry:&Value)->Result<Option<Prepared>,String> {
    let abilities=&entry["abilities"];
    let raw=&abilities["card"][0];
    if raw["targetResolver"]["mode"]!="dropdownList" {return Ok(None);}
    let fail=||"named Play lock requires the complete reviewed ordinary when-played registration".to_owned();
    if !source(entry) || abilities.as_object().is_none_or(|o|o.len()!=3)
        || !abilities["card"].as_array().is_some_and(|a|a.len()==1)
        || abilities["unit"]!=json!([])||abilities["leader"]!=json!([]) {return Err(fail());}
    let mut normalized=raw.clone();
    // Alpha-renaming is limited to the two proven declarations. Equality below
    // checks every use, role, path, optional flag and lexical select capture.
    let system=bounded_binding(&raw["then"]["effect"]["parameters"][0]["root"]).ok_or_else(fail)?.to_owned();
    let attempted=bounded_binding(&raw["then"]["effect"]["body"]["effects"][0]["condition"]["parameters"][0]["root"]).ok_or_else(fail)?.to_owned();
    if system==attempted||system=="select"||attempted=="select" {return Err(fail());}
    fn rename(v:&mut Value,system:&str,attempted:&str) {
        if v["kind"]=="ref" {
            if v["path"][0]==system {v["path"][0]=json!("attackBinding1_0");}
            else if v["path"][0]==attempted {v["path"][0]=json!("attackBinding2_0");}
        }
        match v {Value::Array(a)=>for v in a {rename(v,system,attempted)},Value::Object(o)=>for v in o.values_mut(){rename(v,system,attempted)},_=>{}}
    }
    rename(&mut normalized,&system,&attempted);
    normalized["then"]["effect"]["parameters"][0]["root"]=json!("attackBinding1_0");
    normalized["then"]["effect"]["body"]["effects"][0]["condition"]["parameters"][0]["root"]=json!("attackBinding2_0");
    let expected:Value=serde_json::from_str(EXPECTED).expect("reviewed named-play source template");
    if normalized!=expected {return Err(fail());}
    Ok(Some(Prepared {registration:raw.clone(),descriptor:Descriptor {
        trigger:"whenPlayed".into(),choice_binding:"select".into(),registry:Registry::PlayableCardTitles,
        recipient:Recipient::Opponent,lifetime:Lifetime::WhileSourceInPlay,
    }}))
}
#[cfg(test)]
#[path="named_play_contract_tests.rs"]
mod tests;
