//! Interpreter for the bounded, data-only card IR. No TypeScript runs in search.
use super::*;
use crate::ability_ir::{Cost, Effect, Expr, Instruction, Program, RestrictionVerb, Target, TargetResolver};
use crate::ability_ir::activation_contract::{base_zone_source, Source};
#[path = "prepared_runtime.rs"]
mod prepared_runtime;

// Scalar field structure is independent of legality, checked amount conversion
// and capture timing. Callers retain those distinct semantic boundaries.
impl Effect {
    fn scalar_args(&self) -> Option<(&Target, Option<&Expr>)> {
        Some(match self {
            Self::DiscardDeck { target, amount }
            | Self::DiscardOwnHand { target, amount }
            | Self::CreateTokenUnit { target, amount, .. }
            | Self::ReadyResources { target, amount }
            | Self::Damage { target, amount }
            | Self::Heal { target, amount }
            | Self::Draw { target, amount }
            | Self::GiveShield { target, amount }
            | Self::GiveExperience { target, amount }
            | Self::GiveWeakness { target, amount }
            | Self::GiveAdvantage { target, amount, .. }
            | Self::GainKeyword { target, amount, .. } => (target, Some(amount)),
            Self::ResourceCard { target }
            | Self::MoveToBottomOfDeck { target }
            | Self::UseForce { target }
            | Self::GainForce { target }
            | Self::ReturnToHand { target }
            | Self::ModifyStats { target, .. }
            | Self::LoseKeyword { target, .. }
            | Self::Ready { target }
            | Self::Exhaust { target }
            | Self::Defeat { target } => (target, None),
            _ => return None,
        })
    }
}

// Callers capture values before target iteration; shared mutations preserve
// the distinct scalar and prepared-event capture boundaries.
#[derive(Clone, Copy)]
struct PhaseStatDelta {
    power: i64,
    hp: i64,
}

struct DrawResult {
    drawn: u16,
    resolved: bool,
}

enum TraitChange<'a> {Gain(&'a str),Lose(&'a str)}

#[derive(Clone, Copy)]
enum ConstantScope<'a> { Stats, AmbushBase, Traits(Option<&'a str>), Keyword(Option<&'a str>), Restriction, CombatDamageFirst, #[cfg(test)] All }
impl ConstantScope<'_> {
    fn accepts(self,e:&Effect)->bool {match (self,e) {
        (Self::Traits(filter),Effect::GainTrait{trait_name,..}|Effect::LoseTrait{trait_name,..})=>filter.is_none_or(|t|t==trait_name),
        (Self::Traits(filter),Effect::ConstantEffectRegistration(r))=>matches!(r.native(),Ok(crate::static_ongoing::NativeModifier::Traits(rule)) if rule.may_produce(filter)),
        (Self::Stats,Effect::ModifyStats{..})=>true,
        (Self::Stats,Effect::ConstantEffectRegistration(r))=>matches!(r.native(),Ok(crate::static_ongoing::NativeModifier::StatCount(_))),
        (Self::AmbushBase,Effect::ConstantEffectRegistration(r))=>matches!(r.native(),Ok(crate::static_ongoing::NativeModifier::FriendlyAmbushBase)),
        (Self::Keyword(filter),Effect::CopyKeywords{..})=>filter.is_none_or(|name|crate::ability_ir::COPY_KEYWORDS.contains(&name)),
        (Self::Keyword(filter),Effect::GainKeyword{keyword,..}|Effect::LoseKeyword{keyword,..})=>filter.is_none_or(|name|name==keyword),
        (Self::Restriction,Effect::CardRestriction{..})=>true,
        (Self::CombatDamageFirst,Effect::DealsCombatDamageFirst{..})=>true,
        #[cfg(test)] (Self::All,_)=>true,
        _=>false,
    }}
}
#[derive(Clone, Debug, PartialEq)]
enum Eval {
    ArenaCards(Vec<CardId>),
    Array(Vec<Eval>),
    Number(i64),
    Bool(bool),
    Text(String),
    Card(CardId),
    Player(Seat),
    Null,
}
impl Eval {
    fn number(&self) -> i64 {
        match self {
            Self::Number(n) => *n,
            Self::Bool(b) => i64::from(*b),
            _ => 0,
        }
    }
    fn truth(&self) -> bool {
        match self {
            Self::Null | Self::ArenaCards(_) => false,
            Self::Bool(b) => *b,
            Self::Number(n) => *n != 0,
            Self::Text(t) => !t.is_empty(),
            _ => true,
        }
    }
}
// The receiver is materialized once before callback card binding. This keeps
// nested iterators separate from the source, chosen target and live modifier.
fn evaluate_collection(mut receiver:Eval,name:&str,mut callback:impl FnMut(CardId)->Eval,needle:Option<Eval>)->Eval {
    if let Eval::ArenaCards(ids)=receiver {receiver=Eval::Array(ids.into_iter().map(Eval::Card).collect());}
    let Eval::Array(items)=receiver else{return Eval::Null;};
    assert!(items.len()<=256,"validated physical inventory exceeds collection capacity");
    if name=="includes" {return Eval::Bool(items.iter().any(|item|Some(item)==needle.as_ref()));}
    let evaluate=|item:&Eval,callback:&mut dyn FnMut(CardId)->Eval|match item {Eval::Card(id)=>callback(*id),_=>unreachable!("validated callback inventory contains cards")};
    match name {
        "some"=>Eval::Bool(items.iter().any(|item|evaluate(item,&mut callback).truth())),
        "every"=>Eval::Bool(items.iter().all(|item|evaluate(item,&mut callback).truth())),
        "filter"=>Eval::Array(items.into_iter().filter(|item|evaluate(item,&mut callback).truth()).collect()),
        "map"=>Eval::Array(items.iter().map(|item|evaluate(item,&mut callback)).collect()),
        _=>unreachable!("validated collection operation"),
    }
}
#[derive(Clone, Copy)]
enum Trigger {
    Keyword(u8),
    Program(usize, u16),
}

fn token_unit_code(token_type: &str) -> Option<&'static str> {
    Some(match token_type {
        "battleDroid" => "TOKEN_BATTLE_DROID",
        "cloneTrooper" => "TOKEN_CLONE_TROOPER",
        "beast" => "TOKEN_BEAST",
        "spy" => "TOKEN_SPY",
        "tieFighter" => "TOKEN_TIE_FIGHTER",
        "xwing" => "TOKEN_XWING",
        "mandalorian" => "TOKEN_MANDALORIAN",
        _ => return None,
    })
}
/// Discover creation and native-state dependencies together. Creation order is
/// retained for existing missing-token diagnostics; Force presence is separate.
#[derive(Debug, Default, PartialEq, Eq)]
struct TokenRequirements {
    created: Vec<String>,
    force: bool,
    weakness: bool,
    predicates: Vec<String>,
}
impl TokenRequirements {
    fn for_program(program: &Program) -> Self {
        fn walk(effect: &Effect, out: &mut TokenRequirements) {
            match effect {
                Effect::GiveShield { .. } => out.created.push("TOKEN_SHIELD".into()),
                Effect::GiveExperience { .. } => out.created.push("TOKEN_EXPERIENCE".into()),
                Effect::GiveWeakness { .. } => out.weakness = true,
                Effect::GiveAdvantage { .. } => out.created.push("TOKEN_ADVANTAGE".into()),
                Effect::CreateTokenUnit { token_type, .. } => out.created.push(
                    token_unit_code(token_type).map_or_else(
                        || format!("unsupported token unit {token_type}"), str::to_owned)),
                Effect::Select { effect, .. } => walk(effect, out),
                Effect::Sequence { effects } | Effect::Simultaneous { effects }
                | Effect::PhaseModifiers { effects } => {
                    for effect in effects { walk(effect, out); }
                }
                Effect::Conditional { on_true, on_false, .. } => {
                    walk(on_true, out);
                    if let Some(effect) = on_false { walk(effect, out); }
                }
                _ => {}
            }
        }
        let mut out = Self {
            force: program_needs_force(program),
            ..Self::default()
        };
        for instruction in &program.instructions {
            if let Some(effect) = match instruction {
                Instruction::Apply(effect) => Some(effect),
                other => other.group_effect(),
            } { walk(effect, &mut out); }
        }
        // Inspect every typed expression position, including target predicates,
        // continuations and costs; presence alone needs an exact token identity.
        fn presence(value:&Value, out:&mut Vec<String>) {
            match value {
                Value::Object(o)=> {
                    if o.get("kind").is_some_and(|v|v=="nativeTokenPredicate") {
                        if let Some(code)=match o.get("tokenType").and_then(Value::as_str) {
                            Some("shield")=>Some("TOKEN_SHIELD"),Some("experience")=>Some("TOKEN_EXPERIENCE"),Some("weakness")=>Some("TOKEN_WEAKNESS"),_=>None
                        } {if !out.iter().any(|v|v==code) {out.push(code.into());}}
                    }
                    for value in o.values() {presence(value,out);}
                },Value::Array(a)=>for value in a {presence(value,out);},_=>{}
            }
        }
        presence(&serde_json::to_value(program).expect("typed program serializes"),&mut out.predicates);
        out
    }
    fn needs(&self, token: &str) -> bool {
        if self.predicates.iter().any(|code|code==token) {return true;}
        match token {
            "TOKEN_THE_FORCE" => self.force,
            "TOKEN_WEAKNESS" => self.weakness,
            _ => self.created.iter().any(|created| created == token),
        }
    }
}
fn card_needs_token(programs: &CardPrograms, requirements: &[TokenRequirements], token: &str) -> bool {
    programs.card.iter().chain(&programs.unit).chain(&programs.leader)
        .any(|program| requirements[*program as usize].needs(token))
}
/// The first token a card's programs create that `has` lacks.
fn missing_token(catalogue: &Catalogue, requirements: &[TokenRequirements], code: &str, has: impl Fn(&str) -> bool) -> Option<String> {
    let programs = catalogue.cards.get(code)?;
    programs.card.iter().chain(&programs.leader).chain(&programs.unit)
        .flat_map(|program| requirements[*program as usize].created.iter())
        .find(|token| !has(token)).cloned()
}
fn reviewed_weakness_entry(entry:&Value)->bool {
    if entry["setCode"]!="TOKEN_WEAKNESS" {return false;}
    let expected:Value=serde_json::from_str(r#"{
  "title": "Weakness",
  "subtitle": "",
  "cost": 0,
  "hp": -1,
  "power": -1,
  "text": "",
  "deployBox": null,
  "epicAction": "",
  "unique": false,
  "rules": null,
  "upgradePower": -1,
  "upgradeHp": -1,
  "id": "7816991190",
  "aspects": [],
  "traits": [
    "condition"
  ],
  "keywords": [],
  "types": [
    "token",
    "upgrade"
  ],
  "setId": {
    "set": "HMW"
  },
  "internalName": "weakness"
}"#).expect("reviewed Weakness metadata");
    entry["setCode"]=="TOKEN_WEAKNESS" && entry["sourceHash"]=="dd546e4886757d77fffeb8be6e7381144fa7378b6493ce444aa87e4ba1202778" && entry["baseClass"]=="TokenUpgradeCard" && entry["card"]==expected && entry["keywordValues"]==serde_json::json!({}) && entry["abilities"]==serde_json::json!({"leader":[],"unit":[],"card":[]}) && entry["attachCondition"].is_null()
}
fn effect_has(e:&Effect,predicate:fn(&Effect)->bool)->bool {predicate(e)||match e {Effect::Simultaneous{effects}|Effect::PhaseModifiers{effects}|Effect::Sequence{effects}=>effects.iter().any(|e|effect_has(e,predicate)),Effect::Conditional{on_true,on_false,..}=>effect_has(on_true,predicate)||on_false.as_deref().is_some_and(|e|effect_has(e,predicate)),Effect::Select{effect,..}=>effect_has(effect,predicate),_=>false}}
// State import asks only about Force; do not allocate creation diagnostics there.
fn program_needs_force(program: &Program) -> bool {
    fn title_force(v:&Value)->bool {
        (v["kind"]=="query" && v["name"]=="hasControlledInPlayTitle" && v["args"][0]["value"]=="The Force") || match v {Value::Object(o)=>o.values().any(title_force),Value::Array(a)=>a.iter().any(title_force),_=>false}
    }
    program.costs.iter().any(|c| matches!(c, Cost::UseForceCost { .. })) || title_force(&serde_json::to_value(program).expect("typed program serializes")) || crate::ability_ir::contains_force_presence(program) || program.instructions.iter().any(|instruction| {
        let effect = match instruction {
            Instruction::Apply(effect) => Some(effect),
            other => other.group_effect(),
        };
        effect.is_some_and(|effect| effect_has(effect,
            |effect| matches!(effect, Effect::UseForce { .. } | Effect::GainForce { .. })))
    })
}
fn reviewed_shield_entry(entry:&Value)->bool {let expected:Value=serde_json::from_str(r#"{"title":"Shield","subtitle":null,"cost":0,"hp":0,"power":0,"text":"If damage would be dealt to attached unit, prevent that damage. If you do, defeat a Shield token on it.","deployBox":null,"epicAction":null,"unique":false,"rules":null,"upgradePower":0,"upgradeHp":0,"id":"8752877738","aspects":[],"traits":["armor"],"keywords":[],"types":["token","upgrade"],"setId":{"set":"SOR"},"internalName":"shield"}"#).expect("reviewed Shield metadata");entry["setCode"]=="TOKEN_SHIELD"&&entry["status"]=="unsupported"&&entry["baseClass"]=="TokenUpgradeCard"&&entry["sourcePath"]=="game/cards/01_SOR/tokens/Shield.ts"&&entry["sourceHash"]=="39f1c6699c36ba309a8fe8e053f9b94ddbb8877991767b6664fcd88d447be91f"&&entry["card"]==expected&&entry["attachCondition"].is_null()&&entry["piloting"].is_null()&&entry["keywordValues"]==serde_json::json!({})&&entry["abilities"]==serde_json::json!({"leader":[],"unit":[],"card":[]})}
fn reviewed_experience_entry(entry:&Value)->bool {
 let expected:Value=serde_json::from_str(r#"{"title":"Experience","subtitle":null,"cost":0,"hp":1,"power":1,"text":null,"deployBox":null,"epicAction":null,"unique":false,"rules":null,"upgradePower":1,"upgradeHp":1,"id":"2007868442","aspects":[],"traits":["learned"],"keywords":[],"types":["token","upgrade"],"setId":{"set":"SOR"},"internalName":"experience"}"#).expect("reviewed Experience metadata");
 entry["setCode"]=="TOKEN_EXPERIENCE"&&entry["sourcePath"]=="game/cards/01_SOR/tokens/Experience.ts"&&entry["status"]=="unsupported"&&entry["baseClass"]=="TokenUpgradeCard"&&entry["sourceHash"]=="fccf00d13061bfc15447101a0dc82fc360f9b5d2e28a9da678684e8c93bc9d67"&&entry["card"]==expected&&entry["attachCondition"].is_null()&&entry["piloting"].is_null()&&entry["keywordValues"]==serde_json::json!({})&&entry["abilities"]==serde_json::json!({"leader":[],"unit":[],"card":[]})
}


fn reviewed_beast_entry(entry:&Value)->bool {
    let expected:Value=serde_json::from_str(r#"{"title":"Beast","subtitle":"","cost":0,"hp":3,"power":3,"text":"","deployBox":null,"epicAction":"","unique":false,"rules":null,"upgradePower":null,"upgradeHp":null,"id":"6445363199","aspects":[],"traits":["creature"],"arena":"ground","keywords":[],"types":["token","unit"],"setId":{"set":"HMW"},"internalName":"beast"}"#).expect("reviewed Beast metadata");
    entry["setCode"]=="TOKEN_BEAST"&&entry["status"]=="supported"&&entry["attachCondition"].is_null()&&entry["piloting"].is_null()&&entry["sourcePath"]=="game/cards/09_HMW/tokens/Beast.ts"&&entry["sourceHash"]=="e9ce58041d8cb93f9a98e773c016dc56bad9271ee30b7954411a7d63357c2566"&&entry["baseClass"]=="TokenUnitCard"&&entry["card"]==expected&&["card","unit","leader"].iter().all(|side|entry["abilities"][side].as_array().is_some_and(|a|a.is_empty()))&&entry["keywordValues"].as_object().is_some_and(|v|v.is_empty())
}
fn reviewed_spy_entry(entry:&Value)->bool {let expected:Value=serde_json::from_str(r##"{"status":"supported","baseClass":"TokenUnitCard","sourcePath":"game/cards/06_SEC/tokens/Spy.ts","sourceHash":"aae6ac20865fac5b7c5eace59f33b781c75a578c07cf494bdc28b337c5d42f1f","card":{"title":"Spy","subtitle":"","cost":0,"hp":2,"power":0,"text":"Raid 2 (This unit gets +2/+0 while attacking.)","deployBox":null,"epicAction":"","unique":false,"rules":null,"upgradePower":null,"upgradeHp":null,"id":"6665455613","aspects":[],"traits":["official"],"arena":"ground","keywords":["raid"],"types":["token","unit"],"setId":{"set":"SEC"},"internalName":"spy"},"abilities":{"leader":[],"unit":[],"card":[]},"keywordValues":{"raid":2}}"##).unwrap();entry["setCode"]=="TOKEN_SPY"&&expected.as_object().unwrap().iter().all(|(k,v)|entry.get(k)==Some(v))&&entry["attachCondition"].is_null()&&entry["piloting"].is_null()}
fn reviewed_force_entry(entry:&Value)->bool {
    if entry["setCode"]!="TOKEN_THE_FORCE" {return false;}
    let metadata:Value=serde_json::from_str(r#"{
  "title": "The Force",
  "subtitle": "",
  "cost": 0,
  "hp": null,
  "power": null,
  "text": "(The Force is with you. When you use the Force, remove this token from play.)",
  "deployBox": null,
  "epicAction": "",
  "unique": false,
  "rules": "An individual player may only control one Force token at a time. If a player already controls a Force token when they are instructed to create their Force token, nothing happens.\nEach player may create their own Force token. It is not shared between players.\n",
  "upgradePower": null,
  "upgradeHp": null,
  "id": "4571900905",
  "aspects": [],
  "traits": [],
  "keywords": [],
  "types": [
    "token"
  ],
  "setId": {
    "set": "LOF"
  },
  "internalName": "the-force"
}"#).expect("reviewed Force metadata");
    entry["sourceHash"]=="3ff3888f5186845acdd8d3ef96f396875b2daf4f9b808fa9b4a2ef03a1b954b6" && entry["baseClass"]=="TokenCard" && entry["card"]==metadata && entry["abilities"]==serde_json::json!({"leader":[],"unit":[],"card":[]}) && entry["keywordValues"]==serde_json::json!({}) && entry["attachCondition"].is_null()
}

impl Engine {
    // Player.getInPlayCards: controller-owned arena cards plus the player's BaseZone.
    // Credits and Fortify cannot occur in the admitted state model.
    fn controlled_in_play_card(&self,s:&State,player:Seat,id:CardId)->bool {
        let c=s.card(id);c.present&&c.owner==player&&(self.in_arena(s,id)||self.attached_in_play(s,id)||id==s.player(player).base||id==s.player(player).leader)
    }
    fn printed_trait_names<'a>(&'a self,s:&State,id:CardId)->impl Iterator<Item=&'a str> {
        let definition=self.def(s,id);
        let back=if definition.info.is("leader") && s.card(id).deployed {
            self.catalogue.as_ref().and_then(|c|c.metadata.get(&definition.code))
                .and_then(|m|m["backSideTraits"].as_array()).filter(|a|!a.is_empty())
        } else {None};
        let front=if back.is_some() {&[][..]}else{definition.traits.as_slice()};
        back.into_iter().flat_map(|a|a.iter().filter_map(Value::as_str))
            .chain(front.iter().map(String::as_str))
    }
    /// One modifier traversal serves full trait inventories and membership.
    /// Losses apply after every gain, regardless of registration order.
    pub(super) fn card_traits(&self,s:&State,id:CardId)->std::collections::BTreeSet<String> {
        let mut traits=self.printed_trait_names(s,id).map(str::to_owned).collect::<std::collections::BTreeSet<_>>();
        let mut lost=std::collections::BTreeSet::new();
        self.visit_trait_changes(s,id,None,|change|match change {
            TraitChange::Gain(name)=>{traits.insert(name.to_owned());},
            TraitChange::Lose(name)=>{lost.insert(name.to_owned());},
        });
        for name in lost {traits.remove(&name);}
        traits
    }
    pub(super) fn card_has_trait(&self,s:&State,id:CardId,trait_name:&str)->bool {
        let mut gained=self.printed_trait_names(s,id).any(|name|name==trait_name);
        let mut lost=false;
        self.visit_trait_changes(s,id,Some(trait_name),|change|match change {
            TraitChange::Gain(name) if name==trait_name=>gained=true,
            TraitChange::Lose(name) if name==trait_name=>lost=true,
            _=>{},
        });
        gained && !lost
    }
    fn visit_trait_changes(&self,s:&State,id:CardId,requested:Option<&str>,mut visit:impl FnMut(TraitChange<'_>)) {
        for (source,effect) in self.constant_effects_matching(s,id,ConstantScope::Traits(requested)) {
            match effect {
                Effect::GainTrait{trait_name,..}=>visit(TraitChange::Gain(trait_name)),
                Effect::LoseTrait{trait_name,..}=>visit(TraitChange::Lose(trait_name)),
                Effect::ConstantEffectRegistration(r)=>match r.native() {
                    Ok(crate::static_ongoing::NativeModifier::Traits(crate::static_ongoing::TraitRule::UnitAura{gained,..}))=>visit(TraitChange::Gain(gained)),
                    Ok(crate::static_ongoing::NativeModifier::Traits(crate::static_ongoing::TraitRule::LeaderUnion{excluded}))=>{
                        let owner=s.card(source).owner;
                        for leader in 1..s.next_id {
                            if self.def(s,leader).info.is("leader") && self.controlled_in_play_card(s,owner,leader) {
                                if let Some(name)=requested {
                                    if name!=excluded && self.card_has_trait(s,leader,name) {visit(TraitChange::Gain(name));}
                                } else {
                                    for name in self.card_traits(s,leader) {
                                        if &name!=excluded {visit(TraitChange::Gain(&name));}
                                    }
                                }
                            }
                        }
                    },_=>{},
                },_=>{},
            }
        }
    }
    fn controlled_in_play_trait(&self,s:&State,player:Seat,trait_name:&str,excluded:CardId)->bool {
        (1..s.next_id).any(|id|id!=excluded&&self.controlled_in_play_card(s,player,id)&&self.card_has_trait(s,id,trait_name))
    }
    fn controlled_in_play_title(&self,s:&State,player:Seat,title:&str)->bool {
        (1..s.next_id).any(|id| {
            self.controlled_in_play_card(s,player,id) && self.def(s,id).info.title==title
        }) || (s.force_state_exact && s.player(player).force_held && self.force_def.is_some_and(|d|self.definitions[d as usize].info.title==title))
    }
    // Native tokens and ordinary upgrades share this physical attachment chain.
    fn attached_trait(&self,s:&State,host:CardId,trait_name:&str)->bool {
        host!=0 && self.in_arena(s,host) && self.upgrades(s,host).into_iter().any(|id|self.card_has_trait(s,id,trait_name))
    }
    fn attached_native_shield(&self,s:&State,host:CardId)->bool { self.attached_native_token(s,host,"shield") }
    fn attached_native_token(&self,s:&State,host:CardId,token_type:&str)->bool {
        let definition=match token_type {"shield"=>Some(self.shield_def),"experience"=>self.experience_def,"weakness"=>self.weakness_def,_=>None};
        host!=0 && self.in_arena(s,host) && definition.is_some_and(|definition|self.upgrades(s,host).into_iter().any(|id|s.card(id).def==definition))
    }

    pub(super) fn needs_force_state(&self)->bool {
        self.catalogue.as_ref().is_some_and(|catalogue|catalogue.metadata.keys().any(|code|catalogue.cards.get(code).is_some_and(|card|card.card.iter().chain(&card.unit).chain(&card.leader).any(|p|program_needs_force(&catalogue.programs[*p as usize])))))
    }
    pub(super) fn needs_hidden_history(&self) -> bool {
        fn grants(v:&Value)->bool {match v {Value::Object(o)=>o.get("keyword").is_some_and(|k| k=="hidden") || o.values().any(grants),Value::Array(a)=>a.iter().any(grants),_=>false}}
        let Some(catalogue)=&self.catalogue else {return self.definitions.iter().any(|d|d.flags & (1<<8)!=0);};
        catalogue.metadata.keys().any(|code| {
            self.by_code.get(code).is_some_and(|i|self.definitions[*i as usize].flags & (1<<8)!=0) ||
            catalogue.cards.get(code).is_some_and(|c|c.card.iter().chain(&c.unit).chain(&c.leader).any(|id|grants(&serde_json::to_value(&catalogue.programs[*id as usize]).expect("serializable program"))))
        })
    }
    /// Load every fully executable card, retaining explicit reasons for all others.
    pub fn from_catalogue_value(value: &Value) -> Result<Self, String> {
        let mut catalogue = Catalogue::load(value)?;
        let requirements: Vec<_> = catalogue.programs.iter().map(TokenRequirements::for_program).collect();
        let shield_available=value["cards"].as_array().ok_or("catalogue missing cards")?.iter().any(reviewed_shield_entry);
        let experience_available=value["cards"].as_array().ok_or("catalogue missing cards")?.iter().any(reviewed_experience_entry);
        let force_available=value["cards"].as_array().ok_or("catalogue missing cards")?.iter().any(reviewed_force_entry);
        let advantage_available=value["cards"].as_array().ok_or("catalogue missing cards")?.iter().any(crate::advantage::reviewed_entry);
        let weakness_available=value["cards"].as_array().ok_or("catalogue missing cards")?.iter().any(reviewed_weakness_entry);
        let beast_available=catalogue.cards.contains_key("TOKEN_BEAST")&&value["cards"].as_array().is_some_and(|cards|cards.iter().any(reviewed_beast_entry));
        let spy_available=catalogue.cards.contains_key("TOKEN_SPY")&&value["cards"].as_array().is_some_and(|cards|cards.iter().any(reviewed_spy_entry));
        let mut definitions = Vec::new();
        let mut by_code = HashMap::new();
        let mut shield_def = None;
        let mut experience_def = None;
        let mut weakness_def = None;
        let mut advantage_def = None;
        let mut force_def = None;
        for entry in value["cards"].as_array().ok_or("catalogue missing cards")? {
            let code = entry["setCode"]
                .as_str()
                .ok_or("catalogue card missing code")?
                .to_owned();
            let native_token = matches!(code.as_str(), "TOKEN_SHIELD" | "TOKEN_EXPERIENCE" | "TOKEN_WEAKNESS" | "TOKEN_THE_FORCE" | "TOKEN_ADVANTAGE");
            if !native_token && !catalogue.cards.contains_key(&code) {
                continue;
            }
            let result = (|| -> Result<Definition, String> {
                if entry["card"]["types"].as_array().is_some_and(|a|a.iter().any(|v|v=="unit"||v=="leader")) && !entry["card"]["unique"].is_boolean(){return Err("arena candidate requires explicit boolean unique metadata".into());}
                let info: CardInfo =
                    serde_json::from_value(entry["card"].clone()).map_err(|e| e.to_string())?;
                for field in ["cost", "power", "hp"] {
                    let value = &entry["card"][field];
                    if !value.is_null() && !(if info.is("upgrade") && field!="cost" {value.as_i64().is_some_and(|n|(-1000..=1000).contains(&n))}else{value.as_u64().is_some_and(|n| n <= 1000)}) {
                        return Err(format!("{field} must be an integer in 0..=1000"));
                    }
                }
                let programs = catalogue.cards.get(&code).cloned().unwrap_or_default();
                fn gains_support(v:&Value)->bool {match v {Value::Object(o)=>o.get("kind").is_some_and(|k|k=="gainKeyword")&&o.get("keyword").is_some_and(|k|k=="support")||o.values().any(gains_support),Value::Array(a)=>a.iter().any(gains_support),_=>false}}
                if gains_support(&entry["abilities"]) {return Err("granting Support requires a separately reviewed entry lifecycle".into());}
                if entry["keywordValues"].get("support").is_some() {
                    let unit_side=if info.is("leader") {&programs.unit}else{&programs.card};
                    let printed=entry["card"][if info.is("leader") {"deployBox"}else{"text"}].as_str().is_some_and(|t|t.lines().any(|line|line.starts_with("Support")));
                    let source=if info.is("leader") {entry["baseClass"]=="LeaderUnitCard"&&entry["card"]["types"]==json!(["leader"])&&programs.card.is_empty()&&!entry["card"]["text"].as_str().is_some_and(|t|t.lines().any(|l|l.starts_with("Support")))}else{entry["card"]["types"]==json!(["unit"])&&programs.leader.is_empty()&&programs.unit.is_empty()&&(entry["baseClass"]=="NonLeaderUnitCard"||(entry["baseClass"].is_null()&&entry["sourcePath"].is_null()&&unit_side.is_empty()))};
                    if !source||!printed||info.is("token")||!entry["piloting"].is_null()||!entry["attachCondition"].is_null()||unit_side.len()>1||unit_side.iter().any(|&id| {let p=&catalogue.programs[id as usize];!matches!(p.trigger.as_str(),"constant"|"onAttack"|"whenAttackEnds")||p.coordinate||p.attached_context.is_some()||p.gained_host.is_some()}) {return Err("Support donors require a bounded ordinary constant/OnAttack/attack-end registration and intrinsic unit-side keyword".into());}
                }

                for (available, token, reason) in [
                    (shield_available, "TOKEN_SHIELD", "reviewed Shield token unavailable"),
                    (experience_available, "TOKEN_EXPERIENCE", "reviewed Experience token unavailable"),
                    (force_available, "TOKEN_THE_FORCE", "reviewed Force token unavailable; dependent source cannot execute"),
                    (advantage_available,"TOKEN_ADVANTAGE","reviewed Advantage token unavailable; dependent source cannot execute"),
                    (weakness_available, "TOKEN_WEAKNESS", "Weakness token unavailable; dependent source cannot execute"),
                    (spy_available, "TOKEN_SPY", "reviewed Spy token unavailable"),
                    (beast_available, "TOKEN_BEAST", "reviewed Beast token unavailable; dependent source cannot execute"),
                ] {
                    if !available && card_needs_token(&programs, &requirements, token) {
                        return Err(reason.into());
                    }
                }
                if programs.card.iter().chain(&programs.unit).chain(&programs.leader).any(|p|catalogue.programs[*p as usize].trigger=="onDefense"&&!catalogue.programs[*p as usize].attached_host) && (!info.is("unit") || info.is("leader") || info.is("token") || entry["baseClass"]!="NonLeaderUnitCard" || !programs.unit.is_empty() || !programs.leader.is_empty()) {return Err("OnDefense currently requires an ordinary nonleader unit source".into());}
                if native_token {
                    if code=="TOKEN_ADVANTAGE" {
                        if !crate::advantage::reviewed_entry(entry) {return Err("native Advantage source/metadata differs from reviewed adapter".into());}
                    }else if code=="TOKEN_THE_FORCE" {
                        if !reviewed_force_entry(entry) {return Err("native Force source/metadata differs from reviewed adapter".into());}
                    }else {
                    let expected_hash = if code == "TOKEN_SHIELD" {
                        "39f1c6699c36ba309a8fe8e053f9b94ddbb8877991767b6664fcd88d447be91f"
                    } else if code=="TOKEN_EXPERIENCE" {
                        "fccf00d13061bfc15447101a0dc82fc360f9b5d2e28a9da678684e8c93bc9d67"
                    } else {"dd546e4886757d77fffeb8be6e7381144fa7378b6493ce444aa87e4ba1202778"};
                    if entry["sourceHash"] != expected_hash {
                        return Err("native token source changed; adapter requires review".into());
                    }
                    let bonus = if code == "TOKEN_SHIELD" { 0 } else if code=="TOKEN_EXPERIENCE" {1} else {-1};
                    for field in ["power", "hp", "upgradePower", "upgradeHp"] {
                        if entry["card"][field] != bonus {
                            return Err(format!(
                                "native token {field} differs from reviewed adapter"
                            ));
                        }
                    }
                    let expected = if code == "TOKEN_SHIELD" {
                        "shield"
                    } else if code=="TOKEN_EXPERIENCE" {
                        "experience"
                    } else {"weakness"};
                    if code=="TOKEN_SHIELD"&&!reviewed_shield_entry(entry) {return Err("native Shield metadata differs from reviewed adapter".into());}
                    if code=="TOKEN_EXPERIENCE"&&!reviewed_experience_entry(entry) {return Err("native Experience metadata differs from reviewed adapter".into());}
                    if code=="TOKEN_WEAKNESS" && !reviewed_weakness_entry(entry) {return Err("native Weakness metadata differs from reviewed adapter".into());}
                    if info.internal_name != expected || !info.is("token") || !info.is("upgrade") {
                        return Err("invalid native token metadata".into());
                    }
                    }
                } else if info.is("base") {
                    // Pinned DataVault/ThermalOscillator are plain BaseCard
                    // subclasses whose source comment reads "no implementation
                    // here, the 'implementation' is in DeckValidator": their
                    // printed minimum-deck-size sentence is a deck-construction
                    // rule, not an in-game ability, and a position is a caller-
                    // supplied deck. Admit exactly those printed sentences on a
                    // base with no registered programs; any other non-plain base
                    // still needs the reviewed base-zone ongoing source.
                    fn construction_only(text: &str) -> bool {
                        matches!(
                            text,
                            "Your minimum deck size is increased by 10 cards."
                                | "Your minimum deck size is decreased by 5 cards."
                        )
                    }
                    let text = info.text.as_deref().unwrap_or("");
                    let construction_only =
                        programs.card.is_empty() && construction_only(text.trim());
                    if !info.hp.is_some_and(|n| (1..=1000).contains(&n))
                        || ((!text.trim().is_empty()||!programs.card.is_empty())&&base_zone_source(entry)!=Some(Source::Base)&&!construction_only)
                    {
                        return Err("only plain bases with bounded positive HP execute".into());
                    }
                } else if info.is("event") {
                    if programs.card.len() != 1
                        || catalogue.programs[programs.card[0] as usize].trigger != "event"
                    {
                        return Err("event needs exactly one event program".into());
                    }
                } else if info.is("upgrade") {
                    if entry["baseClass"]!="UpgradeCard" || entry["card"]["pilotText"].as_str().is_some_and(|v|!v.trim().is_empty()) {return Err("pilot/base upgrade execution is not implemented".into());}
                    for &p in &programs.card {
                        let program=&catalogue.programs[p as usize];
                        if !["constant","whenPlayed"].contains(&program.trigger.as_str()) && !((program.gained_host==Some(true)&&crate::ability_ir::gained_host_upgrade_kind(entry).is_some())||program.attached_host) {return Err("upgrade trigger requires dedicated framework event support".into());}
                        if program.trigger=="constant" && program.attached_context.as_deref()!=Some("host") && !crate::ability_ir::parent_defense_upgrade_entry(entry) && program.instructions.iter().any(|i|match i {Instruction::Apply(Effect::GainKeyword {target:Target::Name(n),..}|Effect::GainTrait{target:Target::Name(n),..}|Effect::LoseTrait{target:Target::Name(n),..})=>n!="parent",Instruction::Apply(Effect::CardRestriction {target:Target::Name(n),duration,condition:None,..})=>n!="parent"||duration!="persistent",Instruction::EndStep=>false,_=>true}) {return Err("upgrade constants require attached keyword grants".into());}
                    }
                    for field in ["upgradePower","upgradeHp"] {if !entry["card"][field].as_i64().is_some_and(|n|(-1000..=1000).contains(&n)) {return Err("upgrade printed stats must be bounded integers".into());}}
                } else if !info.is("unit") && !info.is("leader") {
                    return Err("runtime card type is not implemented".into());
                }
                // Coordinate instances carry independent ability definitions. The
                // bounded loader supports exactly one printed card-side instance,
                // and inventory may rely on metadata only after this agreement.
                let coordinate_names=entry["card"]["keywords"].as_array().map_or(0,|names|names.iter().filter(|n|*n=="coordinate").count());
                let coordinate_programs=programs.card.iter().chain(&programs.unit).chain(&programs.leader).filter(|&&id|catalogue.programs[id as usize].coordinate).count();
                let coordinate_value=entry["keywordValues"].get("coordinate");
                if coordinate_names>1||coordinate_names!=coordinate_programs||if coordinate_names==0 {coordinate_value.is_some()}else{coordinate_value.and_then(Value::as_u64)!=Some(1)} {return Err("Coordinate printed metadata, keyword value and registration count must agree at zero or one".into());}
                let mut flags = 0;
                let mut raid = 0;
                let mut restore = 0;
                for keyword in entry["card"]["keywords"].as_array().into_iter().flatten() {
                    let name = keyword.as_str().ok_or("invalid keyword")?;
                    if name=="coordinate" {
                        if entry["keywordValues"][name].as_u64()!=Some(1) || programs.card.iter().filter(|&&id|catalogue.programs[id as usize].coordinate).count()!=1 || info.is("leader") {return Err("unimplemented printed Coordinate registration".into());}
                        continue;
                    }
                    if !crate::ability_ir::CORE_KEYWORDS.contains(&name) { return Err(format!("unimplemented runtime keyword {name}")); }
                    // Provider metadata includes keyword mentions in abilities.
                    // Only the compiler's engine-matching printed-text parse
                    // establishes an intrinsic keyword instance.
                    if entry["keywordValues"].get(name).is_none() { continue; }
                    if !["raid", "restore"].contains(&name) && entry["keywordValues"][name].as_u64() != Some(1) { return Err(format!("invalid printed boolean keyword {name}")); }

                    match name {
                        "sentinel" => flags |= SENTINEL,
                        "overwhelm" => flags |= OVERWHELM,
                        "saboteur" => flags |= SABOTEUR,
                        "ambush" => flags |= AMBUSH,
                        "grit" => flags |= GRIT,
                        "shielded" => flags |= SHIELDED,
                        "hidden" => flags |= 1 << 8,
                        "support" => flags |= SUPPORT,
                        "raid" | "restore" => {
                            let amount = entry["keywordValues"][name]
                                .as_u64()
                                .filter(|n| *n <= 1000)
                                .ok_or_else(|| format!("missing bounded keyword amount {name}"))?
                                as u16;
                            if name == "restore" && amount == 0 { return Err("zero printed Restore requires verified nullable trigger titles".into()); }
                            if name == "raid" {
                                flags |= 1 << 6;
                                raid = amount;
                            } else {
                                flags |= 1 << 7;
                                restore = amount;
                            }
                        }
                        _ => return Err(format!("unimplemented runtime keyword {name}")),
                    }
                }
                for side in [&programs.card, &programs.leader, &programs.unit] {
                    if side.len() > 16 {
                        return Err("card has more than 16 abilities on one side".into());
                    }
                }
                if info.is("leader") && programs.leader.iter().any(|p| catalogue.programs[*p as usize].trigger == "constant")&&base_zone_source(entry)!=Some(Source::Leader) {
                    return Err("leader-front constants require verified base-zone ongoing sources".into());
                }
                // A leader's deployment trigger requires a separate registration; never silently drop it.
                if info.is("leader")
                    && programs
                        .unit
                        .iter()
                        .any(|p| catalogue.programs[*p as usize].trigger == "whenPlayed")
                {
                    return Err("leader deployment triggers require whenDeployed semantics".into());
                }
                let mut aspects = [0; 6];
                for a in info.aspects() {
                    aspects[aspect(a)?] += 1;
                }
                if (info.is("unit") || info.is("leader"))
                    && !matches!(info.arena.as_deref(), Some("ground" | "space"))
                {
                    return Err("unit is missing arena".into());
                }
                Ok(Definition {
                    code: code.clone(),
                    info,
                    flags,
                    raid,
                    restore,
                    aspects,
                    programs,
                    traits: entry["card"]["traits"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(Value::as_str)
                        .map(str::to_owned)
                        .collect(),
                })
            })();
            match result {
                Ok(d) => {
                    let id = definitions.len() as u16;
                    if code == "TOKEN_SHIELD" {
                        shield_def = Some(id);
                    }
                    if code == "TOKEN_EXPERIENCE" {
                        experience_def = Some(id);
                    }
                    if code=="TOKEN_WEAKNESS" {weakness_def=Some(id);}
                    if code=="TOKEN_ADVANTAGE" {advantage_def=Some(id);}
                    if code=="TOKEN_THE_FORCE" {force_def=Some(id);}
                    catalogue.rejected.remove(&code);
                    catalogue
                        .metadata
                        .insert(code.clone(), entry["card"].clone());
                    catalogue.cards.entry(code.clone()).or_default();
                    by_code.insert(code, id);
                    definitions.push(d);
                }
                Err(reason) => {
                    catalogue.cards.remove(&code);
                    catalogue.metadata.remove(&code);
                    catalogue.rejected.insert(code, reason);
                }
            }
        }
        // fail closed: a card whose programs create a token that was not admitted (Experience, unit tokens) is rejected
        let mut lacking: Vec<_> = by_code.keys().filter_map(|code| Some((code.clone(), missing_token(&catalogue, &requirements, code, |t| by_code.contains_key(t))?))).collect();
        lacking.sort();
        for (code, token) in lacking {
            by_code.remove(&code);
            catalogue.cards.remove(&code);
            catalogue.metadata.remove(&code);
            catalogue.rejected.insert(code, format!("creates {token}, which is not admitted"));
        }
        // Native registration is not a compiler conversion. The exact reviewed
        // two-class token uses one shared mandatory scalar defeat program.
        let advantage_program=if advantage_def.is_some() {
            let ability:crate::ability_ir::Ability=serde_json::from_value(json!({"trigger":"whenAttackEnds","title":"Defeat Advantage token","costs":[],"effect":{"kind":"defeat","target":"self"}})).map_err(|e|e.to_string())?;
            let program=Program::compile(&ability)?;let index=catalogue.programs.len();
            if index>u16::MAX as usize {return Err("native token program capacity exceeded".into());}
            catalogue.programs.push(program);Some(index as u16)
        }else {None};
        Ok(Self {
            pool_hash: format!(
                "{:x}",
                Sha256::digest(serde_json::to_vec(value).map_err(|e| e.to_string())?)
            ),
            definitions,
            by_code,
            shield_def: shield_def.ok_or("catalogue must include native Shield token")?,
            experience_def,
            weakness_def,
            advantage_def,
            advantage_program,
            force_def,
            rules_backend: RulesBackend::Auto,
            rules_instrumentation: false,
            rules_hits: Default::default(),
            expression_hits: Default::default(),
            catalogue: Some(catalogue),
        })
    }
    pub fn from_catalogue_bytes(bytes: &[u8]) -> Result<Self, String> {
        Self::from_catalogue_value(&serde_json::from_slice(bytes).map_err(|e| e.to_string())?)
    }
    /// Export the admitted subset for the independent engine's pool-restricted oracle.
    pub fn compiled_pool(&self) -> Result<Value, String> {
        let catalogue = self
            .catalogue
            .as_ref()
            .ok_or("engine has no compiled catalogue")?;
        let mut codes: Vec<_> = catalogue.metadata.keys().collect();
        codes.sort();
        let cards: Vec<_> = codes
            .into_iter()
            .map(|code| {
                let mut card = catalogue.metadata[code].clone();
                card["setCode"] = json!(code);
                card
            })
            .collect();
        Ok(
            json!({"schema":"omni-rs-pool-v1","description":"Cards admitted by the bounded generated-IR runtime; admission is not differential validation.","cards":cards}),
        )
    }
    /// Bind generated programs to an independently exported pool and its exact bytes.
    pub fn from_compiled_pool_bytes(
        pool_bytes: &[u8],
        catalogue_bytes: &[u8],
    ) -> Result<Self, String> {
        let pool: Value = serde_json::from_slice(pool_bytes).map_err(|e| e.to_string())?;
        if pool["schema"] != "omni-rs-pool-v1" {
            return Err("unsupported pool schema".into());
        }
        let mut engine = Self::from_catalogue_bytes(catalogue_bytes)?;
        let catalogue = engine.catalogue.as_mut().unwrap();
        let requirements: Vec<_> = catalogue.programs.iter().map(TokenRequirements::for_program).collect();
        let mut included = std::collections::HashSet::new();
        for card in pool["cards"].as_array().ok_or("pool needs cards array")? {
            let code = card["setCode"]
                .as_str()
                .ok_or("pool card missing setCode")?;
            if !included.insert(code.to_owned()) {
                return Err(format!("duplicate pool card {code}"));
            }
            let approved = catalogue.metadata.get(code).ok_or_else(|| {
                format!(
                    "unsupported pool card {code}: {}",
                    catalogue
                        .rejected
                        .get(code)
                        .map(String::as_str)
                        .unwrap_or("not in compiled catalogue")
                )
            })?;
            for field in [
                "title",
                "subtitle",
                "internalName",
                "cost",
                "hp",
                "power",
                "text",
                "deployBox",
                "epicAction",
                "unique",
                "upgradePower",
                "upgradeHp",
                "aspects",
                "keywords",
                "types",
                "arena",
                "traits",
                "backSideTraits",
                "pilotText",
            ] {
                if card[field] != approved[field] {
                    return Err(format!(
                        "pool card {code}: {field} differs from compiled catalogue"
                    ));
                }
            }
        }
        if !included.contains("TOKEN_SHIELD") {
            return Err("pool must include TOKEN_SHIELD".into());
        }
        for (token, reason) in [
            ("TOKEN_ADVANTAGE","pool containing GiveAdvantage cards must include reviewed TOKEN_ADVANTAGE"),
            ("TOKEN_WEAKNESS", "pool containing GiveWeakness cards must include reviewed TOKEN_WEAKNESS"),
            ("TOKEN_THE_FORCE", "pool containing Force gain/use/presence cards must include reviewed TOKEN_THE_FORCE"),
            ("TOKEN_SPY", "pool containing Spy creation must include reviewed TOKEN_SPY"),
            ("TOKEN_BEAST", "pool containing Beast creation must include reviewed TOKEN_BEAST"),
            ("TOKEN_EXPERIENCE", "pool containing Experience creation must include reviewed TOKEN_EXPERIENCE"),
        ] {
            if !included.contains(token) && included.iter().any(|code| catalogue.cards.get(code)
                .is_some_and(|programs| card_needs_token(programs, &requirements, token))) {
                return Err(reason.into());
            }
        }
        // a pool must hold the tokens its cards create, or a game could put a card in play the pool lacks
        let mut codes: Vec<_> = included.iter().collect();
        codes.sort();
        for code in codes {
            if let Some(token) = missing_token(catalogue, &requirements, code, |t| included.contains(t)) {
                return Err(format!("pool card {code} creates {token}, which the pool omits"));
            }
        }
        catalogue.metadata.retain(|code, _| included.contains(code));
        engine.pool_hash = format!("{:x}", Sha256::digest(pool_bytes));
        Ok(engine)
    }

    pub fn rejected_cards(&self) -> Option<&std::collections::BTreeMap<String, String>> {
        self.catalogue.as_ref().map(|c| &c.rejected)
    }
    pub(super) fn token_count(&self, s: &State, mut id: CardId) -> u16 {
        let mut count = 0;
        while id != 0 {
            count += 1;
            id = s.card(id).next_token;
        }
        count
    }
    pub(super) fn check_stat_capacity(&self, s: &State) -> Result<(), String> {
        self.check_keyword_copy_graph(s)?;
        for id in 1..s.next_id {
            if !self.in_arena(s, id) {
                continue;
            }
            let d = self.def(s, id);
            for keyword in ["raid", "restore"] {
                if !(0..=65535).contains(&self.raw_keyword_amount(s, id, keyword)) { return Err(format!("computed {keyword} exceeds bounded keyword capacity")); }
            }
            let (constant_power, constant_hp) = self.constant_stats(s, id);
            let (attack_power, attack_hp) = self.attack_stat_bonus(s, id);
            let power = d.info.power.unwrap_or(0) as i64
                + self.token_count(s, s.card(id).experience) as i64
                + s.card(id).phase_power as i64
                + constant_power + self.upgrade_stats(s,id).0 + self.native_attack_power(s,id)
                + attack_power
                + if self.keyword_amount(s, id, "grit") > 0 {
                    s.card(id).damage as i64
                } else {
                    0
                }
                + self.keyword_amount(s, id, "raid") as i64;
            let hp = d.info.hp.unwrap_or(0) as i64
                + self.token_count(s, s.card(id).experience) as i64
                + s.card(id).phase_hp as i64
                + constant_hp + self.upgrade_stats(s,id).1
                + attack_hp;
            if power > 65535 || hp > 65535 {
                return Err("computed stats exceed compact capacity".into());
            }
        }
        Ok(())
    }
    pub(super) fn sweep_stat_defeats(&self, s: &mut State) -> Result<(), String> {
        // The uniqueness prompt precedes the engine game-state resolution.
        // Even a newly entered lethal copy remains selectable until it resolves.
        if s.decision == Decision::Unique {
            return Ok(());
        }
        loop {
            self.check_stat_capacity(s)?;
            let dead:Vec<_>=self.arena_units(s).into_iter().filter(|id|s.card(*id).damage>=self.hp(s,*id)).collect();
            if dead.is_empty() {
                return Ok(());
            }
            self.defeat_many(s,&dead)?;
        }
    }
    #[cfg(test)]
    fn constant_effects<'a>(&'a self,s:&State,target:CardId,keywords:bool)->Vec<(CardId,&'a Effect)> {self.constant_effects_matching(s,target,if keywords {ConstantScope::Keyword(None)}else{ConstantScope::All})}
    pub(super) fn card_restricted(&self, s: &State, id: CardId, verb: RestrictionVerb) -> bool {
        id != 0 && s.card(id).present && (s.card(id).phase_restrictions & verb.bit() != 0 || self.constant_effects_matching(s, id, ConstantScope::Restriction).iter().any(|(_, effect)| matches!(effect, Effect::CardRestriction { restriction, duration, condition: None, .. } if *restriction == verb && duration == "persistent")))
    }
    fn constant_effects_matching<'a>(&'a self, s: &State, target: CardId, scope: ConstantScope<'_>) -> Vec<(CardId, &'a Effect)> {
        // Bases sit outside the arena, but a live "can't be healed" still names them.
        let restriction_base = matches!(scope, ConstantScope::Restriction) && self.def(s, target).info.is("base");
        if self.catalogue.is_none() || (!self.in_arena(s,target) && !matches!(scope,ConstantScope::Traits(_)) && !restriction_base) {return vec![];}
        let mut effects = vec![];
        for source in 1..s.next_id {
            // Most allocated cards have no constant registration. Inspect the
            // current definition side before paying for arena/attachment scans;
            // keep activation, conditions and targets live below.
            let mut programs = self.card_programs(s,source).iter().copied()
                .chain(self.support_programs(s,source))
                .map(|id|self.program(id))
                .filter(|program|program.trigger=="constant" && program.instructions.iter().any(|i|matches!(i,Instruction::Apply(effect) if scope.accepts(effect))))
                .peekable();
            if programs.peek().is_none() { continue; }
            // Incoming unique copies have not activated their ongoing abilities:
            // the engine resolves uniqueness before resolveGameState.
            let basezone=s.players.iter().any(|p|p.base==source||(p.leader==source&&!s.card(source).deployed));
            let active_in_play=self.in_arena(s,source)||self.attached_in_play(s,source)||basezone;
            for program in programs {
                let any_zone=matches!(scope,ConstantScope::Traits(_)) && s.card(source).present && program.instructions.iter().any(|i|matches!(i,Instruction::Apply(Effect::ConstantEffectRegistration(r)) if matches!(r.native(),Ok(crate::static_ongoing::NativeModifier::Traits(crate::static_ongoing::TraitRule::LeaderUnion{..})))));
                if !(active_in_play||any_zone) || (source==s.disabled_ongoing_source && self.in_arena(s,source)) || (!any_zone && s.decision==Decision::Unique && source==s.source) {continue;}

                let effect_source=if program.attached_context.as_deref()==Some("host") {if !self.attached_in_play(s,source) {continue;} s.card(source).parent}else{source};
                if program.attached_context.is_some() && !self.attached_in_play(s,source) {continue;}
                if (program.coordinate && !self.coordinate_active(s,source))
                    || program
                        .condition
                        .as_ref()
                        .is_some_and(|c| !self.eval(s, effect_source, target, c).truth())
                {
                    continue;
                }
                let resolver = program.instructions.iter().find_map(|i| {
                    if let Instruction::Select(r) = i {
                        Some(r)
                    } else {
                        None
                    }
                });
                let parent_target=program.instructions.iter().any(|i|matches!(i,Instruction::Apply(Effect::GainKeyword {target:Target::Name(n),..}|Effect::GainTrait{target:Target::Name(n),..}|Effect::LoseTrait{target:Target::Name(n),..}|Effect::CardRestriction{target:Target::Name(n),..}) if n=="parent"));
                let friendly_ambush=program.instructions.iter().any(|i|matches!(i,Instruction::Apply(Effect::ConstantEffectRegistration(r)) if matches!(r.native(),Ok(crate::static_ongoing::NativeModifier::FriendlyAmbushBase))));
                let native_trait=program.instructions.iter().find_map(|i|match i {Instruction::Apply(Effect::ConstantEffectRegistration(r))=>match r.native(){Ok(crate::static_ongoing::NativeModifier::Traits(rule))=>Some(rule),_=>None},_=>None});
                if resolver.map_or(if let Some(rule)=native_trait {!self.native_trait_target(s,effect_source,target,rule)}else if friendly_ambush {s.card(source).owner!=s.card(target).owner}else if parent_target {s.card(source).parent!=target}else{effect_source != target}, |r| {
                    !self.matching_targets(s, effect_source, r).contains(&target)
                }) {
                    continue;
                }
                for instruction in &program.instructions {
                    if let Instruction::Apply(effect) = instruction { effects.push((effect_source, effect)); }
                }
            }
        }
        effects
    }
    fn native_trait_target(&self,s:&State,source:CardId,target:CardId,rule:&crate::static_ongoing::TraitRule)->bool {
        match rule {
            crate::static_ongoing::TraitRule::LeaderUnion{..}=>source==target,
            crate::static_ongoing::TraitRule::UnitAura{required,..}=>{
                let ordinary_unit=self.def(s,target).info.is("unit") || (self.def(s,target).info.is("leader") && s.card(target).deployed);
                // Current admitted state imports forbid foreign control. In play
                // Card.owner is its controller; in nonplay zones it is its owner.
                // No ownership history is synthesized for unsupported control.
                s.card(target).present && ordinary_unit && s.card(target).owner==s.card(source).owner && self.card_has_trait(s,target,required)
            }
        }
    }
    /// Source-only entry conditions are evaluated at the actual hand-play entry
    /// boundary. They do not invent entry events for snapshot or plain placement.
    pub(super) fn self_static_ongoing(&self, s:&State, source:CardId, kind:crate::static_ongoing::SelfEffect)->bool {
        if self.catalogue.is_none() || source==s.disabled_ongoing_source {return false;}
        self.card_programs(s,source).iter().any(|&id| {
            let p=self.program(id);
            p.trigger=="constant" && p.instructions.iter().any(|i|match i {
                Instruction::Apply(Effect::ConstantEffectRegistration(r)) => match r.native() {
                    Ok(crate::static_ongoing::NativeModifier::SelfBoolean {kind:actual,condition}) => *actual==kind && condition.as_ref().is_none_or(|c|self.native_ongoing_condition(s,source,c)),
                    _=>false,
                },_=>false,
            })
        })
    }
    fn native_ongoing_condition(&self,s:&State,source:CardId,c:&crate::static_ongoing::Condition)->bool {
        use crate::static_ongoing::Condition;
        let player=s.card(source).owner;
        match c {
            Condition::RoundEquals(round)=>s.round==*round,
            Condition::ControlledTitle(title)=>self.controlled_in_play_title(s,player,title),
            Condition::FriendlyNonUnique=>s.player(player).ground.as_slice().iter().chain(s.player(player).space.as_slice()).any(|id|!self.def(s,*id).info.unique),
            Condition::OpponentArenaEmpty(arena)=>if *arena=="ground" {s.player(player.other()).ground.len==0}else{s.player(player.other()).space.len==0},
        }
    }
    pub(super) fn ambush_can_attack_base(&self,s:&State,target:CardId)->bool {
        !self.constant_effects_matching(s,target,ConstantScope::AmbushBase).is_empty()
    }
    fn native_ongoing_count(&self,s:&State,source:CardId,c:&crate::static_ongoing::Count)->i64 {
        use crate::static_ongoing::Count;
        let player=s.card(source).owner;
        match c {
            Count::OtherFriendlySameTitle=>s.player(player).ground.as_slice().iter().chain(s.player(player).space.as_slice()).filter(|id|**id!=source&&self.def(s,**id).info.title==self.def(s,source).info.title).count() as i64,
            Count::DiscardUnitsWithTrait(trait_name)=>s.player(player).discard.as_slice().iter().filter(|id|self.def(s,**id).info.is("unit")&&self.card_has_trait(s,**id,trait_name)).count() as i64,
        }
    }
    pub(super) fn constant_stats(&self, s: &State, target: CardId) -> (i64, i64) {
        let mut total = (0, 0);
        for (source, effect) in self.constant_effects_matching(s, target, ConstantScope::Stats) {
            if let Effect::ModifyStats { power, hp, .. } = effect {
                total.0 += self.eval_live_stat(s,source,target,power).number();
                total.1 += self.eval_live_stat(s,source,target,hp).number();
            } else if let Effect::ConstantEffectRegistration(r)=effect {
                if let Ok(crate::static_ongoing::NativeModifier::StatCount(count))=r.native() {total.0+=self.native_ongoing_count(s,source,count);}
            }
        }
        total
    }
    fn raw_keyword_amount(&self, s: &State, target: CardId, keyword: &str) -> i64 {
        let Some(index) = crate::ability_ir::CORE_KEYWORDS.iter().position(|k| *k == keyword) else { return 0; };
        if s.card(target).lost_keyword_mask() & (1 << index) != 0 { return 0; }
        let definition = self.def(s, target);
        let mut total = match index {
            6 => definition.raid as i64,
            7 => definition.restore as i64,
            _ => i64::from(definition.flags & (1 << index) != 0),
        } + s.card(target).phase_keyword_amount(index) as i64 + self.support_keyword(s,target,index) as i64;
        for (source, effect) in self.constant_effects_matching(s, target, ConstantScope::Keyword(Some(keyword))) {
            match effect {
                Effect::LoseKeyword { keyword: name, .. } if name == keyword => return 0,
                Effect::GainKeyword { keyword: name, amount, .. } if name == keyword => total += self.eval(s, source, target, amount).number(),
                Effect::CopyKeywords{..}=>total+=self.raw_keyword_amount(s,source,keyword),
                _ => {}
            }
        }
        if crate::ability_ir::numeric_keyword_slot(index).is_none() { i64::from(total > 0) } else { total }
    }
    pub(super) fn keyword_amount(&self, s: &State, target: CardId, keyword: &str) -> u16 {
        self.raw_keyword_amount(s, target, keyword).clamp(0, u16::MAX as i64) as u16
    }
    /// First-strike state is either a live reviewed constant registration or an
    /// until-end-of-attack grant captured before defender selection. Both are
    /// attacker-only: the admitted source shapes all require the unit to be
    /// attacking, so a defender with this ability is never admitted.
    pub(super) fn deals_combat_damage_first(&self, s: &State, id: CardId) -> bool {
        if s.support.grants[..s.support.len as usize]
            .iter()
            .any(|g| g.target == id && g.deals_combat_damage_first)
        {
            return true;
        }
        self.constant_effects_matching(s, id, ConstantScope::CombatDamageFirst)
            .iter()
            .any(|(_, e)| matches!(e, Effect::DealsCombatDamageFirst { target: Target::Name(n), duration } if n == "self" && duration == "persistent"))
    }
    pub(super) fn has_keyword(&self, s: &State, target: CardId, keyword: &str) -> bool {
        let Some(index) = crate::ability_ir::CORE_KEYWORDS.iter().position(|k| *k == keyword) else { return false; };
        if s.card(target).lost_keyword_mask() & (1 << index) != 0 { return false; }
        let effects = self.constant_effects_matching(s, target, ConstantScope::Keyword(Some(keyword)));
        if effects.iter().any(|(_,e)| matches!(e, Effect::LoseKeyword {keyword: name, ..} if name == keyword)) { return false; }
        (s.card(target).phase_keyword_mask()|self.support_presence(s,target)) & (1 << index) != 0 || (self.def(s, target).flags & (1 << index) != 0 || (index == 6 && self.def(s, target).raid > 0) || (index == 7 && self.def(s, target).restore > 0)) || effects.iter().any(|(source,e)| matches!(e, Effect::GainKeyword {keyword: name, ..} if name == keyword)||matches!(e,Effect::CopyKeywords{..})&&self.has_keyword(s,*source,keyword))
    }
    fn keyword_inventory_len(&self,s:&State,target:CardId)->u16 {
        let mask=crate::ability_ir::CORE_KEYWORDS.iter().enumerate().fold(0u16,|m,(i,k)|m|if self.has_keyword(s,target,k){1<<i}else{0});
        let coordinate=self.catalogue.as_ref().and_then(|c|c.metadata.get(&self.def(s,target).code)).is_some_and(|m|m["keywords"].as_array().is_some_and(|names|names.iter().any(|n|n=="coordinate"))) && self.def(s,target).programs.card.iter().any(|&id|self.program(id).coordinate);
        (mask.count_ones() as u16)+u16::from(coordinate)
    }
    pub(super) fn effective_flags(&self, s: &State, target: CardId) -> u16 {
        crate::ability_ir::CORE_KEYWORDS.iter().enumerate().filter(|(i,_)| crate::ability_ir::numeric_keyword_slot(*i).is_none()).fold(0, |flags, (i, k)| flags | if self.keyword_amount(s, target, k) > 0 { 1 << i } else { 0 })
    }
    fn raw_hp(&self,s:&State,id:CardId)->i64 {self.def(s,id).info.hp.unwrap_or(0) as i64+self.token_count(s,s.card(id).experience) as i64+s.card(id).phase_hp as i64+self.constant_stats(s,id).1+self.upgrade_stats(s,id).1+self.attack_stat_bonus(s,id).1}
    pub fn hp(&self,s:&State,id:CardId)->u16 {u16::try_from(self.raw_hp(s,id).max(0)).expect("HP exceeds validated compact capacity")}
    pub(super) fn program(&self, id: u16) -> &Program {
        &self.catalogue.as_ref().expect("IR catalogue").programs[id as usize]
    }
    pub(super) fn card_programs<'a>(&'a self, s: &State, id: CardId) -> &'a [u16] {
        let d = self.def(s, id);
        if !d.info.is("leader") { &d.programs.card }
        else if s.card(id).deployed { &d.programs.unit }
        else { &d.programs.leader }
    }
    fn can_pay_force_cost(&self, s: &State, source: CardId) -> bool {
        let player = s.player(s.card(source).owner);
        s.force_state_exact && self.force_def.is_some() && player.force_id != 0 && player.force_held
    }
    pub(super) fn available_programs(&self, s: &State, id: CardId, trigger: &str) -> Vec<u16> {
        if self.catalogue.is_none() {
            return vec![];
        }
        self.card_programs(s, id)
            .iter()
            .copied()
            .filter(|p| {
                let p = self.program(*p);
                p.trigger == trigger
                    && p.condition
                        .as_ref()
                        .is_none_or(|c| self.eval(s, id, 0, c).truth())
                    && p.costs.iter().all(|c| match c {
                        Cost::ExhaustSelf => !s.card(id).exhausted,
                        Cost::Resources { .. } => true,
                        Cost::UseForceCost { .. } => self.can_pay_force_cost(s, id),
                    })
                    && p.costs
                        .iter()
                        .map(|c| match c {
                            Cost::Resources { amount } => self.eval(s, id, 0, amount).number(),
                            _ => 0,
                        })
                        .sum::<i64>()
                        <= self.ready_resources(s, s.card(id).owner) as i64
            })
            .collect()
    }
    fn active_attack_pair(&self,s:&State,id:CardId)->(CardId,CardId) {
        if s.attack_source!=0 && (s.attack_source==id||s.attack_defender==id) {(s.attack_source,s.attack_defender)}else{s.death_frames[..s.death_depth as usize].iter().rev().find(|f|f.context.attack_source!=0&&(f.context.attack_source==id||f.context.attack_defender==id)).map(|f|(f.context.attack_source,f.context.attack_defender)).unwrap_or((0,0))}
    }
    /// The attack currently resolving, independent of which card owns the
    /// observing ability. `getAllAttackTargets` reads the event's attack.
    fn current_attack_pair(&self,s:&State)->(CardId,CardId) {
        if s.attack_source!=0 {return (s.attack_source,s.attack_defender);}
        s.death_frames[..s.death_depth as usize].iter().rev().find(|f|f.context.attack_source!=0).map(|f|(f.context.attack_source,f.context.attack_defender)).unwrap_or((0,0))
    }
    pub(super) fn attack_participant_departed(&self,s:&State,id:CardId)->bool {
        if s.attack_source!=0 && (s.attack_source==id || s.attack_defender==id) {return if s.attack_source==id {s.attack_attacker_departed}else{s.attack_defender_departed};}
        s.death_frames[..s.death_depth as usize].iter().rev().find(|f|f.context.attack_source!=0 && (f.context.attack_source==id || f.context.attack_defender==id)).is_some_and(|f|if f.context.attack_source==id {f.context.attack_attacker_departed}else{f.context.attack_defender_departed})
    }
    pub(super) fn is_attacking(&self,s:&State,id:CardId)->bool {self.in_arena(s,id)&&id==self.active_attack_pair(s,id).0&&!self.attack_participant_departed(s,id)}
    fn is_defending(&self,s:&State,id:CardId)->bool {(self.in_arena(s,id)||self.def(s,id).info.is("base"))&&id==self.active_attack_pair(s,id).1&&!self.attack_participant_departed(s,id)}
    fn evaluated_power(&self,s:&State,id:CardId)->i64 {
        if self.def(s,id).info.is("upgrade") {self.def(s,id).info.upgrade_power.unwrap_or(0) as i64}
        else {self.raw_power(s,id,self.is_attacking(s,id)).max(0)}
    }
    fn evaluated_hp(&self,s:&State,id:CardId)->i64 {
        if self.def(s,id).info.is("upgrade") {self.def(s,id).info.upgrade_hp.unwrap_or(0) as i64}
        else {self.raw_hp(s,id).max(0)}
    }
    pub(super) fn attachment_condition(&self,s:&State,source:CardId,parent:CardId,e:&Expr)->bool {self.eval(s,source,parent,e).truth()}
    pub(super) fn selected_definition_any(&self, mut predicate: impl FnMut(&Definition) -> bool) -> bool {
        match &self.catalogue {
            Some(catalogue) => catalogue.metadata.keys().any(|code| self.by_code.get(code)
                .is_some_and(|&id| predicate(&self.definitions[id as usize]))),
            None => self.definitions.iter().any(predicate),
        }
    }
    pub(super) fn needs_phase_history(&self)->bool {
        fn expression(e:&Expr)->bool {match e.original() {
            Expr::HistoryQuery(_)|Expr::DataMember{..}=>true,
            Expr::Binary{left,right,..}=>expression(left)||expression(right),
            _=>false,
        }}
        self.catalogue.is_some() && self.selected_definition_any(|definition| {
            let p = &definition.programs;
            p.card.iter().chain(&p.leader).chain(&p.unit).any(|&id|self.program(id).instructions.iter().any(|i|match i {Instruction::Conditional{condition,..}=>expression(condition),Instruction::Select(r)=>r.predicate.as_ref().is_some_and(expression),_=>false}))
        })
    }
    fn history_boolean(&self,s:&State,source:CardId,card:CardId,q:&crate::phase_history::Query)->bool {
        use crate::model::PhaseHistory;use crate::phase_history::QueryKind;
        assert!(s.phase_history.exact,"phase history must be exact before execution");
        let player=s.card(source).owner;
        match q.admitted_kind() {
            QueryKind::EnteredCard => [Seat::P1,Seat::P2].into_iter().any(|seat|PhaseHistory::contains(&s.phase_history.entered,seat,card)),
            QueryKind::EnteredOtherFriendly => (1..s.next_id).any(|id|id!=source&&PhaseHistory::contains(&s.phase_history.entered,player,id)),
            QueryKind::PlayedForce => (1..s.next_id).any(|id|PhaseHistory::contains(&s.phase_history.played,player,id)&&self.card_has_trait(s,id,"force")),
            QueryKind::PlayedOpponentCount => unreachable!("history list requires length consumer"),
        }
    }
    fn eval(&self, s: &State, source: CardId, target: CardId, e: &Expr) -> Eval {
        self.eval_with_card(s, source, target, target, e)
    }
    fn eval_live_stat(&self,s:&State,source:CardId,affected:CardId,e:&Expr)->Eval {self.eval_with_binding(s,source,affected,affected,affected,e)}
    fn eval_with_card(&self,s:&State,source:CardId,target:CardId,card:CardId,e:&Expr)->Eval {self.eval_with_binding(s,source,target,card,0,e)}
    fn eval_with_binding(
        &self,
        s: &State,
        source: CardId,
        target: CardId,
        card: CardId,
        modifier_target:CardId,
        e: &Expr,
    ) -> Eval {
        if let Expr::Compiled(compiled) = e {
            let generated = self.rules_backend != RulesBackend::Interpreter;
            if self.rules_instrumentation {
                self.expression_hits.lock().expect("expression metrics lock")[usize::from(!generated)] += 1;
            }
            return if generated {
                self.generated_expression(s, source, target, card, modifier_target, compiled.index)
            } else {
                self.eval_with_binding(s, source, target, card, modifier_target, &compiled.original)
            };
        }
        let player = s.card(source).owner;
        let sub = |e| self.eval_with_binding(s, source, target, card, modifier_target,e);
        match e {
            Expr::Compiled(_) => unreachable!("compiled expression handled above"),
            Expr::HistoryQuery(q) => Eval::Bool(self.history_boolean(s,source,card,q)),
            Expr::DataMember {receiver:crate::phase_history::List::HistoryQuery(_),..} => {assert!(s.phase_history.exact,"phase history must be exact before execution");Eval::Number(s.phase_history.play_counts[player.other().index()] as i64)},
            Expr::HistoryAttackCheck {..}=>Eval::Bool(s.attack_source!=0 && s.attack_defender_defeated),
            Expr::ArenaList {receiver,predicate} => {let seat=match receiver.as_ref(){Expr::Ref{path} if path==&["player"]=>Some(player),Expr::Ref{path} if path==&["opponent"]=>Some(player.other()),_=>None};let ids=self.arena_units(s).into_iter().filter(|id|self.in_arena(s,*id)&&seat.is_none_or(|p|s.card(*id).owner==p)&&self.eval_with_binding(s,source,target,*id,modifier_target,predicate).truth()).collect();Eval::ArenaCards(ids)},
            Expr::ArenaFilter {receiver,predicate} => {let Eval::ArenaCards(ids)=sub(receiver) else{return Eval::Null;};Eval::ArenaCards(ids.into_iter().filter(|id|self.eval_with_binding(s,source,target,*id,modifier_target,predicate).truth()).collect())},
            Expr::ArenaSome {receiver,predicate} => {let Eval::ArenaCards(ids)=sub(receiver) else{return Eval::Null;};Eval::Bool(ids.into_iter().any(|id|self.eval_with_binding(s,source,target,id,modifier_target,predicate).truth()))},
            Expr::ArenaLength {receiver} => {let Eval::ArenaCards(ids)=sub(receiver) else{return Eval::Null;};Eval::Number(ids.len() as i64)},
            Expr::NativeTokenPredicate {receiver,token_type,..} => Eval::Bool(match sub(receiver) {Eval::Card(id)=>self.attached_native_token(s,id,token_type),_=>false}),
            Expr::Collection {name,receiver,args,..} => {
                let inventory=sub(receiver);
                let needle=(name=="includes").then(||sub(&args[0]));
                evaluate_collection(inventory,name,|id|self.eval_with_binding(s,source,target,id,modifier_target,&args[0]),needle)
            },
            Expr::Array { items } => Eval::Array(items.iter().map(sub).collect()),
            Expr::Math { name, args } => {
                if let Some((numerator,divisor))=e.bounded_floor_division() {return Eval::Number(sub(numerator).number()/divisor);}
                let values: Vec<i64> = args.iter().map(|a| sub(a).number()).collect();
                Eval::Number(if name == "min" {
                    values.into_iter().min().unwrap_or(0)
                } else {
                    values.into_iter().max().unwrap_or(0)
                })
            }
            Expr::Const { value } => {
                if let Some(n) = value.as_i64() {
                    Eval::Number(n)
                } else if let Some(b) = value.as_bool() {
                    Eval::Bool(b)
                } else if let Some(t) = value.as_str() {
                    Eval::Text(t.into())
                } else {
                    Eval::Null
                }
            }
            Expr::Ref { path } => {
                let root = path.first().map(String::as_str).unwrap_or("");
                let field = path.get(1).map(String::as_str);
                if root == "event" {
                    return match path.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
                        ["event","lastKnownInformation","power"] if s.last_known_source==source=>Eval::Number(s.last_known_power as i64),
                        ["event","damageDealt"]=>Eval::Number(s.event_result.damage_dealt as i64),
                        ["event","totalDistributed"]=>Eval::Number(s.ability_result as i64),_=>Eval::Null,
                    };
                }
                if matches!(root, "player" | "opponent") {
                    let p = if root == "player" {
                        player
                    } else {
                        player.other()
                    };
                    return match field {
                        None => Eval::Player(p),
                        Some("base") => {
                            if path.get(2).map(String::as_str) == Some("damage") {
                                Eval::Number(s.card(s.player(p).base).damage as i64)
                            } else {
                                Eval::Card(s.player(p).base)
                            }
                        }
                        Some("hasTheForce") if root=="player" && path.len()==2 => Eval::Bool(s.force_state_exact && s.player(p).force_held),
                        Some("readyResourceCount") => {
                            Eval::Number(self.ready_resources(s, p) as i64)
                        }
                        Some("exhaustedResourceCount") => Eval::Number(
                            s.player(p).resources.len as i64 - self.ready_resources(s, p) as i64,
                        ),
                        Some("resources") if path.len()==2 => Eval::Array(s.player(p).resources.as_slice().iter().copied().map(Eval::Card).collect()),
                        Some("resources") => Eval::Number(s.player(p).resources.len as i64),
                        Some("hand") if path.len()==2 => Eval::Array(s.player(p).hand.as_slice().iter().copied().map(Eval::Card).collect()),
                        Some("hand") => Eval::Number(s.player(p).hand.len as i64),
                        Some("drawDeck") if path.len()==2 => Eval::Array(s.player(p).deck.as_slice().iter().copied().map(Eval::Card).collect()),
                        Some("drawDeck") => Eval::Number(s.player(p).deck.len as i64),
                        Some("discard") if path.len()==2 => Eval::Array(s.player(p).discard.as_slice().iter().copied().map(Eval::Card).collect()),
                        Some("discard") => Eval::Number(s.player(p).discard.len as i64),
                        _ => Eval::Null,
                    };
                }
                if root=="snapshot" {return match field{Some("power")=>Eval::Number(s.maximum_capture.power as i64),Some("count")=>Eval::Number(s.maximum_capture.count as i64),Some("sole")=>Eval::Card(s.maximum_capture.sole),_=>Eval::Null};}
                if root=="modifierTarget" {
                    assert!(modifier_target!=0,"modifierTarget unavailable outside persistent stat evaluation");
                    return match field {None=>Eval::Card(modifier_target),Some("damage")=>Eval::Number(s.card(modifier_target).damage as i64),Some("controller") if path.len()==2=>Eval::Player(s.card(modifier_target).owner),Some("controller") if path.iter().map(String::as_str).collect::<Vec<_>>()==["modifierTarget","controller","resources","length"]=>Eval::Number(s.player(s.card(modifier_target).owner).resources.len as i64),_=>Eval::Null};
                }
                if path.iter().map(String::as_str).collect::<Vec<_>>()==["card","keywords","length"] {return Eval::Number(if card==0 {0}else{self.keyword_inventory_len(s,card) as i64});}
                if path.iter().map(String::as_str).collect::<Vec<_>>()==["card","activeAttack","attacker"] {return Eval::Card(self.active_attack_pair(s,card).0);}
                if root=="source" && matches!(field,Some("parentCard"|"parentUnit")) {
                    let parent=s.card(source).parent;
                    // Compiler permits parent access only on ordinary attached upgrades.
                    // Source registrations are evaluated while attachment is valid.
                    assert!(parent!=0 && self.in_arena(s,parent),"parentCard/parentUnit requires a live attached source");
                    if let Some(property)=path.get(2) {return self.eval_with_card(s,source,target,parent,&Expr::Ref {path:vec!["card".into(),property.clone()]});}
                    return Eval::Card(parent);
                }
                let id = if root == "source" {
                    source
                } else if root == "card" {
                    card
                } else {
                    target
                };
                if id == 0 {
                    return Eval::Null;
                }
                match field {
                    None => Eval::Card(id),
                    Some("upgrades") => Eval::Array(self.upgrades(s,id).into_iter().map(Eval::Card).collect()),
                    Some("controller") => Eval::Player(s.card(id).owner),
                    Some("damage") => Eval::Number(s.card(id).damage as i64),
                    Some("power") => {
                        Eval::Number(
                            self.evaluated_power(s,id)
                        )
                    }
                    Some("hp") => Eval::Number(self.evaluated_hp(s,id)),
                    Some("remainingHp") => {
                        Eval::Number(self.hp(s, id).saturating_sub(s.card(id).damage) as i64)
                    }
                    Some("title") => Eval::Text(self.def(s, id).info.title.clone()),
                    Some("cost" | "printedCost") => Eval::Number(self.def(s, id).info.cost() as i64),
                    Some("exhausted") => Eval::Bool(s.card(id).exhausted),
                    Some("zoneName") => {
                        if self.attached_in_play(s,id) {return Eval::Text(if self.def(s,s.card(id).parent).info.arena.as_deref()==Some("space") {"spaceArena"}else{"groundArena"}.into());}
                        let p = s.player(s.card(id).owner);
                        let zone = if p.ground.contains(id) {
                            "groundArena"
                        } else if p.space.contains(id) {
                            "spaceArena"
                        } else if p.hand.contains(id) {
                            "hand"
                        } else if p.discard.contains(id) {
                            "discard"
                        } else if p.resources.contains(id) {
                            "resource"
                        } else if p.deck.contains(id) {
                            "deck"
                        } else if p.base == id {
                            "base"
                        } else {
                            "leader"
                        };
                        Eval::Text(zone.into())
                    }
                    Some("unique") => Eval::Bool(self.def(s, id).info.unique),
                    _ => Eval::Null,
                }
            }
            Expr::Binary { op, left, right } => {
                let a = sub(left);
                if op == "??" {
                    // Only the exact parent-upgrade operational title admits this.
                    return if matches!(a,Eval::Null) {sub(right)} else {a};
                }
                if op == "&&" && !a.truth() {
                    return a;
                }
                if op == "||" && a.truth() {
                    return a;
                }
                let b = sub(right);
                match op.as_str() {
                    "+" => Eval::Number(a.number().saturating_add(b.number())),
                    "-" => Eval::Number(a.number().saturating_sub(b.number())),
                    "*" => Eval::Number(a.number().saturating_mul(b.number())),
                    "/" => Eval::Number(a.number().checked_div(b.number()).unwrap_or(0)),
                    "%" => Eval::Number(a.number().checked_rem(b.number()).unwrap_or(0)),
                    "===" | "==" => Eval::Bool(a == b),
                    "!==" | "!=" => Eval::Bool(a != b),
                    "<" => Eval::Bool(a.number() < b.number()),
                    "<=" => Eval::Bool(a.number() <= b.number()),
                    ">" => Eval::Bool(a.number() > b.number()),
                    ">=" => Eval::Bool(a.number() >= b.number()),
                    "&&" | "||" => b,
                    _ => Eval::Null,
                }
            }
            Expr::Unary { op, value } => {
                let v = sub(value);
                if op == "!" {
                    Eval::Bool(!v.truth())
                } else {
                    Eval::Number(v.number().saturating_neg())
                }
            }
            Expr::Conditional {
                condition,
                then,
                r#else,
            } => {
                if sub(condition).truth() {
                    sub(then)
                } else {
                    sub(r#else)
                }
            }
            Expr::Query {
                name,
                receiver,
                args,
            } => {
                if name=="getAllAttackTargets" {
                    // `context.event.attack.getAllTargets()`: the attack's captured
                    // target list. Kessel admits one target per attack, and the
                    // pinned lasting-effect system's own canAffect filter accepts
                    // units only, so a unit still in play is the complete list.
                    let (_,defender)=self.current_attack_pair(s);
                    return Eval::Array(if defender!=0 && self.in_arena(s,defender) {vec![Eval::Card(defender)]}else{vec![]});
                }
                if name=="leaderTraits" {
                    let Eval::Player(player)=sub(receiver) else {return Eval::Null;};
                    let mut traits=std::collections::BTreeSet::new();
                    for id in 1..s.next_id {
                        if s.card(id).present&&s.card(id).owner==player&&self.def(s,id).info.is("leader")&&(self.in_arena(s,id)||self.attached_in_play(s,id)||s.players.iter().any(|p|p.leader==id&&!s.card(id).deployed)) {
                            traits.extend(self.card_traits(s,id));
                        }
                    }
                    return Eval::Array(traits.into_iter().map(Eval::Text).collect());
                }
                if name==crate::live_power_maximum::DISPLAY_QUERY {
                    let Eval::Player(owner)=sub(receiver) else{return Eval::Null};
                    let maximum=s.player(owner).ground.as_slice().iter().chain(s.player(owner).space.as_slice()).map(|&id|self.evaluated_power(s,id)).max();
                    return Eval::Text(maximum.map_or_else(||"-Infinity".into(),|value|value.to_string()));
                }
                if crate::bounded_reductions::QUERIES.contains(&name.as_str()) {
                    let Eval::Player(owner)=sub(receiver) else{return Eval::Null};
                    let values=s.player(owner).ground.as_slice().iter().chain(s.player(owner).space.as_slice()).map(|&id|match name.as_str(){"arenaDamageSum"=>s.card(id).damage as i64,"arenaUpgradeSum"=>self.upgrades(s,id).len() as i64,"arenaPowerMax"=>self.evaluated_power(s,id),_=>unreachable!()});
                    return Eval::Number(if name=="arenaPowerMax" {values.fold(0,i64::max)}else{values.sum()});
                }
                if name=="resolvedDefeatedUnitCount" {return if let Eval::Player(p)=sub(receiver){Eval::Number(s.event_result.defeated_units[p.index()] as i64)}else{Eval::Null};}
                if name=="hasControlledInPlayTrait" {return if let (Eval::Player(p),Eval::Text(trait_name),Eval::Card(excluded))=(sub(receiver),sub(&args[0]),sub(&args[1])){Eval::Bool(self.controlled_in_play_trait(s,p,&trait_name,excluded))}else{Eval::Null};}
                if name=="hasControlledInPlayTitle" {
                    return if let Eval::Player(p)=sub(receiver) {Eval::Bool(self.controlled_in_play_title(s,p,if let Expr::Const{value}=&args[0] {value.as_str().unwrap_or("")} else {""}))} else {Eval::Null};
                }
                if name == "hasSomeArenaUnit" || name == "hasSomeArenaCard" || name == "arenaUnitCount" {
                    let Eval::Player(player) = sub(receiver) else {
                        return Eval::Null;
                    };
                    let predicate = &args[0];
                    let count=if name=="hasSomeArenaCard" {
                        // Controller belongs to the upgrade even on an opposing host.
                        (1..s.next_id).filter(|id|s.card(*id).present && s.card(*id).owner==player && (self.in_arena(s,*id)||self.attached_in_play(s,*id)) && self.eval_with_binding(s,source,target,*id,modifier_target,predicate).truth()).count() as i64
                    }else{
                        s.player(player).ground.as_slice().iter().chain(s.player(player).space.as_slice()).filter(|&&card|self.eval_with_binding(s,source,target,card,modifier_target,predicate).truth()).count() as i64
                    };
                    return if name != "arenaUnitCount" {
                        Eval::Bool(count > 0)
                    } else {
                        Eval::Number(count)
                    };
                }
                if name=="isDeployed" {return Eval::Bool(matches!(sub(receiver),Eval::Card(id) if id!=0 && self.def(s,id).info.is("leader") && s.card(id).deployed && self.in_arena(s,id)));}
                if name=="resourcePairs" {return if let Eval::Player(p)=sub(receiver) {Eval::Number(s.player(p).resources.len as i64/2)}else{Eval::Null};}
                if name == "hasInitiative" {
                    return Eval::Bool(matches!(sub(receiver), Eval::Player(p) if s.initiative==p));
                }
                let Eval::Card(id) = sub(receiver) else {
                    return Eval::Null;
                };
                if id == 0 {
                    return Eval::Null;
                }
                let d = self.def(s, id);
                match name.as_str() {
                    "hasAttachedTrait" => Eval::Bool(self.attached_trait(s,id,if let Expr::Const{value}=&args[0] {value.as_str().unwrap_or("")} else {""})),
                    "hasAttachedNativeToken" => Eval::Bool(self.attached_native_shield(s,id)),
                    "isUnit" => Eval::Bool(
                        d.info.is("unit") || (d.info.is("leader") && s.card(id).deployed),
                    ),
                    "isAttacking"=>Eval::Bool(self.is_attacking(s,id)),
                    "isDefending"=>Eval::Bool(self.is_defending(s,id)),
                    "hasCost"=>Eval::Bool(["unit","leader","upgrade","event","token"].iter().any(|kind|d.info.is(kind))),
                    "isUpgrade"=>Eval::Bool(d.info.is("upgrade")),
                    "isTokenUpgrade"=>Eval::Bool(d.info.is("token")&&d.info.is("upgrade")),
                    "isTokenUnit"=>Eval::Bool(d.info.is("token")&&d.info.is("unit")),
                    "isShield"=>Eval::Bool(s.card(id).def==self.shield_def),
                    "isNonLeaderUnit" => Eval::Bool(d.info.is("unit") && !d.info.is("leader")),
                    "isInPlay" => Eval::Bool(self.in_arena(s, id) || self.attached_in_play(s,id) || d.info.is("base")),
                    "canBeExhausted" => Eval::Bool(
                        d.info.is("unit")
                            || d.info.is("leader")
                            || d.info.is("event")
                            || d.info.is("upgrade"),
                    ),
                    // a base is never upgraded here: Fortify, the only way to upgrade one, is not admitted
                    "isUpgraded" => {
                        Eval::Bool(s.card(id).shield != 0 || s.card(id).experience != 0 || s.card(id).upgrades!=0)
                    }
                    "isToken" => Eval::Bool(d.info.is("token")),
                    "isBase" => Eval::Bool(d.info.is("base")),
                    "isLeader" => Eval::Bool(d.info.is("leader")),
                    "getPower" => {
                        Eval::Number(
                            self.evaluated_power(s,id)
                        )
                    }
                    "getHp" => Eval::Number(self.evaluated_hp(s,id)),
                    "getCost" | "getPrintedCost" => Eval::Number(d.info.cost() as i64),
                    "hasSomeTrait" | "hasSomeAspect" | "hasSomeKeyword" => {
                        let values = match args.first().map(sub) {
                            Some(Eval::Array(v)) => v,
                            Some(v) => vec![v],
                            None => vec![],
                        };
                        Eval::Bool(values.iter().any(|v| {
                            if let Eval::Text(t) = v {
                                match name.as_str() {
                                    "hasSomeTrait" => self.card_has_trait(s,id,t),
                                    "hasSomeAspect" => d.info.aspects().contains(t),
                                    _ => self.has_keyword(s, id, t),
                                }
                            } else {
                                false
                            }
                        }))
                    }

                    _ => Eval::Null,
                }
            }
        }
    }
    fn matching_targets(&self, s: &State, source: CardId, r: &TargetResolver) -> Vec<CardId> {
        if let Some(native)=&r.native {return self.native_selector_candidates(s,source,r,native).into_iter().filter(|id|r.predicate.as_ref().is_none_or(|p|self.eval(s,source,*id,p).truth())).collect();}
        let owner = s.card(source).owner;
        let mut out = Vec::new();
        for seat in [Seat::P1, Seat::P2] {
            if (r.owner == "self" && seat != owner) || (r.owner == "opponent" && seat == owner) {
                continue;
            }
            let p = s.player(seat);
            let candidates: Vec<CardId> = if r.zone.as_deref() == Some("discard") {
                p.discard.as_slice().to_vec()
            } else if r.zone.as_deref() == Some("hand") {
                p.hand.as_slice().to_vec()
            } else if r.zone.as_deref() == Some("resource") {
                p.resources.as_slice().to_vec()
            } else if r.is_upgrade_domain() {
                (1..s.next_id).filter(|id|s.card(*id).owner==seat && self.attached_in_play(s,*id)).collect()
            } else {
                p.ground
                    .as_slice()
                    .iter()
                    .chain(p.space.as_slice())
                    .copied()
                    .chain(std::iter::once(p.base))
                    .chain((r.card_type!="any" && p.leader!=0 && !s.card(p.leader).deployed && self.selector_type_matches(s,p.leader,&r.card_type)).then_some(p.leader))
                    .collect()
            };
            for id in candidates {
                let d = self.def(s, id);
                let base = id == p.base;
                if !(if r.card_type=="any" && r.is_upgrade_domain() {d.info.is("upgrade")}else{self.selector_type_matches(s,id,&r.card_type)}) {continue;}
                if r.zone.as_deref() == Some("arena") && base
                    || r.zone.as_deref() == Some("base") && !base && !(id==p.leader&&!s.card(id).deployed)
                {
                    continue;
                }
                if let Some(arena) = &r.arena {
                    if d.info.arena.as_ref() != Some(arena) {
                        continue;
                    }
                }
                if r.predicate
                    .as_ref()
                    .is_some_and(|p| !self.eval(s, source, id, p).truth())
                {
                    continue;
                }
                out.push(id);
            }
        }
        out
    }
    pub(super) fn native_attack_predicate(&self,s:&State,source:CardId,candidate:CardId,e:&Expr)->bool {
        self.eval_with_card(s,source,candidate,candidate,e).truth()
    }
    /// Disclose commits the play before reading the hand, then suspends for the
    /// owner selection and opponent reveal acknowledgment. Only acknowledgment
    /// publishes success; cancellation and masked failure skip the continuation.
    /// Selection does not move cards or open child event windows, and is cleared
    /// before the ordinary continuation can select targets or resolve effects.
    fn disclose_missing(&self,s:&State,aspects:&[String],selected:&[CardId])->[u8;6] {
        let mut missing=[0u8;6];for name in aspects {missing[aspect(name).expect("validated disclose aspect")]+=1;}
        for &id in selected {for (i,count) in self.def(s,id).aspects.iter().enumerate() {missing[i]=missing[i].saturating_sub(*count);}}
        missing
    }
    fn can_disclose(&self,s:&State,player:Seat,aspects:&[String])->bool {self.disclose_missing(s,aspects,s.player(player).hand.as_slice()).iter().all(|n|*n==0)}
    fn disclose_aspects<'a>(&'a self,s:&State)->&'a [String] {let Instruction::Apply(Effect::Disclose{aspects,..})=&self.program(s.ability_program-1).instructions[s.ability_pc as usize] else{panic!("disclose prompt has no disclosure instruction")};aspects}
    fn disclose_complete(&self,s:&State)->bool {self.disclose_missing(s,self.disclose_aspects(s),&s.disclose.selected.ids[..s.disclose.selected.len as usize]).iter().all(|n|*n==0)}
    fn disclose_candidates(&self,s:&State)->Vec<CardId> {
        let picks=&s.disclose.selected;let missing=self.disclose_missing(s,self.disclose_aspects(s),&picks.ids[..picks.len as usize]);
        s.player(s.disclose.player).hand.as_slice().iter().copied().filter(|id|picks.ids[..picks.len as usize].contains(id)||(!self.disclose_complete(s)&&self.def(s,*id).aspects.iter().enumerate().any(|(i,n)|*n>0&&missing[i]>0))).collect()
    }
    fn finish_disclose(&self,s:&mut State,success:bool)->Result<(),String> {
        s.active=s.disclose.player;s.disclose=DiscloseState::default();s.ability_result=u16::from(success);s.ability_succeeded=success;s.ability_pc+=1;self.run_program(s)
    }
    fn effect_can_affect(
        &self,
        s: &State,
        source: CardId,
        target: CardId,
        e: &Effect,
        require_change: bool,
    ) -> bool {
        match e {
            // These zone moves need their own target/eligibility review instead
            // of the scalar target table below.
            Effect::MoveToBottomOfDeck { target: destination } => {
                let mut context=*s;context.source=source;context.ability_target=target;
                if context.event_group.active {context.event_group.context=Selection::one(target);}
                return self.effect_targets(&context,destination).into_iter().any(|value|match value {
                    Eval::Card(id)=>self.move_to_bottom_eligible(&context,id),
                    _=>false,
                });
            }
            Effect::ResourceCardSystem { target: destination, target_player, .. } => {
                return matches!(self.resource_card_system_card(s,source,destination,target_player),Ok(Some(id)) if self.resource_card_system_eligible(s,id));
            }
            Effect::Disclose{aspects,..}=>return !self.def(s,source).info.is("event")||self.can_disclose(s,s.card(source).owner,aspects),
            // Card-effect payment is legal exactly when the source controller's
            // ready resources cover the constant charge (the engine payment
            // system's canAffect is its canPay check). Paying always consumes
            // resources, so the change-required flag does not change legality.
            Effect::PayAdjustedResources {..} => {
                let Ok(amount)=crate::ability_ir::card_effect_payment(e) else {return false;};
                return self.ready_resources(s,s.card(source).owner)>=amount as usize;
            }
            Effect::InitiateAttack {properties,..} => return self.native_attack_eligible(s,source,target,properties),
            // Native distribution hasLegalTarget delegates to its selector; the
            // nested HealSystem checks legality with no change requirement.
            Effect::DistributeHealing {target_resolver,..} => return !self.matching_targets(s,source,target_resolver).is_empty(),
            Effect::Simultaneous { effects } | Effect::PhaseModifiers { effects } => return effects.iter().any(|e| self.effect_can_affect(s,source,target,e,require_change)),
            Effect::Select { target_resolver,effect } => return self.matching_targets(s,source,target_resolver).into_iter().any(|id| self.effect_can_affect(s,source,id,effect,require_change)),
            Effect::Conditional { condition,on_true,on_false } => return if self.eval(s,source,target,condition).truth() { self.effect_can_affect(s,source,target,on_true,require_change) } else { on_false.as_ref().is_some_and(|e| self.effect_can_affect(s,source,target,e,require_change)) },
            Effect::CardRestriction { duration, condition, target: restriction_target, .. } => {
                if condition.is_some() || duration != "phase" { return false; }
                let mut context = *s;
                context.source = source;
                context.ability_target = target;
                return self.effect_targets(&context, restriction_target).into_iter().any(|value| matches!(value, Eval::Card(id) if id != 0 && (self.in_arena(s, id) || self.def(s, id).info.is("base"))));
            }
            _ => {},
        }
        let mut context = *s;
        context.source = source;
        context.ability_target = target;
        if context.event_group.active { context.event_group.context=Selection::one(target); }
        let Some((dest, amount)) = e.scalar_args() else { return false; };
        let amount = amount.map_or(1, |amount| {
            self.eval(&context, source, context.ability_target, amount).number()
        });
        self.effect_targets(&context, dest)
            .into_iter()
            .any(|dest| match dest {
                Eval::Player(player) => {
                    if matches!(e,Effect::UseForce{..}) {s.force_state_exact && s.player(player).force_held}
                    else if matches!(e,Effect::GainForce{..}) {s.force_state_exact && (!require_change || !s.player(player).force_held)}
                    else if matches!(e,Effect::DiscardDeck {..}) {amount>0 && (!require_change || s.player(player).deck.len>0)}
                    else if matches!(e,Effect::DiscardOwnHand {..}) {!require_change || (amount>0 && s.player(player).hand.len>0)}
                    else if matches!(e, Effect::ReadyResources { .. }) {
                        !require_change
                            || s.player(player)
                                .resources
                                .as_slice()
                                .iter()
                                .any(|&id| s.card(id).exhausted)
                    } else {
                        matches!(e, Effect::Draw { .. } | Effect::CreateTokenUnit {..}) && (!require_change || amount > 0)
                    }
                }
                Eval::Card(id) if id != 0 => {
                    let unit = self.in_arena(s, id);
                    let base = self.def(s, id).info.is("base");
                    match e {
                        Effect::Damage { .. } => (unit || base) && (!require_change || amount > 0),
                        Effect::Heal { .. } => {
                            (unit || base)
                                && (!require_change || (amount > 0 && s.card(id).damage > 0 && !self.card_restricted(s, id, RestrictionVerb::BeHealed)))
                        }
                        Effect::Ready { .. } => self.exhaustible_card(s,id) && !self.card_restricted(s, id, RestrictionVerb::Ready) && (!require_change || s.card(id).exhausted),
                        Effect::Exhaust { .. } => {
                            self.exhaustible_card(s,id) && (!require_change || !s.card(id).exhausted)
                        }
                        Effect::Defeat { .. } => unit || self.attached_in_play(s,id) || self.resource_zone_card(s,id),
                        Effect::ResourceCard {target:Target::Name(n)} if n=="ownTopDeck" => self.resource_top_deck_eligible(&context,id),
                        Effect::ResourceCard {target:Target::Name(n)} if n=="self" => self.resource_source_eligible(&context,id) && (!require_change || !s.player(s.card(id).owner).resources.contains(id)),
                        Effect::ResourceCard { .. } => self.resource_card_eligible(&context,id),
                        Effect::ReturnToHand { .. } => {
                            !self.def(s, id).info.is("leader")
                                && (unit || self.attached_in_play(s,id) || s.player(s.card(id).owner).discard.contains(id) || self.resource_zone_card(s,id))
                        }
                        Effect::ModifyStats { .. } | Effect::GainKeyword { .. } | Effect::LoseKeyword { .. } => unit,
                        Effect::GiveShield { .. } | Effect::GiveExperience { .. } | Effect::GiveWeakness{..}|Effect::GiveAdvantage{..} => {
                            unit && amount > 0
                        }
                        _ => false,
                    }
                }
                _ => false,
            })
    }
    fn instructions_can_affect(
        &self,
        s: &State,
        source: CardId,
        target: CardId,
        p: u16,
        start: usize,
        end: usize,
        require_change: bool,
    ) -> bool {
        let instructions = &self.program(p).instructions;
        let mut pc = start;
        while let Some(instruction) = instructions.get(pc).filter(|_| pc < end) {
            match instruction {
                Instruction::EndStep => {}
                Instruction::Commit => {}
                Instruction::CaptureMaximum{aspect}=>{let mut captured=*s;if self.capture_maximum(&mut captured,source,aspect).is_err(){return false;}return self.instructions_can_affect(&captured,source,target,p,pc+1,end,require_change);}

                Instruction::Select(resolver) => {
                    let scope = self.program(p).select_end(pc).min(end);
                    if self.matching_targets(s, source, resolver).into_iter().any(
                        |candidate| {
                            self.instructions_can_affect(
                                s,
                                source,
                                candidate,
                                p,
                                pc + 1,
                                scope,
                                require_change,
                            )
                        },
                    ) {
                        return true;
                    }
                    // a selection with no target it can change is skipped, and the steps after it still resolve
                    pc = scope;
                    continue;
                }
                Instruction::IfResult { .. } => break,
                Instruction::Apply(effect) | Instruction::Group(effect) | Instruction::NamedGroup(effect) | Instruction::NamedChooserGroup {effect,..} => {
                    if self.effect_can_affect(s, source, target, effect, require_change) {
                        return true;
                    }
                }
                Instruction::Conditional { condition, skip } => {
                    if !self.eval(s, source, target, condition).truth() {
                        pc = *skip;
                        continue;
                    }
                }
                Instruction::Jump { to } => {
                    pc = *to;
                    continue;
                }
            }
            pc += 1;
        }
        false
    }
    fn program_targets(
        &self,
        s: &State,
        source: CardId,
        p: u16,
        pc: usize,
        r: &TargetResolver,
        require_change: bool,
    ) -> Vec<CardId> {
        self.matching_targets(s, source, r)
            .into_iter()
            .filter(|id| self.instructions_can_affect(s, source, *id, p, pc + 1, self.program(p).select_end(pc), require_change))
            .collect()
    }
    fn program_has_effect(&self, s: &State, source: CardId, p: u16) -> bool {
        self.program_has_effect_mode(s, source, p, self.program(p).trigger != "action")
    }
    /// Whether a trigger window counts this ability as able to resolve (forceteki hasAnyLegalEffects): titles,
    /// order and the all-no-effect skip. An ability's own target is checked without requiring a change, since
    /// forceteki builds a triggered ability's target resolvers before it sets the ability's change-required mode.
    /// Resolving it still requires a change (begin_program), so such a trigger can be ordered and then do nothing.
    fn trigger_has_effect(&self, s: &State, source: CardId, p: u16) -> bool {
        if self.program(p).attacker_must_survive && s.attack_attacker_defeated {return false;}
        match self.program(p).instructions.first() {
            Some(Instruction::Select(_)) => self.program_has_effect_mode(s, source, p, false),
            _ => self.program_has_effect(s, source, p),
        }
    }
    fn program_has_effect_mode(
        &self,
        s: &State,
        source: CardId,
        p: u16,
        require_change: bool,
    ) -> bool {
        if self.program(p).trigger=="event"&&self.program(p).instructions.first().is_some_and(|i|matches!(i,Instruction::Group(Effect::Simultaneous{effects}) if effects.len()==2&&matches!(&effects[0],Effect::Select{effect,..} if matches!(effect.as_ref(),Effect::Damage{..}))&&matches!(&effects[1],Effect::GainForce{..}))) {return true;}
        match self.program(p).instructions.first() {
            Some(Instruction::Select(r)) => !self
                .program_targets(s, source, p, 0, r, require_change)
                .is_empty(),
            Some(Instruction::Apply(Effect::DistributeHealing {
                target_resolver, ..
            })) => !self.matching_targets(s, source, target_resolver).is_empty(),
            _ => self.instructions_can_affect(s, source, 0, p, 0, usize::MAX, require_change),
        }
    }
    pub(super) fn program_menu_title(&self, s: &State, source: CardId, p: u16) -> String {
        let program = self.program(p);
        let title = if program.context_title.is_empty() { program.title.clone() } else {
            program.context_title.iter().map(|fragment| match self.eval(s, source, s.ability_target, fragment) {
                Eval::Text(value) => value,
                Eval::Number(value) => value.to_string(),
                Eval::Bool(value) => value.to_string(),
                Eval::Null => "null".into(),
                _ => "[object Object]".into(),
            }).collect::<String>()
        };
        if self.trigger_has_effect(s, source, p) {
            title.clone()
        } else {
            format!("(No effect) {title}")
        }
    }
    pub(super) fn is_ability_decision(&self, d: Decision) -> bool {
        matches!(
            d,
            Decision::DeathPlayer | Decision::DeathOrder | Decision::DeathBatch | Decision::AbilityMenu
                | Decision::AbilityTarget
                | Decision::AbilityOptional
                | Decision::AbilityNoEffect
                | Decision::AbilityDistribute
                | Decision::AbilityGroupShield | Decision::AbilityDiscard
                | Decision::DiscloseSelect | Decision::DisclosePause | Decision::DiscloseReveal
        )
    }
    pub(super) fn ability_actions(&self, s: &State) -> Vec<Action> {
        let mut out = Vec::new();
        let source = s.source;
        match s.decision {
            Decision::DisclosePause=>out.push(Action::button("Skip","done")),
            Decision::DiscloseReveal=>out.push(Action::button("Done","done")),
            Decision::DiscloseSelect=>{
                if s.disclose.selected.len==0 {out.push(Action::button("Choose nothing","done"));}
                else if s.disclose.selected.len>1&&self.disclose_complete(s) {out.push(Action::button("Done","done"));}
                out.push(Action::button("Cancel","cancel"));for id in self.disclose_candidates(s) {out.push(self.card_action(s,id));}
            }
            Decision::AbilityDiscard => {
                let g=&s.event_group;
                if g.discard_count>1 && g.picks.len==g.discard_count {out.push(Action::button("Done","done"));}
                for &id in s.player(g.discard_owner).hand.as_slice() {if g.picks.len<g.discard_count || g.picks.ids[..g.picks.len as usize].contains(&id) {out.push(self.card_action(s,id));}}
            }
            Decision::DeathPlayer => {out.push(Action::button("You",0));out.push(Action::button("Opponent",1));}
            Decision::DeathOrder => {
                for g in self.death_groups(s) {let e=s.death_window.entries[g[0]];
                    let title=if g.len()>1 {let title=self.death_title(s,e,true);if !g.iter().any(|i|self.death_has_effect(s,s.death_window.entries[*i])) {format!("(No effect) {title}")}else{title}}else{self.death_title(s,e,false)};
                    out.push(Action::button(title,out.len().to_string()));}
            }
            Decision::DeathBatch => {
                let count=s.death_window.entries[..s.death_window.len as usize].iter().filter(|e|e.player==s.active && self.batch_member(s,**e)).count();
                out.push(Action::button("Resolve next","next"));out.push(Action::button(format!("Resolve all ({count})"),"all"));
            }
            Decision::AbilityGroupShield => {
                if s.event_group.shield_count==0 { out.push(Action::button("You",0)); out.push(Action::button("Opponent",1)); }
                else { out.push(Action::button("Resolve next","next")); out.push(Action::button(format!("Resolve all ({})",s.event_group.shield_count),"all")); }
            }
            Decision::AbilityMenu => {
                for p in self.available_programs(s, source, "action") {
                    out.push(Action::button(
                        self.program_menu_title(s, source, p),
                        out.len(),
                    ));
                }
                if !s.card(source).exhausted && self.has_attack_target(s, source, false) {
                    out.push(Action::button("Attack", out.len()));
                }
                out.push(Action::button("Cancel", out.len()));
            }
            Decision::AbilityNoEffect => {
                out.push(Action::button(
                    if self.def(s, s.source).info.is("event") {
                        "Play anyway"
                    } else {
                        "Use it anyway"
                    },
                    0,
                ));
                out.push(Action::button("Cancel", 1));
            }
            Decision::AbilityOptional => {
                out.push(Action::button("Trigger", "trigger"));
                out.push(Action::button("Pass", "pass"));
            }
            Decision::AbilityTarget => {
                let p=self.program(s.ability_program-1);
                let (r,child)=if let Some((r,e))=self.current_group_resolver(s,p) { (r,Some(e)) } else {
                    let Instruction::Select(r)=&p.instructions[s.ability_pc as usize] else { unreachable!() }; (r,None)
                };
                let named=p.instructions[s.ability_pc as usize].is_named_group();
                if !named && !s.ability_committed && s.ability_return==ProgramContinuation::Finish.raw() { out.push(Action::button("Cancel","cancel")); }
                if child.is_none() && p.optional && s.ability_pc==0 { out.push(Action::button("Pass","passAbility")); }
                let picks=if child.is_some() { s.event_group.picks } else { s.ability_selection };
                if r.is_multiple() {
                    if picks.len>0 { out.push(Action::button("Done","done")); }
                    else if r.optional { out.push(Action::button("Choose nothing","done")); }
                } else if r.optional {
                    if child.is_some() || s.ability_pc>0 && matches!(p.instructions[s.ability_pc as usize-1],Instruction::Commit) {
                        out.push(Action::button("Choose nothing","done"));
                    } else { out.push(Action::button("Choose no target","noTarget")); }
                }
                if named && !s.ability_committed && s.ability_return==ProgramContinuation::Finish.raw() { out.push(Action::button("Cancel","cancel")); }
                let candidates=if let Some(e)=child { self.group_candidates(s,r,e,false) } else {
                    self.program_targets(s,source,s.ability_program-1,s.ability_pc as usize,r,false)
                };
                for id in candidates {
                    if !r.is_multiple() || picks.len<r.max_targets().expect("validated selector") as u8 || picks.ids[..picks.len as usize].contains(&id) { out.push(self.card_action(s,id)); }
                }
            }
            Decision::AbilityDistribute => {
                let p = self.program(s.ability_program - 1);
                let Instruction::Apply(Effect::DistributeHealing {
                    amount,
                    target_resolver,
                    can_choose_no_targets,
                    can_distribute_less,
                    ..
                }) = &p.instructions[s.ability_pc as usize]
                else {
                    unreachable!()
                };
                let amount = self.eval(s, source, 0, amount).number().clamp(0, 3);
                let label = format!("distribute {amount}");
                if *can_choose_no_targets {
                    out.push(Action {
                        kind: "distribute".into(),
                        card: None,
                        arg: Some(json!([])),
                        method: None,
                        label: label.clone(),
                    });
                }
                for id in self.matching_targets(s, source, target_resolver) {
                    for n in 1..=amount {
                        if n != amount && !can_distribute_less {
                            continue;
                        }
                        out.push(Action {
                            kind: "distribute".into(),
                            card: None,
                            arg: Some(json!([[format!("c{}", id - 1), n]])),
                            method: None,
                            label: label.clone(),
                        });
                    }
                }
            }
            _ => {}
        }
        out
    }
    /// New work chooses a named continuation; only suspended state stores its
    /// historical numeric representation. Legacy raw return decoding stays in end_program.
    pub(super) fn begin_program(&self, s: &mut State, p: u16, return_to: ProgramContinuation) -> Result<(), String> {
        s.disclose=DiscloseState::default();
        s.ability_program = p + 1;
        s.ability_pc = 0;
        s.ability_target = 0;
        s.maximum_capture=MaximumCapture::default();
        s.ability_selection=Selection::default();
        s.event_group=EventGroupState::default();
        s.event_result=StepEventResult::default();
        if self.program(p).trigger!="whenDefeated" {s.last_known_source=0;s.last_known_power=0;}
        s.ability_result = 0;
        s.ability_succeeded = false;
        s.ability_return = return_to.raw();
        s.ability_committed = return_to != ProgramContinuation::Finish;
        if self.program(p).attacker_must_survive && s.attack_attacker_defeated {return self.end_program(s);}
        if self.program(p).optional
            && !matches!(
                self.program(p).instructions.first(),
                Some(Instruction::Select(_))
            )
            && self.program_has_effect(s, s.source, p)
        {
            s.decision = Decision::AbilityOptional;
            return Ok(());
        }
        if !self.program_has_effect_mode(s, s.source, p, false) {
            if return_to == ProgramContinuation::Finish {
                s.decision = Decision::AbilityNoEffect;
                return Ok(());
            }
            return self.end_program(s);
        }
        if !self.program_has_effect(s, s.source, p) {
            if self.program(p).trigger == "event" {
                self.commit_program(s)?;
            }
            return self.end_program(s);
        }
        self.run_program(s)
    }
    fn commit_program(&self, s: &mut State) -> Result<(), String> {
        if s.ability_committed {
            return Ok(());
        }
        let source = s.source;
        // Mandatory Force payment must still be possible after targeting. Check
        // before mutating any other cost, so a failed payment stays atomic.
        if self.program(s.ability_program - 1).costs.iter().any(|c| matches!(c, Cost::UseForceCost { .. }))
            && !self.can_pay_force_cost(s, source) {
            return Err("Force cost requires exact held Force identity".into());
        }
        if self.def(s, source).info.is("event") && s.player(s.active).hand.contains(source) {
            s.phase_history.preflight_play(s.active)?;
            self.spend(s, s.active, self.cost(s, source));
            s.player_mut(s.active).hand.remove(source);
            s.player_mut(s.active).discard.push(source)?;
            s.phase_history.played(s.active,source)?;
        }
        for cost in &self.program(s.ability_program - 1).costs {
            match cost {
                Cost::ExhaustSelf => s.card_mut(source).exhausted = true,
                Cost::Resources { amount } => self.spend(
                    s,
                    s.active,
                    self.eval(s, source, s.ability_target, amount)
                        .number()
                        .clamp(0, u16::MAX as i64) as u16,
                ),
                Cost::UseForceCost { .. } => s.player_mut(s.card(source).owner).force_held = false,
            }
        }
        s.ability_committed = true;
        Ok(())
    }
    fn end_program(&self, s: &mut State) -> Result<(), String> {
        s.disclose=DiscloseState::default();
        let next = s.ability_return;
        s.ability_program = 0;
        s.maximum_capture=MaximumCapture::default();
        s.ability_selection=Selection::default(); s.event_group=EventGroupState::default();
        match ProgramContinuation::from_raw(next) {
            Some(ProgramContinuation::Deaths) => self.process_deaths(s),
            Some(ProgramContinuation::Triggers) => self.process_triggers(s),
            // Historical raw values outside the two resumable cases finish.
            _ => self.finish(s),
        }
    }
    pub(super) fn run_program(&self, s: &mut State) -> Result<(), String> {
        loop {
            if self.open_deaths(s,DeathContinuation::Program)? {return Ok(());}
            let p = self.program(s.ability_program - 1);
            if self.rules_backend != RulesBackend::Interpreter {
                match p.executable {
                    Ok(index)=>{if self.rules_instrumentation {self.rules_hits.lock().expect("rules metrics lock").entry(s.ability_program as usize-1).or_default()[0]+=1;}if self.generated_step(s,p,index)? {return Ok(());} continue;},
                    Err(reason) if self.rules_backend==RulesBackend::Aot=>return Err(format!("AOT backend unavailable: {reason}")),
                    Err(_)=>{},
                }
            }
            if self.rules_instrumentation {self.rules_hits.lock().expect("rules metrics lock").entry(s.ability_program as usize-1).or_default()[1]+=1;}
            let Some(instruction) = p.instructions.get(s.ability_pc as usize) else {
                return self.end_program(s);
            };
            match instruction {
                Instruction::EndStep => {s.ability_pc+=1;}
                Instruction::CaptureMaximum{aspect}=>{self.capture_maximum(s,s.source,aspect)?;s.ability_pc+=1;}
                Instruction::Commit => {
                    self.commit_program(s)?;
                    s.ability_pc += 1;
                }
                Instruction::Select(r) => {
                    if self
                        .program_targets(
                            s,
                            s.source,
                            s.ability_program - 1,
                            s.ability_pc as usize,
                            r,
                            true,
                        )
                        .is_empty()
                    {
                        s.ability_target = 0;
                        s.ability_selection=Selection::default();
                        s.ability_pc += 1;
                        continue;
                    }
                    s.decision = Decision::AbilityTarget;
                    return Ok(());
                }
                Instruction::Conditional { condition, skip } => {
                    s.ability_pc = if self.eval(s, s.source, s.ability_target, condition).truth() {
                        s.ability_pc + 1
                    } else {
                        *skip as u8
                    };
                }
                Instruction::IfResult { skip } => {
                    s.ability_pc = if s.ability_succeeded {
                        s.ability_pc + 1
                    } else {
                        *skip as u8
                    };
                }
                Instruction::Jump { to } => s.ability_pc = *to as u8,
                Instruction::Apply(Effect::InitiateAttack{properties,..}) => {
                    self.commit_program(s)?;
                    if self.native_attack_eligible(s,s.source,s.ability_target,properties) {
                        self.begin_native_attack(s,properties,p.optional)?;
                        return Ok(());
                    }
                    s.ability_result=0;s.ability_succeeded=false;s.ability_pc+=1;
                }
                Instruction::Apply(Effect::Disclose{aspects,..})=>{
                    self.commit_program(s)?;s.ability_result=0;s.ability_succeeded=false;
                    let player=s.card(s.source).owner;s.disclose=DiscloseState{player,selected:Selection::default()};s.active=player;
                    if self.can_disclose(s,player,aspects) {s.decision=Decision::DiscloseSelect;return Ok(());}
                    if p.trigger!="event" {s.decision=Decision::DisclosePause;return Ok(());}
                    s.disclose=DiscloseState::default();s.ability_pc+=1;
                }
                Instruction::Apply(Effect::DistributeHealing { .. }) => {
                    s.decision = Decision::AbilityDistribute;
                    return Ok(());
                }
                Instruction::Group(effect) | Instruction::NamedGroup(effect) | Instruction::NamedChooserGroup{effect,..} => {
                    let named=instruction.is_named_group();
                    if !named { self.commit_program(s)?; }
                    if !s.event_group.active {
                        s.ability_succeeded=false;
                        s.event_group.active=true; s.event_group.initiator=s.active;
                        s.event_group.context=if s.ability_selection.len>0 { s.ability_selection } else { Selection::one(s.ability_target) };
                    }
                    // Target prompts may temporarily hand active to the opposite chooser.
                    // Replacement prompts retain their own actor until resolve_group consumes them.
                    if matches!(instruction,Instruction::NamedChooserGroup{..}) && !s.event_group.replacement_done {s.active=s.event_group.initiator;}
                    if !self.generate_group(s,effect)? { return Ok(()); }
                    if named { self.commit_program(s)?; self.allocate_named_tokens(s,effect)?; }
                    if !self.resolve_group(s,effect)? { return Ok(()); }
                    s.event_group=EventGroupState::default();
                    s.ability_selection=Selection::default();
                    s.ability_pc+=1;
                }
                Instruction::Apply(effect) => {
                    self.commit_program(s)?;
                    let (amount, resolved) = self.apply_effect(s, effect)?;
                    s.ability_result = amount;
                    s.ability_succeeded = resolved;
                    if resolved&&matches!(effect,Effect::UseForce{..})&&p.instructions.iter().skip(s.ability_pc as usize+1).any(|i|matches!(i,Instruction::Apply(Effect::Defeat{target:Target::ArenaList(_)}))){self.sweep_stat_defeats(s)?;}
                    s.ability_pc += 1;
                }
            }
        }
    }
    fn effect_targets(&self, s: &State, target: &Target) -> Vec<Eval> {
        let owner = s.card(s.source).owner;
        match target {
            Target::ArenaList(list)=>{
                let seat=match self.eval(s,s.source,s.ability_target,&list.receiver) {Eval::Player(seat)=>Some(seat),_=>None};
                // Snapshot candidates before modifiers and the single lethal sweep.
                // Candidate card and outer selected target retain separate bindings.
                let mut ids=self.arena_units(s).into_iter().filter(|id|seat.is_none_or(|seat|s.card(*id).owner==seat) && self.eval_with_card(s,s.source,s.ability_target,*id,&list.predicate).truth()).collect::<Vec<_>>();
                if let Some(order)=list.controller_order {
                    let first=match order {crate::ability_ir::ControllerOrder::PlayerFirst=>owner,crate::ability_ir::ControllerOrder::OpponentFirst=>owner.other(),crate::ability_ir::ControllerOrder::SeatOrder=>Seat::P1};
                    ids.sort_by_key(|id|u8::from(s.card(*id).owner!=first));
                }
                ids.into_iter().map(Eval::Card).collect()
            }
            Target::Expression(e) => {
                fn flatten(value: Eval, out: &mut Vec<Eval>) {
                    match value {
                        Eval::Array(values) => {
                            for value in values {
                                flatten(value, out);
                            }
                        }
                        value => out.push(value),
                    }
                }
                let mut out = vec![];
                flatten(self.eval(s, s.source, s.ability_target, e), &mut out);
                out
            }
            Target::Name(n) => match n.as_str() {
                "self" => vec![Eval::Card(s.source)],
                "ownTopDeck" => s.player(owner).deck.as_slice().first().copied().map(|id|vec![Eval::Card(id)]).unwrap_or_default(),
                "parent" if self.attached_in_play(s,s.source) => vec![Eval::Card(s.card(s.source).parent)],
                "chosen" if s.event_group.active => s.event_group.context.ids[..s.event_group.context.len as usize].iter().map(|id| Eval::Card(*id)).collect(),
                "chosen" => vec![Eval::Card(s.ability_target)],
                "ownBase" => vec![Eval::Card(s.player(owner).base)],
                "enemyBase" => vec![Eval::Card(s.player(owner.other()).base)],
                "bothBases" => vec![Eval::Card(s.players[0].base), Eval::Card(s.players[1].base)],
                "ownPlayer" => vec![Eval::Player(owner)],
                "enemyPlayer" => vec![Eval::Player(owner.other())],
                "bothPlayers" => vec![Eval::Player(Seat::P1), Eval::Player(Seat::P2)],
                _ => vec![],
            },
        }
    }
    fn create_token(&self, s: &mut State, target: CardId, experience: bool) -> Result<(), String> {
        let def = if experience {
            self.experience_def
                .ok_or("catalogue needs Experience token")?
        } else {
            self.shield_def
        };
        if s.next_id as usize >= MAX_CARDS {
            return Err("card capacity exceeded creating token".into());
        }
        if experience {
            let mut check = *s;
            check.card_mut(target).phase_power = check
                .card(target)
                .phase_power
                .checked_add(1)
                .ok_or("computed power capacity")?;
            check.card_mut(target).phase_hp = check
                .card(target)
                .phase_hp
                .checked_add(1)
                .ok_or("computed HP capacity")?;
            self.check_stat_capacity(&check)?;
        }
        let id = s.next_id;
        s.next_id += 1;
        let previous = if experience {
            s.card(target).experience
        } else {
            s.card(target).shield
        };
        *s.card_mut(id) = Card {
            present: true,
            def,
            owner: s.card(target).owner,
            next_token: previous,
            ..Card::default()
        };
        if experience {
            s.card_mut(target).experience = id;
        } else {
            s.card_mut(target).shield = id;
        }
        self.attach(s,id,target);
        Ok(())
    }
    fn effect_amount(&self, s: &State, e: &Expr) -> Result<u16, String> {
        let Eval::Number(n) = self.eval(s, s.source, s.ability_target, e) else {
            return Err("effect amount did not evaluate to an integer".into());
        };
        u16::try_from(n).map_err(|_| format!("effect amount {n} exceeds bounded runtime"))
    }
    fn resource_zone_card(&self, s: &State, id: CardId) -> bool {
        id != 0 && s.player(s.card(id).owner).resources.contains(id)
    }
    /// `CardTargetSystem.leavesResourceZoneEventHandler` (pinned): when a ready
    /// resource controlled by the ability's player leaves the resource zone, it
    /// first swaps ready state with the first exhausted resource; the effect then
    /// consumes readiness rather than a particular physical resource identity.
    fn swap_resource_ready_state_on_departure(&self, s: &mut State, id: CardId) {
        if s.card(id).exhausted || s.card(id).owner != s.card(s.source).owner {
            return;
        }
        let owner = s.card(id).owner;
        let other = s
            .player(owner)
            .resources
            .as_slice()
            .iter()
            .copied()
            .find(|&resource| resource != id && s.card(resource).exhausted);
        if let Some(other) = other {
            s.card_mut(id).exhausted = true;
            s.card_mut(other).exhausted = false;
        }
    }
    /// `DefeatCardSystem` on a card in the Resource zone: swap readiness, leave
    /// the zone, and move the physical card to its owner's discard. This is a
    /// zone move, not a leaves-play event; no defeat registration is captured.
    fn defeat_resource(&self, s: &mut State, id: CardId) -> Result<u16, String> {
        if !self.resource_zone_card(s, id) {
            return Ok(0);
        }
        self.swap_resource_ready_state_on_departure(s, id);
        let owner = s.card(id).owner;
        let p = s.player_mut(owner);
        p.resources.remove(id);
        p.discard.push(id)?;
        Ok(1)
    }
    /// `MoveCardSystem(returnToHand)` on a card in the Resource zone: the same
    /// readiness swap, then the card moves to its owner's hand.
    fn return_resource_to_hand(&self, s: &mut State, id: CardId) -> Result<u16, String> {
        if !self.resource_zone_card(s, id) {
            return Ok(0);
        }
        self.swap_resource_ready_state_on_departure(s, id);
        let owner = s.card(id).owner;
        let p = s.player_mut(owner);
        p.resources.remove(id);
        p.hand.push(id)?;
        Ok(1)
    }
    fn resource_top_deck_eligible(&self,s:&State,id:CardId)->bool {
        if id==0||!s.card(id).present||s.source==0{return false;}
        let owner=s.card(s.source).owner;let info=&self.def(s,id).info;
        s.card(id).owner==owner&&s.player(owner).deck.as_slice().first()==Some(&id)&&!info.is("leader")&&!info.is("token")&&(info.is("unit")||info.is("event")||info.is("upgrade"))
    }
    fn resource_top_deck(&self,s:&mut State,id:CardId)->Result<u16,String> {
        if !self.resource_top_deck_eligible(s,id){return Ok(0);}
        let owner=s.card(s.source).owner;
        if s.player(owner).resources.len as usize>=crate::model::MAX_ZONE{return Err("resource zone capacity exceeded".into());}
        // Preserve the ability source. The target vector captured this physical
        // deck-front identity before this single event handler mutates zones.
        let p=s.player_mut(owner);p.deck.remove(id);p.resources.push(id)?;
        s.card_mut(id).exhausted=true;Ok(1)
    }
    /// A reviewed selected card entering the owner's resource zone (hand or
    /// discard source shapes). Resource entry exhausts it.
    fn resource_card_eligible(&self,s:&State,id:CardId)->bool {
        if id==0||!s.card(id).present {return false;}
        let info=&self.def(s,id).info;
        if info.is("leader")||info.is("token")||info.is("base")||!(info.is("unit")||info.is("event")||info.is("upgrade")) {return false;}
        let owner=s.card(id).owner;
        if s.player(owner).resources.contains(id) {return false;}
        s.player(owner).hand.contains(id)||s.player(owner).discard.contains(id)
    }
    fn resource_card_card(&self,s:&mut State,id:CardId)->Result<u16,String> {
        if !self.resource_card_eligible(s,id) {return Ok(0);}
        let owner=s.card(id).owner;
        if s.player(owner).resources.len as usize>=crate::model::MAX_ZONE {return Err("resource zone capacity exceeded".into());}
        let p=s.player_mut(owner);
        p.hand.remove(id);p.discard.remove(id);p.resources.push(id)?;
        s.card_mut(id).exhausted=true;
        Ok(1)
    }
    fn resource_source_eligible(&self,s:&State,id:CardId)->bool {
        if id==0 || id!=s.source {return false;}
        let info=&self.def(s,id).info;
        if info.is("leader") || info.is("token") || !(info.is("unit")||info.is("event")||info.is("upgrade")) {return false;}
        let p=s.player(s.card(id).owner);
        p.hand.contains(id)||p.deck.contains(id)||p.discard.contains(id)||p.resources.contains(id)
    }
    fn resource_source(&self,s:&mut State,id:CardId)->Result<u16,String> {
        if !self.resource_source_eligible(s,id) {return Ok(0);}
        let owner=s.card(id).owner;
        if s.player(owner).resources.contains(id) {return Ok(0);}
        if s.player(owner).resources.len as usize>=crate::model::MAX_ZONE {return Err("resource zone capacity exceeded".into());}
        let p=s.player_mut(owner);
        p.hand.remove(id);p.deck.remove(id);p.discard.remove(id);p.resources.push(id)?;
        s.card_mut(id).exhausted=true;
        Ok(1)
    }
    /// Move one admitted card to the bottom of its owner's deck. The admission
    /// gate restricts this effect to a single chosen card from a hand or discard
    /// pile; a play-zone move would need the pinned engine's leaving-play window
    /// and stays an explicit runtime error rather than an approximation.
    fn move_to_bottom_of_deck(&self,s:&mut State,id:CardId)->Result<u16,String> {
        if id==0||!s.card(id).present {return Ok(0);}
        if self.in_arena(s,id)||self.attached_in_play(s,id) {return Err("moveToBottomOfDeck requires a card outside the play zone".into());}
        if !self.move_to_bottom_eligible(s,id) {return Ok(0);}
        let owner=s.card(id).owner;
        if s.player(owner).deck.len as usize>=crate::model::MAX_ZONE {return Err("deck capacity exceeded".into());}
        let p=s.player_mut(owner);
        p.hand.remove(id);p.discard.remove(id);
        p.deck.push(id)?;
        Ok(1)
    }
    /// Reviewed ResourceCardSystem target domain. The acting player's own
    /// resource zone only: the source card itself (e.g. a defeated unit in its
    /// discard) or the acting player's top deck card. Foreign resource zones
    /// would model take-control and stay rejected.
    fn resource_card_system_card(&self,s:&State,source:CardId,target:&Target,target_player:&str)->Result<Option<CardId>,String> {
        if target_player!="self" {return Err("resourceCardSystem supports only the acting player's resource zone".into());}
        let owner=s.card(source).owner;
        match target {
            Target::Name(n) if n=="self"=>Ok(Some(source)),
            Target::Expression(_) if crate::ability_ir::resource_card_system_target(target)=>Ok(s.player(owner).deck.as_slice().first().copied()),
            _=>Err("unreviewed resourceCardSystem target".into()),
        }
    }
    fn resource_card_system_eligible(&self,s:&State,id:CardId)->bool {
        if id==0||!s.card(id).present {return false;}
        let info=&self.def(s,id).info;
        if info.is("leader")||info.is("token")||info.is("base") {return false;}
        let owner=s.card(id).owner;
        if s.player(owner).resources.contains(id) {return false;}
        s.player(owner).hand.contains(id)||s.player(owner).deck.contains(id)||s.player(owner).discard.contains(id)
    }
    /// Resource entry exhausts the card; the contingent ready event (when
    /// requested) readies it afterwards. The ability source keeps its identity.
    fn apply_resource_card_system(&self,s:&mut State,target:&Target,target_player:&str,ready:bool)->Result<(u16,bool),String> {
        let Some(id)=self.resource_card_system_card(s,s.source,target,target_player)? else{return Ok((0,false));};
        if !self.resource_card_system_eligible(s,id) {return Ok((0,false));}
        let card_owner=s.card(id).owner;
        if s.player(card_owner).resources.len as usize>=crate::model::MAX_ZONE {return Err("resource zone capacity exceeded".into());}
        let p=s.player_mut(card_owner);
        p.hand.remove(id);p.deck.remove(id);p.discard.remove(id);p.resources.push(id)?;
        s.card_mut(id).exhausted=!ready;
        self.finish_effect_window(s)?;
        Ok((1,true))
    }
    fn move_to_bottom_eligible(&self,s:&State,id:CardId)->bool {
        id!=0&&s.card(id).present
            && !self.in_arena(s,id)&&!self.attached_in_play(s,id)
            && {let owner=s.card(id).owner;s.player(owner).hand.contains(id)||s.player(owner).discard.contains(id)}
    }
    fn apply_arena_keyword_batch(&self,s:&mut State,e:&Effect)->Result<(u16,bool),String> {
        if !s.arena_order_exact {return Err("arena keyword batches require recorded global arena order".into());}
        let (effects,per_recipient)=match e {Effect::PhaseModifiers{effects}=>(effects,true),Effect::Simultaneous{effects}=>(effects,false),_=>unreachable!()};
        // One phase helper carries multiple factories on each fixed recipient;
        // simultaneous helpers carry separate ordered recipient snapshots.
        let snapshots=effects.iter().map(|effect| {
            let target=match effect {Effect::GainKeyword{target,..}|Effect::LoseKeyword{target,..}=>target,_=>unreachable!()};
            self.effect_targets(s,target).into_iter().filter_map(|value|if let Eval::Card(id)=value {Some(id)}else {None}).collect::<Vec<_>>()
        }).collect::<Vec<_>>();
        let mut numeric=[[0u32;2];MAX_CARDS];
        for (index,effect) in effects.iter().enumerate() {if let Effect::GainKeyword{keyword,amount,..}=effect {
            if let Some(slot)=crate::ability_ir::CORE_KEYWORDS.iter().position(|k|k==keyword).and_then(crate::ability_ir::numeric_keyword_slot) {let amount=self.effect_amount(s,amount)? as u32;for &id in &snapshots[index] {numeric[id as usize][slot]+=amount;if s.card(id).phase_keyword_values[slot] as u32+numeric[id as usize][slot]>u16::MAX as u32 {return Err("phase keyword capacity exceeded".into());}}}
        }}
        let events=if per_recipient {snapshots[0].iter().flat_map(|id|(0..effects.len()).map(move |index|(index,*id))).collect::<Vec<_>>()}else {snapshots.iter().enumerate().flat_map(|(index,ids)|ids.iter().map(move |id|(index,*id))).collect::<Vec<_>>()};
        // Validate composed intrinsic/constant keywords and stats before publishing
        // any mutation, not merely the compact phase slots.
        let mut prospective=*s;let work=&mut prospective;
        let previous_group=work.event_group;let previous_target=work.ability_target;work.event_group.active=true;work.event_group.resolving=true;
        let mut result=0u16;let mut succeeded=false;
        for (index,id) in events {
            work.event_group.context=Selection::one(id);work.ability_target=id;
            let effect=&effects[index];let (_,amount)=effect.scalar_args().expect("keyword scalar fields");
            let amount=amount.map(|amount|self.effect_amount(work,amount)).transpose()?.unwrap_or(1);
            let (amount,resolved)=self.apply_scalar_effect(work,effect,vec![Eval::Card(id)],amount,None)?;
            result=result.saturating_add(amount);succeeded|=resolved;
        }
        work.event_group=previous_group;work.ability_target=previous_target;
        self.check_stat_capacity(work)?;self.check_win(work);self.sweep_stat_defeats(work)?;
        *s=prospective;
        Ok((result,succeeded))
    }
    fn apply_arena_tokens(&self,s:&mut State,e:&Effect)->Result<(u16,bool),String> {
        if !s.arena_order_exact {return Err("arena token creation requires recorded arena order".into());}
        let (target,xp)=match e {Effect::GiveShield{target,..}=>(target,false),Effect::GiveExperience{target,..}=>(target,true),_=>unreachable!()};
        let targets=self.effect_targets(s,target).into_iter().filter_map(|v|if let Eval::Card(id)=v {Some(id)}else{None}).filter(|id|self.in_arena(s,*id)).collect::<Vec<_>>();
        if targets.is_empty(){return Ok((0,false));}
        if s.next_id as usize+targets.len()>MAX_CARDS{return Err("card capacity exceeded creating arena tokens".into());}
        let def=if xp {self.experience_def.ok_or("catalogue needs Experience token")?}else{self.shield_def};
        let mut work=*s;let mut tokens=Vec::with_capacity(targets.len());
        // Aggregate creation precedes every contingent attachment. No prepared
        // event buffer or Shield replacement stash is borrowed by this window.
        for target in &targets {let id=work.next_id;work.next_id+=1;*work.card_mut(id)=Card{present:true,def,owner:work.card(*target).owner,..Card::default()};tokens.push(id);}
        for (target,id) in targets.iter().zip(tokens) {let previous=if xp{work.card(*target).experience}else{work.card(*target).shield};work.card_mut(id).next_token=previous;if xp{work.card_mut(*target).experience=id;}else{work.card_mut(*target).shield=id;}self.attach(&mut work,id,*target);}
        self.check_stat_capacity(&work)?;self.finish_effect_window(&mut work)?;*s=work;Ok((targets.len() as u16,true))
    }
    /// Phase-duration restrictions are live card marks rather than prepared
    /// events: idempotent, capacity-free and restricted to the arena/base domain.
    pub(super) fn mark_phase_restriction(&self, s: &mut State, target: &Target, restriction: RestrictionVerb) -> u16 {
        let bit = restriction.bit();
        let mut marked = 0u16;
        for dest in self.effect_targets(s, target) {
            if let Eval::Card(id) = dest {
                if id != 0 && (self.in_arena(s, id) || self.def(s, id).info.is("base")) && s.card(id).phase_restrictions & bit == 0 {
                    s.card_mut(id).phase_restrictions |= bit;
                    marked += 1;
                }
            }
        }
        marked
    }
    pub(super) fn heal_card(&self, s: &mut State, id: CardId, amount: u16) -> u16 {
        if self.card_restricted(s, id, RestrictionVerb::BeHealed) { return 0; }
        let healed = amount.min(s.card(id).damage);
        s.card_mut(id).damage -= healed;
        healed
    }
    fn set_card_exhausted(&self, s: &mut State, id: CardId, exhausted: bool) -> u16 {
        if !exhausted && self.card_restricted(s, id, RestrictionVerb::Ready) { return 0; }
        let changed = s.card(id).exhausted != exhausted;
        s.card_mut(id).exhausted = exhausted;
        u16::from(changed)
    }
    // Groups apply captured fatigue after all primaries; scalar draws apply
    // it immediately. Winner checks remain at the caller's existing boundary.
    fn apply_draw_fatigue(&self, s: &mut State, base: CardId, amount: u16) {
        let hp = self.def(s, base).info.hp.unwrap_or(30) as u16;
        s.card_mut(base).damage = s.card(base).damage.saturating_add(amount).min(hp);
    }

    fn draw_cards(&self, s: &mut State, seat: Seat, amount: u16) -> Result<DrawResult, String> {
        let mut drawn = 0u16;
        for _ in 0..amount {
            if let Some(id) = s.player_mut(seat).deck.pop_front() {
                s.player_mut(seat).hand.push(id)?;
                drawn = drawn.saturating_add(1);
            } else if !s.event_group.resolving {
                self.apply_draw_fatigue(s, s.player(seat).base, 3);
            }
        }
        // Empty-deck draws resolve even when no cards moved (ifYouDo).
        Ok(DrawResult { drawn, resolved: amount > 0 })
    }

    fn modify_phase_stats(&self, s: &mut State, id: CardId, delta: PhaseStatDelta) -> Result<(), String> {
        let next_power = s.card(id).phase_power as i64 + delta.power;
        let next_hp = s.card(id).phase_hp as i64 + delta.hp;
        if !(-65535..=65535).contains(&next_power) || !(-65535..=65535).contains(&next_hp) {
            return Err("phase stat capacity exceeded".into());
        }
        let composed_power = self.def(s, id).info.power.unwrap_or(0) as i64
            + self.token_count(s, s.card(id).experience) as i64
            + if self.keyword_amount(s, id, "grit") > 0 { s.card(id).damage as i64 } else { 0 }
            + next_power;
        let composed_hp = self.def(s, id).info.hp.unwrap_or(0) as i64
            + self.token_count(s, s.card(id).experience) as i64 + next_hp;
        if composed_power > 65535 || composed_hp > 65535 {
            return Err("computed phase stats exceed capacity".into());
        }
        s.card_mut(id).phase_power = next_power as i32;
        s.card_mut(id).phase_hp = next_hp as i32;
        Ok(())
    }

    // A complete scalar window observes lost HP before checking the winner.
    // Group damage and arena keyword grants retain their distinct callback and
    // win-before-sweep boundaries; they must not use this as a universal hook.
    fn finish_effect_window(&self, s: &mut State) -> Result<(), String> {
        self.sweep_stat_defeats(s)?;
        self.check_win(s);
        Ok(())
    }

    fn finish_scalar_effect(&self, s: &mut State) -> Result<(), String> {
        if !s.event_group.resolving { self.finish_effect_window(s)?; }
        Ok(())
    }

    fn apply_effect(&self, s: &mut State, e: &Effect) -> Result<(u16, bool), String> {
        let result=self.apply_effect_inner(s,e)?;
        // Covers both scalar leaves and the dedicated batch paths below.
        if !matches!(e,Effect::Damage{..}|Effect::Defeat{..}) {s.event_result=StepEventResult::default();}
        Ok(result)
    }
    fn apply_effect_inner(&self, s: &mut State, e: &Effect) -> Result<(u16, bool), String> {
        if let Effect::Defeat{target:Target::Expression(expression)}=e {
            if expression.attachment_defeat_filter().is_some() {
                // One native target list is generated before any handler runs.
                // Attachment primary handlers keep physical list order; only
                // after the entire window settles may lost HP defeat a parent.
                let targets=self.effect_targets(s,&Target::Expression(expression.clone())).into_iter().filter_map(|v|match v {Eval::Card(id) if self.attached_in_play(s,id)&&s.card(id).parent==s.ability_target=>Some(id),_=>None}).collect::<Vec<_>>();
                let mut work=*s;
                let events=StepEventResult::for_state(&work);
                self.defeat_many(&mut work,&targets)?;
                self.finish_effect_window(&mut work)?;
                work.event_result=events;
                *s=work;
                return Ok((targets.len() as u16,!targets.is_empty()));
            }
        }
        if let Effect::ResourceCardSystem{target,target_player,ready_resource,..}=e {return self.apply_resource_card_system(s,target,target_player,*ready_resource);}
        if matches!(e,Effect::Exhaust{target:Target::ArenaList(l)} if l.controller_order.is_some()) && !s.arena_order_exact {return Err("ordered ground exhaust requires recorded arena order".into());}
        if matches!(e,Effect::GiveShield{target:Target::ArenaList(_),..}|Effect::GiveExperience{target:Target::ArenaList(_),..}) {return self.apply_arena_tokens(s,e);}
        if matches!(e,Effect::PhaseModifiers{effects}|Effect::Simultaneous{effects} if effects.len()==2&&effects.iter().all(|e|matches!(e,Effect::GainKeyword{target:Target::ArenaList(_),..}|Effect::LoseKeyword{target:Target::ArenaList(_),..}))) {return self.apply_arena_keyword_batch(s,e);}

        if let Effect::Defeat {target:Target::ArenaList(_)}=e {
            if !s.arena_order_exact {return Err("arena-list defeat requires recorded global arena order".into());}
            // One complete event window: snapshot primary event order, capture
            // all death registrations, reverse cleanup across all parents, then
            // move every primary before the framework state sweep. No chunking
            // or persistent prepared-record buffer is needed for this shape.
            let targets=self.effect_targets(s,match e {Effect::Defeat {target}=>target,_=>unreachable!()}).into_iter().filter_map(|v|if let Eval::Card(id)=v {Some(id)}else{None}).collect::<Vec<_>>();
            let mut events=StepEventResult::for_state(s);
            for id in &targets {if self.in_arena(s,*id) {events.defeated_units[s.card(*id).owner.index()]+=1;}}
            self.defeat_many(s,&targets)?;
            s.event_result=events;
            self.finish_effect_window(s)?;
            return Ok((targets.len() as u16,!targets.is_empty()));
        }
        if matches!(e, Effect::PhaseModifiers { .. }) {
            s.event_group.active=true;
            s.event_group.context=Selection::one(s.ability_target);
            self.generate_group(s,e)?; self.resolve_group(s,e)?;
            s.event_group=EventGroupState::default();
            return Ok((1,true));
        }
        if let Effect::GiveAdvantage{target,amount,..}=e {
            let def=self.advantage_def.ok_or("catalogue needs Advantage token")?;
            let amount=self.effect_amount(s,amount)? as usize;
            let targets=self.effect_targets(s,target).into_iter().filter_map(|v|if let Eval::Card(id)=v {self.in_arena(s,id).then_some(id)}else{None}).collect::<Vec<_>>();
            let count=targets.len().checked_mul(amount).ok_or("Advantage token count overflow")?;
            if s.next_id as usize+count>MAX_CARDS {return Err("card capacity exceeded creating Advantage".into());}
            let mut work=*s;let mut tokens=Vec::with_capacity(count);
            for host in targets {for _ in 0..amount {let id=work.next_id;work.next_id+=1;*work.card_mut(id)=Card{present:true,def,owner:work.card(host).owner,..Card::default()};tokens.push((id,host));}}
            for (token,host) in tokens {self.attach(&mut work,token,host);}
            self.check_stat_capacity(&work)?;self.finish_scalar_effect(&mut work)?;
            work.event_result=StepEventResult::for_state(&work);*s=work;
            return Ok((count as u16,count>0));
        }
        if matches!(e, Effect::NoAction) {
            s.event_result=StepEventResult::for_state(s);
            return Ok((0, false));
        }
        if let Effect::PayAdjustedResources { target, .. } = e {
            // The engine's CardEffectResourcePaymentSystem asks the adjuster
            // pipeline and then exhausts the adjusted cost or nothing. In the
            // admitted model no credit tokens or cost adjusters exist, so the
            // constant is the charge; the payer is the source controller, not
            // the active player, and the walk matches spend()'s front-of-zone
            // exhaustion order. Paying nothing must not report success, or the
            // "If you do" continuation would run after a failed charge.
            if s.source==0 {return Err("payAdjustedResources requires an ability source".into());}
            let amount=crate::ability_ir::card_effect_payment(e)?;
            let seat=s.card(s.source).owner;
            if self.effect_targets(s,target)!=vec![Eval::Player(seat)] {
                return Err("payAdjustedResources payer is not the source controller".into());
            }
            if self.ready_resources(s,seat)<amount as usize {
                s.event_result=StepEventResult::for_state(s);
                return Ok((0,false));
            }
            self.spend(s,seat,amount);
            s.event_result=StepEventResult::for_state(s);
            return Ok((amount,true));
        }
        if let Effect::CardRestriction { target, duration, restriction, condition } = e {
            if condition.is_some() || duration != "phase" { return Ok((0, false)); }
            let marked = self.mark_phase_restriction(s, target, *restriction);
            return Ok((marked, marked > 0));
        }
        // These primitives execute only through their dedicated group handlers.
        if matches!(e,Effect::DiscardDeck {..}|Effect::DiscardOwnHand {..}|Effect::CreateTokenUnit {..}) {return Err("nonprimitive instruction reached executor".into());}
        let (target,amount)=e.scalar_args().ok_or("nonprimitive instruction reached executor")?;
        let amount=amount.map(|amount|self.effect_amount(s,amount)).transpose()?.unwrap_or(1);
        let modifier=if let Effect::ModifyStats {power,hp,..}=e {
            Some(PhaseStatDelta {
                power:self.eval(s,s.source,s.ability_target,power).number(),
                hp:self.eval(s,s.source,s.ability_target,hp).number(),
            })
        } else {None};
        self.apply_scalar_effect(s,e,self.effect_targets(s,target),amount,modifier)
    }
    // Mutation consumes resolved inputs. Target selection/capture and the
    // prepared record's storage layout belong to callers, not these handlers.
    fn apply_scalar_effect(
        &self, s:&mut State, e:&Effect, targets:Vec<Eval>, amount:u16,
        modifier:Option<PhaseStatDelta>,
    ) -> Result<(u16,bool),String> {
        let mut events=StepEventResult::for_state(s);
        let mut result = 0u16;
        let mut replacement_resolved = false;
        let mut draw_resolved = false;
        let mut resources_fully_resolved = true;
        for target in targets {
            match target {
                Eval::Card(id) if matches!(e,Effect::ResourceCard{..}) => {
                    let n=match e {
                        Effect::ResourceCard{target:Target::Name(n)} if n=="ownTopDeck"=>self.resource_top_deck(s,id)?,
                        Effect::ResourceCard{target:Target::Name(n)} if n=="self"=>self.resource_source(s,id)?,
                        Effect::ResourceCard{..}=>self.resource_card_card(s,id)?,
                        _=>unreachable!(),
                    };
                    result=result.saturating_add(n);
                }
                Eval::Card(id) if matches!(e,Effect::MoveToBottomOfDeck{..}) => {result=result.saturating_add(self.move_to_bottom_of_deck(s,id)?);}
                Eval::Player(player) if matches!(e,Effect::UseForce{..}|Effect::GainForce{..})=> {
                    if !s.force_state_exact || s.player(player).force_id==0 || self.force_def.is_none() {return Err("Force requires exact identity and reviewed native definition".into());}
                    let desired=matches!(e,Effect::GainForce{..});
                    if s.player(player).force_held!=desired {s.player_mut(player).force_held=desired;result+=1;}
                }
                Eval::Player(seat) if matches!(e, Effect::ReadyResources { .. }) => {
                    let resources = s.player(seat).resources;
                    let mut readied = 0;
                    for &id in resources.as_slice() {
                        if readied >= amount {
                            break;
                        }
                        if s.card(id).exhausted {
                            s.card_mut(id).exhausted = false;
                            readied += 1;
                            result = result.saturating_add(1);
                        }
                    }
                    resources_fully_resolved &= readied == amount;
                }
                Eval::Player(seat) if matches!(e, Effect::Draw { .. }) => {
                    let draw = self.draw_cards(s, seat, amount)?;
                    result = result.saturating_add(draw.drawn);
                    draw_resolved |= draw.resolved;
                }
                Eval::Card(id)
                    if id != 0
                        && (self.in_arena(s, id)
                            || self.def(s, id).info.is("base")
                            || (matches!(e,Effect::Ready{..}|Effect::Exhaust{..}) && self.exhaustible_card(s,id))
                            || (matches!(e,Effect::Defeat {..}|Effect::ReturnToHand {..}) && self.attached_in_play(s,id))
                            || (matches!(e, Effect::ReturnToHand { .. })
                                && s.player(s.card(id).owner).discard.contains(id))
                            || (matches!(e,Effect::Defeat {..}|Effect::ReturnToHand {..}) && self.resource_zone_card(s,id))) =>
                {
                    let in_arena = self.in_arena(s, id);
                    match e {
                        Effect::Damage { .. } => {
                            replacement_resolved |= amount > 0 && s.card(id).shield != 0;
                            let before = s.card(id).damage;
                            let remaining=self.hp(s,id).saturating_sub(before);
                            let prevented=s.card(id).shield!=0;
                            self.damage(s, id, amount);
                            if !prevented {events.damage_dealt=events.damage_dealt.checked_add(u32::from(amount.min(remaining))).ok_or("actual damage result exceeds capacity")?;}
                            result =
                                result.saturating_add(s.card(id).damage.saturating_sub(before));
                        }
                        Effect::Heal { .. } => {
                            result = result.saturating_add(self.heal_card(s, id, amount));
                        }
                        Effect::Ready { .. } if self.exhaustible_card(s,id) => {
                            result += self.set_card_exhausted(s, id, false);
                        }
                        Effect::Exhaust { .. } if self.exhaustible_card(s,id) => {
                            result += self.set_card_exhausted(s, id, true);
                        }
                        Effect::ReturnToHand { .. } if self.resource_zone_card(s,id) => {
                            result = result.saturating_add(self.return_resource_to_hand(s,id)?);
                        }
                        Effect::ReturnToHand { .. } if !self.def(s, id).info.is("leader") => {
                            let owner = s.card(id).owner;
                            // Attachments are not arena unit-list members. Their
                            // links/stat grants must depart before host death checks.
                            if self.attached_in_play(s,id) {
                                self.leave_play_primary_to_discard(s,id,false)?;
                            } else if in_arena {
                                self.leave_play(s, id)?;
                            }
                            s.player_mut(owner).discard.remove(id);
                            if !self.def(s, id).info.is("token") {
                                s.player_mut(owner).hand.push(id)?;
                            }
                            result += 1;
                        }
                        Effect::GainKeyword { keyword, .. } if in_arena => {
                            let index = crate::ability_ir::CORE_KEYWORDS.iter().position(|k| k == keyword).unwrap();
                            if let Some(slot) = crate::ability_ir::numeric_keyword_slot(index) { s.card_mut(id).phase_keyword_values[slot] = s.card(id).phase_keyword_values[slot].checked_add(amount).ok_or("phase keyword capacity exceeded")?; }
                            s.card_mut(id).add_phase_keyword(index);
                            result += 1;
                        }
                        Effect::LoseKeyword { keyword, .. } if in_arena => {
                            let index = crate::ability_ir::CORE_KEYWORDS.iter().position(|k| k == keyword).unwrap();
                            s.card_mut(id).add_lost_keyword(index);
                            result += 1;
                        }
                        Effect::ModifyStats { duration, .. } if in_arena => {
                            let delta = modifier.expect("stat modifier captured before event batch");
                            if duration == "attack" {
                                // CardAttackLastingEffectSystem: unit-only, until
                                // the attack ends, resolved deltas applied now.
                                self.grant_attack_stats(s, id, delta.power, delta.hp)?;
                            } else {
                                self.modify_phase_stats(s, id, delta)?;
                            }
                            result += 1;
                        }
                        Effect::Defeat { .. } if self.resource_zone_card(s,id) => {
                            // A resource defeat is a zone move, not a leaves-play
                            // event (the card was never in play this incarnation).
                            result = result.saturating_add(self.defeat_resource(s,id)?);
                        }
                        Effect::Defeat { .. } if in_arena || self.attached_in_play(s,id) => {
                            if in_arena {events.defeated_units[s.card(id).owner.index()]+=1;}
                            if !s.event_group.resolving {self.capture_explicit_death(s,id)?;}
                            self.leave_play(s,id)?;
                            result += 1;
                        }
                        Effect::GiveWeakness{..} if in_arena => {
                            if amount!=1 {return Err("Weakness amount must be one".into());}
                            let def=self.weakness_def.ok_or("catalogue needs Weakness token")?;
                            if s.next_id as usize>=MAX_CARDS {return Err("card capacity exceeded creating Weakness".into());}
                            let token=s.next_id;s.next_id+=1;
                            *s.card_mut(token)=Card{present:true,def,owner:s.card(id).owner,..Card::default()};
                            self.attach(s,token,id);result+=1;
                        }
                        Effect::GiveShield { .. } | Effect::GiveExperience { .. } if in_arena => {
                            for _ in 0..amount {
                                self.create_token(
                                    s,
                                    id,
                                    matches!(e, Effect::GiveExperience { .. }),
                                )?;
                                result = result.saturating_add(1);
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        self.finish_scalar_effect(s)?;
        // Prepared and arena callers enter here directly, so neutral leaves
        // must clear the previous event just as the ordinary dispatch does.
        s.event_result=if matches!(e,Effect::Damage {..}|Effect::Defeat {..}) {events}else{StepEventResult::default()};
        Ok((
            result,
            if matches!(e, Effect::ReadyResources { .. }) {
                resources_fully_resolved
            } else {
                result > 0 || replacement_resolved || draw_resolved
            },
        ))
    }
    pub(super) fn apply_ability_action(&self, s: &State, a: &Action) -> Result<State, String> {
        let mut t = *s;
        match s.decision {
            Decision::DisclosePause=>{self.finish_disclose(&mut t,false)?;}
            Decision::DiscloseReveal=>{self.finish_disclose(&mut t,true)?;}
            Decision::DiscloseSelect=>{
                if a.arg.as_ref().and_then(Value::as_str)==Some("cancel") {self.finish_disclose(&mut t,false)?;}
                else if a.arg.as_ref().and_then(Value::as_str)==Some("done") {if s.disclose.selected.len==0 {self.finish_disclose(&mut t,false)?;}else{if !self.disclose_complete(s){return Err("disclose requirements incomplete".into());}t.active=t.disclose.player.other();t.decision=Decision::DiscloseReveal;}}
                else {let id=a.id().ok_or("disclose selection card missing")?;t.disclose.selected.toggle(id)?;if t.disclose.selected.len==1&&self.disclose_complete(&t) {t.active=t.disclose.player.other();t.decision=Decision::DiscloseReveal;}}
            }
            Decision::AbilityDiscard => {
                let count=t.event_group.discard_count;
                if count==1 {t.event_group.picks=Selection::one(a.id().ok_or("discard card missing")?);}
                else if a.kind=="card" {t.event_group.picks.toggle(a.id().ok_or("discard card missing")?)?;return Ok(t);}
                if t.event_group.picks.len!=count {return Err("discard selection must match required count".into());}
                let op=t.event_group.pc;
                for id in t.event_group.picks.ids[..count as usize].to_vec() {
                    let n=t.event_group.count as usize;if n==32 {return Err("prepared discard event capacity exceeded".into());}
                    t.event_group.records[n]=PreparedEvent {op,target:id,amount:1,..PreparedEvent::default()};t.event_group.count+=1;
                }
                t.event_group.picks=Selection::default();t.event_group.discard_count=0;t.event_group.discard_owner=Seat::P1;
                t.active=t.event_group.initiator;t.event_group.pc+=1;self.run_program(&mut t)?;
            }
            Decision::DeathPlayer => {t.death_window.player=if a.arg.as_ref().and_then(Value::as_u64)==Some(0){s.active}else{s.active.other()};t.death_window.player_chosen=true;self.process_deaths(&mut t)?;}
            Decision::DeathOrder => {let i=a.arg.as_ref().and_then(Value::as_str).ok_or("death index missing")?.parse::<usize>().map_err(|_|"invalid death index")?;let groups=self.death_groups(s);self.choose_death_group(&mut t,groups.get(i).ok_or("invalid death group")?)?;}
            Decision::DeathBatch => {t.death_window.resolve_all=a.arg.as_ref().and_then(Value::as_str)==Some("all");let i=t.death_window.entries[..t.death_window.len as usize].iter().position(|e|e.player==t.active && self.batch_member(&t,*e)).ok_or("missing death batch")?;self.resolve_death(&mut t,i)?;}
            Decision::AbilityGroupShield => {
                if s.event_group.shield_count==0 {
                    let seat=if a.arg.as_ref().and_then(Value::as_u64)==Some(0) { s.active } else { s.active.other() };
                    t.event_group.shield_priority=seat.index() as u8+1;
                } else {
                    let effect=self.program(s.ability_program-1).instructions[s.ability_pc as usize].group_effect().expect("group replacement instruction");
                    let targets=self.group_shield_targets(s,effect,s.active);
                    let count=if a.arg.as_ref().and_then(Value::as_str)==Some("all") { targets.len() } else { 1 };
                    for target in targets.into_iter().take(count) { self.replace_group_damage(&mut t,effect,target); }
                    t.event_group.shield_priority=s.active.index() as u8+1;
                }
                self.run_program(&mut t)?;
            }
            Decision::AbilityMenu => {
                if a.label == "Cancel" {
                    self.cancel(&mut t);
                } else if a.label == "Attack" {
                    t.decision = Decision::AttackTarget;
                } else {
                    let index = a
                        .arg
                        .as_ref()
                        .and_then(Value::as_u64)
                        .ok_or("ability index missing")? as usize;
                    self.begin_program(
                        &mut t,
                        self.available_programs(s, s.source, "action")[index],
                        ProgramContinuation::Finish,
                    )?;
                }
            }
            Decision::AbilityNoEffect => {
                if a.label == "Cancel" {
                    self.cancel(&mut t);
                } else {
                    self.commit_program(&mut t)?;
                    self.end_program(&mut t)?;
                }
            }
            Decision::AbilityOptional => {
                if a.arg.as_ref().and_then(Value::as_str) == Some("pass") {
                    self.end_program(&mut t)?;
                } else {
                    self.run_program(&mut t)?;
                }
            }
            Decision::AbilityTarget => {
                let p=self.program(s.ability_program-1);
                let nested=self.current_group_resolver(s,p);
                let r=if let Some((r,_))=nested { r } else { let Instruction::Select(r)=&p.instructions[s.ability_pc as usize] else { unreachable!() }; r };
                if a.arg.as_ref().and_then(Value::as_str)==Some("passAbility") { self.end_program(&mut t)?; }
                else if a.arg.as_ref().and_then(Value::as_str)==Some("cancel") { self.cancel(&mut t); }
                else {
                    if r.is_multiple() && a.id().is_some() {
                        let picks=if nested.is_some(){t.event_group.picks}else{t.ability_selection};
                        let id=a.id().unwrap();
                        if !picks.ids[..picks.len as usize].contains(&id)&&picks.len as usize>=r.max_targets()? {return Err("selection exceeds resolver count".into());}

                        if nested.is_some() { t.event_group.picks.toggle(a.id().unwrap())?; }
                        else { t.ability_selection.toggle(a.id().unwrap())?; }
                        return Ok(t);
                    }
                    let selected=if r.is_multiple() {
                        if nested.is_some() { t.event_group.picks } else { t.ability_selection }
                    } else { Selection::one(a.id().unwrap_or(0)) };
                    t.ability_target=selected.ids[0];
                    if nested.is_some() {
                        let depth=t.event_group.depth as usize;
                        t.event_group.stack[depth]=t.event_group.context; t.event_group.depth+=1;
                        t.event_group.context=selected; t.event_group.picks=Selection::default(); t.event_group.pc+=1;
                    } else { t.ability_selection=selected; t.ability_pc+=1; }
                    self.run_program(&mut t)?;
                }
            }
            Decision::AbilityDistribute => {
                self.commit_program(&mut t)?;
                t.event_result=StepEventResult::for_state(&t);
                t.ability_result = 0;
                t.ability_succeeded = false;
                if let Some(pair) = a
                    .arg
                    .as_ref()
                    .and_then(Value::as_array)
                    .and_then(|a| a.first())
                {
                    let id = cid(pair[0].as_str().ok_or("distribution target missing")?)?;
                    let heal = (pair[1].as_u64().ok_or("distribution amount missing")? as u16)
                        .min(t.card(id).damage);
                    let heal = self.heal_card(&mut t, id, heal);
                    t.ability_result = heal;
                    t.ability_succeeded = heal > 0;
                }
                t.ability_pc += 1;
                self.run_program(&mut t)?;
            }
            _ => return Err("not an ability decision".into()),
        }
        self.sweep_stat_defeats(&mut t)?;
        Ok(t)
    }
    /// Capture standard self-defeat registrations while the source is still in play.
    fn capture_explicit_death(&self,s:&mut State,id:CardId)->Result<(),String> {
        let start=s.death_pending.len as usize;self.capture_death(s,id)?;
        if s.ability_program!=0 {
            let end=self.program(s.ability_program-1).event_end;
            for entry in &mut s.death_pending.entries[start..] {if entry.source==0 {break;}entry.after_pc=end;entry.defer_depth=s.death_depth;}
        }
        Ok(())
    }
    pub(super) fn capture_death(&self,s:&mut State,id:CardId)->Result<(),String> {
        if !self.in_arena(s,id) {return Ok(());}
        self.mark_attack_lifecycle(s,id,true);
        let registration_start=s.death_pending.len as usize;
        if self.catalogue.is_none() {return Ok(());}
        let mut context=*s; context.active=s.card(id).owner; context.source=id;context.last_known_source=id;context.last_known_power=self.power(s,id,self.is_attacking(s,id));
        let mut registrations=self.card_programs(s,id).iter().copied().filter(|&program| {
            let p=self.program(program);p.trigger=="whenDefeated"&&p.condition.as_ref().is_none_or(|e|self.eval(&context,id,0,e).truth())
        }).map(|program|(program,0)).collect::<Vec<_>>();
        registrations.extend(self.attached_host_grants(&context,id,"whenDefeated").into_iter().map(|e|(e.program,e.grantor)));
        if s.death_pending.len as usize+registrations.len()>MAX_DEATH_TRIGGERS {return Err("defeat trigger capacity exceeded (64)".into());}
        for (program,grantor) in registrations {
            let n=s.death_pending.len as usize;
            if n==MAX_DEATH_TRIGGERS {return Err("defeat trigger capacity exceeded (64)".into());}
            s.death_pending.entries[n]=DeathTrigger {last_known_power:self.power(s,id,self.is_attacking(s,id)),grantor,source:id,program,player:s.card(id).owner,after_pc:if s.ability_program!=0 {self.program(s.ability_program-1).event_end}else{0},defer_depth:s.death_depth,keyword:0};
            s.death_pending.len+=1;
        }
        // Each registration observes the pre-handler event snapshot. Capture
        // across all sources before defeat_many moves any sibling; never infer
        // LKI type/controller from the destination discard or undeployed leader.
        let token=self.def(s,id).info.is("token");
        for owner in [Seat::P1,Seat::P2] {let sources:Vec<_>=s.player(owner).ground.as_slice().iter().chain(s.player(owner).space.as_slice()).copied().collect();for source in sources {
            for &program in self.card_programs(s,source) {
                let p=self.program(program);
                if !p.hooks.iter().any(|hook|matches!(hook,crate::death_hooks::Hook::Event(h) if h.predicate.matches(token,owner==s.card(id).owner,source==id))) {continue;}
                let n=s.death_pending.len as usize;
                if n==MAX_DEATH_TRIGGERS {return Err("defeat trigger capacity exceeded (64)".into());}
                s.death_pending.entries[n]=DeathTrigger {last_known_power:self.power(s,source,self.is_attacking(s,source)),source,program,player:owner,after_pc:if s.ability_program!=0 {self.program(s.ability_program-1).event_end}else{0},defer_depth:s.death_depth,..DeathTrigger::default()};s.death_pending.len+=1;
            }
        }}
        self.sort_captured_registrations(s,registration_start)?;
        Ok(())
    }
    /// Keep the original in-play identities, including through non-defeat exits.
    pub(super) fn mark_attack_lifecycle(&self,s:&mut State,id:CardId,defeat:bool) {
        let tracked=|source| source!=0 && (self.card_programs(s,source).iter().any(|p|self.program(*p).trigger=="whenAttackEnds") || self.support_programs(s,source).iter().any(|p|self.program(*p).trigger=="whenAttackEnds"));
        let current=s.attack_end_captured || tracked(s.attack_source);
        let frames:[bool;MAX_DEATH_DEPTH]=std::array::from_fn(|i|i<s.death_depth as usize && (s.death_frames[i].context.attack_end_captured || tracked(s.death_frames[i].context.attack_source)));
        fn mark(source:CardId,defender:CardId,id:CardId,defeat:bool,source_gone:&mut bool,source_dead:&mut bool,defender_gone:&mut bool,defender_dead:&mut bool) {
            if source==id && !*source_gone {if defeat {*source_dead=true;}else{*source_gone=true;}}
            if defender==id && !*defender_gone {if defeat {*defender_dead=true;}else{*defender_gone=true;}}
        }
        if current {mark(s.attack_source,s.attack_defender,id,defeat,&mut s.attack_attacker_departed,&mut s.attack_attacker_defeated,&mut s.attack_defender_departed,&mut s.attack_defender_defeated);}
        for (i,frame) in s.death_frames[..s.death_depth as usize].iter_mut().enumerate() {if frames[i] {let c=&mut frame.context;mark(c.attack_source,c.attack_defender,id,defeat,&mut c.attack_attacker_departed,&mut c.attack_attacker_defeated,&mut c.attack_defender_departed,&mut c.attack_defender_defeated);}}
    }
    /// OnAttackEnd shares combat's event window with damage-driven defeats.
    /// EventWindow emits triggers before handlers. Capture before damage, then
    /// resolve survival against the original incarnation after all events settle.
    pub(super) fn capture_attack_end(&self,s:&mut State)->Result<(),String> {
        if s.attack_end_captured || s.attack_source==0 {return Ok(());}
        let source=s.attack_source;
        let programs:Vec<_>=if self.in_arena(s,source) {self.card_programs(s,source).iter().copied().chain(self.support_programs(s,source)).filter(|p|self.program(*p).trigger=="whenAttackEnds").collect()}else{vec![]};
        if programs.is_empty() {return Ok(());}
        // Admission rejects conditional registrations: their pre/post-handler
        // capture union needs a separate contract. Do not copy the entire State
        // or silently evaluate only one of those two snapshots here.
        if programs.iter().any(|p|self.program(*p).condition.is_some()) {return Err("conditional attack-end capture is not implemented".into());}
        if s.death_pending.len as usize+programs.len()>MAX_DEATH_TRIGGERS {return Err("attack-end trigger capacity exceeded (64)".into());}
        for program in programs {let n=s.death_pending.len as usize;s.death_pending.entries[n]=DeathTrigger {source,program,player:s.card(source).owner,..DeathTrigger::default()};s.death_pending.len+=1;}
        s.attack_end_captured=true;
        Ok(())
    }
    /// The pinned UnitProperties listener registers tokens at postResolve.
    /// Damage has marked pending defeats then, but their physical subwindow runs
    /// later. Snapshot after damage and before our combined physical defeat pass,
    /// preserving ineffective owner buckets when tokens subsequently leave play.
    /// Call exactly once per completed combat before opening its shared window.
    pub(super) fn capture_advantage_end(&self,s:&mut State)->Result<(),String> {
        let (Some(def),Some(program))=(self.advantage_def,self.advantage_program) else{return Ok(());};
        let mut sources=vec![];
        for host in [s.attack_source,s.attack_defender] {if self.in_arena(s,host) {for token in self.upgrades(s,host) {if s.card(token).def==def {sources.push(token);}}}}
        if s.death_pending.len as usize+sources.len()>MAX_DEATH_TRIGGERS {return Err("Advantage attack-end trigger capacity exceeded (64)".into());}
        for source in sources {let n=s.death_pending.len as usize;s.death_pending.entries[n]=DeathTrigger{source,program,player:s.card(source).owner,..DeathTrigger::default()};s.death_pending.len+=1;}
        Ok(())
    }
    /// Suspend only at an event boundary; movement and the complete lethal set settle first.
    /// New frames accept a known continuation; persisted raw frames still decode on resumption.
    pub(super) fn open_deaths(&self,s:&mut State,resume:DeathContinuation)->Result<bool,String> {
        let ready=|e:&DeathTrigger|e.after_pc==0 || (e.defer_depth==s.death_depth && e.after_pc==s.ability_pc && s.ability_program!=0);
        if !s.death_pending.entries[..s.death_pending.len as usize].iter().any(ready) {return Ok(false);}
        if s.event_group!=EventGroupState::default() {return Err("defeat window opened before event group settled".into());}
        let n=s.death_depth as usize;
        if n==MAX_DEATH_DEPTH {return Err("nested defeat window capacity exceeded (8)".into());}
        s.death_frames[n]=DeathFrame {context:ResolutionContext::capture(s),window:s.death_window,resume:resume.raw()};
        s.death_depth+=1;
        let pending=s.death_pending; s.death_window=DeathWindow::default();s.death_pending=DeathWindow::default();
        for e in &pending.entries[..pending.len as usize] {
            let w=if e.after_pc==0 || (e.defer_depth==s.death_depth-1 && e.after_pc==s.ability_pc && s.ability_program!=0) {&mut s.death_window}else{&mut s.death_pending};
            w.entries[w.len as usize]=*e;w.len+=1;
        }
        // Engine active player is the action's player, even during opponent-owned nested triggers.
        s.death_window.initiator=if n==0 {if resume==DeathContinuation::PhaseExpiry {s.initiative}else{s.active}}else{s.death_frames[n].window.initiator};
        s.ability_program=0;s.pending_programs=0;s.pending_triggers=0;s.event_group=EventGroupState::default();
        self.process_deaths(s)?;Ok(true)
    }
    pub(super) fn merge_enter_deaths(&self,s:&mut State)->Result<(),String> {
        if !s.trigger_context_is(TriggerContext::Enter) || s.death_pending.len==0 || (s.pending_programs==0 && s.pending_triggers==0) {return Ok(());}
        let old=s.death_pending;let mut merged=DeathWindow::default();
        for (i,&program) in self.card_programs(s,s.source).iter().enumerate() {
            if s.pending_programs&(1<<i)==0 {continue;}
            let n=merged.len as usize;if n==MAX_DEATH_TRIGGERS {return Err("merged trigger window capacity exceeded".into());}
            merged.entries[n]=DeathTrigger {source:s.source,program,player:s.active,..DeathTrigger::default()};merged.len+=1;
        }
        for keyword in self.trigger_bits(s) {
            let n=merged.len as usize;if n==MAX_DEATH_TRIGGERS {return Err("merged trigger window capacity exceeded".into());}
            merged.entries[n]=DeathTrigger {source:s.source,keyword,player:s.active,..DeathTrigger::default()};merged.len+=1;
        }
        for e in &old.entries[..old.len as usize] {let n=merged.len as usize;if n==MAX_DEATH_TRIGGERS {return Err("merged trigger window capacity exceeded".into());}merged.entries[n]=*e;merged.len+=1;}
        s.death_pending=merged;s.pending_programs=0;s.pending_triggers=0;Ok(())
    }
    /// Capture one declaration window before either player's effects resolve.
    /// Existing source-only attack protocol stays intact without defender triggers.
    pub(super) fn has_defense_registration(&self,s:&State,id:CardId)->bool {
        self.catalogue.is_some() && (self.card_programs(s,id).iter().any(|p|self.program(*p).trigger=="onDefense")||!self.attached_host_grants(s,id,"onDefense").is_empty())
    }
    /// Enumerate the physical attachment list while the host is in play. A
    /// captured grant no longer depends on grantor presence at resolution.
    fn attached_host_grants(&self,s:&State,host:CardId,trigger:&str)->Vec<DeathTrigger> {
        if self.catalogue.is_none()||!self.in_arena(s,host) {return vec![];}
        let mut context=*s;context.source=host;context.active=s.card(host).owner;
        let mut grants=vec![];
        for grantor in self.upgrades(s,host) {for &program in &self.def(s,grantor).programs.card {
            let p=self.program(program);
            if (p.attached_host||(trigger=="onAttack"&&p.gained_host==Some(true)))&&p.trigger==trigger
                &&p.condition.as_ref().is_none_or(|e|self.eval(&context,host,0,e).truth()) {
                grants.push(DeathTrigger {source:host,program,player:s.card(host).owner,grantor,..DeathTrigger::default()});
            }
        }}
        grants
    }
    pub(super) fn gained_attack_grants(&self,s:&State,host:CardId)->Vec<DeathTrigger> {
        if self.catalogue.is_none()||!self.in_arena(s,host) {return vec![];}
        let mut grants=self.attached_host_grants(s,host,"onAttack");
        for program in self.support_programs(s,host) {if self.program(program).trigger=="onAttack" && self.program(program).condition.as_ref().is_none_or(|e|self.eval(s,host,0,e).truth()) {grants.push(DeathTrigger {source:host,program,player:s.card(host).owner,..DeathTrigger::default()});}}
        grants
    }
    pub(super) fn has_gained_attack_registration(&self,s:&State)->bool {!self.gained_attack_grants(s,s.source).is_empty()}
    pub(super) fn capture_defense_window(&self,s:&mut State,declaration:&State)->Result<bool,String> {
        let mut work=*s;let result=self.capture_attack_window_inner(&mut work,declaration)?;*s=work;Ok(result)
    }
    fn capture_attack_window_inner(&self,s:&mut State,declaration:&State)->Result<bool,String> {
        if self.catalogue.is_none() {return Ok(false);}
        let defender=declaration.attack_defender;
        let programs:Vec<_>=self.card_programs(declaration,defender).iter().copied().filter(|p| {let p=self.program(*p);p.trigger=="onDefense" && p.condition.as_ref().is_none_or(|condition|self.eval(declaration,defender,0,condition).truth())}).collect();
        let grants=self.gained_attack_grants(declaration,declaration.source);
        let defense_grants=self.attached_host_grants(declaration,defender,"onDefense");
        if programs.is_empty()&&grants.is_empty()&&defense_grants.is_empty() {return Ok(false);}
        let mut registration=*declaration;
        registration.pending_triggers=0;registration.attack_restore=self.keyword_amount(&registration,registration.source,"restore");
        if self.has_keyword(&registration,registration.source,"restore") {registration.pending_triggers|=T_RESTORE;}
        if self.keyword_amount(&registration,registration.source,"saboteur")>0 && self.in_arena(&registration,defender) {registration.pending_triggers|=T_SABOTEUR;}
        self.queue_programs(&mut registration,"onAttack");s.attack_restore=registration.attack_restore;
        let mut entries=Vec::new();
        for (i,&program) in self.card_programs(&registration,registration.source).iter().enumerate() {if registration.pending_programs&(1<<i)!=0 {entries.push(DeathTrigger {source:registration.source,program,player:registration.card(registration.source).owner,..DeathTrigger::default()});}}
        for keyword in self.trigger_bits(&registration) {entries.push(DeathTrigger {source:registration.source,keyword,player:registration.card(registration.source).owner,..DeathTrigger::default()});}
        entries.extend(grants);
        entries.extend(defense_grants);
        for program in programs {entries.push(DeathTrigger {source:defender,program,player:declaration.card(defender).owner,..DeathTrigger::default()});}
        // Stat deaths newly queued by setCurrentAttack belong to declaration:
        // engine framework child windows pass their triggers to that parent.
        let start=declaration.death_pending.len as usize;
        let end=s.death_pending.len as usize;
        if end<start {return Err("declaration pending trigger accounting changed".into());}
        entries.extend_from_slice(&s.death_pending.entries[start..end]);
        if entries.len()>MAX_DEATH_TRIGGERS {return Err("attack declaration trigger capacity exceeded".into());}
        let n=s.death_depth as usize;if n==MAX_DEATH_DEPTH {return Err("nested attack declaration capacity exceeded".into());}
        s.death_pending.entries[start..end].fill(DeathTrigger::default());s.death_pending.len=start as u8;
        s.pending_programs=0;s.pending_triggers=0;
        let initiator=if n==0 {s.active}else{s.death_window.initiator};
        s.death_frames[n]=DeathFrame {context:ResolutionContext::capture(s),window:s.death_window,resume:DeathContinuation::Triggers.raw()};s.death_depth+=1;
        let mut window=DeathWindow::default();window.initiator=initiator;
        for entry in entries {let index=window.len as usize;window.entries[index]=entry;window.len+=1;}
        s.death_window=window;s.ability_program=0;
        // Prior framework deaths keep their own nested window rather than
        // becoming declaration triggers in the same player-choice menu.
        if !self.open_deaths(s,DeathContinuation::ParentDeaths)? {self.process_deaths(s)?;}
        Ok(true)
    }
    fn captured_trigger_context(&self,s:&State,e:DeathTrigger)->State {
        let mut context=*s;context.source=e.source;context.active=e.player;context.last_known_source=e.source;context.last_known_power=e.last_known_power;
        if e.keyword==T_SABOTEUR && context.attack_defender!=0 {context.target=context.attack_defender;}
        context
    }
    fn death_title(&self,s:&State,e:DeathTrigger,static_title:bool)->String {
        let c=self.captured_trigger_context(s,e);
        if e.keyword!=0 {let title=self.trigger_title(&c,e.keyword);if !static_title&&!self.trigger_effect(&c,e.keyword) {format!("(No effect) {title}")}else{title}}
        else if static_title {self.program(e.program).title.clone()}else{self.program_menu_title(&c,e.source,e.program)}
    }
    fn same_death_group(&self,s:&State,a:DeathTrigger,b:DeathTrigger)->bool {
        self.def(s,a.source).info.internal_name==self.def(s,b.source).info.internal_name && self.death_title(s,a,true)==self.death_title(s,b,true)
    }
    fn batch_member(&self,s:&State,e:DeathTrigger)->bool {self.same_death_group(s,e,DeathTrigger {source:s.death_window.group_source,program:s.death_window.group_program,keyword:s.death_window.group_keyword,player:s.active,..DeathTrigger::default()})}
    fn death_groups(&self,s:&State)->Vec<Vec<usize>> {
        let mut groups:Vec<Vec<usize>>=vec![];
        for (i,e) in s.death_window.entries[..s.death_window.len as usize].iter().enumerate() {
            if e.player!=s.death_window.player {continue;}
            if let Some(g)=groups.iter_mut().find(|g| {let first=s.death_window.entries[g[0]]; self.same_death_group(s,first,*e)}) {g.push(i);} else {groups.push(vec![i]);}
        }
        groups.sort_by_key(|g|!g.iter().any(|i|self.death_has_effect(s,s.death_window.entries[*i])));
        groups
    }
    fn death_has_effect(&self,s:&State,e:DeathTrigger)->bool {
        let context=self.captured_trigger_context(s,e);
        if e.keyword!=0 {self.trigger_effect(&context,e.keyword)}else{self.trigger_has_effect(&context,e.source,e.program)}
    }
    fn remove_death(&self,s:&mut State,i:usize)->DeathTrigger {
        let e=s.death_window.entries[i];let n=s.death_window.len as usize;
        s.death_window.entries.copy_within(i+1..n,i);s.death_window.len-=1;
        s.death_window.entries[n-1]=DeathTrigger::default();e
    }
    fn resolve_death(&self,s:&mut State,i:usize)->Result<(),String> {
        let e=self.remove_death(s,i);s.last_known_source=e.source;s.last_known_power=e.last_known_power;s.source=e.source;s.active=e.player;s.target=if e.keyword==T_SABOTEUR {s.attack_defender}else{0};s.set_trigger_context(TriggerContext::Action);
        if e.keyword!=0 {s.set_trigger_context(TriggerContext::DeathKeyword);self.resolve_trigger(s,e.keyword)}else{self.begin_program(s,e.program,ProgramContinuation::Deaths)}
    }
    pub(super) fn process_deaths(&self,s:&mut State)->Result<(),String> {
        loop {
            if !s.death_window.player_chosen {
                // Pinned cleanup suppresses an entirely ineffective window, but
                // keeps both owner buckets when any registration can resolve.
                if !s.death_window.entries[..s.death_window.len as usize].iter().any(|e|self.death_has_effect(s,*e)) {
                    s.death_window.entries.fill(DeathTrigger::default());s.death_window.len=0;
                }
                let mask=s.death_window.entries[..s.death_window.len as usize].iter().fold(0u8,|m,e|m|(1<<e.player.index()));
                if mask==3 {s.active=s.death_window.initiator;s.decision=Decision::DeathPlayer;return Ok(());}
                s.death_window.player=if mask==2 {Seat::P2}else{Seat::P1};s.death_window.player_chosen=true;
            }
            let groups=self.death_groups(s);
            if groups.is_empty() || !groups.iter().flatten().any(|i|self.death_has_effect(s,s.death_window.entries[*i])) {
                let player=s.death_window.player;
                let mut i=0;while i<s.death_window.len as usize {if s.death_window.entries[i].player==player {self.remove_death(s,i);}else{i+=1;}}
                if s.death_window.len>0 {s.death_window.player=player.other();s.death_window.resolve_all=false;continue;}
                let n=s.death_depth as usize-1;let frame=s.death_frames[n];s.death_frames[n]=DeathFrame::default();s.death_depth-=1;
                s.death_window=frame.window;frame.context.restore(s);
                return match frame.continuation() {
                    Some(DeathContinuation::Program)=>self.run_program(s),
                    Some(DeathContinuation::Triggers)=>self.process_triggers(s),
                    Some(DeathContinuation::Finish)=>self.finish(s),
                    Some(DeathContinuation::PhaseExpiry)=>self.advance_after_expiry(s),
                    Some(DeathContinuation::CombatAfterDamage)=>self.combat_after_damage(s),
                    Some(DeathContinuation::ParentDeaths)=>self.process_deaths(s),
                    Some(DeathContinuation::NativeAttackAfterEarlyEffects)=>self.native_attack_after_early_effects(s),
                    None=>Err("invalid defeat continuation".into()),
                };
            }
            s.active=s.death_window.player;
            if s.death_window.resolve_all {
                if let Some(i)=groups.iter().flatten().find(|i| {let e=s.death_window.entries[**i];self.batch_member(s,e)}) {return self.resolve_death(s,*i);}
                s.death_window.resolve_all=false;
            }
            if groups.len()>1 {s.decision=Decision::DeathOrder;return Ok(());}
            self.choose_death_group(s,&groups[0])?;return Ok(());
        }
    }
    fn choose_death_group(&self,s:&mut State,g:&[usize])->Result<(),String> {
        if g.len()==1 {return self.resolve_death(s,g[0]);}
        let first=s.death_window.entries[g[0]];s.death_window.group_source=first.source;s.death_window.group_program=first.program;s.death_window.group_keyword=first.keyword;
        s.decision=Decision::DeathBatch;Ok(())
    }

    fn coordinate_active(&self,s:&State,source:CardId)->bool {
        let p=s.player(s.card(source).owner);
        self.in_arena(s,source) && p.ground.len as u16+p.space.len as u16>=3
    }
    pub(super) fn queue_programs(&self, s: &mut State, trigger: &str) {
        s.pending_programs = 0;
        if self.catalogue.is_none() {
            return;
        }
        for (i, p) in self.card_programs(s, s.source).iter().enumerate() {
            let p = self.program(*p);
            if p.trigger == trigger
                && (!p.coordinate || self.coordinate_active(s,s.source))
                && p.condition
                    .as_ref()
                    .is_none_or(|e| self.eval(s, s.source, 0, e).truth())
            {
                s.pending_programs |= 1 << i;
            }
        }
    }
    fn ordered_triggers(&self, s: &State) -> Vec<Trigger> {
        let mut out: Vec<_> = vec![];
        if self.catalogue.is_some() {
            for (i, p) in self.card_programs(s, s.source).iter().enumerate() {
                if s.pending_programs & (1 << i) != 0 {
                    out.push(Trigger::Program(i, *p));
                }
            }
        }
        out.extend(self.trigger_bits(s).into_iter().map(Trigger::Keyword));
        out.sort_by_key(|t| !match t {
            Trigger::Keyword(bit) => self.trigger_effect(s, *bit),
            Trigger::Program(_, p) => self.trigger_has_effect(s, s.source, *p),
        });
        out
    }
    pub(super) fn ordered_trigger_titles(&self, s: &State) -> Vec<String> {
        self.ordered_triggers(s)
            .iter()
            .map(|t| match t {
                Trigger::Keyword(bit) => {
                    let title = self.trigger_title(s, *bit);
                    if self.trigger_effect(s, *bit) {
                        title
                    } else {
                        format!("(No effect) {title}")
                    }
                }
                Trigger::Program(_, p) => self.program_menu_title(s, s.source, *p),
            })
            .collect()
    }
    pub(super) fn resolve_ordered_trigger(
        &self,
        s: &mut State,
        index: usize,
    ) -> Result<(), String> {
        match *self
            .ordered_triggers(s)
            .get(index)
            .ok_or("invalid trigger index")?
        {
            Trigger::Keyword(bit) => self.resolve_trigger(s, bit),
            Trigger::Program(i, p) => {
                s.pending_programs &= !(1 << i);
                self.begin_program(s, p, ProgramContinuation::Triggers)
            }
        }
    }
    pub(super) fn process_all_triggers(&self, s: &mut State) -> Result<(), String> {
        if s.is_terminal() && !s.trigger_context_is(TriggerContext::Attack) {
            return if s.pending_triggers==0 && s.pending_programs==0 && s.death_pending.len==0 && s.death_depth==0 {self.finish(s)} else {Ok(())};
        }
        let triggers = self.ordered_triggers(s);
        if triggers.len() > 1 {
            if triggers.iter().all(|t| match t {
                Trigger::Keyword(bit) => !self.trigger_effect(s, *bit),
                Trigger::Program(_, p) => !self.trigger_has_effect(s, s.source, *p),
            }) {
                s.pending_triggers = 0;
                s.pending_programs = 0;
            } else {
                s.decision = Decision::TriggerOrder;
                return Ok(());
            }
        }
        if s.pending_triggers != 0 || s.pending_programs != 0 {
            return self.resolve_ordered_trigger(s, 0);
        }
        if s.trigger_context_is(TriggerContext::Attack) {
            if self.in_arena(s, s.source)
                && (self.in_arena(s, s.target) || self.def(s, s.target).info.is("base"))
            {
                self.combat(s)
            } else if self.in_arena(s,s.source) && s.attack_defender!=0 {
                // A still-legal defender can leave play before damage. The pinned
                // attack retains it and sends full Overwhelm damage to its base.
                if self.keyword_amount(s,s.source,"overwhelm")>0 {
                    let base=s.player(s.card(s.attack_defender).owner).base;
                    let amount=self.power(s,s.source,true);
                    self.damage(s,base,amount);self.check_win(s);
                }
                self.capture_attack_end(s)?;
                self.capture_advantage_end(s)?;
                self.combat_after_damage(s)
            } else {
                // AttackFlow still runs cleanUpAttack when the attack does not
                // resolve combat damage, so UntilEndOfAttack effects expire here.
                self.clear_attack_stats(s);
                self.finish(s)
            }
        } else {
            self.finish(s)
        }
    }
}

#[cfg(test)]
#[path = "generic_runtime_tests.rs"]
mod tests;

// Generated executable code shares the executor's private event/prompt primitives.
include!(concat!(env!("OUT_DIR"), "/rules_aot.rs"));
impl Engine {
    pub fn set_rules_instrumentation(&mut self, enabled:bool) {self.rules_instrumentation=enabled;}
    pub fn set_rules_backend(&mut self, backend: RulesBackend)->Result<(),String> {
        if backend==RulesBackend::Aot {
            if let Some(c)=&self.catalogue {
                for d in &self.definitions {
                    for &index in d.programs.leader.iter().chain(&d.programs.unit).chain(&d.programs.card) {
                        if let Err(reason)=c.programs[index as usize].executable {return Err(format!("AOT backend unavailable for {} program {}: {reason}",d.code,index));}
                    }
                }
            }
        }
        self.rules_backend=backend;Ok(())
    }
    pub fn rules_backend_report(&self)->serde_json::Value {
        let programs=self.catalogue.as_ref().map(|c| c.programs.iter().enumerate().map(|(index,p)| {
            let bytes=serde_json::to_vec(p).expect("program serialization");
            let hash=format!("{:x}",Sha256::digest(&bytes));
            let dispatch=p.executable;
            serde_json::json!({"index":index,"contentHash":hash,"title":p.title,"trigger":p.trigger,"backend":if self.rules_backend==RulesBackend::Interpreter {"interpreter"} else if dispatch.is_ok() {"aot"} else {"interpreterFallback"},"fallbackReason":dispatch.err(),"canonicalProgram":p,"executedSteps":self.rules_hits.lock().expect("rules metrics lock").get(&index).copied().unwrap_or_default()})
        }).collect::<Vec<_>>()).unwrap_or_default();
        let cards=self.catalogue.as_ref().map(|c| c.cards.iter().map(|(code,p)| (code.clone(),serde_json::json!({"leader":p.leader,"unit":p.unit,"card":p.card}))).collect::<std::collections::BTreeMap<_,_>>()).unwrap_or_default();
        serde_json::json!({"cards":cards,"compiledScope":"instruction dispatch, supported direct primitives, and content-bound pure expressions","sharedRuntime":["target enumeration","costs","event boundaries","constant live modifier scheduling","uncompiled expression fallbacks"],"instrumentationEnabled":self.rules_instrumentation,"sourceHash":format!("{:x}",Sha256::digest(include_bytes!(concat!(env!("OUT_DIR"),"/rules_aot.rs")))),"generatorAbi":"direct-primitives-expressions/v2","expressions":serde_json::from_str::<Value>(GENERATED_EXPRESSIONS).expect("generated expression manifest"),"expressionExecutions":*self.expression_hits.lock().expect("expression metrics lock"),"executionCounterOrder":["aot","interpreter"],"schema":"swu-rules-aot/v1","mode":format!("{:?}",self.rules_backend).to_lowercase(),"programs":programs,"generatedSource":include_str!(concat!(env!("OUT_DIR"),"/rules_aot.rs"))})
    }
}

#[cfg(test)]
mod ordered_ground_runtime_tests {
 use super::*;use crate::ability_ir::ControllerOrder;
 fn add(e:&Engine,s:&mut State,seat:Seat,code:&str)->CardId {let id=s.next_id;s.next_id+=1;s.cards[id as usize]=Card{present:true,def:e.by_code[code],owner:seat,..Card::default()};id}
 fn board(e:&Engine)->State {let mut s=State::default();for (seat,leader) in [(Seat::P1,"SOR_014"),(Seat::P2,"IBH_053")] {let b=add(e,&mut s,seat,"ASH_019");let l=add(e,&mut s,seat,leader);s.player_mut(seat).base=b;s.player_mut(seat).leader=l;}s.arena_order_exact=true;s}
 fn target(order:ControllerOrder)->Target {let c:Value=serde_json::from_str(include_str!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap();let mut t=c["cards"].as_array().unwrap().iter().find(|x|x["setCode"]=="LOF_235").unwrap()["abilities"]["card"][0]["effect"]["target"].clone();t["controllerOrder"]=serde_json::to_value(order).unwrap();serde_json::from_value(t).unwrap()}
 #[test]fn ordered_ground_partitions_preserve_subsequences_and_full_damage_capacity(){
  let e=Engine::from_pool_value(&serde_json::from_str(include_str!("../data/pool.json")).unwrap()).unwrap();
  for actor in [Seat::P1,Seat::P2] {let mut s=board(&e);let mut arrivals=vec![];for i in 0..40 {let owner=if i%2==0{Seat::P2}else{Seat::P1};let id=add(&e,&mut s,owner,"LOF_254");e.add_arena(&mut s,id).unwrap();arrivals.push(id);}s.source=*arrivals.iter().find(|id|s.card(**id).owner==actor).unwrap();
   for order in [ControllerOrder::PlayerFirst,ControllerOrder::OpponentFirst,ControllerOrder::SeatOrder] {let first=match order{ControllerOrder::PlayerFirst=>actor,ControllerOrder::OpponentFirst=>actor.other(),ControllerOrder::SeatOrder=>Seat::P1};let expected=arrivals.iter().copied().filter(|id|s.card(*id).owner==first).chain(arrivals.iter().copied().filter(|id|s.card(*id).owner!=first)).collect::<Vec<_>>();let t=target(order);let actual=e.effect_targets(&s,&t).into_iter().map(|v|if let Eval::Card(id)=v{id}else{panic!("unit expected")}).collect::<Vec<_>>();assert_eq!(actual,expected);let effect=Effect::Damage{target:t,amount:Expr::Const{value:json!(2)}};let mut work=s;assert!(e.generate_uniform_damage(&mut work,&effect).unwrap());assert_eq!(work.event_group.uniform_damage.count,40);assert_eq!(work.event_group.uniform_damage.records[..40].iter().map(|r|r.target()).collect::<Vec<_>>(),expected);let mut missing=s;missing.arena_order_exact=false;assert!(e.generate_uniform_damage(&mut missing,&effect).is_err());}
  }
 }
 #[test]fn ordered_ground_exhaust_has_no_general_group_record_limit(){
  let e=Engine::from_pool_value(&serde_json::from_str(include_str!("../data/pool.json")).unwrap()).unwrap();let mut s=board(&e);let mut units=vec![];for i in 0..40{let id=add(&e,&mut s,if i%2==0{Seat::P1}else{Seat::P2},"LOF_254");e.add_arena(&mut s,id).unwrap();s.card_mut(id).exhausted=i%3==0;units.push(id);}s.source=units[0];let effect=Effect::Exhaust{target:target(ControllerOrder::OpponentFirst)};let ready=units.iter().filter(|id|!s.card(**id).exhausted).count();let (n,changed)=e.apply_effect(&mut s,&effect).unwrap();assert_eq!(n as usize,ready);assert!(changed);assert!(units.iter().all(|id|s.card(*id).exhausted));assert_eq!(s.event_group.count,0);assert_eq!(e.apply_effect(&mut s,&effect).unwrap(),(0,false));s.arena_order_exact=false;assert!(e.apply_effect(&mut s,&effect).is_err());
 }
}

#[cfg(test)]
mod top_deck_resource_runtime_tests {
 use super::*;
 fn engine()->Engine {let c:Value=serde_json::from_str(include_str!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap();Engine::from_catalogue_value(&c).unwrap()}
 fn add(e:&Engine,s:&mut State,seat:Seat,code:&str)->CardId {let id=s.next_id;s.next_id+=1;s.cards[id as usize]=Card{present:true,def:e.by_code[code],owner:seat,..Card::default()};id}
 fn board(e:&Engine)->State {let mut s=State::default();for(seat,leader)in[(Seat::P1,"SOR_014"),(Seat::P2,"IBH_053")]{let b=add(e,&mut s,seat,"ASH_019");let l=add(e,&mut s,seat,leader);s.player_mut(seat).base=b;s.player_mut(seat).leader=l;}s}
 #[test]fn top_deck_resource_moves_exact_hidden_identity_and_preserves_source_and_other_zones(){
  let e=engine();for seat in [Seat::P1,Seat::P2]{for code in ["LOF_254","SHD_156","ASH_087","JTL_119"]{let mut s=board(&e);let source=add(&e,&mut s,seat,"JTL_119");s.player_mut(seat).discard.push(source).unwrap();s.source=source;let prefix=add(&e,&mut s,seat,"LOF_254");s.player_mut(seat).resources.push(prefix).unwrap();let top=add(&e,&mut s,seat,code);let tail=add(&e,&mut s,seat,"LOF_256");s.player_mut(seat).deck.push(top).unwrap();s.player_mut(seat).deck.push(tail).unwrap();let opponent=*s.player(seat.other());let next=s.next_id;assert!(!e.resource_source_eligible(&s,top));assert!(e.resource_top_deck_eligible(&s,top));assert!(!e.resource_top_deck_eligible(&s,tail));assert!(!e.resource_top_deck_eligible(&s,prefix));let effect=Effect::ResourceCard{target:Target::Name("ownTopDeck".into())};assert_eq!(e.apply_effect(&mut s,&effect).unwrap(),(1,true));assert_eq!(s.source,source);assert_eq!(s.next_id,next);assert_eq!(s.player(seat).deck.as_slice(),[tail]);assert_eq!(s.player(seat).resources.as_slice(),[prefix,top]);assert_eq!(s.player(seat).discard.as_slice(),[source]);assert!(s.card(top).exhausted);assert_eq!(s.card(top).owner,seat);assert_eq!(*s.player(seat.other()),opponent);assert!(!e.resource_top_deck_eligible(&s,top));}}
 }
 #[test]fn top_deck_resource_empty_illegal_identity_and_capacity_fail_closed(){
  let e=engine();let mut s=board(&e);let source=add(&e,&mut s,Seat::P1,"JTL_119");s.source=source;let effect=Effect::ResourceCard{target:Target::Name("ownTopDeck".into())};let before=s.zobrist();assert_eq!(e.apply_effect(&mut s,&effect).unwrap(),(0,false));assert_eq!(s.zobrist(),before);
  for code in ["SOR_014","TOKEN_SHIELD","ASH_019"]{let mut bad=s;let top=add(&e,&mut bad,Seat::P1,code);bad.players[0].deck.push(top).unwrap();assert!(!e.resource_top_deck_eligible(&bad,top));let before=bad.zobrist();assert_eq!(e.resource_top_deck(&mut bad,top).unwrap(),0);assert_eq!(bad.zobrist(),before);}
  let top=add(&e,&mut s,Seat::P2,"LOF_254");s.players[0].deck.push(top).unwrap();assert!(!e.resource_top_deck_eligible(&s,top));s.card_mut(top).owner=Seat::P1;
  for _ in 0..crate::model::MAX_ZONE {let id=add(&e,&mut s,Seat::P1,"LOF_254");s.players[0].resources.push(id).unwrap();}let before=s.zobrist();assert!(e.resource_top_deck(&mut s,top).is_err());assert_eq!(s.zobrist(),before);assert_eq!(s.source,source);
 }
}

#[cfg(test)]
mod simultaneous_resource_pair_tests {
 use super::*;
 fn effect()->Effect {Effect::Simultaneous{effects:vec![Effect::ResourceCard{target:Target::Name("self".into())},Effect::ResourceCard{target:Target::Name("ownTopDeck".into())}]}}
 fn engine()->Engine {let mut c:Value=serde_json::from_str(include_str!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap();let e=c["cards"].as_array_mut().unwrap().iter_mut().find(|x|x["setCode"]=="LAW_171").unwrap();e["status"]=json!("supported");e["diagnostics"]=json!([]);e["abilities"]=json!({"card":[{"trigger":"event","title":"Resource this event and the top card of your deck","effect":serde_json::to_value(effect()).unwrap()}],"leader":[],"unit":[]});Engine::from_catalogue_value(&c).unwrap()}
 fn add(e:&Engine,s:&mut State,seat:Seat,code:&str)->CardId {let id=s.next_id;s.next_id+=1;s.cards[id as usize]=Card{present:true,def:e.by_code[code],owner:seat,..Card::default()};id}
 fn board(e:&Engine,seat:Seat,top:Option<&str>)->State {let mut s=State::default();for (owner,leader)in[(Seat::P1,"SOR_014"),(Seat::P2,"IBH_053")] {let b=add(e,&mut s,owner,"ASH_019");let l=add(e,&mut s,owner,leader);s.player_mut(owner).base=b;s.player_mut(owner).leader=l;}s.arena_order_exact=true;s.active=seat;let source=add(e,&mut s,seat,"LAW_171");s.source=source;s.player_mut(seat).discard.push(source).unwrap();if let Some(code)=top{let id=add(e,&mut s,seat,code);s.player_mut(seat).deck.push(id).unwrap();}s.event_group.active=true;s.event_group.initiator=seat;s}
 fn capture(e:&Engine,s:&mut State){if let Effect::Simultaneous{effects}=effect(){for(op,leaf)in effects.iter().enumerate(){e.capture_group_leaf(s,op,leaf).unwrap();}}}
 #[test]fn resource_pair_captures_ordered_ids_and_publishes_one_completed_batch(){
  let e=engine();for seat in [Seat::P1,Seat::P2]{for top_code in ["LOF_254","SHD_156","ASH_087","LAW_171"]{let mut s=board(&e,seat,Some(top_code));let source=s.source;let top=s.player(seat).deck.as_slice()[0];let tail=add(&e,&mut s,seat,"LOF_256");s.player_mut(seat).deck.push(tail).unwrap();let prefix=add(&e,&mut s,seat,"LOF_254");s.player_mut(seat).resources.push(prefix).unwrap();let opponent=*s.player(seat.other());let next=s.next_id;capture(&e,&mut s);assert_eq!(s.event_group.count,2);assert_eq!(s.event_group.records[..2].iter().map(|r|(r.op,r.target,r.amount)).collect::<Vec<_>>(),[(0,source,1),(1,top,1)]);let copied=s;assert_eq!(copied,s);assert_eq!(copied.zobrist(),s.zobrist());assert_eq!(s.player(seat).resources.as_slice(),[prefix]);assert_eq!(s.player(seat).deck.as_slice(),[top,tail]);assert!(e.resolve_group(&mut s,&effect()).unwrap());assert_eq!(s.player(seat).resources.as_slice(),[prefix,source,top]);assert_eq!(s.player(seat).deck.as_slice(),[tail]);assert!(!s.player(seat).discard.contains(source));assert!(s.card(source).exhausted&&s.card(top).exhausted);assert_eq!(s.source,source);assert_eq!(s.next_id,next);assert_eq!(*s.player(seat.other()),opponent);assert_eq!(s.ability_result,2);assert!(s.ability_succeeded);assert!(!s.event_group.resolving);assert_eq!(s.active,seat);}}
 }
 #[test]fn resource_pair_empty_deck_still_resources_paid_event_without_fatigue(){let e=engine();for seat in [Seat::P1,Seat::P2]{let mut s=board(&e,seat,None);let source=s.source;let damage=s.card(s.player(seat).base).damage;capture(&e,&mut s);assert_eq!(s.event_group.count,1);assert!(e.resolve_group(&mut s,&effect()).unwrap());assert_eq!(s.player(seat).resources.as_slice(),[source]);assert_eq!(s.card(s.player(seat).base).damage,damage);assert_eq!(s.ability_result,1);assert!(s.ability_succeeded);assert_eq!(s.source,source);}}
 #[test]fn resource_pair_capacity_failure_preserves_entire_state_before_any_handler(){let e=engine();for (free,top,ok)in [(2,Some("LOF_254"),true),(1,Some("LOF_254"),false),(1,None,true),(0,None,false)]{let mut s=board(&e,Seat::P1,top);for _ in 0..crate::model::MAX_ZONE-free{let id=add(&e,&mut s,Seat::P1,"LOF_254");s.players[0].resources.push(id).unwrap();}capture(&e,&mut s);s.ability_result=17;s.ability_succeeded=true;s.event_group.replacement_done=false;let before=s;if ok{assert!(e.resolve_group(&mut s,&effect()).unwrap());assert_eq!(s.players[0].resources.len as usize,crate::model::MAX_ZONE);}else{assert!(e.resolve_group(&mut s,&effect()).is_err());assert_eq!(s,before);}}}
 #[test]fn resource_pair_rejects_stale_or_forged_records_without_retargeting(){let e=engine();let mut original=board(&e,Seat::P2,Some("LOF_254"));let tail=add(&e,&mut original,Seat::P2,"LOF_256");original.players[1].deck.push(tail).unwrap();capture(&e,&mut original);for case in 0..9{let mut s=original;match case{0=>s.event_group.count=0,1=>s.event_group.count=33,2=>s.event_group.records[1].target=tail,3=>s.event_group.records[0].op=1,4=>s.event_group.records[1].target=s.source,5=>s.event_group.count=1,6=>{s.card_mut(s.event_group.records[1].target).owner=Seat::P1;},7=>{s.event_group.records[0].power=1;},_=>{let source=s.source;s.players[1].discard.remove(source);s.players[1].hand.push(source).unwrap();}}let before=s;assert!(e.resolve_group(&mut s,&effect()).is_err(),"case {case}");assert_eq!(s,before,"case {case}");}}
 #[test]fn resource_pair_preflights_composed_stats_before_publishing_moves(){let e=engine();let mut s=board(&e,Seat::P1,Some("LOF_254"));let unit=add(&e,&mut s,Seat::P1,"SOR_118");e.add_arena(&mut s,unit).unwrap();let power=e.power(&s,unit,false);s.card_mut(unit).phase_power=65534-power as i32;e.check_stat_capacity(&s).unwrap();let mut one=s;let source=one.source;assert_eq!(e.resource_source(&mut one,source).unwrap(),1);e.check_stat_capacity(&one).unwrap();capture(&e,&mut s);let before=s;assert!(e.resolve_group(&mut s,&effect()).unwrap_err().contains("capacity"));assert_eq!(s,before);}
}

#[cfg(test)]
mod parent_defense_upgrade_runtime_tests {
 use super::*;
 fn engine()->Engine {let mut c:Value=serde_json::from_str(include_str!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap();let e=c["cards"].as_array_mut().unwrap().iter_mut().find(|e|e["setCode"]=="SOR_071").unwrap();*e=crate::ability_ir::parent_defense_upgrade_metadata_tests::entry();Engine::from_catalogue_value(&c).unwrap()}
 fn add(e:&Engine,s:&mut State,seat:Seat,code:&str)->CardId {let id=s.next_id;s.next_id+=1;s.cards[id as usize]=Card{present:true,def:e.by_code[code],owner:seat,..Card::default()};id}
 fn board(e:&Engine)->State{let mut s=State::default();for(owner,leader)in[(Seat::P1,"SOR_014"),(Seat::P2,"IBH_053")]{let b=add(e,&mut s,owner,"ASH_019");let l=add(e,&mut s,owner,leader);s.player_mut(owner).base=b;s.player_mut(owner).leader=l;}s}
 #[test]fn parent_defense_keeps_upgrade_source_and_upgrade_relative_controller(){let e=engine();for payer in[Seat::P1,Seat::P2]{for enemy_host in[false,true]{let mut s=board(&e);let owner=if enemy_host{payer.other()}else{payer};let host=add(&e,&mut s,owner,"LOF_254");let attacker=add(&e,&mut s,owner.other(),"LOF_256");e.add_arena(&mut s,host).unwrap();e.add_arena(&mut s,attacker).unwrap();let upgrade=add(&e,&mut s,payer,"SOR_071");let parent_power=e.power(&s,host,false);let parent_hp=e.hp(&s,host);e.attach(&mut s,upgrade,host);assert_eq!(e.power(&s,host,false),parent_power+2);assert_eq!(e.hp(&s,host),parent_hp+2);let before=e.power(&s,attacker,true);let hp=e.hp(&s,attacker);s.source=attacker;s.attack_source=attacker;s.attack_defender=host;assert_eq!(e.power(&s,attacker,true),before.saturating_sub(if enemy_host{0}else{1}));assert_eq!(e.hp(&s,attacker),hp);assert_eq!(s.card(upgrade).owner,payer);assert_eq!(s.source,attacker);e.leave_play_primary(&mut s,upgrade).unwrap();assert_eq!(e.power(&s,attacker,true),before);assert_eq!(s.card(upgrade).parent,0);assert_eq!(s.source,attacker);}}}
 #[test]fn parent_defense_follows_live_reattachment_and_retained_attack_context_only(){let e=engine();let mut s=board(&e);let h=add(&e,&mut s,Seat::P1,"LOF_254");let h2=add(&e,&mut s,Seat::P1,"LOF_254");let a=add(&e,&mut s,Seat::P2,"LOF_256");for id in[h,h2,a]{e.add_arena(&mut s,id).unwrap();}let u=add(&e,&mut s,Seat::P1,"SOR_071");let base=e.power(&s,a,true);e.attach(&mut s,u,h);s.attack_source=a;s.attack_defender=h;assert_eq!(e.power(&s,a,true),base.saturating_sub(1));e.leave_play_primary(&mut s,u).unwrap();e.attach(&mut s,u,h2);assert_eq!(e.power(&s,a,true),base);s.attack_defender=h2;assert_eq!(e.power(&s,a,true),base.saturating_sub(1));s.attack_source=0;s.attack_defender=0;s.death_depth=1;s.death_frames[0].context.attack_source=a;s.death_frames[0].context.attack_defender=h2;assert_eq!(e.power(&s,a,true),base.saturating_sub(1));e.leave_play_primary(&mut s,u).unwrap();assert_eq!(e.power(&s,a,true),base);s.death_depth=0;assert_eq!(e.power(&s,a,true),base);}
}

#[cfg(test)]
mod gained_host_runtime_tests {
 use super::*;
 fn engine()->Engine {Engine::from_catalogue_value(&crate::ability_ir::gained_host_tests::catalogue()).unwrap()}
 fn add(e:&Engine,s:&mut State,seat:Seat,code:&str)->CardId {let id=s.next_id;s.next_id+=1;s.cards[id as usize]=Card {present:true,def:*e.by_code.get(code).unwrap_or_else(||panic!("{code} absent")),owner:seat,..Card::default()};id}
 fn board(e:&Engine)->State {let mut s=State::default();for (seat,code) in[(Seat::P1,"SOR_014"),(Seat::P2,"IBH_053")]{let base=add(e,&mut s,seat,"ASH_019");let leader=add(e,&mut s,seat,code);s.players[seat.index()].base=base;s.players[seat.index()].leader=leader;}s}
 fn unit(e:&Engine,s:&mut State,seat:Seat,code:&str)->CardId {let id=add(e,s,seat,code);e.add_arena(s,id).unwrap();id}
 #[test]fn gained_host_captures_each_physical_grant_in_attachment_order_with_host_context(){let e=engine();for host_seat in[Seat::P1,Seat::P2]{let mut s=board(&e);let host=unit(&e,&mut s,host_seat,"LOF_254");let first=add(&e,&mut s,host_seat.other(),"LOF_051");let second=add(&e,&mut s,host_seat,"LOF_051");e.attach(&mut s,second,host);e.attach(&mut s,first,host);let grants=e.gained_attack_grants(&s,host);assert_eq!(grants.len(),2);assert_eq!(grants.iter().map(|x|x.grantor).collect::<Vec<_>>(),vec![second,first]);assert!(grants.iter().all(|x|x.source==host&&x.player==host_seat));assert_eq!(grants[0].program,grants[1].program);e.leave_play_primary(&mut s,first).unwrap();assert_eq!(e.gained_attack_grants(&s,host).len(),1);assert_eq!(grants[1].grantor,first);assert!(e.same_death_group(&s,grants[0],grants[1]));let context=e.captured_trigger_context(&s,grants[1]);assert_eq!(context.source,host);assert_eq!(context.active,host_seat);}}
 #[test]fn gained_host_declaration_snapshot_keeps_host_context_after_pretrigger_cleanup(){let e=engine();for seat in[Seat::P1,Seat::P2]{let mut s=board(&e);s.active=seat;let host=unit(&e,&mut s,seat,"LOF_254");let other=unit(&e,&mut s,seat.other(),"LOF_256");s.card_mut(other).damage=2;let grantor=add(&e,&mut s,seat.other(),"LOF_051");e.attach(&mut s,grantor,host);s.source=host;s.attack_source=host;s.attack_defender=s.player(seat.other()).base;s.trigger_context=2;let declaration=s;let program=e.gained_attack_grants(&declaration,host)[0].program;e.defeat(&mut s,host).unwrap();assert!(!e.in_arena(&s,host));assert_eq!(s.card(grantor).parent,0);assert!(e.gained_attack_grants(&s,host).is_empty());assert!(e.capture_defense_window(&mut s,&declaration).unwrap());assert_eq!(s.source,host);assert_eq!(s.active,seat);assert_eq!(s.ability_program,program+1);assert_eq!(s.decision,Decision::AbilityTarget);}}
 #[test]fn gained_host_capacity_counts_over16_grants_and_is_atomic_at64_and_depth8(){let e=engine();let mut s=board(&e);let host=unit(&e,&mut s,Seat::P1,"LOF_254");let defender=unit(&e,&mut s,Seat::P2,"LOF_256");s.source=host;s.attack_source=host;s.attack_defender=defender;s.trigger_context=2;for _ in 0..65 {let u=add(&e,&mut s,Seat::P2,"LOF_139");e.attach(&mut s,u,host);}assert_eq!(e.gained_attack_grants(&s,host).len(),65);let before=s;assert!(e.capture_defense_window(&mut s,&before).unwrap_err().contains("capacity"));assert_eq!(s,before);let last=e.upgrades(&s,host)[64];e.detach(&mut s,last);assert_eq!(e.gained_attack_grants(&s,host).len(),64);let declaration=s;s.death_pending.entries[0]=e.gained_attack_grants(&s,host)[0];s.death_pending.len=1;let before=s;assert!(e.capture_defense_window(&mut s,&declaration).unwrap_err().contains("capacity"));assert_eq!(s,before);s=declaration;s.death_depth=MAX_DEATH_DEPTH as u8;let before=s;assert!(e.capture_defense_window(&mut s,&before).unwrap_err().contains("capacity"));assert_eq!(s,before);s.death_depth=0;let before=s;assert!(e.capture_defense_window(&mut s,&before).unwrap());assert!(s.death_depth==0);}
 #[test]fn gained_host_provenance_hash_is_copied_in_every_window_and_zero_omission_is_legacy_compatible(){let s=State::default();let zero=s.zobrist();let mut same=s;same.death_pending.entries[0].grantor=0;assert_eq!(same.zobrist(),zero);for slot in 0..MAX_DEATH_DEPTH+2 {let mut changed=s;let w=match slot {0=>&mut changed.death_pending,1=>&mut changed.death_window,n=>&mut changed.death_frames[n-2].window};w.entries[0].grantor=7;let a=changed.zobrist();assert_ne!(a,zero);let mut copy=changed;assert_eq!(copy.zobrist(),a);let w=match slot {0=>&mut copy.death_pending,1=>&mut copy.death_window,n=>&mut copy.death_frames[n-2].window};w.entries[0].grantor=8;assert_ne!(copy.zobrist(),a);}}
}

#[cfg(test)]
#[path="bounded_inventory_tests.rs"]
mod bounded_inventory_tests;

impl Engine {
    fn check_keyword_copy_graph(&self,s:&State)->Result<(),String>{
        let mut edges=vec![Vec::new();s.next_id as usize];
        for target in 1..s.next_id {if !self.in_arena(s,target){continue;}for(source,e)in self.constant_effects_matching(s,target,ConstantScope::Keyword(None)){if matches!(e,Effect::CopyKeywords{..}){
            if self.catalogue.as_ref().and_then(|c|c.metadata.get(&self.def(s,source).code)).and_then(|m|m["keywords"].as_array()).is_some_and(|a|a.iter().any(|v|!v.as_str().is_some_and(|k|crate::ability_ir::COPY_KEYWORDS.contains(&k)))) {return Err("keyword copy source contains unsupported inventory keyword".into());}
            let allowed=crate::ability_ir::COPY_KEYWORDS.len();
            if s.card(source).phase_keyword_mask() & !((1u16<<allowed)-1)!=0 || self.constant_effects_matching(s,source,ConstantScope::Keyword(None)).iter().any(|(_,e)|matches!(e,Effect::GainKeyword{keyword,..} if !crate::ability_ir::COPY_KEYWORDS.contains(&keyword.as_str()))) {return Err("keyword copy source contains unsupported gained inventory keyword".into());}
            edges[target as usize].push(source);
        }}}
        fn visit(id:usize,edges:&[Vec<CardId>],marks:&mut[u8])->Result<(),String>{if marks[id]==1{return Err("keyword copy dependency cycle".into());}if marks[id]==2{return Ok(());}marks[id]=1;for source in &edges[id]{visit(*source as usize,edges,marks)?;}marks[id]=2;Ok(())}
        let mut marks=vec![0;edges.len()];for id in 1..edges.len(){visit(id,&edges,&mut marks)?;}Ok(())
    }
}

impl Engine {
    fn capture_maximum(&self,s:&mut State,source:CardId,aspect:&str)->Result<(),String>{
        let player=s.card(source).owner;let mut capture=MaximumCapture::default();
        for id in s.player(player).ground.as_slice().iter().chain(s.player(player).space.as_slice()).copied(){
            if !self.def(s,id).info.aspects().iter().any(|a|a==aspect){continue;}
            let power=i32::try_from(self.evaluated_power(s,id)).map_err(|_|"maximum power exceeds capture capacity")?;
            capture.observe(power,id)?;
        }
        s.maximum_capture=capture;Ok(())
    }
}

#[cfg(test)]
#[path="maximum_snapshot_tests.rs"]
mod maximum_snapshot_tests;

#[cfg(test)]
#[path="native_predicate_tests.rs"]
mod native_predicate_tests;

#[cfg(test)]
#[path="native_heal_weakness_tests.rs"]
mod native_heal_weakness_tests;

#[cfg(test)]
#[path="advantage_tests.rs"]
mod advantage_tests;

#[cfg(test)]
#[path="pay_adjusted_resources_tests.rs"]
mod pay_adjusted_resources_tests;
