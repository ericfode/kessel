//! Reviewed base-zone constant sources. Runtime side selection and physical
//! activation use their existing state directly; admission needs only source kind.
use super::{Ability, Effect, Expr, RecipeSelector, Sides, Target, TargetResolver};
use serde_json::{json, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source { Base, Leader }

// Keep this family's original header policy. Program admission owns later
// confirmation/attachment diagnostics, so this check must not absorb those guards.
fn constant<'a>(a: &'a Ability, recipient: &str) -> Option<&'a TargetResolver> {
    if a.trigger != "constant" || a.optional || !a.costs.is_empty() || a.condition.is_some()
        || a.coordinate || a.then.is_some() || a.if_you_do.is_some()
        || !a.context_title.is_empty() || !a.target_resolvers.is_empty() { return None; }
    a.target_resolver.as_ref().filter(|r|RecipeSelector::one("self", recipient, "arena").matches(r))
}
fn reference(e: &Expr, expected: &[&str]) -> bool {
    matches!(e, Expr::Ref { path } if path.iter().map(String::as_str).eq(expected.iter().copied()))
}
fn number(e: &Expr, expected: u64) -> bool { matches!(e, Expr::Const { value } if value.as_u64() == Some(expected)) }
fn cost_three(e: &Expr) -> bool {
    matches!(e, Expr::Binary { op, left, right } if op == "&&"
        && matches!(left.as_ref(), Expr::Query { name, receiver, args } if name == "isUnit" && reference(receiver, &["card"]) && args.is_empty())
        && matches!(right.as_ref(), Expr::Binary { op, left, right } if op == ">=" && reference(left, &["card", "cost"]) && number(right, 3)))
}
fn raid(a: &Ability, exclude_source: bool) -> bool {
    let Some(resolver) = constant(a, "unit") else { return false; };
    matches!(&a.effect, Effect::GainKeyword { target: Target::Name(name), keyword, amount, duration }
        if name == "chosen" && keyword == "raid" && number(amount, 1) && duration == "persistent")
        && resolver.predicate.as_ref().is_some_and(|p| {
            if exclude_source {
                matches!(p, Expr::Binary { op, left, right } if op == "&&" && cost_three(left)
                    && matches!(right.as_ref(), Expr::Binary { op, left, right } if op == "!=="
                        && reference(left, &["card"]) && reference(right, &["source"])))
            } else { cost_three(p) }
        })
}

fn original_base_zone_source(entry: &Value) -> Option<Source> {
    if crate::bounded_reductions::leader_entry(entry) { return Some(Source::Leader); }
    let sides: Sides = serde_json::from_value(entry["abilities"].clone()).ok()?;
    if !entry["card"]["unique"].is_boolean() || !entry["attachCondition"].is_null() || !entry["piloting"].is_null()
        || ["pilotText", "pilotBox", "pilotingBox"].iter().any(|key| !entry["card"][key].is_null()) { return None; }
    if entry["baseClass"] == "BaseCard" && entry["card"]["types"] == json!(["base"])
        && entry["card"]["unique"] == false && entry["card"]["hp"] == 26 && entry["card"]["cost"].is_null()
        && entry["card"]["power"].is_null() && entry["keywordValues"] == json!({}) && entry["card"]["keywords"] == json!([])
        && sides.card.len() == 1 && sides.leader.is_empty() && sides.unit.is_empty() {
        let a = &sides.card[0];
        let hp_bonus = matches!(&a.effect, Effect::ModifyStats { power, hp, .. } if number(power, 0) && number(hp, 1));
        let text = if hp_bonus { "Each leader unit you control gets +0/+1." } else { "Each leader unit you control gets +1/+0." };
        let grant = matches!(&a.effect, Effect::ModifyStats { target: Target::Name(name), power, hp, duration }
            if name == "chosen" && duration == "persistent"
                && ((number(power, 0) && number(hp, 1)) || (number(power, 1) && number(hp, 0))));
        return (entry["card"]["text"] == text && (entry["card"]["epicAction"].is_null() || entry["card"]["epicAction"] == "")
            && ["rules", "deployBox", "upgradePower", "upgradeHp"].iter().all(|key| entry["card"][key].is_null())
            && grant && constant(a, "leaderUnit").is_some_and(|r|r.predicate.is_none())).then_some(Source::Base);
    }
    (entry["card"]["text"] == "Friendly units that cost 3 or more gain Raid 1."
        && entry["card"]["deployBox"] == "Raid 1\nOther friendly units that cost 3 or more gain Raid 1."
        && entry["card"]["epicAction"] == "Epic Action: If you control 6 or more resources, deploy this leader."
        && entry["card"]["rules"] == "Abilities that refer to a card’s cost always refer to its printed cost, regardless of modifiers.\n"
        && entry["card"]["upgradePower"].is_null() && entry["card"]["upgradeHp"].is_null()
        && entry["baseClass"] == "LeaderUnitCard" && entry["card"]["types"] == json!(["leader"])
        && entry["card"]["unique"] == true && entry["card"]["cost"] == 6 && entry["card"]["hp"] == 5
        && entry["card"]["power"] == 5 && entry["card"]["arena"] == "ground"
        && entry["keywordValues"] == json!({"raid":1}) && entry["card"]["keywords"] == json!(["raid"])
        && sides.card.is_empty() && sides.leader.len() == 1 && sides.unit.len() == 1
        && raid(&sides.leader[0], false) && raid(&sides.unit[0], true)).then_some(Source::Leader)
}


#[cfg(test)]
#[path = "activation_contract_tests.rs"]
mod tests;

/// Combine the original activation contracts with the checkpoint's complete
/// Krennic, epic-base and deployment-hook registrations. Original base/raid
/// shapes retain their stricter selector contract and never enter the fallback.
pub fn base_zone_source(entry: &Value) -> Option<Source> {
    original_base_zone_source(entry).or_else(|| {
        if entry["card"]["hp"]==26 || entry["card"]["text"]=="Friendly units that cost 3 or more gain Raid 1." {return None;}
        checkpoint_source_kind(entry).map(|leader|if leader {Source::Leader}else{Source::Base})
    })
}

fn checkpoint_source_kind(entry:&Value)->Option<bool> {
 if crate::bounded_reductions::leader_entry(entry){return Some(true);}
 let sides: Sides=serde_json::from_value(entry["abilities"].clone()).ok()?;
 if !entry["card"]["unique"].is_boolean()||!entry["attachCondition"].is_null()||!entry["piloting"].is_null()||["pilotText","pilotBox","pilotingBox"].iter().any(|k|!entry["card"][k].is_null()){return None;}
 fn header(a:&Ability)->bool {a.trigger=="constant"&&!a.optional&&a.costs.is_empty()&&a.condition.is_none()&&!a.coordinate&&a.then.is_none()&&a.if_you_do.is_none()&&a.context_title.is_empty()&&a.target_resolvers.is_empty()}
 fn resolver(a:&Ability,kind:&str)->bool {a.target_resolver.as_ref().is_some_and(|r|r.owner=="self"&&r.card_type==kind&&r.zone.as_deref()==Some("arena")&&!r.optional&&r.arena.is_none()&&r.count.is_none()&&r.mode.is_none()&&r.prompt_title.is_none())}
 fn num(e:&Expr,n:u64)->bool {matches!(e,Expr::Const{value} if value.as_u64()==Some(n))}
 if entry["baseClass"]=="BaseCard"&&entry["card"]["types"]==serde_json::json!(["base"])&&entry["card"]["unique"]==false&&entry["card"]["hp"]==26&&entry["card"]["cost"].is_null()&&entry["card"]["power"].is_null()&&entry["keywordValues"]==serde_json::json!({})&&entry["card"]["keywords"]==serde_json::json!([])&&sides.card.len()==1&&sides.leader.is_empty()&&sides.unit.is_empty() {
  let a=&sides.card[0];let text=match &a.effect {Effect::ModifyStats{power,hp,..} if num(power,0)&&num(hp,1)=>"Each leader unit you control gets +0/+1.",_=>"Each leader unit you control gets +1/+0."};if entry["card"]["text"]!=text||!(entry["card"]["epicAction"].is_null()||entry["card"]["epicAction"].as_str()==Some(""))||!entry["card"]["rules"].is_null()||!entry["card"]["deployBox"].is_null()||!entry["card"]["upgradePower"].is_null()||!entry["card"]["upgradeHp"].is_null(){return None;}if header(a)&&resolver(a,"leaderUnit")&&a.target_resolver.as_ref().unwrap().predicate.is_none()&&matches!(&a.effect,Effect::ModifyStats{target:Target::Name(n),power,hp,duration} if n=="chosen"&&duration=="persistent"&&((num(power,0)&&num(hp,1))||(num(power,1)&&num(hp,0)))){return Some(false);}return None;
 }
 fn reference(e:&Expr,p:&[&str])->bool {matches!(e,Expr::Ref{path} if path.iter().map(String::as_str).collect::<Vec<_>>()==p)}
 fn cost_predicate(e:&Expr)->bool {matches!(e,Expr::Binary{op,left,right} if op=="&&"&&matches!(left.as_ref(),Expr::Query{name,receiver,args} if name=="isUnit"&&reference(receiver,&["card"])&&args.is_empty())&&matches!(right.as_ref(),Expr::Binary{op,left,right} if op==">="&&reference(left,&["card","cost"])&&num(right,3)))}
 fn raid(a:&Ability,exclude:bool)->bool {if !header(a)||!resolver(a,"unit")||!matches!(&a.effect,Effect::GainKeyword{target:Target::Name(n),keyword,amount,duration} if n=="chosen"&&keyword=="raid"&&num(amount,1)&&duration=="persistent"){return false;}a.target_resolver.as_ref().unwrap().predicate.as_ref().is_some_and(|p| if exclude{matches!(p,Expr::Binary{op,left,right} if op=="&&"&&cost_predicate(left)&&matches!(right.as_ref(),Expr::Binary{op,left,right} if op=="!=="&&reference(left,&["card"])&&reference(right,&["source"])))}else{cost_predicate(p)})}
 if entry["card"]["text"]=="Friendly units that cost 3 or more gain Raid 1."&&entry["card"]["deployBox"]=="Raid 1\nOther friendly units that cost 3 or more gain Raid 1."&&entry["card"]["epicAction"]=="Epic Action: If you control 6 or more resources, deploy this leader."&&entry["card"]["rules"]=="Abilities that refer to a card’s cost always refer to its printed cost, regardless of modifiers.\n"&&entry["card"]["upgradePower"].is_null()&&entry["card"]["upgradeHp"].is_null()&&entry["baseClass"]=="LeaderUnitCard"&&entry["card"]["types"]==serde_json::json!(["leader"])&&entry["card"]["unique"]==true&&entry["card"]["cost"]==6&&entry["card"]["hp"]==5&&entry["card"]["power"]==5&&entry["card"]["arena"]=="ground"&&entry["keywordValues"]==serde_json::json!({"raid":1})&&entry["card"]["keywords"]==serde_json::json!(["raid"])&&sides.card.is_empty()&&sides.leader.len()==1&&sides.unit.len()==1&&raid(&sides.leader[0],false)&&raid(&sides.unit[0],true){return Some(true);}
 // Director Krennic: both sides grant +1/+0 to friendly damaged units (damage is not a
 // live power/HP dependency, so constant evaluation stays acyclic).
 fn damaged_unit_predicate(e:&Expr)->bool {
  matches!(e,Expr::Binary{op,left,right} if op=="&&"
   && matches!(left.as_ref(),Expr::Query{name,receiver,args} if name=="isUnit"&&reference(receiver,&["card"])&&args.is_empty())
   && matches!(right.as_ref(),Expr::Binary{op,left,right} if op=="!=="&&reference(left,&["card","damage"])&&num(right,0)))
 }
 fn krennic_aura(a:&Ability)->bool {
  header(a)&&resolver(a,"unit")&&a.target_resolver.as_ref().unwrap().predicate.as_ref().is_some_and(damaged_unit_predicate)
   && matches!(&a.effect,Effect::ModifyStats{target:Target::Name(n),power,hp,duration} if n=="chosen"&&duration=="persistent"&&num(power,1)&&num(hp,0))
 }
 if entry["card"]["text"]=="Each friendly damaged unit gets +1/+0. "&&entry["card"]["deployBox"]=="Restore 2 (When this unit attacks, heal 2 damage from your base.)\n\nEach friendly damaged unit gets +1/+0."&&entry["card"]["epicAction"]=="Epic Action: If you control 5 or more resources, deploy this leader. "&&entry["card"]["rules"].is_null()&&entry["card"]["upgradePower"].is_null()&&entry["card"]["upgradeHp"].is_null()&&entry["baseClass"]=="LeaderUnitCard"&&entry["card"]["types"]==serde_json::json!(["leader"])&&entry["card"]["unique"]==true&&entry["card"]["cost"]==5&&entry["card"]["hp"]==7&&entry["card"]["power"]==2&&entry["card"]["arena"]=="ground"&&entry["keywordValues"]==serde_json::json!({"restore":2})&&entry["card"]["keywords"]==serde_json::json!(["restore"])&&sides.card.is_empty()&&sides.leader.len()==1&&sides.unit.len()==1&&krennic_aura(&sides.leader[0])&&krennic_aura(&sides.unit[0]){return Some(true);}
 // Reviewed base epic actions: sole card-side epicAction with existing effect leaves.
 fn epic_header(a:&Ability)->bool {
  a.trigger=="epicAction"&&!a.optional&&!a.requires_confirmation&&!a.coordinate&&a.costs.is_empty()
   &&a.condition.is_none()&&a.then.is_none()&&a.if_you_do.is_none()&&a.context_title.is_empty()
   &&a.target_resolvers.is_empty()&&a.hooks.is_empty()&&a.attached_context.is_none()&&a.gained_host.is_none()
 }
 fn epic_base_meta(entry:&Value,sides:&Sides)->bool {
  entry["baseClass"]=="BaseCard"&&entry["card"]["types"]==serde_json::json!(["base"])&&entry["card"]["unique"]==false
   &&entry["card"]["hp"]==25&&entry["card"]["cost"].is_null()&&entry["card"]["power"].is_null()
   &&(entry["card"]["epicAction"].is_null()||entry["card"]["epicAction"].as_str()==Some(""))
   &&entry["card"]["rules"].is_null()&&entry["card"]["deployBox"].is_null()
   &&entry["card"]["upgradePower"].is_null()&&entry["card"]["upgradeHp"].is_null()
   &&entry["keywordValues"]==serde_json::json!({})&&entry["card"]["keywords"]==serde_json::json!([])
   &&sides.card.len()==1&&sides.leader.is_empty()&&sides.unit.is_empty()
 }
 fn non_leader_unit_any(a:&Ability)->bool {
  a.target_resolver.as_ref().is_some_and(|r|r.owner=="any"&&r.card_type=="nonLeaderUnit"&&r.zone.as_deref()==Some("anyAttackable")
   &&!r.optional&&r.arena.is_none()&&r.count.is_none()&&r.mode.is_none()&&r.prompt_title.is_none()&&r.predicate.is_none())
 }
 if epic_base_meta(entry,&sides) {
  let a=&sides.card[0];
  if epic_header(a) {
   if entry["card"]["text"]=="Epic Action: Give a Shield token to a non-leader unit."
    && non_leader_unit_any(a)
    && matches!(&a.effect,Effect::GiveShield{target:Target::Name(n),amount:Expr::Const{value}} if n=="chosen"&&value.as_u64()==Some(1))
   {return Some(false);}
   if entry["card"]["text"]=="Epic Action: Deal 3 damage to a damaged non-leader unit."
    && a.target_resolver.as_ref().is_some_and(|r|r.owner=="any"&&r.card_type=="any"&&r.zone.as_deref()==Some("anyAttackable")
     &&!r.optional&&r.arena.is_none()&&r.count.is_none()&&r.mode.is_none()&&r.prompt_title.is_none()
     && r.predicate.as_ref().is_some_and(|p|matches!(p,Expr::Binary{op,left,right} if op=="&&"
      && matches!(left.as_ref(),Expr::Query{name,receiver,args} if name=="isNonLeaderUnit"&&reference(receiver,&["card"])&&args.is_empty())
      && matches!(right.as_ref(),Expr::Binary{op,left,right} if op=="!=="&&reference(left,&["card","damage"])&&num(right,0)))))
    && matches!(&a.effect,Effect::Damage{target:Target::Name(n),amount:Expr::Const{value}} if n=="chosen"&&value.as_u64()==Some(3))
   {return Some(false);}
   if entry["card"]["text"]=="Epic Action: Give a non-leader unit -4/-0 for this phase."
    && non_leader_unit_any(a)
    && matches!(&a.effect,Effect::ModifyStats{target:Target::Name(n),power,hp,duration}
     if n=="chosen"&&duration=="phase"
     && matches!(power,Expr::Unary{op,value} if op=="-"&&num(value,4))
     && num(hp,0))
   {return Some(false);}
   return None;
  }
 }
 // Reviewed base OnLeaderDeployed watchers (Droid Manufactory / Shadow Collective Camp).
 fn deploy_base_meta(entry:&Value,sides:&Sides,hp:u64)->bool {
  entry["baseClass"]=="BaseCard"&&entry["card"]["types"]==serde_json::json!(["base"])&&entry["card"]["unique"]==false
   &&entry["card"]["hp"]==hp&&entry["card"]["cost"].is_null()&&entry["card"]["power"].is_null()
   &&(entry["card"]["epicAction"].is_null()||entry["card"]["epicAction"].as_str()==Some(""))
   &&entry["card"]["rules"].is_null()&&entry["card"]["deployBox"].is_null()
   &&entry["card"]["upgradePower"].is_null()&&entry["card"]["upgradeHp"].is_null()
   &&entry["keywordValues"]==serde_json::json!({})&&entry["card"]["keywords"]==serde_json::json!([])
   &&sides.card.len()==1&&sides.leader.is_empty()&&sides.unit.is_empty()
   &&crate::death_hooks::leader_deploy_base_registration(&sides.card[0])
 }
 if deploy_base_meta(entry,&sides,24)
  &&entry["card"]["text"]=="When you deploy a leader: Create 2 Battle Droid tokens."
  && matches!(&sides.card[0].effect,Effect::CreateTokenUnit{token_type,amount:Expr::Const{value},target:Target::Name(t),enters_ready}
   if token_type=="battleDroid"&&value.as_u64()==Some(2)&&t=="ownPlayer"&&!*enters_ready)
 {return Some(false);}
 if deploy_base_meta(entry,&sides,25)
  &&entry["card"]["text"]=="When you deploy a leader: Draw a card."
  && matches!(&sides.card[0].effect,Effect::Draw{target:Target::Name(t),amount:Expr::Const{value}}
   if t=="ownPlayer"&&value.as_u64()==Some(1))
 {return Some(false);}
 None
}
