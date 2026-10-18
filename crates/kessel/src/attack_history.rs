//! The bounded projection of the pinned defeat watcher used by attack helpers.
//! The original in-play identities are retained for one attack, including all
//! suspended trigger contexts; this is not a query over current arena contents.
use serde::{Deserialize,Serialize};
use serde_json::{Value,json};

#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct AttackContextPath {
    kind:String,
    root:Vec<String>,
    context_binding:AttackContextBinding,
    steps:[AttackContextStep;2],
}
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
struct AttackContextBinding {id:String,role:String,bind_at:String,read_at:String,reference:String}
#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct AttackContextStep {key:String,optional:bool}
impl AttackContextPath {
    pub fn validate(&self)->Result<(),String> {
        let b=&self.context_binding;
        let id:Vec<_>=b.id.split(':').collect();
        let first_context=id.len()==3 && id[0]=="callback" && id[1].parse::<u32>().is_ok() && id[2]=="0";
        if self.kind!="dataPath" || self.root!=["context"] || self.steps[0].key!="event" || self.steps[1].key!="attack" || self.steps.iter().any(|s|s.optional) || b.role!="condition" || b.bind_at!="callbackInvocation" || b.read_at!="expressionEvaluation" || b.reference!="lexicalContextObject" || !first_context || b.id.len()>128 {return Err("attack history requires the lexical condition context's event.attack".into());}
        Ok(())
    }
}
pub fn validate(test:&str,attack:&AttackContextPath,watcher:&str,binding:&str)->Result<(),String> {
    if test!="defenderWasDefeated" || watcher!="cardsDefeatedThisPhase" || binding.is_empty() || binding.len()>128 {return Err("unsupported attack defeat history helper".into());}
    attack.validate()
}
pub fn contains(v:&Value)->bool { crate::ability_ir::contains(v,|v|v["kind"]=="historyAttackCheck") }
/// Every history read must resolve to the declared pinned watcher instance.
pub fn declarations(entry:&Value)->bool {
    fn bindings(v:&Value,out:&mut Vec<String>) {if v["kind"]=="historyAttackCheck" {if let Some(b)=v["binding"].as_str(){if !out.iter().any(|s|s==b){out.push(b.into());}}}match v{Value::Array(a)=>for v in a {bindings(v,out)},Value::Object(o)=>for v in o.values(){bindings(v,out)},_=>{}}}
    let mut used=vec![];bindings(&entry["abilities"],&mut used);
    if used.is_empty() {return entry["stateWatchers"].is_null() || entry["stateWatchers"]==json!([]);}
    let Some(declarations)=entry["stateWatchers"].as_array() else{return false;};
    declarations.len()==used.len() && used.iter().all(|binding|declarations.iter().any(|d|d==&json!({"kind":"phaseHistory","binding":binding,"watcher":"cardsDefeatedThisPhase","sharedScope":"game","registerAt":"setupStateWatchers","resetEvent":"onPhaseEnded","implementation":"CardsDefeatedThisPhaseWatcher","queryContract":"pinnedWatcherApi"})))
}
