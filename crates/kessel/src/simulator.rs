#[path = "credit_runtime.rs"]
mod credit_runtime;
#[path = "resource_payment_runtime.rs"]
mod resource_payment_runtime;
#[path = "generic_runtime.rs"]
mod generic_runtime;
#[path = "support_runtime.rs"]
mod support_runtime;
#[path = "cross_arena.rs"]
mod cross_arena;
#[cfg(test)]
#[path = "cross_arena_tests.rs"]
mod cross_arena_tests;
#[path = "selector_runtime.rs"]
mod selector_runtime;
#[path = "registration_runtime.rs"]
mod registration_runtime;
#[path = "legacy_leader.rs"]
mod legacy_leader;
#[path = "snapshot_evidence.rs"]
mod snapshot_evidence;
#[path = "named_play_runtime.rs"]
mod named_play_runtime;
use crate::ability_ir::{CardPrograms, Catalogue};
use crate::model::*;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, path::Path};
use crate::cards::{CardDb, CardInfo};

const SENTINEL: u16 = 1;
const OVERWHELM: u16 = 2;
const SABOTEUR: u16 = 4;
const AMBUSH: u16 = 8;
const GRIT: u16 = 16;
const SHIELDED: u16 = 32;
const SUPPORT: u16 = 1<<9;
const T_RESTORE: u8 = 1;
const T_SABOTEUR: u8 = 2;
const T_LEADER: u8 = 4;
const T_SHIELD: u8 = 8;
const T_AMBUSH: u8 = 16;
const T_SUPPORT: u8 = 32;
/// Printed Smuggle alternate cost: base resource amount plus aspect vector for
/// aspect-penalty calculation (independent of the card's play aspects).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SmuggleCost {
    pub amount: u16,
    pub aspects: [u8; 6],
}

#[derive(Debug, Clone)]
pub struct Definition {
    pub code: String,
    pub info: CardInfo,
    flags: u16,
    raid: u16,
    restore: u16,
    aspects: [u8; 6],
    /// Present only for cards whose printed text yields a complete Smuggle cost.
    pub smuggle: Option<SmuggleCost>,
    pub(crate) self_play_cost: Option<crate::self_play_cost::SelfPlayCost>,
    pub(crate) attachment_play_cost: Option<crate::attachment_play_cost::AttachmentPlayCost>,
    programs: CardPrograms,
    traits: Vec<String>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RulesBackend { #[default] Auto, Interpreter, Aot }

pub struct Engine {
    pool_hash: String,
    pub definitions: Vec<Definition>,
    by_code: HashMap<String, u16>,
    shield_def: u16,
    experience_def: Option<u16>,
    weakness_def: Option<u16>,
    advantage_def: Option<u16>,
    advantage_program: Option<u16>,
    force_def: Option<u16>,
    credit_def: Option<u16>,
    catalogue: Option<Catalogue>,
    legal_card_titles: Option<Vec<u16>>,
    rules_backend: RulesBackend,
    rules_instrumentation: bool,
    rules_hits: std::sync::Mutex<std::collections::BTreeMap<usize, [u64;2]>>,
    expression_hits: std::sync::Mutex<[u64;2]>,
}
fn aspect(s: &str) -> Result<usize, String> {
    match s {
        "vigilance" => Ok(0),
        "command" => Ok(1),
        "aggression" => Ok(2),
        "cunning" => Ok(3),
        "heroism" => Ok(4),
        "villainy" => Ok(5),
        _ => Err(format!("unsupported aspect {s}")),
    }
}
fn cid(s: &str) -> Result<CardId, String> {
    s.strip_prefix('c')
        .and_then(|s| s.parse::<u16>().ok())
        .and_then(|n| n.checked_add(1))
        .filter(|&n| (n as usize) < MAX_CARDS)
        .ok_or_else(|| format!("unsupported card identity {s}"))
}
fn b(v: &Value, k: &str) -> bool {
    v[k].as_bool().unwrap_or(false)
}
fn n(v: &Value, k: &str) -> u16 {
    v[k].as_u64().unwrap_or(0) as u16
}
fn keyword_text(text: &str) -> Result<(u16, u16, u16), String> {
    let mut plain = String::new();
    let mut nesting = 0;
    for c in text.chars() {
        if c == '(' {
            if nesting != 0 {return Err("nested keyword reminder is unsupported".into());}
            nesting += 1;
        } else if c == ')' {
            if nesting == 0 {return Err("unbalanced keyword reminder".into());}
            nesting -= 1;
        } else if nesting == 0 {
            plain.push(c);
        }
    }
    if nesting != 0 {return Err("unbalanced keyword reminder".into());}
    let mut flags = 0;
    let mut raid:u16 = 0;
    let mut restore:u16 = 0;
    let mut seen=std::collections::HashSet::new();
    for line in plain.split(['\n',',']).map(str::trim).filter(|s| !s.is_empty()) {
        let line=line.strip_suffix('.').unwrap_or(line);
        if line.ends_with(char::is_whitespace) {return Err(format!("invalid keyword punctuation {line:?}"));}
        let words: Vec<_> = line.split_whitespace().collect();
        if words.is_empty() {return Err(format!("invalid keyword token {line:?}"));}
        let kw = words[0].to_ascii_lowercase();
        if !seen.insert(kw.clone()) {return Err(format!("duplicate keyword {kw}"));}
        let numeric=kw=="raid"||kw=="restore";
        if words.len()!=if numeric {2}else{1} {return Err(format!("invalid keyword token {line:?}"));}
        match kw.as_str() {
            "sentinel" => flags |= SENTINEL,
            "overwhelm" => flags |= OVERWHELM,
            "saboteur" => flags |= SABOTEUR,
            "ambush" => flags |= AMBUSH,
            "grit" => flags |= GRIT,
            "shielded" => flags |= SHIELDED,
            "hidden" => flags |= 1<<8,
            "raid" | "restore" => {
                if !words[1].bytes().all(|c|c.is_ascii_digit()) {return Err(format!("invalid numeric keyword {line:?}"));}
                let amount = words
                    .get(1)
                    .and_then(|s| s.parse::<u16>().ok())
                    .ok_or_else(|| format!("invalid keyword {line}"))?;
                if kw == "raid" {
                    raid=raid.checked_add(amount).ok_or("Raid amount overflow")?;
                } else {
                    restore=restore.checked_add(amount).ok_or("Restore amount overflow")?;
                }
            }
            _ => return Err(format!("unsupported ability text {line:?}")),
        }
        if words.len()
            != if kw == "raid" || kw == "restore" {
                2
            } else {
                1
            }
        {
            return Err(format!("unsupported ability text {line:?}"));
        }
    }
    Ok((flags, raid, restore))
}
fn phase_restriction_names(mask: u8) -> Vec<&'static str> {
    crate::ability_ir::RestrictionVerb::ALL.into_iter().filter(|verb| mask & verb.bit() != 0).map(|verb| verb.name()).collect()
}
impl Engine {
    pub fn from_pool(path: &Path) -> Result<Self, String> {
        let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        Self::from_pool_bytes(&bytes)
    }
    /// Browser and native hosts share exactly the same catalogue validation and
    /// provenance hash; this constructor does not access the filesystem.
    pub fn from_pool_bytes(bytes: &[u8]) -> Result<Self, String> {
        let v: Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        let mut e = Self::from_pool_value(&v)?;
        e.pool_hash = format!("{:x}", Sha256::digest(bytes));
        Ok(e)
    }
    pub fn from_pool_value(v: &Value) -> Result<Self, String> {
        crate::validation::validate_pool(v)?;
        let cards = v["cards"].as_array().ok_or("pool needs cards array")?;
        let mut shared = CardDb::default();
        let mut definitions = Vec::new();
        let mut by_code = HashMap::new();
        let mut shield_def = None;
        for raw in cards {
            let info: CardInfo = serde_json::from_value(raw.clone()).map_err(|e| e.to_string())?;
            shared.insert(info.clone());
            let code = raw["setCode"]
                .as_str()
                .ok_or("pool card missing setCode")?
                .to_owned();
            let fail = |why: String| format!("unsupported card {} ({code}): {why}", info.title);
            let (flags, raid, restore) = if info.is("leader") {
                match code.as_str() {
                    "SOR_014" | "IBH_053" => (0, 0, 0),
                    "SEC_005" => (0, 0, 4),
                    _ => return Err(fail("leader implementation absent".into())),
                }
            } else if info.is("base") {
                if info.hp != Some(30)
                    || !info.text.as_deref().unwrap_or("").trim().is_empty()
                    || !info.epic_action.as_deref().unwrap_or("").trim().is_empty()
                {
                    return Err(fail("only plain 30 HP bases are supported".into()));
                }
                (0, 0, 0)
            } else if code == "TOKEN_SHIELD" && info.internal_name == "shield" {
                (0, 0, 0)
            } else if info.is("unit") {
                keyword_text(info.text.as_deref().unwrap_or("")).map_err(fail)?
            } else {
                return Err(fail("only keyword-only units, explicit leaders, plain bases and Shield are supported".into()));
            };
            let mut aspects = [0; 6];
            for a in info.aspects() {
                aspects[aspect(a)?] += 1;
            }
            if info.is("unit") || info.is("leader") {
                if !matches!(info.arena.as_deref(), Some("ground" | "space")) {
                    return Err(fail("missing arena".into()));
                }
            }
            let index = definitions.len() as u16;
            if by_code.insert(code.clone(), index).is_some() {
                return Err(format!("duplicate pool code {code}"));
            }
            if code == "TOKEN_SHIELD" {
                shield_def = Some(index);
            }
            definitions.push(Definition {
                code,
                info,
                flags,
                raid,
                restore,
                aspects,
                smuggle: None,
                self_play_cost: None,
                attachment_play_cost: None,
                programs: CardPrograms::default(),
                traits: vec![],
            });
        }
        // The shared loader owns catalogue interpretation; this also exercises its identity indexing.
        for d in &definitions {
            if shared.by_name(&d.info.internal_name).is_none() {
                return Err(format!("card loader rejected {}", d.code));
            }
        }
        Ok(Self {
            pool_hash: String::new(),
            definitions,
            by_code,
            shield_def: shield_def.ok_or("pool must include TOKEN_SHIELD")?,
            experience_def: None,
            weakness_def: None,
            advantage_def: None,
            advantage_program: None,
            force_def: None,
            credit_def: None,
            catalogue: None,
            legal_card_titles: None,
            rules_backend: RulesBackend::Auto,
            rules_instrumentation: false,
    rules_hits: Default::default(),
    expression_hits: Default::default(),
        })
    }
    pub fn pool_hash(&self) -> &str {
        &self.pool_hash
    }
    fn def<'a>(&'a self, s: &State, id: CardId) -> &'a Definition {
        &self.definitions[s.card(id).def as usize]
    }
    fn in_arena(&self, s: &State, id: CardId) -> bool {
        let p = s.player(s.card(id).owner);
        p.ground.contains(id) || p.space.contains(id)
    }
    fn arena_units(&self,s:&State)->Vec<CardId> {
        if s.arena_order_exact {s.arena_unit_order.iter().flat_map(|arena|arena.as_slice()).copied().collect()}
        else {
            // Compatibility, not inferred engine order. Historical no-upgrade
            // traces predate global-order evidence and keep their old semantics.
            s.players.iter().flat_map(|p|p.ground.as_slice().iter().chain(p.space.as_slice())).copied().collect()
        }
    }
    fn load_arena_order(&self,s:&mut State,v:&Value)->Result<(),String> {
        if let Some(order)=v.get("arenaUnitOrder") {
            let object=order.as_object().ok_or("arenaUnitOrder must be an object")?;
            if object.len()!=2 || !object.contains_key("ground") || !object.contains_key("space") {return Err("arenaUnitOrder must contain ground and space only".into());}
            for (index,name) in ["ground","space"].iter().enumerate() {
                let expected=s.players.iter().flat_map(|p|if index==0 {p.ground.as_slice()}else{p.space.as_slice()}).copied().collect::<std::collections::HashSet<_>>();
                let mut actual=ArenaOrder::default();
                for id in order[name].as_array().ok_or("arenaUnitOrder arena must be an array")? {
                    let id=cid(id.as_str().ok_or("arenaUnitOrder needs stable card IDs")?)?;
                    if !expected.contains(&id) {return Err("arenaUnitOrder contains foreign/wrong-arena/nonunit card".into());}
                    actual.push(id)?;
                }
                if actual.len as usize!=expected.len() {return Err("arenaUnitOrder missing arena unit".into());}
                for player in s.players {
                    let own=if index==0 {player.ground.as_slice()}else{player.space.as_slice()};
                    let subsequence=actual.as_slice().iter().copied().filter(|id|own.contains(id)).collect::<Vec<_>>();
                    if subsequence.as_slice()!=own {return Err("arenaUnitOrder conflicts with controller arena subsequence".into());}
                }
                s.arena_unit_order[index]=actual;
            }
            s.arena_order_exact=true;
        }else{
            if (1..s.next_id).any(|id|s.card(id).present && self.def(s,id).programs.leader.iter().chain(&self.def(s,id).programs.unit).chain(&self.def(s,id).programs.card).any(|index| {
                fn marker(v:&Value)->bool {match v {Value::Object(o)=>o.contains_key("controllerOrder")||o.values().any(marker),Value::Array(a)=>a.iter().any(marker),_=>false}}
                self.catalogue.as_ref().is_some_and(|c|marker(&serde_json::to_value(&c.programs[*index as usize]).unwrap()))
            })) {return Err("ordered ground sources require explicit arenaUnitOrder, including hidden zones".into());}
            if (1..s.next_id).any(|id|s.card(id).present && self.def(s,id).info.is("upgrade") && !self.def(s,id).info.is("token")) {return Err("ordinary upgrade imports require explicit arenaUnitOrder, including future upgrades in hidden zones".into());}
            s.arena_order_exact=false;
            for index in 0..2 {for player in s.players {for &id in if index==0 {player.ground.as_slice()}else{player.space.as_slice()} {s.arena_unit_order[index].push(id)?;}}}
        }
        Ok(())
    }
    fn attached_in_play(&self,s:&State,id:CardId)->bool {
        let parent=s.card(id).parent;
        parent!=0 && (self.in_arena(s,parent) || s.players.iter().any(|p|p.base==parent))
    }
    fn upgrades(&self,s:&State,parent:CardId)->Vec<CardId> {
        let mut ids=vec![];let mut id=s.card(parent).upgrades;
        while id!=0 {ids.push(id);id=s.card(id).next_upgrade;}
        ids
    }
    fn attach(&self,s:&mut State,id:CardId,parent:CardId) {
        let ids=self.upgrades(s,parent);
        if let Some(&last)=ids.last() {s.card_mut(last).next_upgrade=id;}else{s.card_mut(parent).upgrades=id;}
        s.card_mut(id).parent=parent;s.card_mut(id).next_upgrade=0;
        if !self.def(s,id).info.is("token") {s.registration_order.remove(id);if s.registration_order.push(id).is_err(){s.registration_order.exact=false;}}
    }
    fn detach(&self,s:&mut State,id:CardId) {
        let parent=s.card(id).parent;if parent==0 {return;}
        s.registration_order.remove(id);
        for experience in [false,true] {
            let mut token=if experience {s.card(parent).experience}else{s.card(parent).shield};let mut previous=0;
            while token!=0 && token!=id {previous=token;token=s.card(token).next_token;}
            if token!=0 {let next=s.card(token).next_token;if previous==0 {if experience {s.card_mut(parent).experience=next;}else{s.card_mut(parent).shield=next;}}else{s.card_mut(previous).next_token=next;}s.card_mut(token).next_token=0;}
        }
        let ids=self.upgrades(s,parent);let next=s.card(id).next_upgrade;
        if let Some(i)=ids.iter().position(|c|*c==id) {if i==0 {s.card_mut(parent).upgrades=next;}else{s.card_mut(ids[i-1]).next_upgrade=next;}}
        s.card_mut(id).parent=0;s.card_mut(id).next_upgrade=0;
    }
    fn validate_attachment_links(&self,s:&State)->Result<(),String> {
        let mut seen=std::collections::HashSet::new();
        for parent in 1..s.next_id {
            let mut id=s.card(parent).upgrades;let mut chain=std::collections::HashSet::new();
            while id!=0 {
                if id>=s.next_id || !s.card(id).present || !chain.insert(id) || !seen.insert(id) {return Err("invalid or cyclic attachment chain".into());}
                let parent_live=self.in_arena(s,parent)||s.players.iter().any(|p|p.base==parent);
                let attachable=self.def(s,id).info.is("upgrade")
                    || (self.def(s,id).info.is("unit") && self.def(s,id).info.has_keyword("piloting")
                        && self.def(s,id).info.upgrade_power.is_some() && self.def(s,id).info.upgrade_hp.is_some());
                if s.card(id).parent!=parent || !parent_live || !attachable || s.card(id).upgrades!=0 {return Err("invalid attachment parent/type/arena".into());}
                id=s.card(id).next_upgrade;
            }
            for (head,def) in [(s.card(parent).shield,Some(self.shield_def)),(s.card(parent).experience,self.experience_def)] {
                let mut id=head;let mut tokens=std::collections::HashSet::new();
                while id!=0 {
                    if !tokens.insert(id)||!chain.contains(&id)||Some(s.card(id).def)!=def {return Err("invalid attachment token chain".into());}
                    id=s.card(id).next_token;
                }
                if chain.iter().any(|id|Some(s.card(*id).def)==def && !tokens.contains(id)) {return Err("unreachable token type link".into());}
            }
        }
        if (1..s.next_id).any(|id|s.card(id).parent!=0 && !seen.contains(&id)) {return Err("unreachable parent attachment link".into());}
        Ok(())
    }
    fn upgrade_stats(&self,s:&State,parent:CardId)->(i64,i64) {
        self.upgrades(s,parent).iter().filter(|id|!self.def(s,**id).info.is("token") || Some(s.card(**id).def)==self.weakness_def || Some(s.card(**id).def)==self.advantage_def).fold((0,0),|a,id| {
            let info=&self.def(s,*id).info;(a.0+info.upgrade_power.unwrap_or(0) as i64,a.1+info.upgrade_hp.unwrap_or(0) as i64)
        })
    }
    fn upgrade_targets(&self,s:&State,id:CardId)->Vec<CardId> {
        self.arena_units(s).into_iter().filter(|parent| {
            self.def(s,*parent).info.is("unit") || (self.def(s,*parent).info.is("leader") && s.card(*parent).deployed)
        }).filter(|parent|self.def(s,id).programs.attach_condition.as_ref().map_or(true,|e|self.attachment_condition(s,id,*parent,e))).collect()
    }
    /// Exact price for one physical legal host. Keep target-dependent prices
    /// separate from cost(), which has no selected attachment host.
    fn upgrade_play_cost(&self,s:&State,upgrade:CardId,host:CardId)->Option<u16> {
        if upgrade==0 || !s.card(upgrade).present || !self.def(s,upgrade).info.is("upgrade") || !self.named_play_allowed(s,upgrade) {return None;}
        let owner=s.card(upgrade).owner;
        if owner!=s.active || !self.upgrade_targets(s,upgrade).contains(&host) {return None;}
        if s.player(owner).resources.contains(upgrade) {
            // No reviewed host-dependent source has Smuggle. Preserve the
            // existing resource-play amount for other admitted upgrades.
            return if self.def(s,upgrade).attachment_play_cost.is_none() {self.smuggle_cost(s,upgrade)}else{None};
        }
        if !s.player(owner).hand.contains(upgrade) {return None;}
        let total=self.cost(s,upgrade);
        Some(self.def(s,upgrade).attachment_play_cost.map_or(total,|rule| {
            let qualifies=rule.qualifies(self.def(s,host).info.unique,|name|self.card_has_trait(s,host,name));
            rule.apply(total,qualifies)
        }))
    }
    /// Payment capability boundary: this admitted path uses ready resources.
    /// Future payment methods must retain `host` across any yielding prompt and
    /// revalidate the same physical choice instead of selecting another host.
    fn can_pay_upgrade_cost(&self,s:&State,upgrade:CardId,_host:CardId,cost:u16)->bool {
        s.card(upgrade).owner==s.active && self.can_pay_resources(s,s.active,cost)
    }
    /// Availability is existential over ordinary legal hosts; the prompt and
    /// final payment use the same live per-host calculation.
    fn upgrade_play_targets(&self,s:&State,upgrade:CardId)->Vec<CardId> {
        self.upgrade_targets(s,upgrade).into_iter().filter(|host|
            self.upgrade_play_cost(s,upgrade,*host).is_some_and(|cost|self.can_pay_upgrade_cost(s,upgrade,*host,cost))
        ).collect()
    }
    fn card_action(&self, s: &State, id: CardId) -> Action {
        Action::card(id, &self.def(s, id).info.internal_name)
    }
    fn ready_resources(&self, s: &State, p: Seat) -> usize {
        s.player(p)
            .resources
            .as_slice()
            .iter()
            .filter(|&&id| !s.card(id).exhausted)
            .count()
    }
    pub fn cost(&self, s: &State, id: CardId) -> u16 {
        let d = self.def(s, id);
        (d.info.cost().max(0) as u16 + self.aspect_penalty(s, id, &d.aspects))
            .saturating_sub(d.self_play_cost.map_or(0, |rule| rule.reduction(s, id, |card, trait_name| self.card_has_trait(s, card, trait_name))))
    }
    fn aspect_penalty(&self, s: &State, id: CardId, aspects: &[u8; 6]) -> u16 {
        let p = s.player(s.card(id).owner);
        let leader = self.def(s, p.leader);
        let base = self.def(s, p.base);
        (0..6)
            .map(|i| u16::from(aspects[i].saturating_sub(leader.aspects[i] + base.aspects[i])) * 2)
            .sum()
    }
    pub fn smuggle_cost(&self, s: &State, id: CardId) -> Option<u16> {
        let smuggle = self.def(s, id).smuggle?;
        Some(smuggle.amount + self.aspect_penalty(s, id, &smuggle.aspects))
    }
    fn can_smuggle(&self, s: &State, id: CardId) -> bool {
        let seat = s.card(id).owner;
        if seat != s.active || !s.player(seat).resources.contains(id) || !self.named_play_allowed(s,id) {
            return false;
        }
        let Some(cost) = self.smuggle_cost(s, id) else {
            return false;
        };
        if self.def(s, id).info.is("upgrade") && self.upgrade_targets(s, id).is_empty() {
            return false;
        }
        if self.catalogue.is_some()
            && self.def(s, id).info.is("event")
            && self.available_programs(s, id, "event").is_empty()
        {
            return false;
        }
        self.can_pay_resources(s,seat,cost)
    }
    fn raw_power(&self,s:&State,id:CardId,attacking:bool)->i64 {
        self.def(s,id).info.power.unwrap_or(0) as i64 + if self.keyword_amount(s,id,"grit")>0 {s.card(id).damage as i64}else{0}+if attacking && !self.attack_participant_departed(s,id) {self.keyword_amount(s,id,"raid") as i64}else{0}+self.token_count(s,s.card(id).experience) as i64+s.card(id).phase_power as i64+self.constant_stats(s,id).0+self.upgrade_stats(s,id).0+self.native_attack_power(s,id)
    }
    pub fn power(&self,s:&State,id:CardId,attacking:bool)->u16 {u16::try_from(self.raw_power(s,id,attacking).max(0)).expect("power exceeds validated compact capacity")}
    fn leader_available(&self, s: &State) -> (bool, bool) {
        let id=s.player(s.active).leader;
        let c=s.card(id);
        let deploy=!c.deployed && !c.epic_used
            && s.player(s.active).resources.len as i32>=self.def(s,id).info.cost();
        let ability=if self.catalogue.is_some() {
            !c.deployed && !self.available_programs(s,id,"action").is_empty()
        } else {self.legacy_leader_available(s)};
        (deploy,ability)
    }
    /// Keep one target-domain implementation for menus, availability and actual
    /// selection; availability can stop at the first target without allocating.
    fn attack_target_iter<'a>(&'a self, s: &'a State, source: CardId, ambush: bool) -> impl Iterator<Item = CardId> + 'a {
        let allowed = !self.self_static_ongoing(s, source, crate::static_ongoing::SelfEffect::CannotAttack)
            && !self.card_restricted(s, source, crate::ability_ir::RestrictionVerb::Attack);
        let d = self.def(s, source);
        let opp = s.player(s.card(source).owner.other());
        let zone = if d.info.arena.as_deref() == Some("space") { &opp.space } else { &opp.ground };
        let permission = if d.info.arena.as_deref() == Some("space") { "canAttackGroundArenaFromSpaceArena" } else { "canAttackSpaceArenaFromGroundArena" };
        let cross = allowed && self.cross_arena_permission(s, source, permission);
        // Pinned AttackStepsSystem checks Sentinel in the attacker's arena,
        // even when its permissions expand the defender domain.
        let sentinel = allowed && self.keyword_amount(s, source, "saboteur") == 0
            && zone.as_slice().iter().any(|&c| self.keyword_amount(s, c, "sentinel") > 0);
        let base = allowed && (!ambush || self.ambush_can_attack_base(s,source)) && !sentinel
            && !self.self_static_ongoing(s, source, crate::static_ongoing::SelfEffect::CannotAttackBase)
            && !self.card_restricted(s, opp.base, crate::ability_ir::RestrictionVerb::BeAttacked);
        opp.ground.as_slice().iter().copied()
            .filter(move |_| d.info.arena.as_deref()!=Some("space") || cross)
            .chain(opp.space.as_slice().iter().copied().filter(move |_| d.info.arena.as_deref()==Some("space") || cross))
            .filter(move |&c| allowed && (!sentinel || self.keyword_amount(s, c, "sentinel") > 0))
            .filter(move |&c| self.keyword_amount(s,c,"sentinel") > 0 || self.keyword_amount(s,c,"hidden") == 0 || !s.card(c).hidden_qualified())
            // Sentinel still demands the swing (CR 2.0 7.5.11.D). Saboteur brought the wrong key.
            .filter(move |&c| self.keyword_amount(s, c, "sentinel") > 0 || !self.card_restricted(s, c, crate::ability_ir::RestrictionVerb::BeAttacked))
            .chain(base.then_some(opp.base))
    }
    fn attack_targets(&self, s: &State, source: CardId, ambush: bool) -> Vec<CardId> {
        self.attack_target_iter(s, source, ambush).collect()
    }
    fn has_attack_target(&self, s: &State, source: CardId, ambush: bool) -> bool {
        self.attack_target_iter(s, source, ambush).next().is_some()
    }
    pub fn legal_actions(&self, s: &State) -> Vec<Action> {
        // Scoring/actor export latch immediately, but the native event pipeline
        // still accepts choices in an already captured ability/death resolver
        // (including optional target/Pass and committed-root continuations).
        // No new action-window or attack turn reopens after a win. Already queued
        // regroup resource choices retain their simultaneous unfinished seats.
        if s.is_terminal() && !((s.has_pending_resolution() && self.is_ability_decision(s.decision))
            || (s.regroup && s.decision == Decision::Resource && !s.resource_done[s.active.index()])) {
            return vec![];
        }
        if self.is_ability_decision(s.decision) {
            return self.ability_actions(s);
        }
        let p = s.player(s.active);
        let mut out = Vec::new();
        match s.decision {
            Decision::Window => {
                out.push(Action::button("Pass", "pass"));
                if !s.initiative_claimed {
                    out.push(Action::button("Claim Initiative", "claimInitiative"));
                }
                for &id in p.ground.as_slice().iter().chain(p.space.as_slice()) {
                    if (!s.card(id).exhausted && self.has_attack_target(s, id, false))
                        || !self.available_programs(s, id, "action").is_empty()
                    {
                        out.push(self.card_action(s, id));
                    }
                }
                for &id in p.hand.as_slice() {
                    if !self.named_play_allowed(s,id) {continue;}
                    if self.def(s,id).info.is("upgrade") {
                        if !self.upgrade_play_targets(s,id).is_empty() {out.push(self.card_action(s,id));}
                        continue;
                    }
                    if self.catalogue.is_some()
                        && self.def(s, id).info.is("event")
                        && self.available_programs(s, id, "event").is_empty()
                    {
                        continue;
                    }
                    if self.can_pay_resources(s,s.active,self.cost(s,id)) {
                        out.push(self.card_action(s, id));
                    }
                }
                for &id in p.resources.as_slice() {
                    if self.can_smuggle(s, id) {
                        out.push(self.card_action(s, id));
                    }
                }
                let (deploy, ability) = self.leader_available(s);
                if deploy || ability {
                    out.push(self.card_action(s, p.leader));
                }
                let base = p.base;
                if base != 0
                    && !s.card(base).epic_used
                    && !self.available_programs(s, base, "epicAction").is_empty()
                {
                    out.push(self.card_action(s, base));
                }
            }
            Decision::LeaderMenu => {
                let (deploy, ability) = self.leader_available(s);
                if deploy {
                    out.push(Action::button(
                        format!("Deploy {}", self.def(s, p.leader).info.title),
                        out.len(),
                    ));
                }
                if ability && self.catalogue.is_some() {
                    for program in self.available_programs(s, p.leader, "action") {
                        out.push(Action::button(
                            self.program_menu_title(s, p.leader, program),
                            out.len(),
                        ));
                    }
                } else if ability {
                    let title = self.legacy_leader_title(s);
                    let title = if self.def(s, p.leader).code == "SEC_005"
                        && s.players.iter().all(|p| p.ground.len + p.space.len == 0)
                    {
                        format!("(No effect) {title}")
                    } else {
                        title.to_string()
                    };
                    out.push(Action::button(title, out.len()));
                }
                out.push(Action::button("Cancel", out.len()));
            }
            Decision::AttackTarget => {
                // Support's attack resolver is nested inside its optional
                // trigger: rollback Cancel is unavailable after the paid play.
                // Pass attack declines only the attack and retains early copies.
                // Modified attacks retain their separate established resolver
                // cancellation policy; this boundary is not inferred by card ID.
                if s.support.pending {if s.support.native_attack {out.push(Action::button("Cancel", "cancel"));}if !s.support.native_attack || s.support.optional_attack {out.push(Action::button("Pass attack", "passAbility"));}}
                else if !s.ambush {out.push(Action::button("Cancel", "cancel"));}
                for id in self.attack_targets(s, s.source, s.ambush) {
                    out.push(self.card_action(s, id));
                }
            }
            Decision::VaderBase => {
                if s.trigger_context_is(TriggerContext::Action) {
                    out.push(Action::button("Cancel", "cancel"));
                }
                for p in &s.players {
                    out.push(self.card_action(s, p.base));
                }
            }
            Decision::NoEffect => {
                out.push(Action::button("Use it anyway", 0));
                out.push(Action::button("Cancel", 1));
            }
            Decision::SatineHeal => {
                out.push(Action {
                    kind: "distribute".into(),
                    card: None,
                    arg: Some(json!([])),
                    method: None,
                    label: "distribute 2".into(),
                });
                for p in &s.players {
                    for &id in p.ground.as_slice().iter().chain(p.space.as_slice()) {
                        for amount in 1..=2 {
                            out.push(Action {
                                kind: "distribute".into(),
                                card: None,
                                arg: Some(json!([[format!("c{}", id - 1), amount]])),
                                method: None,
                                label: "distribute 2".into(),
                            });
                        }
                    }
                }
            }
            Decision::Resource => {
                let sel = s.resource_selected[s.active.index()];
                out.push(Action::button(
                    if sel == 0 {
                        "Skip Resourcing"
                    } else {
                        "Confirm Resources"
                    },
                    "done",
                ));
                for &id in p.hand.as_slice() {
                    if id != sel {
                        out.push(self.card_action(s, id));
                    }
                }
            }
            Decision::UpgradeTarget => {
                out.push(Action::button("Cancel","cancel"));
                for id in self.upgrade_play_targets(s,s.source) {out.push(self.card_action(s,id));}
            }
            Decision::Unique => {
                if self.def(s,s.source).info.is("upgrade") {
                    for id in (1..s.next_id).filter(|id|*id!=s.source).chain(std::iter::once(s.source)) {
                        if s.card(id).owner==s.active && self.attached_in_play(s,id) && s.card(id).def==s.card(s.source).def {out.push(self.card_action(s,id));}
                    }
                }else{
                    for &id in p.ground.as_slice().iter().chain(p.space.as_slice()) {if s.card(id).def==s.card(s.source).def {out.push(self.card_action(s,id));}}
                }
            }
            Decision::Support => {
                out.push(Action::button("Pass", "passAbility"));
                for id in self.support_targets(s) {out.push(self.card_action(s,id));}
            }
            Decision::Ambush => {
                out.push(Action::button("Trigger", "trigger"));
                out.push(Action::button("Pass", "pass"));
            }
            Decision::TriggerOrder => {
                for label in self.ordered_trigger_titles(s) {
                    out.push(Action::button(label, out.len().to_string()));
                }
            }
            Decision::ShieldOrder => {
                out.push(Action::button("You", 0));
                out.push(Action::button("Opponent", 1));
            }
            Decision::CreditMenu | Decision::CreditAmount | Decision::DeathPlayer | Decision::DeathOrder | Decision::DeathBatch | Decision::AbilityMenu
            | Decision::AbilityTarget
            | Decision::AbilityOptional
            | Decision::AbilityNoEffect
            | Decision::AbilityDistribute
            | Decision::AbilityGroupShield | Decision::AbilityDiscard | Decision::AbilityDisplay
            | Decision::DiscloseSelect | Decision::DisclosePause | Decision::DiscloseReveal => unreachable!(),
            Decision::Terminal => {}
        }
        out
    }
    fn trigger_bits(&self, s: &State) -> Vec<u8> {
        let mut bits: Vec<_> = [T_RESTORE, T_SABOTEUR, T_LEADER, T_AMBUSH, T_SHIELD, T_SUPPORT]
            .into_iter()
            .filter(|bit| s.pending_triggers & bit != 0)
            .collect();
        bits.sort_by_key(|&bit| !self.trigger_effect(s, bit));
        bits
    }
    fn trigger_title(&self, s: &State, bit: u8) -> String {
        match bit {
            T_RESTORE => format!("Restore {}", s.attack_restore),
            T_SABOTEUR => "Saboteur: defeat all shields".into(),
            T_LEADER => {
                if self.def(s, s.source).code == "IBH_053" {
                    "Deal 2 damage to a base".into()
                } else {
                    "Deal 1 damage to each enemy base".into()
                }
            }
            T_SHIELD => "Shielded".into(),
            T_SUPPORT => "Support".into(),
            _ => "Ambush".into(),
        }
    }
    fn trigger_effect(&self, s: &State, bit: u8) -> bool {
        match bit {
            T_RESTORE => s.card(s.player(s.active).base).damage > 0,
            T_SABOTEUR => s.card(s.target).shield != 0,
            T_AMBUSH => {
                self.in_arena(s, s.source) && self.has_attack_target(s, s.source, true)
            }
            T_SHIELD => self.in_arena(s, s.source),
            T_SUPPORT => !self.support_targets(s).is_empty(),
            _ => true,
        }
    }
    pub fn set_actor(&self, s: &mut State, actor: Seat) -> Result<(), String> {
        // A latched winner hides the exported turn actor, but an already
        // captured resolver still has its chooser in active (which can differ
        // from the action owner). Validate that chooser without reopening turns
        // or changing the scoring/export latch. A captured regroup resource prompt
        // has its separate simultaneous-seat protocol below.
        if s.is_terminal() {
            // The queued regroup ResourcePrompt is simultaneous: either unfinished
            // seat may act. This changes only its chooser, never the action owner.
            if s.regroup && s.decision == Decision::Resource && !s.resource_done[actor.index()] {
                s.active = actor;
                return Ok(());
            }
            return if s.has_pending_resolution()
                && self.is_ability_decision(s.decision)
                && actor == s.active
            {
                Ok(())
            } else {
                Err(format!("actor mismatch at terminal {:?}: trace {actor:?}", s.decision))
            };
        }
        if s.decision == Decision::Resource && !s.resource_done[actor.index()] {
            s.active = actor;
            Ok(())
        } else if s.actor() == Some(actor) {
            Ok(())
        } else {
            Err(format!(
                "actor mismatch: Rust {:?}, trace {actor:?}",
                s.actor()
            ))
        }
    }
    pub fn order_regroup(&self, s: &mut State, prover: Seat) {
        if s.decision == Decision::Resource
            && s.resource_done == [false; 2]
            && s.resource_selected == [0; 2]
        {
            s.active = prover;
        }
    }
    pub fn apply(&self, s: &State, a: &Action) -> Result<State, String> {
        self.check_named_play(s)?;
        if (!s.phase_history.exact||!s.phase_history.defeats_exact)&&(self.needs_defeat_history()||self.state_needs_defeat_history(s)) {return Err("selected or imported pool requires exact defeat phase history version 2".into());}
        if !s.phase_history.exact && self.needs_phase_history() {return Err("selected pool requires exact phase history".into());}
        self.check_stat_capacity(s)?;
        if !self.legal_actions(s).contains(a) {
            return Err(format!("illegal action {} at {:?}", a.key(), s.decision));
        }
        if self.is_ability_decision(s.decision) {
            return self.apply_ability_action(s, a);
        }
        let mut t = *s;
        if t.decision==Decision::Window {t.action_player=Some(t.active);}
        let seat = t.active;
        let id = a.id().unwrap_or(0);
        let arg = a.arg.as_ref().and_then(Value::as_str).unwrap_or("");
        match t.decision {
            Decision::Window => {
                if arg == "pass" || arg == "claimInitiative" {
                    if arg == "claimInitiative" {
                        t.initiative = seat;
                        t.initiative_claimed = true;
                        t.player_mut(seat).out = true;
                    }
                    if t.prev_passed {
                        t.players[0].out = true;
                        t.players[1].out = true;
                    } else if t.player(seat.other()).out {
                        t.player_mut(seat).out = true;
                    } else {
                        t.prev_passed = true;
                    }
                    self.advance(&mut t)?;
                } else if id == t.player(seat).leader && !t.card(id).deployed {
                    t.source = id;
                    let (deploy, ability) = self.leader_available(&t);
                    if deploy || self.available_programs(&t, id, "action").len() > 1 {
                        t.decision = Decision::LeaderMenu;
                    } else if ability {
                        self.use_leader(&mut t)?;
                    }
                } else if id == t.player(seat).base {
                    if t.card(id).epic_used {
                        return Err("base epic action already spent".into());
                    }
                    let programs = self.available_programs(&t, id, "epicAction");
                    let program = *programs
                        .first()
                        .ok_or("base has no executable epic action")?;
                    t.source = id;
                    t.trigger_context = 0;
                    self.begin_program(&mut t, program, ProgramContinuation::Finish)?;
                } else if t.player(seat).hand.contains(id) {
                    if self.def(&t,id).info.is("upgrade") {t.source=id;t.decision=Decision::UpgradeTarget;return Ok(t);}
                    if self.catalogue.is_some() && self.def(&t, id).info.is("event") {
                        t.source = id;
                        t.set_trigger_context(TriggerContext::Action);
                        let programs = self.available_programs(&t, id, "event");
                        let program = *programs.first().ok_or("event has no executable ability")?;
                        self.begin_program(&mut t, program, ProgramContinuation::Finish)?;
                        return Ok(t);
                    }
                    self.play_unit_with_payment(&mut t,id,false)?;
                } else if t.player(seat).resources.contains(id) {
                    self.begin_smuggle_play(&mut t, id)?;
                } else {
                    t.source = id;
                    t.ambush = false;
                    t.set_trigger_context(TriggerContext::Action);
                    let programs = self.available_programs(&t, id, "action");
                    if t.card(id).exhausted && programs.len() == 1 {
                        self.begin_program(&mut t, programs[0], ProgramContinuation::Finish)?;
                    } else {
                        t.decision = if programs.is_empty() {
                            Decision::AttackTarget
                        } else {
                            Decision::AbilityMenu
                        };
                    }
                }
            }
            Decision::UpgradeTarget => {
                if arg=="cancel" {self.cancel(&mut t);}else{
                    let upgrade=t.source;
                    self.play_upgrade_with_payment(&mut t,upgrade,id)?;
                }
            }
            Decision::LeaderMenu => {
                if a.label == "Cancel" {
                    self.cancel(&mut t);
                } else if a.label.starts_with("Deploy ") {
                    let leader = t.player(seat).leader;
                    {
                        let c = t.card_mut(leader);
                        c.deployed = true;
                        c.epic_used = true;
                        c.exhausted = false;
                    }
                    t.card_mut(leader).set_hidden_qualified(true);
                    self.add_arena(&mut t, leader)?;
                    t.phase_history.entered(seat,leader);
                    t.source = leader;
                    self.enter_triggers(&mut t)?;
                } else {
                    if self.catalogue.is_some() {
                        let id = t.player(seat).leader;
                        let (deploy, _) = self.leader_available(&t);
                        let index = a
                            .arg
                            .as_ref()
                            .and_then(Value::as_u64)
                            .ok_or("ability index missing")?
                            as usize
                            - usize::from(deploy);
                        let program = self.available_programs(&t, id, "action")[index];
                        self.begin_program(&mut t, program, ProgramContinuation::Finish)?;
                    } else {
                        self.use_leader(&mut t)?;
                    }
                }
            }
            Decision::AttackTarget => {
                if t.support.pending && (arg == "passAbility" || t.support.native_attack && arg == "cancel") {
                    self.resume_support(&mut t)?;
                } else if arg == "cancel" {
                    self.cancel(&mut t);
                } else {
                    t.target = id;
                    let source = t.source;
                    t.card_mut(source).exhausted = true;
                    // The attack cost precedes late effect application; late
                    // effects precede AttackFlow and its declaration windows.
                    self.install_conditional_attack(&mut t)?;
                    t.trigger_context = 2;
                    t.attack_source = source;t.attack_defender=id;t.attack_end_captured=false;t.attack_attacker_defeated=false;t.attack_attacker_departed=false;t.attack_defender_defeated=false;t.attack_defender_departed=false;
                    // The declaration registers before queued framework deaths
                    // physically remove combatants. Copy only for defender rules.
                    let declaration_context=if self.has_defense_registration(&t,id)||self.has_gained_attack_registration(&t) {Some(t)}else{None};
                    self.sweep_stat_defeats(&mut t)?;
                    t.pending_triggers = 0;
                    let d = self.def(&t, source);
                    t.attack_restore = self.keyword_amount(&t, source, "restore");
                    if self.has_keyword(&t, source, "restore") {
                        t.pending_triggers |= T_RESTORE;
                    }
                    if self.keyword_amount(&t, source, "saboteur") > 0 && self.in_arena(&t, id) {
                        t.pending_triggers |= T_SABOTEUR;
                    }
                    if self.catalogue.is_none() && (d.code == "SOR_014" || d.code == "IBH_053") {
                        t.pending_triggers |= T_LEADER;
                    }
                    self.queue_programs(&mut t, &["onAttack"]);
                    if let Some(context)=declaration_context.as_ref() {
                        if !self.capture_defense_window(&mut t,context)? {self.process_triggers(&mut t)?;}
                    } else {self.process_triggers(&mut t)?;}
                }
            }
            Decision::VaderBase => {
                if arg == "cancel" {
                    self.cancel(&mut t);
                } else {
                    if t.trigger_context_is(TriggerContext::Action) {
                        self.spend(&mut t, seat, 1);
                        let leader = t.player(seat).leader;
                        t.card_mut(leader).exhausted = true;
                    }
                    let amount = if t.trigger_context_is(TriggerContext::Action) { 1 } else { 2 };
                    t.card_mut(id).damage += amount;
                    self.check_win(&mut t);
                    if t.trigger_context_is(TriggerContext::Action) {
                        self.finish(&mut t)?;
                    } else {
                        self.process_triggers(&mut t)?;
                    }
                }
            }
            Decision::NoEffect => {
                if a.label == "Cancel" {
                    self.cancel(&mut t);
                } else {
                    let leader = t.player(seat).leader;
                    t.card_mut(leader).exhausted = true;
                    self.finish(&mut t)?;
                }
            }
            Decision::SatineHeal => {
                let leader = t.player(seat).leader;
                t.card_mut(leader).exhausted = true;
                if let Some(pair) = a
                    .arg
                    .as_ref()
                    .and_then(Value::as_array)
                    .and_then(|v| v.first())
                {
                    let target = cid(pair[0].as_str().ok_or("missing healing card")?)?;
                    let heal = (pair[1].as_u64().ok_or("missing healing amount")? as u16)
                        .min(t.card(target).damage);
                    let heal = self.heal_card(&mut t, target, heal);
                    let base = t.player(seat).base;
                    t.card_mut(base).damage += heal;
                }
                self.check_win(&mut t);
                self.finish(&mut t)?;
            }
            Decision::Resource => {
                if id != 0 {
                    if t.resource_selected[seat.index()] == 0 {
                        t.resource_selected[seat.index()] = id;
                    }
                } else {
                    t.resource_done[seat.index()] = true;
                    if t.resource_done == [true; 2] {
                        self.end_regroup(&mut t)?;
                    } else {
                        t.active = seat.other();
                    }
                }
            }
            Decision::Unique => {
                self.settle_enter_triggers(&mut t, Some(id))?;
            }
            Decision::Support => {
                if arg=="passAbility" {self.process_triggers(&mut t)?;}else{self.begin_support_attack(&mut t,id)?;}
            }
            Decision::Ambush => {
                if arg == "trigger" && self.has_attack_target(&t, t.source, true) {
                    t.after_attack_programs = t.pending_programs;
                    t.pending_programs = 0;
                    t.after_attack_triggers = t.pending_triggers;
                    t.pending_triggers = 0;
                    t.ambush = true;
                    t.decision = Decision::AttackTarget;
                } else {
                    self.process_triggers(&mut t)?;
                }
            }
            Decision::TriggerOrder => {
                let index = arg.parse::<usize>().map_err(|e| e.to_string())?;
                self.resolve_ordered_trigger(&mut t, index)?;
            }
            Decision::ShieldOrder => {
                t.attack_shield_opponent_first=a.arg.as_ref().and_then(Value::as_u64)==Some(1);
                t.set_trigger_context(TriggerContext::ShieldOrder);
                self.combat(&mut t)?;
            }
            Decision::CreditMenu | Decision::CreditAmount | Decision::DeathPlayer | Decision::DeathOrder | Decision::DeathBatch | Decision::AbilityMenu
            | Decision::AbilityTarget
            | Decision::AbilityOptional
            | Decision::AbilityNoEffect
            | Decision::AbilityDistribute
            | Decision::AbilityGroupShield | Decision::AbilityDiscard | Decision::AbilityDisplay
            | Decision::DiscloseSelect | Decision::DisclosePause | Decision::DiscloseReveal => unreachable!(),
            Decision::Terminal => return Err("game is over".into()),
        }
        self.sweep_stat_defeats(&mut t)?;
        Ok(t)
    }
    fn duplicates(&self, s: &State, id: CardId) -> usize {
        (1..s.next_id).filter(|c|s.card(*c).owner==s.card(id).owner && s.card(*c).def==s.card(id).def && (self.in_arena(s,*c)||self.attached_in_play(s,*c))).count()
    }
    /// Exhausts amount ready resources from the payer's resource zone in zone
    /// order. The payer is explicit because card-effect payments charge the
    /// ability source's controller, which can differ from the active player
    /// while a trigger resolves.
    fn spend(&self, s: &mut State, seat: Seat, mut amount: u16) {
        let ids = s.player(seat).resources;
        for &id in ids.as_slice() {
            if amount > 0 && !s.card(id).exhausted {
                s.card_mut(id).exhausted = true;
                amount -= 1;
            }
        }
        debug_assert_eq!(amount, 0);
    }
    /// Pay a play-from-resources cost. The leaving resource is exhausted first when
    /// ready (pinned Smuggle/Plot ResourceCost priority), then other ready resources
    /// in zone order. Deterministic; does not shuffle the resource row.
    fn spend_resource_play(&self, s: &mut State, mut amount: u16, played: CardId) {
        if amount > 0 && s.player(s.active).resources.contains(played) && !s.card(played).exhausted {
            s.card_mut(played).exhausted = true;
            amount -= 1;
        }
        let ids = s.player(s.active).resources;
        for &id in ids.as_slice() {
            if amount == 0 {
                break;
            }
            if id != played && !s.card(id).exhausted {
                s.card_mut(id).exhausted = true;
                amount -= 1;
            }
        }
        debug_assert_eq!(amount, 0);
    }
    fn resource_deck_top_after_play(&self, s: &mut State, seat: Seat) -> Result<(), String> {
        let Some(&top) = s.player(seat).deck.as_slice().first() else {
            return Ok(());
        };
        let info = &self.def(s, top).info;
        if info.is("leader") || info.is("token") {
            return Ok(());
        }
        if s.player(seat).resources.len as usize >= crate::model::MAX_ZONE {
            return Err("resource zone capacity exceeded".into());
        }
        s.player_mut(seat).deck.remove(top);
        s.player_mut(seat).resources.push(top)?;
        s.card_mut(top).exhausted = true;
        Ok(())
    }
    fn begin_smuggle_play(&self, s: &mut State, id: CardId) -> Result<(), String> {
        if !self.can_smuggle(s, id) {
            return Err("illegal smuggle".into());
        }
        if self.def(s, id).info.is("upgrade") {
            s.source = id;
            s.decision = Decision::UpgradeTarget;
            return Ok(());
        }
        if self.catalogue.is_some() && self.def(s, id).info.is("event") {
            s.source = id;
            s.trigger_context = 0;
            let programs = self.available_programs(s, id, "event");
            let program = *programs.first().ok_or("event has no executable ability")?;
            return self.begin_program(s, program, ProgramContinuation::Finish);
        }
        self.play_unit_with_payment(s,id,true)?;
        Ok(())
    }
    fn add_arena(&self, s: &mut State, id: CardId) -> Result<(), String> {
        let space = self.def(s, id).info.arena.as_deref() == Some("space");
        let p = s.player_mut(s.card(id).owner);
        if space {p.space.push(id)?;}else{p.ground.push(id)?;}
        s.arena_unit_order[usize::from(space)].push(id)?;
        s.registration_order.push(id)
    }
    fn use_leader(&self, s: &mut State) -> Result<(), String> {
        if self.catalogue.is_some() {
            let id = s.player(s.active).leader;
            s.source = id;
            s.set_trigger_context(TriggerContext::Action);
            let program = *self
                .available_programs(s, id, "action")
                .first()
                .ok_or("leader has no executable action")?;
            return self.begin_program(s, program, ProgramContinuation::Finish);
        }
        self.use_legacy_leader(s)
    }
    fn clear_action_context(&self, s: &mut State) {
        s.resource_payment=crate::resource_payment::ResourcePayment::default();s.resource_cost_index=0;s.resource_cost_locked=false;s.payment_group=EventGroupState::default();
        s.disclose=DiscloseState::default();
        s.attack_source=0;s.attack_defender=0;s.attack_end_captured=false;s.attack_attacker_defeated=false;s.attack_attacker_departed=false;s.attack_defender_defeated=false;s.attack_defender_departed=false;
        s.source = 0;
        s.target = 0;
        s.set_trigger_context(TriggerContext::Action);
        s.pending_triggers = 0;
        s.ambush = false;
        s.ability_program = 0;
        s.ability_pc = 0;
        s.ability_target = 0;
        s.maximum_capture=MaximumCapture::default();
        s.attack_shield_opponent_first=false;
        s.ability_selection = Selection::default();
        s.event_group = EventGroupState::default();
        s.event_result=StepEventResult::default();s.last_known_source=0;s.last_known_power=0;
        s.ability_result = 0;
        s.ability_succeeded = false;
        s.ability_return = ProgramContinuation::Finish.raw();
        s.ability_committed = false;
        s.pending_programs = 0;
        s.after_attack_programs = 0;
    }
    fn cancel(&self, s: &mut State) {
        self.clear_action_context(s);
        s.decision = Decision::Window;
    }
    fn enter_triggers(&self, s: &mut State) -> Result<(), String> {
        self.settle_enter_triggers(s, None)
    }
    fn settle_enter_triggers(
        &self,
        s: &mut State,
        unique_defeat: Option<CardId>,
    ) -> Result<(), String> {
        // Registration sees pending-defeat cards in the arena, with the chosen
        // unique copy's ongoing effects disabled. Resolution sees settled deaths.
        s.decision = Decision::Window;
        s.disabled_ongoing_source = unique_defeat.unwrap_or(0);
        self.capture_enter_triggers(s)?;
        s.disabled_ongoing_source = 0;
        if let Some(id) = unique_defeat {
            self.defeat(s, id)?;
        }
        self.sweep_stat_defeats(s)?;
        self.process_triggers(s)
    }
    fn capture_enter_triggers(&self, s: &mut State) -> Result<(), String> {
        let flags = self.effective_flags(s, s.source);
        s.pending_triggers = if flags & SHIELDED != 0 { T_SHIELD } else { 0 }
            | if flags & AMBUSH != 0 { T_AMBUSH } else { 0 }
            | if flags & SUPPORT != 0 {T_SUPPORT}else{0};
        s.trigger_context = 1;
        let smuggle = s.enter_smuggle;
        s.enter_smuggle = false;
        // Standard WhenPlayed abilities watch OnCardPlayed, while keyword
        // Ambush/Shielded also watch OnLeaderDeployed. Smuggle-only aliases
        // watch OnCardPlayed with PlayType.Smuggle; leader fronts also watch
        // Smuggle playType via capture_card_played_watchers.
        if !self.def(s, s.source).info.is("leader") {
            if smuggle {
                self.queue_programs(s, &["whenPlayed", "whenPlayedUsingSmuggle"]);
            } else {
                self.queue_programs(s, &["whenPlayed"]);
            }
            self.capture_card_played_watchers(s, smuggle)?;
        }
        Ok(())
    }
    fn process_triggers(&self, s: &mut State) -> Result<(), String> {
        if s.trigger_context_is(TriggerContext::DeathKeyword) {return self.process_deaths(s);}
        self.merge_enter_deaths(s)?;
        if self.open_deaths(s,DeathContinuation::Triggers)? {return Ok(());}
        if self.catalogue.is_some() {
            return self.process_all_triggers(s);
        }
        if s.is_terminal() && !s.trigger_context_is(TriggerContext::Attack) {
            return Ok(());
        }
        if s.pending_triggers.count_ones() > 1
            && ![T_RESTORE, T_SABOTEUR, T_LEADER, T_AMBUSH, T_SHIELD, T_SUPPORT]
                .iter()
                .any(|&bit| s.pending_triggers & bit != 0 && self.trigger_effect(s, bit))
        {
            s.pending_triggers = 0;
        }
        if s.pending_triggers.count_ones() > 1 {
            s.decision = Decision::TriggerOrder;
            return Ok(());
        }
        if s.pending_triggers != 0 {
            let bit = s.pending_triggers;
            return self.resolve_trigger(s, bit);
        }
        if s.trigger_context_is(TriggerContext::Attack) {
            self.combat(s)
        } else {
            self.finish(s)
        }
    }
    fn resolve_trigger(&self, s: &mut State, bit: u8) -> Result<(), String> {
        s.pending_triggers &= !bit;
        match bit {
            T_RESTORE => {
                let base = s.player(s.active).base;
                self.heal_card(s, base, s.attack_restore);
            }
            T_SABOTEUR => {let mut token=s.card(s.target).shield;while token!=0 {let next=s.card(token).next_token;self.detach(s,token);token=next;}s.card_mut(s.target).shield=0;}
            T_LEADER => {
                if self.def(s, s.source).code == "IBH_053" {
                    s.decision = Decision::VaderBase;
                    return Ok(());
                }
                let base = s.player(s.active.other()).base;
                s.card_mut(base).damage += 1;
                self.check_win(s);
            }
            T_SHIELD => {
                if !self.in_arena(s, s.source) {
                    return self.process_triggers(s);
                }
                if s.next_id as usize >= MAX_CARDS {
                    return Err("card capacity exceeded while creating Shield".into());
                }
                let id = s.next_id;
                s.next_id += 1;
                *s.card_mut(id) = Card {
                    present: true,
                    def: self.shield_def,
                    owner: s.active,
                    next_token: s.card(s.source).shield,
                    ..Card::default()
                };
                s.card_mut(s.source).shield = id;
                self.attach(s,id,s.source);
            }
            T_SUPPORT => {if self.trigger_effect(s,T_SUPPORT) {s.decision=Decision::Support;return Ok(());}}
            T_AMBUSH => {
                if self.trigger_effect(s, T_AMBUSH) {
                    s.decision = Decision::Ambush;
                    return Ok(());
                }
            }
            _ => return Err("unknown trigger".into()),
        }
        self.process_triggers(s)
    }
    fn damage(&self, s: &mut State, id: CardId, amount: u16) {
        if amount == 0 {
            return;
        }
        if s.card(id).shield != 0 {
            let mut token = s.card(id).shield;
            let mut previous = 0;
            while s.card(token).next_token != 0 {
                previous = token;
                token = s.card(token).next_token;
            }
            if previous == 0 {
                s.card_mut(id).shield = 0;
            } else {
                s.card_mut(previous).next_token = 0;
            }
            self.detach(s,token);s.card_mut(token).next_token=0;
        } else {
            s.card_mut(id).damage = s.card(id).damage.saturating_add(amount);
        }
    }
    fn combat_damage(&self,s:&State,id:CardId,attacking:bool)->u16 {
        if self.card_restricted(s,id,crate::ability_ir::RestrictionVerb::DealCombatDamage) {0} else {self.power(s,id,attacking)}
    }
    fn combat(&self, s: &mut State) -> Result<(), String> {
        let source = s.source;
        let target = s.target;
        if self.in_arena(s,target) && self.deals_combat_damage_first(s,source) {
            return self.combat_first_window(s);
        }
        let attack=self.combat_damage(s,source,true);
        let target_unit=self.in_arena(s,target);
        let defend=if target_unit {self.combat_damage(s,target,false)}else{0};
        if s.trigger_context != 3
            && s.card(source).shield != 0
            && s.card(target).shield != 0
            && attack > 0
            && defend > 0
        {
            s.decision = Decision::ShieldOrder;
            return Ok(());
        }
        self.capture_attack_end(s)?;
        if target_unit {
            let source_damage_before=s.card(source).damage;
            let target_damage_before=s.card(target).damage;
            let remaining = self.hp(s, target).saturating_sub(s.card(target).damage);
            let excess = if self.keyword_amount(s, source, "overwhelm") > 0 && s.card(target).shield == 0 {
                attack.saturating_sub(remaining)
            } else {
                0
            };
            if s.trigger_context==3 && s.card(source).shield!=0 && s.card(target).shield!=0 && attack>0 && defend>0 {
                let (first,second)=if s.attack_shield_opponent_first { (target,source) } else { (source,target) };
                let (first_token,second_token)=self.combat_shield_contexts(s,first,second);
                self.remove_group_shield(s,first,first_token); self.remove_group_shield(s,second,second_token);
            } else {
                self.damage(s, target, attack);
                self.damage(s, source, defend);
            }
            if excess > 0 {
                let base = s.player(s.active.other()).base;
                s.card_mut(base).damage += excess;
            }
            // Only actual damage invokes deferred damage-defeat callbacks.
            // Shield replacement can instead remove an HP grant: those deaths
            // belong to the later framework sweep in global arena order.
            let mut dead=vec![];
            for (id,before) in [(target,target_damage_before),(source,source_damage_before)] {
                if s.card(id).damage>before && s.card(id).damage>=self.hp(s,id) {dead.push(id);}
            }
            for id in self.arena_units(s) {if s.card(id).damage>=self.hp(s,id) && !dead.contains(&id) {dead.push(id);}}
            self.capture_advantage_end(s)?;
            self.defeat_many(s,&dead)?;
        } else {
            s.card_mut(target).damage += attack;
            self.capture_advantage_end(s)?;
        }
        self.sweep_stat_defeats(s)?;
        self.check_win(s);
        if self.open_deaths(s, DeathContinuation::CombatAfterDamage)? {return Ok(());}
        self.combat_after_damage(s)
    }
    fn combat_after_damage(&self,s:&mut State)->Result<(),String> {
        let source=s.source;
        self.capture_attack_end(s)?;
        if self.open_deaths(s,DeathContinuation::CombatAfterDamage)? {return Ok(());}
        self.clear_support_grants(s);
        s.attack_source=0;s.attack_defender=0;s.attack_end_captured=false;s.attack_attacker_defeated=false;s.attack_attacker_departed=false;s.attack_defender_defeated=false;s.attack_defender_departed=false;
        self.sweep_stat_defeats(s)?;
        if self.open_deaths(s,DeathContinuation::CombatAfterDamage)? {return Ok(());}
        if s.support.pending {return self.resume_support(s);}
        self.check_win(s);
        if s.is_terminal() {
            self.finish(s)?;
        } else {
            if (s.after_attack_triggers != 0 || s.after_attack_programs != 0)
                && (self.in_arena(s, source)
                    // Ordinary unit WhenPlayed registrations were captured before
                    // Ambush; source departure cannot erase the remaining window.
                    // Leader program indices depend on deployed side, so retain
                    // their existing live-source policy (leaders do not queue WP).
                    || (s.after_attack_programs != 0 && !self.def(s, source).info.is("leader")))
            {
                s.pending_programs = s.after_attack_programs;
                s.after_attack_programs = 0;
                s.pending_triggers = s.after_attack_triggers;
                s.after_attack_triggers = 0;
                s.set_trigger_context(TriggerContext::Enter);
                self.process_triggers(s)?;
            } else {
                self.finish(s)?;
            }
        }
        Ok(())
    }
    fn prepare_unit_defeat(&self,s:&State,id:CardId)->Result<Option<(Seat,bool)>,String> {
        let fact=if self.in_arena(s,id) {Some((s.card(id).owner,self.def(s,id).info.is("leader")))}else{None};
        if let Some((owner,leader))=fact {let mut counts=[0;2];counts[owner.index()]=1;s.phase_history.preflight_defeats(counts,u16::from(leader))?;}
        Ok(fact)
    }
    fn finish_physical_defeat(&self,s:&mut State,id:CardId,fact:Option<(Seat,bool)>)->Result<(),String> {
        if let Some((owner,leader))=fact {s.phase_history.record_defeat(owner,leader);}
        self.leave_play(s,id)
    }
    fn defeat(&self, s: &mut State, id: CardId) -> Result<(), String> {
        if !self.in_arena(s,id) && !self.attached_in_play(s,id) {return Ok(());}
        let fact=self.prepare_unit_defeat(s,id)?;
        self.capture_death(s,id)?;
        self.finish_physical_defeat(s,id,fact)
    }
    fn defeat_many(&self,s:&mut State,ids:&[CardId])->Result<(),String> {
        let mut seen=std::collections::BTreeSet::new();
        let ids=ids.iter().copied().filter(|id|seen.insert(*id)&&(self.in_arena(s,*id)||self.attached_in_play(s,*id))).collect::<Vec<_>>();
        let facts=ids.iter().filter(|id|self.in_arena(s,**id)).map(|id|(*id,s.card(*id).owner,self.def(s,*id).info.is("leader"))).collect::<Vec<_>>();
        let mut counts=[0u16;2];let mut leaders=0u16;
        for (_,owner,leader) in &facts {counts[owner.index()]+=1;leaders+=u16::from(*leader);}
        s.phase_history.preflight_defeats(counts,leaders)?;
        // Capture all sibling LKI/registrations before history changes can
        // alter a live ongoing effect on another simultaneous combatant.
        for &id in &ids {self.capture_death(s,id)?;}
        let attachments=ids.iter().flat_map(|id|self.upgrades(s,*id)).collect::<Vec<_>>();
        // The pinned engine gives cleanup events order -1, then uses a unary
        // sort comparator (event => event.order). Its observed order is reverse
        // cleanup insertion, before the primary parent handlers.
        for upgrade in attachments.into_iter().rev() {self.leave_play_primary(s,upgrade)?;}
        for &id in &ids {if self.in_arena(s,id)||self.attached_in_play(s,id) {
            let fact=facts.iter().find(|(card,_,_)|*card==id).map(|(_,owner,leader)|(*owner,*leader));
            self.finish_physical_defeat(s,id,fact)?;
        }}
        Ok(())
    }
    // The menu captures one physical arena recipient; its owner chooses only
    // the destination. This is departure, so reuse contingent attachment cleanup
    // and primary reset without capturing a defeat or incrementing history.
    fn owner_deck_target_eligible(&self,s:&State,id:CardId)->bool {
        id!=0 && self.in_arena(s,id) && self.def(s,id).info.is("unit")
            && !self.def(s,id).info.is("leader")
    }
    fn move_arena_unit_to_owner_deck(&self,s:&mut State,id:CardId,position:crate::ability_ir::DeckPosition)->Result<u16,String> {
        if !self.owner_deck_target_eligible(s,id) {return Ok(0);}
        let owner=s.card(id).owner;
        let token=self.def(s,id).info.is("token");
        // No physical movement or attachment cleanup precedes capacity checks.
        if !token && s.player(owner).deck.len as usize==MAX_ZONE {return Err("owner deck capacity exceeded".into());}
        let attachments=self.upgrades(s,id);
        let mut discard_counts=[0usize;2];
        for &upgrade in &attachments {if !self.def(s,upgrade).info.is("token") {discard_counts[s.card(upgrade).owner.index()]+=1;}}
        for seat in [Seat::P1,Seat::P2] {if s.player(seat).discard.len as usize+discard_counts[seat.index()]>MAX_ZONE {return Err("attachment discard capacity exceeded".into());}}
        for upgrade in attachments.into_iter().rev() {self.leave_play_primary(s,upgrade)?;}
        self.leave_play_primary_to_discard(s,id,false)?;
        if !token {
            let deck=&mut s.player_mut(owner).deck;
            deck.push(id)?;
            if position==crate::ability_ir::DeckPosition::Top {
                let last=deck.len as usize-1;
                deck.ids[..=last].rotate_right(1);
            }
        }
        Ok(1)
    }
    fn leave_play(&self,s:&mut State,id:CardId)->Result<(),String> {
        let attachments=self.upgrades(s,id);
        for upgrade in attachments.into_iter().rev() {self.leave_play_primary(s,upgrade)?;}
        self.leave_play_primary(s,id)?;
        Ok(())
    }
    fn leave_play_primary(&self, s: &mut State, id: CardId) -> Result<(), String> {
        self.leave_play_primary_to_discard(s,id,true)
    }
    // A returned upgrade moves directly to hand, never through discard. Keep
    // the common detach/state-reset path without consuming a discard slot.
    fn leave_play_primary_to_discard(&self, s: &mut State, id: CardId, discard: bool) -> Result<(), String> {
        if self.in_arena(s,id) {self.mark_attack_lifecycle(s,id,false);}
        s.named_play.remove_source(id);
        self.remove_support_target(s,id);
        self.detach(s,id);
        let owner = s.card(id).owner;
        let leader = id == s.player(owner).leader;
        let token = self.def(s, id).info.is("token");
        s.arena_unit_order[0].remove(id);s.arena_unit_order[1].remove(id);s.registration_order.remove(id);
        let p = s.player_mut(owner);
        p.ground.remove(id);
        p.space.remove(id);
        if discard && !leader && !token {
            p.discard.push(id)?;
        }
        let c = s.card_mut(id);
        if !leader && !token {c.set_hidden_qualified(false);}
        c.damage = 0;
        c.phase_power = 0;
        c.phase_hp = 0;
        c.phase_keyword_values = [0; 2];
        c.phase_keyword_presence = 0;
        c.phase_lost_keywords = 0;
        c.phase_restrictions = 0;
        c.clear_hidden_phase_effects();
        c.shield = 0;
        c.experience = 0;
        c.upgrades = 0;
        c.exhausted = leader;
        c.deployed = false;
        Ok(())
    }
    fn check_win(&self, s: &mut State) {
        for seat in [Seat::P1, Seat::P2] {
            let id = s.player(seat).base;
            let hp = self.def(s, id).info.hp.unwrap_or(30) as u16;
            s.card_mut(id).damage = s.card(id).damage.min(hp);
        }
        let dead = [
            s.card(s.players[0].base).damage
                >= self.def(s, s.players[0].base).info.hp.unwrap_or(30) as u16,
            s.card(s.players[1].base).damage
                >= self.def(s, s.players[1].base).info.hp.unwrap_or(30) as u16,
        ];
        if s.outcome == 0 {
            s.outcome = match dead {
                [true, true] => 3,
                [true, false] => 2,
                [false, true] => 1,
                _ => 0,
            };
        }
        if s.outcome != 0 {
            s.decision = Decision::Terminal;
        }
    }
    fn finish(&self, s: &mut State) -> Result<(), String> {
        self.sweep_stat_defeats(s)?;
        if self.open_deaths(s,DeathContinuation::Finish)? {return Ok(());}
        if s.death_depth>0 {return self.process_deaths(s);}
        if s.is_terminal() {
            s.prev_passed = false;
            if !s.regroup && !s.player(s.active.other()).out {
                s.active = s.active.other();
            }
            if !s.regroup {s.action_player=Some(s.active);}
            s.decision = Decision::Terminal;
            return Ok(());
        }
        s.prev_passed = false;
        s.after_attack_triggers = 0;
        self.clear_action_context(s);
        self.advance(s)
    }
    fn advance(&self, s: &mut State) -> Result<(), String> {
        if s.players.iter().all(|p| p.out) {
            s.action_player=None;
            s.phase_history.reset();
            for id in 1..s.next_id {
                s.card_mut(id).set_hidden_qualified(false);
                s.card_mut(id).phase_power = 0;
                s.card_mut(id).phase_hp = 0;
                s.card_mut(id).phase_keyword_values = [0; 2];
                s.card_mut(id).phase_keyword_presence = 0;
                s.card_mut(id).phase_lost_keywords = 0;
                s.card_mut(id).phase_restrictions = 0;
                s.card_mut(id).clear_hidden_phase_effects();
            }
            self.sweep_stat_defeats(s)?;
            if self.open_deaths(s,DeathContinuation::PhaseExpiry)? {return Ok(());}
            return self.advance_after_expiry(s);
        } else {
            if !s.player(s.active.other()).out {s.active=s.active.other();}
            s.decision=Decision::Window;s.action_player=Some(s.active);
        }
        Ok(())
    }
    // Action-phase expiry and its complete defeat window settle first. Entering
    // regroup captures every phase-start registration before resolving any of
    // them. RegroupDraw resumes below, never through this capture function, so
    // nested defeats cannot trigger the phase twice or draw before it finishes.
    fn advance_after_expiry(&self,s:&mut State)->Result<(),String> {
            if s.is_terminal() && !(s.players[0].out && s.players[1].out) {
                s.decision = Decision::Terminal;
                return Ok(());
            }
            // Already queued phase closure continues after expiry-trigger wins.
            // It cannot be entered by a new action once the outcome is latched.
            s.regroup = true;
            s.players[0].out = false;
            s.players[1].out = false;
            s.initiative_claimed = false;
            s.prev_passed = false;
            self.capture_regroup_start(s)?;
            if self.open_deaths(s,DeathContinuation::RegroupDraw)? {return Ok(());}
            self.draw_for_regroup(s)
    }
    fn draw_for_regroup(&self,s:&mut State)->Result<(),String> {
            if s.is_terminal() && !s.regroup {
                s.decision = Decision::Terminal;
                return Ok(());
            }
            for seat in [Seat::P1, Seat::P2] {
                self.draw_cards(s,seat,2)?;
            }
            self.check_win(s);
            // Source queues resourcing and readying after the whole draw window,
            // even if fatigue (or an earlier captured trigger) latched a winner.
            {
                s.decision = Decision::Resource;
                s.resource_done = [false; 2];
                s.resource_selected = [0; 2];
                s.active = Seat::P1;
            }
        Ok(())
    }
    fn end_regroup(&self, s: &mut State) -> Result<(), String> {
        s.phase_history.reset();
        for seat in [Seat::P1, Seat::P2] {
            let id = s.resource_selected[seat.index()];
            if id != 0 {
                s.player_mut(seat).hand.remove(id);
                s.player_mut(seat).resources.push(id)?;
            }
            let p = *s.player(seat);
            for &id in p
                .resources
                .as_slice()
                .iter()
                .chain(p.ground.as_slice())
                .chain(p.space.as_slice())
                .chain(std::iter::once(&p.leader))
            {
                let arena = p.ground.contains(id) || p.space.contains(id);
                if !(arena && (self.card_restricted(s, id, crate::ability_ir::RestrictionVerb::Ready) || self.card_restricted(s, id, crate::ability_ir::RestrictionVerb::DoesNotReadyDuringRegroup))) {
                    s.card_mut(id).exhausted = false;
                }
            }
            s.player_mut(seat).out = false;
        }
        // Marks applied for "this regroup phase" expire at the next phase
        // boundary; the ready step above is their only consumer.
        for id in 1..s.next_id {
            s.card_mut(id).phase_restrictions = 0;
        }
        s.round = s.round.checked_add(1).ok_or("round capacity exceeded")?;
        s.regroup = false;
        s.active = s.initiative;s.action_player=Some(s.active);
        s.decision = Decision::Window;
        s.resource_selected = [0; 2];
        s.resource_done = [false; 2];
        s.prev_passed = false;
        Ok(())
    }
    fn load_card(&self, s: &mut State, v: &Value, owner: Seat) -> Result<CardId, String> {
        if v["phaseKeywords"].get("support").is_some() {return Err("imported phase Support grants require a reviewed lifecycle".into());}
        let text = v["id"].as_str().ok_or("card missing id")?;
        let id = cid(text)?;
        let code = v["setCode"]
            .as_str()
            .ok_or_else(|| format!("card {text} missing setCode"))?;
        let def = *self.by_code.get(code).ok_or_else(|| {
            format!(
                "unsupported card {} ({code})",
                v["name"].as_str().unwrap_or(text)
            )
        })?;
        if s.card(id).present && s.card(id).def != def {
            return Err(format!("conflicting card identity {text}"));
        }
        let c = s.card_mut(id);
        if let Some(q)=v.get("hiddenQualified") {c.set_hidden_qualified(q.as_bool().ok_or("hiddenQualified must be boolean")?);}
        c.present = true;
        c.def = def;
        c.owner = owner;
        if v.get("damage").is_some() {
            c.damage = n(v, "damage");
        }
        if v.get("exhausted").is_some() {
            c.exhausted = b(v, "exhausted");
        }
        if v.get("deployed").is_some() {
            c.deployed = b(v, "deployed");
        }
        if v.get("epicUsed").is_some() {
            c.epic_used = b(v, "epicUsed");
        }
        if let Some(stats) = v.get("phaseStats") {
            c.phase_power = stats["power"]
                .as_i64()
                .ok_or("phase power must be signed integer")? as i32;
            c.phase_hp = stats["hp"]
                .as_i64()
                .ok_or("phase HP must be signed integer")? as i32;
        }
        if let Some(keywords) = v.get("phaseKeywords") {
            for (keyword, amount) in keywords.as_object().ok_or("phaseKeywords must be object")? {
                let index = crate::ability_ir::CORE_KEYWORDS.iter().position(|k| k == keyword).ok_or("unsupported phase keyword")?;
                let amount = amount.as_u64().and_then(|n| u16::try_from(n).ok()).ok_or("phase keyword amount must be bounded integer")?;
                if index == 7 && amount == 0 { return Err("zero phase Restore requires verified nullable trigger titles".into()); }
                if let Some(slot) = crate::ability_ir::numeric_keyword_slot(index) { c.phase_keyword_values[slot] = amount; } else if amount != 1 { return Err("boolean phase keyword amount must be one".into()); }
                c.add_phase_keyword(index);
            }
        }
        if let Some(keywords) = v.get("phaseLostKeywords") {
            for keyword in keywords.as_array().ok_or("phaseLostKeywords must be array")? {
                let index = crate::ability_ir::CORE_KEYWORDS.iter().position(|k| Some(*k) == keyword.as_str()).ok_or("unsupported lost keyword")?;
                c.add_lost_keyword(index);
            }
        }
        if let Some(verbs) = v.get("phaseRestrictions") {
            let mut mask = 0u8;
            for verb in verbs.as_array().ok_or("phaseRestrictions must be array")? {
                mask |= crate::ability_ir::RestrictionVerb::parse(verb.as_str().ok_or("phase restriction must be a verb")?).ok_or("unsupported phase restriction")?.bit();
            }
            c.phase_restrictions = mask;
        }
        s.next_id = s.next_id.max(id + 1);
        if let Some(upgrades) = v["upgrades"].as_array() {
            for u in upgrades {
                let upgrade_owner=if u["token"]==false {Seat::parse(u["controller"].as_str().ok_or("ordinary upgrade missing controller")?)?}else{owner};
                if u["token"]==false && u["owner"].as_str()!=Some(upgrade_owner.as_str()) {return Err("foreign-controlled upgrade imports are unsupported".into());}
                let uid = self.load_card(s, u, upgrade_owner)?;
                let prior = if s.card(uid).def == self.shield_def {
                    let prior = s.card(id).shield;
                    s.card_mut(id).shield = uid;
                    prior
                } else if Some(s.card(uid).def) == self.experience_def {
                    let prior = s.card(id).experience;
                    s.card_mut(id).experience = uid;
                    prior
                } else {
                    let attachable=self.def(s,uid).info.is("upgrade")
                        || (self.def(s,uid).info.is("unit") && self.def(s,uid).info.has_keyword("piloting")
                            && self.def(s,uid).info.upgrade_power.is_some() && self.def(s,uid).info.upgrade_hp.is_some());
                    if !attachable || (self.def(s,uid).info.is("token") && Some(s.card(uid).def)!=self.weakness_def && Some(s.card(uid).def)!=self.advantage_def) {return Err("unsupported attachment type".into());}
                    0
                };
                s.card_mut(uid).next_token = prior;
                self.attach(s,uid,id);
            }
        }
        Ok(id)
    }
    pub fn load_state(&self, v: &Value) -> Result<State, String> {
        if v.get("regroupStartPending").is_some() {return Err("pending regroup-start triggers cannot be imported as an action window".into());}
        if v.get("namedPlayPending").is_some() {return Err("pending card naming cannot be imported as an action window".into());}
        if v.get("dependentChoicePending").is_some(){return Err("pending dependent choice cannot be imported as an action window".into());}
        if v.get("viewSelectionPending").is_some(){return Err("pending view selection cannot be imported as an action window".into());}
        if v.get("creditPaymentPending").is_some() || v.get("resourcePaymentPending").is_some() {return Err("pending Credit/resource payment cannot be imported as an action window".into());}
        if v.get("disclosePending").is_some(){return Err("pending Disclose cannot be imported as an action window".into());}
        if v.get("supportAttackPending").is_some() {return Err("pending Support attacks cannot be imported as action windows".into());}
        if v["supportGrants"].as_array().is_some_and(|grants|grants.iter().any(|g|g.get("conditionalAttack").is_some())) {
            return Err("pending conditional attack grants cannot be imported as action windows".into());
        }
        if let Some(catalogue) = &self.catalogue {
            crate::validation::validate_state_with_catalogue(v, &catalogue.metadata)?;
        } else {
            crate::validation::validate_state(v)?;
        }
        if v["phase"].as_str() != Some("action") {
            return Err("positions must start at an action window in the action phase".into());
        }
        let mut s = State::default();
        self.load_hidden_evidence(&mut s,v)?;
        s.round = n(v, "round");
        if s.round == 0 {
            return Err("invalid round".into());
        }
        s.initiative = Seat::parse(v["initiative"].as_str().ok_or("missing initiative")?)?;
        s.action_player=v["activePlayer"].as_str().map(Seat::parse).transpose()?;
        s.initiative_claimed = b(v, "initiativeClaimed");
        s.prev_passed = b(v, "prevPassed");
        s.active = Seat::parse(
            v["actor"]
                .as_str()
                .or(v["activePlayer"].as_str())
                .ok_or("missing actor")?,
        )?;
        for seat in [Seat::P1, Seat::P2] {
            let p = &v["players"][seat.as_str()];
            let base = self.load_card(&mut s, &p["base"], seat)?;
            let leader = self.load_card(&mut s, &p["leader"], seat)?;
            s.player_mut(seat).base = base;
            s.player_mut(seat).leader = leader;
            s.player_mut(seat).out = b(p, "out");
            for name in ["hand", "deck", "resources", "discard", "ground", "space"] {
                let mut zone = Zone::default();
                for c in p[name]
                    .as_array()
                    .ok_or_else(|| format!("missing {seat:?}.{name}"))?
                {
                    let id = self.load_card(&mut s, c, seat)?;
                    if zone.contains(id) {
                        return Err(format!("duplicate {} in {name}", c["id"]));
                    }
                    zone.push(id)?;
                }
                match name {
                    "hand" => s.player_mut(seat).hand = zone,
                    "deck" => s.player_mut(seat).deck = zone,
                    "resources" => s.player_mut(seat).resources = zone,
                    "discard" => s.player_mut(seat).discard = zone,
                    "ground" => s.player_mut(seat).ground = zone,
                    _ => s.player_mut(seat).space = zone,
                };
            }
        }
        self.load_arena_order(&mut s,v)?;
        self.validate_attachment_links(&s)?;
        self.load_registration_order(&mut s,v)?;
        self.load_named_play(&mut s,v)?;
        if let Some(next) = v["nextCardId"].as_u64() {
            if next >= MAX_CARDS as u64 {
                return Err("card capacity exceeded".into());
            }
            s.next_id = s.next_id.max(next as u16 + 1);
        } else {
            return Err(
                "full state missing nextCardId: regenerate the trace with the current exporter"
                    .into(),
            );
        }
        self.load_force_evidence(&mut s,v)?;
        if v.get("creditStateVersion").is_some() {self.load_credit_state(&mut s,v)?;}
        self.load_phase_history_evidence(&mut s,v)?;
        self.load_support_state(&mut s,v)?;
        self.check_stat_capacity(&s)?;
        for seat in [Seat::P1, Seat::P2] {
            let p = &v["players"][seat.as_str()];
            for name in ["ground", "space"] {
                for c in p[name].as_array().unwrap() {
                    let id = cid(c["id"].as_str().unwrap())?;
                    if c["power"].as_u64() != Some(self.power(&s, id, false) as u64)
                        || c["hp"].as_u64() != Some(self.hp(&s, id) as u64)
                    {
                        return Err(format!(
                            "unit stats mismatch on {}: engine {}/{}, Rust {}/{}",
                            c["id"],
                            c["power"],
                            c["hp"],
                            self.power(&s, id, false),
                            self.def(&s, id).info.hp.unwrap_or(0)
                        ));
                    }
                }
            }
        }
        s.outcome = match v["winner"].as_str() {
            Some("p1") => 1,
            Some("p2") => 2,
            Some("draw") => 3,
            _ => 0,
        };
        if s.outcome != 0 {
            s.decision = Decision::Terminal;
        }
        self.check_stat_capacity(&s)?;
        // Preserve existing import diagnostics before checking the additional
        // selected-pool requirement. No imported state escapes either guard.
        if !s.arena_order_exact && self.needs_modal_arena_order() {
            return Err("selected pool requires recorded arenaUnitOrder for modal global damage".into());
        }
        if v.get("creditStateVersion").is_none() {self.load_credit_state(&mut s,v)?;}
        Ok(s)
    }
    fn card_ref(&self, s: &State, id: CardId) -> Value {
        let d = self.def(s, id);
        json!({"id":format!("c{}",id-1),"setCode":d.code,"name":d.info.internal_name})
    }
    pub fn snapshot(&self, s: &State) -> Value {
        let mut players = serde_json::Map::new();
        for seat in [Seat::P1, Seat::P2] {
            let p = s.player(seat);
            let mut base = self.card_ref(s, p.base);
            base["damage"] = json!(s.card(p.base).damage);
            base["hp"] = json!(self.def(s, p.base).info.hp.unwrap_or(30));
            base["epicUsed"] = json!(s.card(p.base).epic_used);
            let base_restrictions = phase_restriction_names(s.card(p.base).phase_restrictions);
            if !base_restrictions.is_empty() { base["phaseRestrictions"] = json!(base_restrictions); }
            let mut leader = self.card_ref(s, p.leader);
            leader["deployed"] = json!(s.card(p.leader).deployed);
            leader["exhausted"] = json!(s.card(p.leader).exhausted);
            leader["epicUsed"] = json!(s.card(p.leader).epic_used);
            let refs = |z: &Zone| -> Value {
                Value::Array(
                    z.as_slice()
                        .iter()
                        .map(|&id| self.card_ref(s, id))
                        .collect(),
                )
            };
            let resources: Vec<_> = p
                .resources
                .as_slice()
                .iter()
                .map(|&id| {
                    let mut v = self.card_ref(s, id);
                    v["exhausted"] = json!(s.card(id).exhausted);
                    v
                })
                .collect();
            let arena = |z: &Zone| -> Value {
                Value::Array(
                    z.as_slice()
                        .iter()
                        .map(|&id| {
                            let mut v = self.card_ref(s, id);
                            let c = s.card(id);
                            v["damage"] = json!(c.damage);
                            if s.hidden_history_exact {v["hiddenQualified"]=json!(c.hidden_qualified());}
                            if c.phase_power != 0 || c.phase_hp != 0 {
                                v["phaseStats"] = json!({"power":c.phase_power,"hp":c.phase_hp});
                            }
                            if c.phase_keyword_mask() != 0 {
                                let mut keywords = serde_json::Map::new();
                                for (i, name) in crate::ability_ir::CORE_KEYWORDS.iter().enumerate() { if c.phase_keyword_mask() & (1 << i) != 0 { keywords.insert((*name).into(), json!(c.phase_keyword_amount(i))); } }
                                v["phaseKeywords"] = Value::Object(keywords);
                            }
                            if c.lost_keyword_mask() != 0 { v["phaseLostKeywords"] = json!(crate::ability_ir::CORE_KEYWORDS.iter().enumerate().filter_map(|(i,k)| (c.lost_keyword_mask() & (1 << i) != 0).then_some(*k)).collect::<Vec<_>>()); }
                            let restrictions = phase_restriction_names(c.phase_restrictions);
                            if !restrictions.is_empty() { v["phaseRestrictions"] = json!(restrictions); }
                            v["exhausted"] = json!(c.exhausted);
                            v["power"] = json!(self.power(s, id, self.is_attacking(s, id)));
                            v["hp"] = json!(self.hp(s, id));
                            let upgrades: Vec<_> = self.upgrades(s,id).into_iter().map(|uid| {
                                let mut u=self.card_ref(s,uid);let token=self.def(s,uid).info.is("token");u["token"]=json!(token);
                                if !token {u["controller"]=json!(s.card(uid).owner.as_str());u["owner"]=json!(s.card(uid).owner.as_str());}u
                            }).collect();
                            v["upgrades"] = json!(upgrades);
                            v
                        })
                        .collect(),
                )
            };
            players.insert(seat.as_str().into(),json!({"base":base,"leader":leader,"hand":refs(&p.hand),"deck":refs(&p.deck),"resources":resources,"discard":refs(&p.discard),"ground":arena(&p.ground),"space":arena(&p.space),"out":p.out}));
        }
        let winner = if s.outcome == 3 {
            Some("draw")
        } else {
            s.winner().map(Seat::as_str)
        };
        let mut value=json!({"round":s.round,"phase":if s.regroup{"regroup"}else{"action"},"initiative":s.initiative.as_str(),"initiativeClaimed":s.initiative_claimed,"prevPassed":s.prev_passed,"activePlayer":s.action_player.map(Seat::as_str),"actor":if s.is_action_window(){Some(s.active.as_str())}else{None},"winner":winner,"nextCardId":s.next_id-1,"players":players});
        if s.death_frames[..s.death_depth as usize].iter().any(|f|f.continuation()==Some(DeathContinuation::RegroupDraw)) {value["regroupStartPending"]=json!(true);}
        if s.support_state_explicit||self.needs_support_state()||self.state_needs_support_state(s)||s.support.len!=0 {value["supportStateVersion"]=json!(1);value["supportGrants"]=self.support_snapshot(s);}
        if matches!(s.decision,Decision::DiscloseSelect|Decision::DisclosePause|Decision::DiscloseReveal){value["disclosePending"]=json!(true);}
        if s.decision==Decision::AbilityDisplay {value["viewSelectionPending"]=json!(true);}
        if s.support.pending {value["supportAttackPending"]=json!(true);}
        if s.ability_program!=0&&self.program(s.ability_program-1).instructions.iter().any(|i|matches!(i,crate::ability_ir::Instruction::ModalChoice(menu) if menu.preserve_target)) {value["dependentChoicePending"]=json!(true);}
        self.write_snapshot_evidence(s,&mut value);
        self.snapshot_named_play(s,&mut value);
        self.snapshot_credit_state(s,&mut value);
        if s.resource_payment.active() || s.payment_group.active || s.death_frames[..s.death_depth as usize].iter().any(|f|f.context.resource_payment.active()) {value["creditPaymentPending"]=json!(true);}
        value
    }
}

#[cfg(test)]
#[path = "simulator_tests.rs"]
mod tests;

pub(crate) fn program_backend(hash:&str,bytes:&[u8])->Result<usize,&'static str> {generic_runtime::generated_backend(hash,bytes)}

pub(crate) fn expression_backend(hash: &str, bytes: &[u8]) -> Option<usize> {
    generic_runtime::generated_expression_backend(hash, bytes)
}

#[cfg(test)]
#[path = "credit_action_tests.rs"]
mod credit_action_tests;

#[cfg(test)]
#[path = "compatibility_tests.rs"]
mod compatibility_tests;
