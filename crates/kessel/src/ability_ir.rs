//! Portable, data-only card abilities emitted by the TypeScript card compiler.
//!
//! The IR is deliberately smaller than TypeScript. It cannot call back into JS,
//! access host state, or smuggle an unimplemented engine operation through a
//! callback. Loading a card validates every instruction before play can begin.
use serde::{Deserialize, Serialize};
use serde_json::Value;

// Inspect every serialized expression position, including selectors and continuations.
pub(crate) fn contains_force_presence<T: Serialize>(value:&T)->bool {
    fn visit(v:&Value)->bool {match v {
        Value::Object(o)=> (o.get("kind").is_some_and(|k|k=="ref") && o.get("path").and_then(Value::as_array).is_some_and(|p|p.last().is_some_and(|v|v=="hasTheForce"))) || (o.get("kind").is_some_and(|k|k=="query") && o.get("name").is_some_and(|n|n=="hasTheForce")) || o.values().any(visit),
        Value::Array(a)=>a.iter().any(visit),_=>false,
    }}
    visit(&serde_json::to_value(value).expect("typed IR serializes"))
}

#[path = "selector_contract.rs"]
pub mod selector_contract;

pub const SCHEMA: &str = "swu-card-ir/v1";
pub const MAX_PROGRAM_INSTRUCTIONS: usize = 128;
pub const MAX_GROUP_RECORDS: usize = 32;
pub const MAX_GROUP_OPS: usize = 128;
pub const MAX_ABILITY_NESTING: usize = 8;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum Expr {
    NativeTokenPredicate {
        operation: String,
        receiver: Box<Expr>,
        domain: String,
        #[serde(rename = "tokenType")]
        token_type: String,
        #[serde(rename = "optionalReceiver")]
        optional_receiver: bool,
        #[serde(rename = "optionalCall")]
        optional_call: bool,
    },
    HistoryQuery(crate::phase_history::Query),
    DataMember {receiver:crate::phase_history::List,key:crate::phase_history::Length,optional:bool},

    HistoryAttackCheck {test:String,attack:crate::attack_history::AttackContextPath,watcher:String,binding:String},
    ArenaList { receiver: Box<Expr>, predicate: Box<Expr> },
    ArenaFilter { receiver: Box<Expr>, predicate: Box<Expr> },
    ArenaSome { receiver: Box<Expr>, predicate: Box<Expr> },
    ArenaLength { receiver: Box<Expr> },
    Collection { name: String, receiver: Box<Expr>, args: Vec<Expr>, limit: usize },
    Array {
        items: Vec<Expr>,
    },
    Math {
        name: String,
        args: Vec<Expr>,
    },
    Const {
        value: Value,
    },
    Ref {
        path: Vec<String>,
    },
    Binary {
        op: String,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Unary {
        op: String,
        value: Box<Expr>,
    },
    Conditional {
        condition: Box<Expr>,
        then: Box<Expr>,
        r#else: Box<Expr>,
    },
    Query {
        name: String,
        receiver: Box<Expr>,
        #[serde(default)]
        args: Vec<Expr>,
    },
    /// Runtime attachment only: serialized content remains the portable expression.
    /// Raw IR cannot select an executable index.
    #[serde(untagged, skip_deserializing)]
    Compiled(CompiledExpr),
}
// Keep deserialization strictly tagged. Deriving it on Expr with the runtime
// untagged variant would erase useful unknown-kind/field diagnostics, even when
// that variant is marked skip_deserializing. The remote derive constructs Expr
// directly and exposes only its original portable variants to raw IR.
#[derive(Deserialize)]
#[allow(dead_code)]
#[serde(remote = "Expr", tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
enum PortableExpr {
    NativeTokenPredicate {
        operation: String,
        receiver: Box<Expr>,
        domain: String,
        #[serde(rename = "tokenType")]
        token_type: String,
        #[serde(rename = "optionalReceiver")]
        optional_receiver: bool,
        #[serde(rename = "optionalCall")]
        optional_call: bool,
    },
    HistoryQuery(crate::phase_history::Query),
    DataMember {receiver:crate::phase_history::List,key:crate::phase_history::Length,optional:bool},

    HistoryAttackCheck {test:String,attack:crate::attack_history::AttackContextPath,watcher:String,binding:String},
    ArenaList { receiver: Box<Expr>, predicate: Box<Expr> },
    ArenaFilter { receiver: Box<Expr>, predicate: Box<Expr> },
    ArenaSome { receiver: Box<Expr>, predicate: Box<Expr> },
    ArenaLength { receiver: Box<Expr> },
    Collection { name: String, receiver: Box<Expr>, args: Vec<Expr>, limit: usize },
    Array { items: Vec<Expr> },
    Math { name: String, args: Vec<Expr> },
    Const { value: Value },
    Ref { path: Vec<String> },
    Binary { op: String, left: Box<Expr>, right: Box<Expr> },
    Unary { op: String, value: Box<Expr> },
    Conditional { condition: Box<Expr>, then: Box<Expr>, r#else: Box<Expr> },
    Query {
        name: String,
        receiver: Box<Expr>,
        #[serde(default)]
        args: Vec<Expr>,
    },
}
impl<'de> Deserialize<'de> for Expr {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        PortableExpr::deserialize(deserializer)
    }
}
#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct CompiledExpr {
    pub(crate) original: Box<Expr>,
    #[serde(skip)]
    pub(crate) index: usize,
}
impl Expr {
    fn physical_inventory_value(&self)->bool {
        match self.original() {
            Self::Collection{name,..}=>name=="filter"||name=="map",
            Self::Ref{path}=>matches!(path.last().map(String::as_str),Some("upgrades"|"hand"|"discard"|"drawDeck"|"resources")),
            Self::Conditional{then,r#else,..}=>then.physical_inventory_value()||r#else.physical_inventory_value(),
            Self::Binary{op,left,right} if op=="&&"||op=="||"=>left.physical_inventory_value()||right.physical_inventory_value(),
            Self::Array{items}=>items.iter().any(Self::physical_inventory_value),
            _=>false,
        }
    }
    /// Only the two source-proven physical attachment filters are effect lists.
    /// Scalar collections remain separate from event batching admission.
    pub(crate) fn attachment_defeat_filter(&self) -> Option<bool> {
        let Self::Collection {name,receiver,args,limit}=self.original() else {return None;};
        if name!="filter" || *limit!=256 || !matches!(receiver.original(),Self::Ref{path} if path==&["target","upgrades"]) || args.len()!=1 {return None;}
        if matches!(args[0].original(),Self::Query{name,receiver,args} if name=="isShield"&&args.is_empty()&&matches!(receiver.original(),Self::Ref{path} if path==&["card"])) {return Some(true);}
        if matches!(args[0].original(),Self::Unary{op,value} if op=="!"&&matches!(value.original(),Self::Ref{path} if path==&["card","unique"])) {return Some(false);}
        None
    }
    /// Inspect the portable expression, independent of its executable attachment.
    pub fn original(&self) -> &Self {
        match self {
            Self::Compiled(compiled) => compiled.original.original(),
            _ => self,
        }
    }
    fn bind_compiled(&mut self) {
        if matches!(self, Self::Compiled(_)) { return; }
        // Binding happens only after admission. No serialization or content lookup
        // is performed while a search evaluates this expression.
        let bytes = serde_json::to_vec(self).expect("portable expressions serialize");
        if let Some(index) = crate::expression_backend(&bytes) {
            let original = Box::new(std::mem::replace(self, Self::Const { value: Value::Null }));
            *self = Self::Compiled(CompiledExpr { original, index });
        }
    }
    fn references_title(&self) -> bool {
        match self {
            Self::Compiled(compiled) => compiled.original.references_title(),
            Self::ArenaList {receiver,predicate} | Self::ArenaFilter {receiver,predicate} | Self::ArenaSome {receiver,predicate} => receiver.references_title() || predicate.references_title(), Self::ArenaLength {receiver} => receiver.references_title(),
            Self::Query{name,..} if name==crate::live_power_maximum::DISPLAY_QUERY=>true,
            Self::Ref {path}=>path.last().is_some_and(|p|p=="title"),
            Self::Array {items}=>items.iter().any(Self::references_title),Self::Math {args,..}=>args.iter().any(Self::references_title),Self::Unary {value,..}=>value.references_title(),Self::Binary {left,right,..}=>left.references_title()||right.references_title(),Self::Conditional {condition,then,r#else}=>condition.references_title()||then.references_title()||r#else.references_title(),Self::Collection {receiver,args,..} | Self::Query {receiver,args,..}=>receiver.references_title()||args.iter().any(Self::references_title),_=>false,
        }
    }
    fn depends_on_stats(&self) -> bool {
        match self {
            Self::Compiled(compiled) => compiled.original.depends_on_stats(),
            Self::ArenaList {receiver,predicate} | Self::ArenaFilter {receiver,predicate} | Self::ArenaSome {receiver,predicate} => receiver.depends_on_stats() || predicate.depends_on_stats(), Self::ArenaLength {receiver} => receiver.depends_on_stats(),
            Self::NativeTokenPredicate {receiver,..} => receiver.depends_on_stats(),
            Self::Collection {receiver,args,..} => receiver.depends_on_stats() || args.iter().any(Self::depends_on_stats),
            Self::HistoryQuery(_) | Self::DataMember {..} | Self::HistoryAttackCheck {..} | Self::Const { .. } => false,
            Self::Ref { path } => path.iter().any(|p| ["power", "hp", "remainingHp"].contains(&p.as_str())),
            Self::Array { items } => items.iter().any(Self::depends_on_stats),
            Self::Math { args, .. } => args.iter().any(Self::depends_on_stats),
            Self::Binary { left, right, .. } => left.depends_on_stats() || right.depends_on_stats(),
            Self::Unary { value, .. } => value.depends_on_stats(),
            Self::Conditional {
                condition,
                then,
                r#else,
            } => {
                condition.depends_on_stats() || then.depends_on_stats() || r#else.depends_on_stats()
            }
            Self::Query {
                name,
                receiver,
                args,
            } => {
                ["getPower", "getHp", "arenaPowerMax", "arenaPowerMaximumText"].contains(&name.as_str())
                    || receiver.depends_on_stats()
                    || args.iter().any(Self::depends_on_stats)
            }
        }
    }
    fn depends_on_keywords(&self) -> bool {
        match self {
            Self::Compiled(compiled) => compiled.original.depends_on_keywords(),
            Self::ArenaList {receiver,predicate} | Self::ArenaFilter {receiver,predicate} | Self::ArenaSome {receiver,predicate} => receiver.depends_on_keywords() || predicate.depends_on_keywords(), Self::ArenaLength {receiver} => receiver.depends_on_keywords(),
            Self::NativeTokenPredicate {receiver,..} => receiver.depends_on_keywords(),
            Self::Collection {receiver,args,..} => receiver.depends_on_keywords() || args.iter().any(Self::depends_on_keywords),
            Self::HistoryQuery(_) | Self::DataMember {..} | Self::HistoryAttackCheck {..} | Self::Const { .. } => false,
            Self::Ref { path } => path.iter().map(String::as_str).collect::<Vec<_>>()==["card","keywords","length"] || path.iter().any(|p| ["power", "hp", "remainingHp"].contains(&p.as_str())),
            Self::Array { items } => items.iter().any(Self::depends_on_keywords),
            Self::Math { args, .. } => args.iter().any(Self::depends_on_keywords),
            Self::Binary { left, right, .. } => left.depends_on_keywords() || right.depends_on_keywords(),
            Self::Unary { value, .. } => value.depends_on_keywords(),
            Self::Conditional {
                condition,
                then,
                r#else,
            } => {
                condition.depends_on_keywords() || then.depends_on_keywords() || r#else.depends_on_keywords()
            }
            Self::Query {
                name,
                receiver,
                args,
            } => {
                ["getPower", "getHp", "arenaPowerMaximumText", "hasSomeKeyword"].contains(&name.as_str())
                    || receiver.depends_on_keywords()
                    || args.iter().any(Self::depends_on_keywords)
            }
        }
    }
    fn live_only(&self)->bool {
        match self {
            Self::Compiled(compiled)=>compiled.original.live_only(),
            Self::ArenaList {receiver,predicate} | Self::ArenaFilter {receiver,predicate} | Self::ArenaSome {receiver,predicate} => receiver.live_only() || predicate.live_only(), Self::ArenaLength {receiver} => receiver.live_only(),Self::Collection {receiver,args,..}=>receiver.live_only()||args.iter().any(Self::live_only),Self::Ref {path}=>path.first().is_some_and(|p|p=="modifierTarget"),Self::Query {name,receiver,args}=>name=="resourcePairs"||receiver.live_only()||args.iter().any(Self::live_only),Self::Binary {left,right,..}=>left.live_only()||right.live_only(),Self::Unary {value,..}=>value.live_only(),Self::Conditional {condition,then,r#else}=>condition.live_only()||then.live_only()||r#else.live_only(),Self::Array {items}|Self::Math {args:items,..}=>items.iter().any(Self::live_only),_=>false}
    }
    fn live_stat_bounds(&self)->Result<(i64,i64),String> {
        fn root(e:&Expr,paths:&[&[&str]])->bool {matches!(e.original(),Expr::Ref {path} if paths.iter().any(|p|path.iter().map(String::as_str).collect::<Vec<_>>()==*p))}
        fn predicate(e:&Expr,depth:usize,identity:bool)->Result<(),String> {
            if depth>24 {return Err("live predicate depth exceeds capacity".into());}
            match e {
                Expr::Compiled(compiled)=>predicate(&compiled.original,depth,identity),
                Expr::Const {value} if value.is_boolean()||value.is_string()||value.as_i64().is_some_and(|n|(-1000..=1000).contains(&n))=>Ok(()),
                Expr::Ref {path} if (path.len()==2 && path[0]=="card" && ["damage","exhausted","unique","zoneName"].contains(&path[1].as_str())) || (identity && path.len()==1 && ["card","source","modifierTarget"].contains(&path[0].as_str()))=>Ok(()),
                Expr::Unary {op,value} if op=="!"=>predicate(value,depth+1,true),
                Expr::Binary {op,left,right} if ["&&","||","===","!==",">","<",">=","<="].contains(&op.as_str())=>{let identity=op=="!=="||op=="===";predicate(left,depth+1,identity)?;predicate(right,depth+1,identity)},
                Expr::Query {name,receiver,args} if root(receiver,&[&["card"]])=>{
                    if ["isUnit","isTokenUnit"].contains(&name.as_str()) && args.is_empty() {return Ok(());}
                    if ["hasSomeTrait","hasSomeAspect"].contains(&name.as_str()) && args.len()==1 {
                        let valid=match &args[0] {Expr::Const {value}=>value.is_string(),Expr::Array {items}=>!items.is_empty()&&items.len()<=8&&items.iter().all(|e|matches!(e,Expr::Const {value} if value.is_string())),_=>false};
                        if valid {return Ok(());}
                    }
                    Err("unsupported live-stat count predicate query".into())
                },_=>Err("unsupported live-stat predicate or effective-stat dependency".into())
            }
        }
        fn walk(e:&Expr,depth:usize)->Result<(i64,i64),String> {
            if depth>24 {return Err("live stat depth exceeds capacity".into());}
            let range=match e {
                Expr::Compiled(compiled)=>return walk(&compiled.original,depth),
                Expr::Const {value}=>{let n=value.as_i64().filter(|n|(-1000..=1000).contains(n)).ok_or("live stats require bounded integer literals")?;(n,n)},
                Expr::Ref {path} if path==&["modifierTarget","damage"]=>(0,65535),
                Expr::Ref {path} if path==&["modifierTarget","controller","resources","length"] || (path.len()==3&&["player","opponent"].contains(&path[0].as_str())&&["resources","hand"].contains(&path[1].as_str())&&path[2]=="length")=>(0,64),
                Expr::Unary {op,value} if op=="-"=>{let (a,b)=walk(value,depth+1)?;(-b,-a)},
                Expr::Binary {op,left,right} if ["+","-","*"].contains(&op.as_str())=>{let (a,b)=walk(left,depth+1)?;let(c,d)=walk(right,depth+1)?;match op.as_str(){"+"=>(a+c,b+d),"-"=>(a-d,b-c),_=>{let p=[a*c,a*d,b*c,b*d];(*p.iter().min().unwrap(),*p.iter().max().unwrap())}}},
                Expr::Query {name,receiver,args} if root(receiver,&[&["player"],&["opponent"],&["modifierTarget","controller"]])=>{
                    if name=="resourcePairs"&&args.is_empty() {(0,32)}else if name=="arenaUnitCount"&&args.len()==1 {predicate(&args[0],depth+1,false)?;(0,255)}else{return Err("unsupported live-stat query".into());}
                },_=>return Err("unsupported live stat expression or effective-stat dependency".into())
            };
            if range.0< -1_000_000 || range.1>1_000_000 {return Err("live stat arithmetic exceeds proven integer bounds".into());}Ok(range)
        }
        walk(self,0)
    }
    pub fn number(n: i64) -> Self {
        Self::Const {
            value: Value::from(n),
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        fn positive(e:&Expr)->bool {match e.original() {Expr::Query {name,receiver,args}=>name=="isDefending" && args.is_empty() && matches!(receiver.as_ref(),Expr::Ref {path} if path==&["card"]),Expr::Binary {op,left,right} if op=="&&"=>positive(left)||positive(right),_=>false}}
        fn guard(e:&Expr,defending:bool)->Result<(),String> {
            match e {
                Expr::Compiled(compiled)=>guard(&compiled.original,defending),
                Expr::Ref {path} if path.iter().any(|p|p=="activeAttack") => {if defending && path==&["card","activeAttack","attacker"] {Ok(())}else{Err("active attacker reference requires a dominating card.isDefending guard".into())}},
                Expr::Binary {op,left,right}=>{guard(left,defending)?;guard(right,defending||(op=="&&"&&positive(left)))},
                Expr::Collection {receiver,args,..}|Expr::Query {receiver,args,..}=>{guard(receiver,defending)?;for arg in args {guard(arg,false)?;}Ok(())},
                Expr::Array {items}|Expr::Math {args:items,..}=>{for item in items {guard(item,defending)?;}Ok(())},
                Expr::Unary {value,..}=>guard(value,defending),Expr::Conditional {condition,then,r#else}=>{guard(condition,defending)?;guard(then,defending)?;guard(r#else,defending)},_=>Ok(())
            }
        }
        guard(self,false)?;self.validate_depth(0)
    }
    fn contains_typed_query(&self)->bool {
        fn walk(v:&Value)->bool {
            (v["kind"]=="nativeTokenPredicate") || (v["kind"]=="query" && v["name"].as_str().is_some_and(|n|["hasControlledInPlayTitle","hasControlledInPlayTrait","hasAttachedTrait","hasAttachedNativeToken","leaderTraits"].contains(&n))) || match v {Value::Object(o)=>o.values().any(walk),Value::Array(a)=>a.iter().any(walk),_=>false}
        }
        walk(&serde_json::to_value(self).expect("typed expression serializes"))
    }
    // The only admitted floor/division family has a nonnegative base damage
    // numerator and positive literal divisor. Naked division remains rejected.
    pub(crate) fn bounded_floor_division(&self)->Option<(&Expr,i64)> {
        let Self::Math{name,args}=self.original() else{return None;};
        if name!="floor"||args.len()!=1 {return None;}
        let Self::Binary{op,left,right}=args[0].original() else{return None;};
        if op!="/"||!matches!(left.original(),Self::Ref{path} if path==&["player","base","damage"]||path==&["opponent","base","damage"]){return None;}
        let Self::Const{value}=right.original() else{return None;};let divisor=value.as_i64()?;
        (1..=65535).contains(&divisor).then_some((left.as_ref(),divisor))
    }
    pub fn validate_number(&self) -> Result<(), String> {
        self.validate()?;
        let numeric = match self {
            Self::HistoryQuery(_) => false,
            Self::DataMember {..} => true,
            Self::Compiled(compiled) => compiled.original.validate_number().is_ok(),
            Self::ArenaLength {..} => true, Self::ArenaList {..}|Self::ArenaFilter {..}|Self::ArenaSome {..} => false,
            Self::HistoryAttackCheck {..} | Self::Array { .. } | Self::NativeTokenPredicate {..} | Self::Collection { .. } => false,
            Self::Math { name, args } => {
                self.bounded_floor_division().is_some() || ["min", "max"].contains(&name.as_str())
                    && !args.is_empty()
                    && args.iter().all(|a| a.validate_number().is_ok())
            }
            Self::Const { value } => value.is_i64(),
            Self::Ref { path } => matches!(
                path.last().map(String::as_str),
                Some(
                    "damage"
                        | "power"
                        | "hp"
                        | "cost" | "printedCost"
                        | "remainingHp"
                        | "readyResourceCount"
                        | "exhaustedResourceCount"
                        | "length"
                        | "totalDistributed" | "damageDealt"
                )
            ),
            Self::Binary { op, left, right } => {
                ["+", "-", "*", "%"].contains(&op.as_str())
                    && left.validate_number().is_ok()
                    && right.validate_number().is_ok()
            }
            Self::Unary { op, value } => op == "-" && value.validate_number().is_ok(),
            Self::Conditional { then, r#else, .. } => {
                then.validate_number().is_ok() && r#else.validate_number().is_ok()
            }
            Self::Query { name, .. } => {
                ["getPower", "getHp", "getCost", "getPrintedCost", "arenaUnitCount", "resolvedDefeatedUnitCount", "arenaDamageSum", "arenaUpgradeSum", "arenaPowerMax"].contains(&name.as_str())
            }
        };
        if numeric {
            Ok(())
        } else {
            Err("effect amount expression is not numeric".into())
        }
    }
    fn validate_depth(&self, depth: usize) -> Result<(), String> {
        if depth > MAX_ABILITY_NESTING {
            return Err("expression nesting exceeds runtime capacity".into());
        }
        let sub = |e: &Expr| e.validate_depth(depth + 1);
        match self {
            Self::HistoryAttackCheck {test,attack,watcher,binding}=>crate::attack_history::validate(test,attack,watcher,binding),
            Self::Compiled(compiled) => compiled.original.validate_depth(depth),
            Self::NativeTokenPredicate {operation,receiver,domain,token_type,optional_receiver,optional_call} => {
                if *optional_receiver || *optional_call || domain!="physicalAttachments"
                    || !matches!((operation.as_str(),token_type.as_str()),("hasExperience","experience")|("hasShield","shield")|("hasWeakness","weakness"))
                    || !matches!(receiver.as_ref(),Self::Ref{path} if path.len()==1 && ["source","card","target","modifierTarget"].contains(&path[0].as_str())) {
                    return Err("native token predicate requires exact physical attachment operation and scalar card receiver".into());
                }
                sub(receiver)
            }
            Self::Collection {name,receiver,args,limit} => {
                if *limit != 256 || args.len()!=1 || !["some","every","filter","map","includes"].contains(&name.as_str()) {return Err("collection requires reviewed method, one argument and limit 256".into());}
                fn card_list(e:&Expr)->bool {match e.original(){Expr::Ref{path}=>matches!(path.iter().map(String::as_str).collect::<Vec<_>>().as_slice(),["source"|"card"|"target","upgrades"]|["player"|"opponent","hand"|"discard"|"drawDeck"|"resources"]),Expr::ArenaList{..}|Expr::ArenaFilter{..}=>true,Expr::Collection{name,receiver,..} if name=="filter"=>card_list(receiver),_=>false}}
                let mapped=matches!(receiver.original(),Expr::Collection{name,..} if name=="map");
                if !card_list(receiver) && !(name=="includes"&&mapped) {return Err("collection receiver must be a physical card inventory".into());}
                fn scalar(e:&Expr)->bool {match e.original(){Expr::Const{..}=>true,Expr::Ref{..}=>!e.physical_inventory_value(),Expr::Query{name,..}=>name!="leaderTraits",Expr::Binary{left,right,..}=>scalar(left)&&scalar(right),Expr::Unary{value,..}=>scalar(value),Expr::Conditional{condition,then,r#else}=>scalar(condition)&&scalar(then)&&scalar(r#else),Expr::Math{args,..}=>args.iter().all(scalar),Expr::Collection{name,..}=>["some","every","includes"].contains(&name.as_str()),_=>false}}
                if !scalar(&args[0]) {return Err("collection callback/value must return a supported scalar".into());}
                sub(receiver)?;sub(&args[0])
            },
            Self::HistoryQuery(q) => {if q.kind()?==crate::phase_history::QueryKind::PlayedOpponentCount {Err("history list requires exact length consumption".into())}else{Ok(())}},
            Self::DataMember {receiver:crate::phase_history::List::HistoryQuery(q),optional,..} => {if !optional&&q.kind()?==crate::phase_history::QueryKind::PlayedOpponentCount {Ok(())}else{Err("only nonoptional bounded history list length executes".into())}},
            Self::ArenaList {receiver,predicate} => {if !matches!(receiver.as_ref(),Self::Ref{path} if path.len()==1 && ["player","opponent","game"].contains(&path[0].as_str())) {return Err("arena list requires exact player/opponent/game receiver".into());}sub(predicate)},
            Self::ArenaFilter {receiver,predicate}|Self::ArenaSome {receiver,predicate} => {if !matches!(receiver.as_ref(),Self::ArenaList{..}) {return Err("arena filter/some requires direct arena list".into());}sub(receiver)?;sub(predicate)},
            Self::ArenaLength {receiver} => {if !matches!(receiver.as_ref(),Self::ArenaList{..}|Self::ArenaFilter{..}) {return Err("arena length requires bounded list".into());}sub(receiver)},
            Self::Array { items } => {
                for item in items {
                    sub(item)?;
                }
                Ok(())
            }
            Self::Math { name, args } => {
                if self.bounded_floor_division().is_some() {return Ok(());}
                if !["min", "max"].contains(&name.as_str()) || args.is_empty() {
                    return Err("unsupported math expression".into());
                }
                for arg in args {
                    sub(arg)?;
                    arg.validate_number()?;
                }
                Ok(())
            }
            Self::Const { value }
                if value.is_i64() || value.is_boolean() || value.is_string() || value.is_null() =>
            {
                Ok(())
            }
            Self::Const { .. } => {
                Err("only integer, boolean, string and null constants are executable".into())
            }
            Self::Ref { path } => {
                let valid = match path
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>()
                    .as_slice()
                {
                    ["source" | "target" | "card" | "player" | "opponent"] => true,
                    ["source" | "target" | "card", "title" | "upgrades"] => true,
                    ["player" | "opponent", "hand" | "discard" | "drawDeck" | "resources"] => true,
                    ["player", "hasTheForce"] => true,
                    ["card","activeAttack","attacker"] => true,
                    ["card","keywords","length"] => true,
                    ["modifierTarget"] | ["modifierTarget","damage"|"controller"] | ["modifierTarget","controller","resources","length"] => true,
                    // Upgrade abilities read the live host via source.parent*.
                    // Ordinary units/events may filter selected upgrades via card/target.parent*.
                    ["source" | "target" | "card", "parentCard" | "parentUnit"] => true,
                    ["source" | "target" | "card", "parentCard" | "parentUnit", "damage" | "power" | "hp" | "remainingHp" | "controller" | "exhausted" | "unique" | "zoneName"] => true,
                    ["source" | "target" | "card", "parentCard", "title"] => true,
                    ["source" | "target" | "card", "damage" | "power" | "hp" | "cost" | "printedCost" | "remainingHp" | "exhausted"
                    | "controller" | "unique" | "zoneName"] => true,
                    ["player" | "opponent", "readyResourceCount" | "exhaustedResourceCount" | "base"] => {
                        true
                    }
                    ["player" | "opponent", "resources" | "hand" | "drawDeck" | "discard", "length"] => {
                        true
                    }
                    ["player" | "opponent", "base", "damage"] => true,
                    ["event", "totalDistributed" | "damageDealt"] => true,
                    ["event", "lastKnownInformation", "power"] => true,
                    _ => false,
                };
                if valid {
                    Ok(())
                } else {
                    Err(format!("unsupported runtime reference {}", path.join(".")))
                }
            }
            Self::Binary { op, left, right } => {
                if !["&&","||"].contains(&op.as_str())&&(left.physical_inventory_value()||right.physical_inventory_value()){return Err("physical card inventories require collection operations".into());}
                if (left.references_title() || right.references_title()) && !["===","!==","&&","||"].contains(&op.as_str()) { return Err("printed titles support strict equality and logical operators only".into()); }
                if ![
                    "+", "-", "*", "%", "===", "!==", "<", "<=", ">", ">=", "&&", "||",
                ]
                .contains(&op.as_str())
                {
                    return Err(format!("unsupported expression operator {op}"));
                }
                sub(left)?;
                sub(right)
            }
            Self::Unary { op, value } => {
                if op!="!"&&value.physical_inventory_value(){return Err("physical card inventories do not support numeric coercion".into());}
                if value.references_title() && op!="!" { return Err("printed titles do not support numeric unary coercion".into()); }
                if !["!", "-"].contains(&op.as_str()) {
                    return Err(format!("unsupported unary operator {op}"));
                }
                sub(value)
            }
            Self::Conditional {
                condition,
                then,
                r#else,
            } => {
                sub(condition)?;
                sub(then)?;
                sub(r#else)
            }
            Self::Query {
                name,
                receiver,
                args,
            } => {
                if receiver.physical_inventory_value()||args.iter().any(Self::physical_inventory_value){return Err("physical card inventories require typed collection operations".into());}
                if receiver.references_title() || args.iter().any(Self::references_title) { return Err("printed title query coercion is unsupported".into()); }
                let arity = match name.as_str() {
                    "hasControlledInPlayTrait"=>2,
                    "arenaPowerMaximumText" | "arenaDamageSum" | "arenaUpgradeSum" | "arenaPowerMax" | "leaderTraits" | "hasCost" | "isDeployed" | "isUpgrade" | "canBeExhausted" | "isUpgraded" | "isInPlay" | "isNonLeaderUnit"
                    | "hasInitiative" | "isUnit" | "isEvent" | "isBase" | "isLeader" | "getPower" | "getHp"
                    | "getCost" | "getPrintedCost" | "isToken" | "isTokenUnit" | "isTokenUpgrade" | "isShield" | "resourcePairs" | "isAttacking" | "isDefending" | "resolvedDefeatedUnitCount" => 0,
                    "hasSomeTrait" | "hasSomeAspect" | "hasSomeKeyword" | "hasSomeArenaCard" | "hasSomeArenaUnit"
                    | "arenaUnitCount" | "hasControlledInPlayTitle" | "hasAttachedTrait" | "hasAttachedNativeToken" => 1,
                    _ => return Err(format!("unsupported runtime query {name}")),
                };
                if args.len() != arity {
                    return Err(format!("query {name} expects {arity} arguments"));
                }
                if crate::bounded_reductions::QUERIES.contains(&name.as_str()) && !matches!(receiver.as_ref(),Expr::Ref{path} if path==&["player"]||path==&["opponent"]) {return Err("bounded arena reduction requires exact scalar player receiver".into());}
                if name==crate::live_power_maximum::DISPLAY_QUERY&&!matches!(receiver.as_ref(),Expr::Ref{path} if path==&["player"]) {return Err("maximum display requires exact own-player receiver".into());}
                if name=="resolvedDefeatedUnitCount" && !matches!(receiver.as_ref(),Expr::Ref{path} if path==&["player"]||path==&["opponent"]) {return Err("resolved defeat count requires scalar player receiver".into());}
                if name=="hasControlledInPlayTrait" {
                    if !matches!(receiver.as_ref(),Expr::Ref{path} if path==&["player"]||path==&["opponent"])||!matches!(&args[0],Expr::Const{value} if value.as_str().is_some_and(|t|!t.is_empty()&&t.len()<=64))||!matches!(&args[1],Expr::Ref{path} if path==&["source"]) {return Err("controlled trait requires bounded literal trait and source exclusion".into());}
                }
                if name=="leaderTraits" && !matches!(receiver.as_ref(),Expr::Ref{path} if path==&["player"]||path==&["opponent"]) {return Err("leader trait union requires scalar player receiver".into());}
                if name=="hasControlledInPlayTitle" {
                    if !matches!(receiver.as_ref(),Expr::Ref{path} if path==&["player"]||path==&["opponent"]) || !matches!(&args[0],Expr::Const{value} if value.as_str().is_some_and(|t|!t.is_empty()&&t.len()<=128)) {return Err("controlled title requires scalar player and bounded literal title".into());}
                }
                if name=="hasAttachedTrait" || name=="hasAttachedNativeToken" {
                    if !matches!(receiver.as_ref(),Expr::Ref{path} if path.len()==1&&["source","card","target","modifierTarget"].contains(&path[0].as_str())) || !matches!(&args[0],Expr::Const{value} if value.as_str().is_some_and(|t|if name=="hasAttachedNativeToken" {t=="shield"} else {!t.is_empty()&&t.len()<=64})) {return Err("physical attachment query requires scalar card and reviewed literal argument".into());}
                }
                if name=="hasSomeArenaCard" {
                    fn predicate(e:&Expr)->bool {match e {
                        Expr::Const{value}=>value.as_bool()==Some(true),
                        Expr::Query{name,receiver,args} if matches!(receiver.as_ref(),Expr::Ref{path} if path==&["card"])=>match name.as_str(){"isLeader"|"isDeployed"|"isUpgrade"=>args.is_empty(),"hasSomeTrait"=>args.len()==1&&matches!(&args[0],Expr::Const{value} if value.as_str().is_some_and(|s|!s.is_empty()&&s.len()<=64)),_=>false},
                        Expr::Binary{op,left,right} if op=="&&"=>predicate(left)&&predicate(right),_=>false}}
                    if !predicate(&args[0]) {return Err("arena-card existence supports bounded static trait/type predicates".into());}
                }
                if name=="hasSomeArenaCard" && !matches!(receiver.as_ref(),Expr::Ref {path} if path==&["player"] || path==&["opponent"]) {return Err("arena-card query requires a scalar player receiver".into());}
                if name=="isDeployed" && !matches!(receiver.as_ref(),Expr::Ref {path} if path.len()==1 && ["source","card","target"].contains(&path[0].as_str())) {return Err("isDeployed requires an explicit scalar card receiver".into());}
                if name=="isUpgrade" && !matches!(receiver.as_ref(),Expr::Ref {path} if path.len()==1 && ["source","card","target"].contains(&path[0].as_str())) {return Err("isUpgrade requires an explicit card receiver".into());}
                if name=="hasCost" && !matches!(receiver.as_ref(),Expr::Ref {path} if path.len()==1 && ["source","card","target"].contains(&path[0].as_str())) {return Err("hasCost requires an explicit card receiver".into());}
                if ["isTokenUnit","isTokenUpgrade","isShield"].contains(&name.as_str()) && !matches!(receiver.as_ref(),Expr::Ref {path} if path.len()==1 && ["source","card","target"].contains(&path[0].as_str())) {return Err("token identity query requires an explicit card receiver".into());}
                if ["isAttacking","isDefending"].contains(&name.as_str()) && !matches!(receiver.as_ref(),Expr::Ref {path} if (path.len()==1 && ["source","card","target"].contains(&path[0].as_str())) || (name=="isDefending" && path==&["source","parentUnit"])) {return Err("attack query requires a scalar source/card/target or reviewed defending parent receiver".into());}
                sub(receiver)?;
                for arg in args {
                    sub(arg)?;
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Target {
    Name(String),
    ArenaList(ArenaListTarget),
    Expression(Expr),
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArenaListTarget { pub kind: ArenaListKind, pub receiver: Expr, pub predicate: Expr,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub controller_order: Option<ControllerOrder>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ControllerOrder { PlayerFirst, OpponentFirst, SeatOrder }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArenaListKind { #[serde(rename = "arenaList")] ArenaList }
impl ArenaListTarget {
    fn validate_ordered_ground(&self)->Result<(),String> {
        let ground:Value=serde_json::from_str(r##"{"kind":"binary","op":"&&","left":{"kind":"const","value":true},"right":{"kind":"binary","op":"||","left":{"kind":"unary","op":"!","value":{"kind":"const","value":"groundArena"}},"right":{"kind":"binary","op":"||","left":{"kind":"binary","op":"===","left":{"kind":"const","value":"groundArena"},"right":{"kind":"const","value":"anyArena"}},"right":{"kind":"binary","op":"===","left":{"kind":"ref","path":["card","zoneName"]},"right":{"kind":"const","value":"groundArena"}}}}}"##).unwrap();
        if self.controller_order.is_none() || !matches!(&self.receiver,Expr::Ref{path} if path==&["game"]) || serde_json::to_value(&self.predicate).ok()!=Some(ground) {return Err("ordered lists require global controller partitions and exact ground scope".into());}
        Ok(())
    }

    fn validate_phase_stats(&self)->Result<(),String> {
        if self.controller_order.is_some() {return Err("controller ordering requires the standalone ordered-ground family".into());}
        if self.validate_scope(false).is_ok() {return Ok(());}
        self.receiver.validate()?;self.predicate.validate()?;
        if !matches!(&self.receiver,Expr::Ref{path} if path.len()==1&&["player","opponent"].contains(&path[0].as_str())) {return Err("phase-stat lists require one player arena inventory".into());}
        fn reference(e:&Expr,name:&str)->bool {matches!(e,Expr::Ref{path} if path.len()==1&&path[0]==name)}
        fn predicate(e:&Expr)->bool {
            // getArenaUnits({otherThan:source}) generates the nullable exclusion.
            if matches!(e,Expr::Binary{op,left,right} if op=="||" && matches!(left.as_ref(),Expr::Unary{op,value} if op=="!"&&reference(value,"source")) && matches!(right.as_ref(),Expr::Binary{op,left,right} if op=="!=="&&reference(left,"card")&&reference(right,"source"))) {return true;}
            match e {
                Expr::Const{value}=>value.is_boolean(),
                Expr::Binary{op,left,right} if ["&&","||"].contains(&op.as_str())=>predicate(left)&&predicate(right),
                Expr::Query{name,receiver,args} if name=="hasSomeTrait" && reference(receiver,"card") && args.len()==1=>matches!(&args[0],Expr::Const{value} if value.as_str().is_some_and(|v|!v.is_empty())),
                Expr::Binary{op,left,right} if op=="==="=> {
                    let zone=|e:&Expr|matches!(e,Expr::Ref{path} if path.iter().map(String::as_str).collect::<Vec<_>>()==["card","zoneName"]);
                    let name=|e:&Expr|matches!(e,Expr::Const{value} if matches!(value.as_str(),Some("groundArena"|"spaceArena"|"anyArena")));
                    (zone(left)&&name(right))||(name(left)&&name(right))
                },_=>false,
            }
        }
        if !predicate(&self.predicate) {return Err("phase-stat filters require static traits, arenas or source exclusion".into());}Ok(())
    }
    // The bounded attack family selects friendly token units at resolution.
    // Keep token upgrades, opposing tokens and outer-target aliases out of it.
    fn token_unit_attack_scope(&self)->bool {
        if self.controller_order.is_some() {return false;}
        fn token(e:&Expr)->bool {matches!(e,Expr::Query{name,receiver,args} if name=="isTokenUnit" && args.is_empty() && matches!(receiver.as_ref(),Expr::Ref{path} if path.len()==1&&path[0]=="card"))}
        matches!(&self.receiver,Expr::Ref{path} if path.len()==1&&path[0]=="player") &&
            (token(&self.predicate)||matches!(&self.predicate,Expr::Binary{op,left,right} if op=="&&"&&matches!(left.as_ref(),Expr::Const{value} if value.as_bool()==Some(true))&&token(right)))
    }
    fn validate_damage(&self)->Result<(),String> {
        if self.controller_order.is_some() {return Err("controller ordering requires the standalone ordered-ground family".into());}
        if !matches!(&self.receiver,Expr::Ref{path} if path.len()==1&&["game","player","opponent"].contains(&path[0].as_str())) {return Err("arena damage requires an explicit game/player/opponent receiver".into());}
        self.predicate.validate()?;
        fn reference(e:&Expr,name:&str)->bool {matches!(e,Expr::Ref{path} if path.len()==1&&path[0]==name)}
        fn predicate(e:&Expr)->bool {
            if matches!(e,Expr::Binary{op,left,right} if op=="||" && matches!(left.as_ref(),Expr::Unary{op,value} if op=="!"&&reference(value,"source")) && matches!(right.as_ref(),Expr::Binary{op,left,right} if op=="!=="&&reference(left,"card")&&reference(right,"source"))) {return true;}
            match e {
                Expr::Const{value}=>value.is_boolean(),
                Expr::Unary{op,value} if op=="!"=>matches!(value.as_ref(),Expr::Const{value} if matches!(value.as_str(),Some("groundArena"|"spaceArena"))),
                Expr::Binary{op,left,right} if ["&&","||"].contains(&op.as_str())=>predicate(left)&&predicate(right),
                Expr::Binary{op,left,right} if op=="==="=>{
                    let zone=|e:&Expr|matches!(e,Expr::Ref{path} if path.iter().map(String::as_str).collect::<Vec<_>>()==["card","zoneName"]);
                    let name=|e:&Expr|matches!(e,Expr::Const{value} if matches!(value.as_str(),Some("groundArena"|"spaceArena"|"anyArena")));
                    (zone(left)&&name(right))||(name(left)&&name(right))
                },
                Expr::Query{name,receiver,args} if name=="isUnit"=>reference(receiver,"card")&&args.is_empty(),
                Expr::Ref{path}=>path.iter().map(String::as_str).collect::<Vec<_>>()==["card","exhausted"],
                _=>false,
            }
        }
        if !predicate(&self.predicate) {return Err("arena damage predicates require static arena/source/exhausted unit scope".into());}Ok(())
    }
    fn validate_scope(&self, global:bool)->Result<(),String> {
        if self.controller_order.is_some() {return Err("controller ordering requires the standalone ordered-ground family".into());}
        if !(global && matches!(&self.receiver,Expr::Ref {path} if path.iter().map(String::as_str).collect::<Vec<_>>()==["game"])) {self.receiver.validate()?;}
        self.predicate.validate()?;
        if !matches!(&self.receiver,Expr::Ref {path} if path.len()==1 && (["player","opponent"].contains(&path[0].as_str()) || (global && path[0]=="game"))) {return Err("arena-list receiver is outside the verified scope".into());}
        fn predicate(e:&Expr,global:bool)->bool {match e {
            Expr::Const {value}=>value.is_boolean(),
            Expr::Unary {op,value} if op=="!"=>predicate(value,global)||matches!(value.as_ref(),Expr::Const {value} if matches!(value.as_str(),Some("groundArena"|"spaceArena"))),
            Expr::Binary {op,left,right} if ["&&","||"].contains(&op.as_str())=>predicate(left,global)&&predicate(right,global),
            Expr::Binary {op,left,right} if op=="==="=>{
                let zone=|e:&Expr|matches!(e,Expr::Ref {path} if path.iter().map(String::as_str).collect::<Vec<_>>()==["card","zoneName"]);
                let name=|e:&Expr|matches!(e,Expr::Const {value} if matches!(value.as_str(),Some("groundArena"|"spaceArena"|"anyArena")));
                (zone(left)&&name(right))||(name(left)&&name(right))
            },
            Expr::Query {name,receiver,args} if global && ["isUnit","isUpgraded"].contains(&name.as_str())=>args.is_empty() && matches!(receiver.as_ref(),Expr::Ref {path} if path.iter().map(String::as_str).collect::<Vec<_>>()==["card"]),
            _=>false}}
        if !predicate(&self.predicate,global) {return Err("arena-list phase-stat predicate requires verified arena-only scope".into());}Ok(())
    }
}
impl Target {
    pub fn name(name: &str) -> Self {
        Self::Name(name.into())
    }
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::ArenaList(_)=>Err("arena lists require standalone verified phase-stat semantics".into()),
            Self::Name(s)
                if [
                    "self",
                    "parent",
                    "ownBase",
                    "enemyBase",
                    "bothBases",
                    "chosen",
                    "chosenPlayer",
                    "ownPlayer",
                    "enemyPlayer",
                    "bothPlayers",
                ]
                .contains(&s.as_str()) =>
            {
                Ok(())
            }
            Self::Name(s) => Err(format!("unsupported effect target {s}")),
            Self::Expression(e) => {
                fn is_target(e: &Expr) -> bool {
                    match e {
                        Expr::Compiled(compiled) => is_target(&compiled.original),
                        Expr::Ref { path } => matches!(
                            path.iter()
                                .map(String::as_str)
                                .collect::<Vec<_>>()
                                .as_slice(),
                            ["source" | "target" | "card" | "player" | "opponent"]
                                | ["player" | "opponent", "base"]
                                | ["source" | "target" | "card", "controller"]
                                | ["source", "parentCard" | "parentUnit"]
                        ),
                        Expr::Conditional { then, r#else, .. } => {
                            is_target(then) && (is_target(r#else) || matches!(r#else.as_ref(),Expr::Const {value} if value.is_null()))
                        }
                        _ => false,
                    }
                }
                if !is_target(e) {
                    return Err("effect target must be a bounded card or player reference; array targets require verified event batching".into());
                }
                e.validate()
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct TargetResolver {
    pub prompt_title: Option<String>,
    pub prompt_title_parts: Vec<Expr>,
    pub owner: String,
    pub card_type: String,
    pub arena: Option<String>,
    pub zone: Option<String>,
    pub optional: bool,
    pub predicate: Option<Expr>,
    pub mode: Option<String>,
    pub count: Option<Expr>,
    pub native: Option<selector_contract::NativeCardSelection>,
    /// Exact `playerSelection` / mode `player` resolver from the compiler IR.
    pub player_selection: bool,
}
impl TargetResolver {
    /// A positive type predicate supplies the selector inventory domain; ordinary
    /// `any` continues to enumerate units/bases. Do not admit mixed domains.
    pub fn is_upgrade_domain(&self) -> bool {
        fn implies(e:&Expr)->bool { match e.original() {
            Expr::Query {name,receiver,args} if name=="isUpgrade" && args.is_empty()=>matches!(receiver.as_ref(),Expr::Ref {path} if path.len()==1 && path[0]=="card"),
            Expr::Binary {op,left,right} if op=="&&"=>implies(left)||implies(right),
            Expr::Binary {op,left,right} if op=="||"=>implies(left)&&implies(right),
            _=>false,
        }}
        if let Some(native)=&self.native {
            return native.types().is_ok_and(|types|types.iter().all(|t|t.matches(selector_contract::CardTypeFilter::BasicUpgrade)||t.matches(selector_contract::CardTypeFilter::TokenUpgrade))&&types.iter().all(|t|!t.matches(selector_contract::CardTypeFilter::BasicUnit)&&!t.matches(selector_contract::CardTypeFilter::Base)));
        }
        selector_contract::CardTypeFilter::parse(&self.card_type).is_ok_and(|filter|filter.upgrade_only()) || (self.card_type=="any" && self.predicate.as_ref().is_some_and(implies))
    }
    fn mentions_upgrade_query(&self)->bool {
        fn mentions(e:&Expr)->bool {match e.original() {
            Expr::Query {name,receiver,args}=>name=="isUpgrade"||mentions(receiver)||args.iter().any(mentions),
            Expr::Array {items}=>items.iter().any(mentions), Expr::Math {args,..}=>args.iter().any(mentions),
            Expr::Binary {left,right,..}=>mentions(left)||mentions(right), Expr::Unary {value,..}=>mentions(value),
            Expr::Conditional {condition,then,r#else}=>mentions(condition)||mentions(then)||mentions(r#else),
            _=>false,
        }}
        self.predicate.as_ref().is_some_and(mentions)
    }
    pub fn max_targets(&self) -> Result<usize, String> {
        if self.player_selection {
            if self.mode.as_deref() != Some("player") || self.count.is_some() || self.optional {
                return Err("player selection requires exact non-optional mode player".into());
            }
            return Ok(1);
        }
        match self.mode.as_deref() {
            None | Some("single") if self.count.is_none() => Ok(1),
            Some("upTo") => match self.count.as_ref().map(Expr::original) {
                Some(Expr::Const { value }) if value.as_u64().is_some_and(|n| (1..=5).contains(&n)) => Ok(value.as_u64().unwrap() as usize),
                _ => Err("UpTo selectors require literal counts from one to five".into()),
            },
            Some("unlimited") if self.card_type == "base" && self.arena.is_none() && self.count.is_none() && self.optional && matches!(self.zone.as_deref(), None | Some("base" | "anyAttackable")) => Ok(2),
            _ => Err("unsupported target mode/count; unlimited selection is base-only".into()),
        }
    }
    pub fn is_multiple(&self) -> bool { matches!(self.mode.as_deref(), Some("upTo" | "unlimited")) }
    pub fn validate_inline(&self)->Result<(),String> {
        if self.prompt_title.as_ref().is_some_and(|t|t.is_empty()||t.len()>1024){return Err("inline prompt title must be a bounded nonempty literal".into());}
        let mut r=self.clone();r.prompt_title=None;r.validate()
    }
    pub fn validate(&self) -> Result<(), String> {
        if let Some(native)=&self.native {
            native.validate()?;
            if let Some(predicate)=&self.predicate {predicate.validate()?;}
            return Ok(());
        }
        if self.player_selection {
            if self.card_type != "player"
                || self.owner != "any"
                || self.arena.is_some()
                || self.zone.is_some()
                || self.optional
                || self.predicate.is_some()
                || self.count.is_some()
                || self.prompt_title.is_some()
                || self.mode.as_deref() != Some("player")
            {
                return Err("player selection requires exact reviewed single-player resolver shape".into());
            }
            let mut bytes = 0;
            for fragment in &self.prompt_title_parts {
                let Expr::Const { value } = fragment else {
                    return Err("player selection prompt requires bounded static text".into());
                };
                bytes += value.as_str().ok_or("player selection prompt requires string fragments")?.len();
            }
            if bytes > 1024 || self.prompt_title_parts.len() > 32 {
                return Err("player selection prompt exceeds capacity".into());
            }
            return self.max_targets().map(|_| ());
        }
        if !self.prompt_title_parts.is_empty() {
            let expected=serde_json::json!([{"kind":"const","value":"Choose a unit to deal "},{"kind":"query","name":"arenaDamageSum","receiver":{"kind":"ref","path":["player"]},"args":[]},{"kind":"const","value":" damage to"}]);
            let source_power_parts=self.prompt_title_parts.len()<=32 && self.prompt_title_parts.iter().all(|e|match e {Expr::Const{value}=>value.as_str().is_some(),Expr::Query{name,receiver,args}=>name=="getPower"&&args.is_empty()&&matches!(receiver.as_ref(),Expr::Ref{path} if path==&["source"]),_=>false}) && self.prompt_title_parts.iter().map(|e|match e {Expr::Const{value}=>value.as_str().map_or(0,str::len),_=>0}).sum::<usize>()<=1024;
            if serde_json::to_value(&self.prompt_title_parts).ok()!=Some(expected)&&!crate::live_power_maximum::prompt_parts(&self.prompt_title_parts)&&!source_power_parts{return Err("dynamic prompt requires bounded live arena damage sum, maximum, or source power".into());}
        }
        if self.prompt_title.is_some() {return Err("named prompt titles require a named descriptor".into());}
        self.max_targets()?;
        if self.card_type=="any" && self.mentions_upgrade_query() && !self.is_upgrade_domain() {return Err("mixed upgrade selector domains are not modeled".into());}
        if self.is_upgrade_domain() && (self.arena.is_some() || !matches!(self.zone.as_deref(),None|Some("arena"|"anyAttackable"|"discard"))) {return Err("upgrade selectors require arena/anyAttackable without an arena filter".into());}
        if !["self", "opponent", "any"].contains(&self.owner.as_str()) {
            return Err(format!("unsupported target owner {}", self.owner));
        }
        let filter=selector_contract::CardTypeFilter::parse(&self.card_type)?;
        if self.card_type!="any" && self.zone.as_deref()!=Some("discard") && filter.matches(selector_contract::CardTypeFilter::TokenCard) {return Err("selector base-token inventory requires physical base-zone tokens".into());}
        if self.card_type!="any" && !filter.upgrade_only() && self.zone.as_deref()!=Some("discard") && filter.matches(selector_contract::CardTypeFilter::BasicUpgrade) {return Err("mixed upgrade selector domains require a separately reviewed inventory".into());}
        let legal=if self.zone.as_deref()==Some("discard") {filter.legal_in_discard()}else if self.zone.as_deref()==Some("base") {filter.legal_in_base()}else if self.zone.as_deref()==Some("arena")||self.arena.is_some() {filter.legal_in_arena()}else{filter.legal_in_arena()||filter.legal_in_base()};
        if !legal {return Err("selector type/zone filters have no legal overlap".into());}
        if self
            .arena
            .as_deref()
            .is_some_and(|a| !["ground", "space"].contains(&a))
        {
            return Err(format!("unsupported arena {:?}", self.arena));
        }
        if self
            .zone
            .as_deref()
            .is_some_and(|z| !["arena", "base", "anyAttackable", "discard"].contains(&z))
        {
            return Err(format!("unsupported target zone {:?}", self.zone));
        }
        if let Some(p) = &self.predicate {
            p.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum Cost {
    ExhaustSelf,
    Resources { amount: Expr },
    UseForceCost {
        target: String,
        timing: String,
        #[serde(rename = "mustFullyResolve")]
        must_fully_resolve: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum Effect {
    InitiateAttack {properties:crate::native_attack::Properties, #[serde(default,rename="initiationProperty",skip_serializing_if="std::ops::Not::not")] initiation_property:bool},
    ConstantEffectRegistration(crate::static_ongoing::Registration),
    GainTrait { target:Target, #[serde(rename="trait")] trait_name:String, duration:String },
    LoseTrait { target:Target, #[serde(rename="trait")] trait_name:String, duration:String },
    DiscardOwnHand {amount:Expr,target:Target},
    DiscardDeck {amount:Expr,target:Target},
    CreateTokenUnit { #[serde(rename="tokenType")] token_type:String, amount:Expr, target:Target, #[serde(default,rename="entersReady")] enters_ready:bool },
    Damage {
        amount: Expr,
        target: Target,
    },
    Heal {
        amount: Expr,
        target: Target,
    },
    Draw {
        amount: Expr,
        target: Target,
    },
    Select {
        #[serde(rename = "targetResolver")]
        target_resolver: TargetResolver,
        effect: Box<Effect>,
    },
    ReadyResources {
        target: Target,
        amount: Expr,
    },
    NoAction,
    ResourceCard { target: Target },
    ReturnToHand {
        target: Target,
    },
    ModifyStats {
        target: Target,
        power: Expr,
        hp: Expr,
        duration: String,
    },
    MaximumPowerExperience {aspect:String},
    CopyKeywords {target:Target,duration:String,domain:Vec<String>},
    GainKeyword { target: Target, keyword: String, #[serde(default = "one")] amount: Expr, duration: String },
    LoseKeyword { target: Target, keyword: String, duration: String },
    Ready {
        target: Target,
    },
    Exhaust {
        target: Target,
    },
    Defeat {
        target: Target,
    },
    GiveShield {
        #[serde(default = "one")]
        amount: Expr,
        target: Target,
    },
    UseForce {target:Target},
    GainForce {target:Target},
    GiveWeakness { amount:Expr, target:Target },
    GiveAdvantage { amount:Expr, target:Target, #[serde(rename="createdBy")] created_by:crate::advantage::Creator, batch:crate::advantage::Batch, #[serde(rename="tokenOwner")] token_owner:crate::advantage::Owner },
    GiveExperience {
        #[serde(default = "one")]
        amount: Expr,
        target: Target,
    },
    Sequence {
        effects: Vec<Effect>,
    },
    Simultaneous {
        effects: Vec<Effect>,
    },
    PhaseModifiers {
        effects: Vec<Effect>,
    },
    Conditional {
        condition: Expr,
        #[serde(rename = "onTrue")]
        on_true: Box<Effect>,
        #[serde(rename = "onFalse", default)]
        on_false: Option<Box<Effect>>,
    },
    DistributeHealing {
        amount: Expr,
        #[serde(rename = "maxTargets")]
        max_targets: u16,
        #[serde(rename = "canChooseNoTargets", default)]
        can_choose_no_targets: bool,
        #[serde(rename = "canDistributeLess", default)]
        can_distribute_less: bool,
        #[serde(rename = "targetResolver")]
        target_resolver: TargetResolver,
    },
    /// Compiler `selectedPlayerSystem`; lowered to draw/ready/discard at program compile.
    SelectedPlayerSystem {
        system: String,
        target: serde_json::Value,
        #[serde(default)]
        properties: crate::selected_player::SelectedPlayerProperties,
        defaults: String,
        #[serde(rename = "defaultTargetOverride")]
        default_target_override: serde_json::Value,
    },
    /// View-only opponent-hand lookAt with explicit display acknowledgment.
    LookAt {
        target: Expr,
        player: Expr,
        #[serde(rename = "interactMode")]
        interact_mode: String,
        #[serde(rename = "useDisplayPrompt", default)]
        use_display_prompt: Option<bool>,
    },
    /// Look at opponent hand and optionally discard one filtered card from it.
    LookAtAndSelectCard {
        target: Expr,
        player: Expr,
        #[serde(rename = "useDisplayPrompt")]
        use_display_prompt: bool,
        #[serde(rename = "maxCards")]
        max_cards: Expr,
        #[serde(rename = "canChooseFewer")]
        can_choose_fewer: Expr,
        #[serde(rename = "cardCondition")]
        card_condition: Value,
        #[serde(rename = "selectedContextField")]
        selected_context_field: String,
        #[serde(rename = "immediateEffect")]
        immediate_effect: ViewSelectImmediate,
    },
}

/// Immediate effect nested under reviewed `lookAtAndSelectCard`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum ViewSelectImmediate {
    DiscardSpecificCard { target: String },
    /// Remnant Lookouts / Bazine: discard the selected hand card, then opponent draws 1.
    Sequence { effects: Vec<ViewSelectStep> },
}

/// Steps allowed inside a reviewed `lookAtAndSelectCard` sequence.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum ViewSelectStep {
    DiscardSpecificCard { target: String },
    Draw {
        target: String,
        amount: Expr,
    },
}
impl Target {
    fn visit_expression_roots_mut(&mut self, visit: &mut impl FnMut(&mut Expr)) {
        if let Self::Expression(expression) = self { visit(expression); }
    }
}
impl TargetResolver {
    fn visit_expression_roots_mut(&mut self, visit: &mut impl FnMut(&mut Expr)) {
        if let Some(predicate) = &mut self.predicate { visit(predicate); }
        if let Some(count) = &mut self.count { visit(count); }
    }
}
impl Effect {
    fn visit_expression_roots_mut(&mut self, visit: &mut impl FnMut(&mut Expr)) {
        match self {
            Self::DiscardOwnHand { amount, target } | Self::DiscardDeck { amount, target }
            | Self::CreateTokenUnit { amount, target, .. } | Self::Damage { amount, target }
            | Self::Heal { amount, target } | Self::Draw { amount, target }
            | Self::ReadyResources { amount, target } | Self::GiveShield { amount, target }
            | Self::GiveExperience { amount, target } | Self::GainKeyword { amount, target, .. } => {
                visit(amount); target.visit_expression_roots_mut(visit);
            }
            Self::GainTrait{target,..}|Self::LoseTrait{target,..}|Self::CopyKeywords{target,..}|Self::ReturnToHand { target } | Self::LoseKeyword { target, .. }
            | Self::Ready { target } | Self::Exhaust { target } | Self::Defeat { target } => {
                target.visit_expression_roots_mut(visit);
            }
            Self::ModifyStats { target, power, hp, .. } => {
                target.visit_expression_roots_mut(visit); visit(power); visit(hp);
            }
            Self::Select { target_resolver, effect } => {
                target_resolver.visit_expression_roots_mut(visit);
                effect.visit_expression_roots_mut(visit);
            }
            Self::Sequence { effects } | Self::Simultaneous { effects } | Self::PhaseModifiers { effects } => {
                for effect in effects { effect.visit_expression_roots_mut(visit); }
            }
            Self::Conditional { condition, on_true, on_false } => {
                visit(condition); on_true.visit_expression_roots_mut(visit);
                if let Some(effect) = on_false { effect.visit_expression_roots_mut(visit); }
            }
            Self::DistributeHealing { amount, target_resolver, .. } => {
                visit(amount); target_resolver.visit_expression_roots_mut(visit);
            }
            // Resource, Force and Weakness operations keep their expressions portable:
            // their runtime matches on the raw shapes.
            Self::InitiateAttack{..}|Self::MaximumPowerExperience{..}|Self::ResourceCard { .. } | Self::UseForce { .. } | Self::GainForce { .. }
            | Self::GiveWeakness { .. } | Self::GiveAdvantage { .. } | Self::ConstantEffectRegistration(_) | Self::NoAction
            | Self::SelectedPlayerSystem { .. } => {}
            Self::LookAt { target, player, .. } => {
                visit(target);
                visit(player);
            }
            Self::LookAtAndSelectCard {
                target,
                player,
                max_cards,
                can_choose_fewer,
                ..
            } => {
                visit(target);
                visit(player);
                visit(max_cards);
                visit(can_choose_fewer);
            }
        }
    }
}
/// Upgrade departure is scalar and has a single chosen-card event. Conditional
/// no-action branches retain feasibility; groups and other operations stay blocked.
fn scalar_upgrade_operation(effect:&Effect,discard:bool)->bool {
    fn walk(e:&Effect,discard:bool)->(bool,bool) {match e {
        Effect::ReturnToHand {target}=>(matches!(target,Target::Name(name) if name=="chosen"),true),
        Effect::Defeat {..}=>(!discard,true),
        Effect::NoAction=>(true,false),
        Effect::Conditional {on_true,on_false,..}=>{let a=walk(on_true,discard);let b=on_false.as_ref().map_or((true,false),|e|walk(e,discard));(a.0&&b.0,a.1||b.1)},
        _=>(false,false),
    }}
    let (valid,operation)=walk(effect,discard);valid&&operation
}
fn one() -> Expr {
    Expr::number(1)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Ability {
    /// Set only by the strict attached-registration loader, never accepted from raw IR.
    #[serde(skip)]
    pub attached_host: bool,

    #[serde(default, skip_serializing_if="Vec::is_empty")]
    pub hooks: Vec<crate::death_hooks::Hook>,
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub gained_host: Option<bool>,
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub attached_context: Option<String>,
    #[serde(default, skip_serializing_if="Vec::is_empty")]
    pub target_resolvers: Vec<NamedTargetResolver>,
    #[serde(default)]
    pub trigger: String,
    #[serde(default, skip_serializing_if="std::ops::Not::not")]
    pub attacker_must_survive: bool,
    #[serde(default)]
    pub coordinate: bool,
    pub title: String,
    #[serde(default)]
    pub context_title: Vec<Expr>,
    #[serde(default)]
    pub costs: Vec<Cost>,
    #[serde(default)]
    pub optional: bool,
    #[serde(default)]
    pub requires_confirmation: bool,
    #[serde(default)]
    pub target_resolver: Option<TargetResolver>,
    #[serde(default)]
    pub condition: Option<Expr>,
    pub effect: Effect,
    #[serde(default)]
    pub then: Option<Box<Ability>>,
    #[serde(default)]
    pub if_you_do: Option<Box<Ability>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct NamedTargetResolver {
    pub name:String,
    pub target_resolver:TargetResolver,
    pub chooser:String,
    pub effect:Effect,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sides {
    #[serde(default)]
    pub leader: Vec<Ability>,
    #[serde(default)]
    pub unit: Vec<Ability>,
    #[serde(default)]
    pub card: Vec<Ability>,
}

fn contains_resource<T:Serialize>(v:&T)->bool {
    fn walk(v:&Value)->bool {v["kind"]=="resourceCard" || match v {Value::Array(a)=>a.iter().any(walk),Value::Object(o)=>o.values().any(walk),_=>false}}
    serde_json::to_value(v).map(|v|walk(&v)).unwrap_or(true)
}
fn top_deck_marker(v:&Value)->bool {v.get("target").is_some_and(|t|t.as_str()==Some("ownTopDeck"))||match v{Value::Object(o)=>o.values().any(top_deck_marker),Value::Array(a)=>a.iter().any(top_deck_marker),_=>false}}
fn top_deck_resource_registration(a:&Ability)->bool {
    fn reference(e:&Expr,p:&[&str])->bool {matches!(e,Expr::Ref{path} if path.iter().map(String::as_str).collect::<Vec<_>>()==p)}
    fn resource(e:&Effect)->bool {matches!(e,Effect::ResourceCard{target:Target::Name(n)} if n=="ownTopDeck")}
    fn condition(e:&Expr)->bool {
        if matches!(e,Expr::Const{value} if value.is_boolean()){return true;}
        let Expr::Binary{op,left,right}=e else{return false;};
        if !["<","<=",">",">=","===","!=="].contains(&op.as_str()){return false;}
        (reference(left,&["player","resources","length"])&&reference(right,&["opponent","resources","length"])) ||
        (matches!(left.as_ref(),Expr::Query{name,receiver,args} if name=="arenaUnitCount"&&reference(receiver,&["player"])&&args.len()==1&&matches!(&args[0],Expr::Const{value} if value.as_bool()==Some(true)))&&matches!(right.as_ref(),Expr::Const{value} if value.as_u64().is_some_and(|n|n<=64)))
    }
    a.trigger=="whenPlayed"&&a.optional&&!a.requires_confirmation&&!a.coordinate&&a.attached_context.is_none()&&a.costs.is_empty()&&a.context_title.is_empty()&&a.target_resolver.is_none()&&a.target_resolvers.is_empty()&&a.condition.is_none()&&a.then.is_none()&&a.if_you_do.is_none()&&
    (resource(&a.effect)||matches!(&a.effect,Effect::Conditional{condition:c,on_true,on_false} if condition(c)&&resource(on_true)&&on_false.as_deref().is_some_and(|e|matches!(e,Effect::NoAction))))
}
fn plain_resource_header(a:&Ability)->bool {!a.optional&&!a.requires_confirmation&&!a.coordinate&&a.attached_context.is_none()&&a.costs.is_empty()&&a.context_title.is_empty()&&a.target_resolvers.is_empty()&&a.condition.is_none()&&a.then.is_none()}
fn imperial_resource_continuation(a:&Ability)->bool {
    if a.trigger!="event"||!plain_resource_header(a)||!matches!(&a.effect,Effect::Defeat{target:Target::Name(n)} if n=="chosen"){return false;}
    let Some(r)=&a.target_resolver else{return false;};
    if r.validate_inline().is_err()||r.owner!="self"||r.card_type!="unit"||r.zone.as_deref()!=Some("anyAttackable")||r.arena.is_some()||r.optional||r.mode.is_some()||r.count.is_some()||r.prompt_title.is_none(){return false;}
    if !matches!(&r.predicate,Some(Expr::Query{name,receiver,args}) if name=="hasSomeTrait"&&matches!(receiver.as_ref(),Expr::Ref{path} if path==&["card"])&&args.len()==1&&matches!(&args[0],Expr::Const{value} if value.as_str()==Some("imperial"))){return false;}
    a.if_you_do.as_deref().is_some_and(|n|n.trigger.is_empty()&&plain_resource_header(n)&&n.target_resolver.is_none()&&n.if_you_do.is_none()&&matches!(&n.effect,Effect::ResourceCard{target:Target::Name(t)} if t=="ownTopDeck"))
}
fn sequential_draw_resource(a:&Ability)->bool {
    fn condition(e:&Expr,zone:&str)->bool {if matches!(e,Expr::Const{value} if value.is_boolean()){return true;}matches!(e,Expr::Binary{op,left,right} if ["<","<=",">",">=","===","!=="].contains(&op.as_str())&&matches!(left.as_ref(),Expr::Ref{path} if path==&["player",zone,"length"])&&matches!(right.as_ref(),Expr::Ref{path} if path==&["opponent",zone,"length"]))}
    if a.trigger!="whenPlayed"||!plain_resource_header(a)||a.target_resolver.is_some()||a.if_you_do.is_some(){return false;}
    let Effect::Sequence{effects}=&a.effect else{return false;};if effects.len()!=2{return false;}
    let first=matches!(&effects[0],Effect::Conditional{condition:c,on_true,on_false} if condition(c,"hand")&&matches!(on_true.as_ref(),Effect::Draw{target:Target::Name(t),amount:Expr::Const{value}} if t=="ownPlayer"&&value.as_u64().is_some_and(|n|(1..=2).contains(&n)))&&on_false.as_deref().is_some_and(|e|matches!(e,Effect::NoAction)));
    let second=matches!(&effects[1],Effect::Conditional{condition:c,on_true,on_false} if condition(c,"resources")&&matches!(on_true.as_ref(),Effect::ResourceCard{target:Target::Name(t)} if t=="ownTopDeck")&&on_false.as_deref().is_some_and(|e|matches!(e,Effect::NoAction)));
    first&&second
}
pub(crate) fn resource_pair_effect(e:&Effect)->bool {
    matches!(e,Effect::Simultaneous{effects} if effects.len()==2&&matches!(&effects[0],Effect::ResourceCard{target:Target::Name(n)} if n=="self")&&matches!(&effects[1],Effect::ResourceCard{target:Target::Name(n)} if n=="ownTopDeck"))
}
fn simultaneous_resource_registration(a:&Ability)->bool {
    a.trigger=="event"&&plain_resource_header(a)&&a.target_resolver.is_none()&&a.if_you_do.is_none()&&resource_pair_effect(&a.effect)
}
fn reviewed_top_deck_registration(a:&Ability)->bool {top_deck_resource_registration(a)||imperial_resource_continuation(a)||sequential_draw_resource(a)||simultaneous_resource_registration(a)}
fn parent_defense_marker(value:&Value)->bool {
    (value["kind"]=="query"&&value["name"]=="isDefending"&&value["receiver"]["path"].as_array().is_some_and(|p|p.iter().any(|v|v=="parentUnit"||v=="parentCard")))||match value {Value::Array(a)=>a.iter().any(parent_defense_marker),Value::Object(o)=>o.values().any(parent_defense_marker),_=>false}
}
fn parent_defense_registration(a:&Ability)->bool {
    fn query(e:&Expr,name:&str,path:&[&str])->bool {matches!(e.original(),Expr::Query{name:n,receiver,args} if n==name&&args.is_empty()&&matches!(receiver.original(),Expr::Ref{path:p} if p.iter().map(String::as_str).eq(path.iter().copied())))}
    if a.trigger!="constant"||a.optional||a.requires_confirmation||a.coordinate||a.attached_context.is_some()||!a.target_resolvers.is_empty()||!a.context_title.is_empty()||!a.costs.is_empty()||a.then.is_some()||a.if_you_do.is_some()||!a.condition.as_ref().is_some_and(|e|query(e,"isDefending",&["source","parentUnit"])) {return false;}
    if !matches!(&a.effect,Effect::ModifyStats{target:Target::Name(n),power:Expr::Const{value:p},hp:Expr::Const{value:h},duration} if n=="chosen"&&duration=="persistent"&&p.as_i64()==Some(-1)&&h.as_i64()==Some(0)) {return false;}
    let Some(r)=&a.target_resolver else{return false;};
    if r.owner!="opponent"||r.card_type!="unit"||r.zone.as_deref()!=Some("arena")||r.arena.is_some()||r.optional||r.mode.is_some()||r.count.is_some()||r.prompt_title.is_some(){return false;}
    matches!(r.predicate.as_ref().map(Expr::original),Some(Expr::Binary{op,left,right}) if op=="&&"&&query(right,"isAttacking",&["card"])&&matches!(left.original(),Expr::Binary{op,left,right} if op=="&&"&&query(left,"isUnit",&["card"])&&query(right,"isInPlay",&["card"])))
}
pub(crate) fn parent_defense_upgrade_entry(entry:&Value)->bool {
    let Ok(sides)=serde_json::from_value::<Sides>(entry["abilities"].clone()) else{return false;};
    entry["baseClass"]=="UpgradeCard"&&entry["card"]["types"]==serde_json::json!(["upgrade"])&&entry["card"]["unique"]==false&&sides.card.len()==1&&sides.unit.is_empty()&&sides.leader.is_empty()&&parent_defense_registration(&sides.card[0])&&
    entry["keywordValues"]==serde_json::json!({})&&entry["card"]["keywords"]==serde_json::json!([])&&entry["piloting"].is_null()&&["pilotText","pilotBox","pilotingBox","deployBox","deployCost"].iter().all(|k|entry["card"][*k].is_null())&&(entry["card"]["epicAction"].is_null()||entry["card"]["epicAction"]=="")&&
    [("upgradePower","power"),("upgradeHp","hp")].iter().all(|(derived,printed)|entry["card"][*derived].as_i64().is_some_and(|n|(-1000..=1000).contains(&n))&&entry["card"][*derived]==entry["card"][*printed])&&
    entry["attachCondition"]==serde_json::json!({"kind":"unary","op":"!","value":{"kind":"query","name":"hasSomeTrait","receiver":{"kind":"ref","path":["card"]},"args":[{"kind":"const","value":"vehicle"}]}})
}
fn maximum_registration(a:&Ability)->bool {
    let effect=matches!(&a.effect,Effect::MaximumPowerExperience{aspect} if ["vigilance","command","aggression","cunning","heroism","villainy"].contains(&aspect.as_str()));
    let costs=if a.trigger=="action"{a.costs.len()==2&&matches!(&a.costs[0],Cost::Resources{amount:Expr::Const{value}} if value.as_u64()==Some(1))&&matches!(&a.costs[1],Cost::ExhaustSelf)}else{a.trigger=="onAttack"&&a.costs.is_empty()};
    effect&&costs&&!a.optional&&!a.coordinate&&!a.requires_confirmation&&a.context_title.is_empty()&&a.attached_context.is_none()&&a.condition.is_none()&&a.target_resolver.is_none()&&a.target_resolvers.is_empty()&&a.then.is_none()&&a.if_you_do.is_none()
}
fn resource_self(e:&Effect)->bool {matches!(e,Effect::ResourceCard{target:Target::Name(n)} if n=="self")}
#[derive(Clone,Copy,PartialEq,Eq)]
enum ForceContinuation { Drain, Wave, Shields }
fn plain_success(a:&Ability)->bool {a.trigger.is_empty()&&!a.optional&&!a.requires_confirmation&&!a.coordinate&&a.context_title.is_empty()&&a.costs.is_empty()&&a.target_resolvers.is_empty()&&a.condition.is_none()&&a.then.is_none()&&a.if_you_do.is_none()}
fn force_continuation(a:&Ability)->Option<ForceContinuation> {
    if a.coordinate||a.requires_confirmation||!a.context_title.is_empty()||!a.costs.is_empty()||a.condition.is_some()||a.target_resolver.is_some()||!a.target_resolvers.is_empty()||a.then.is_some(){return None;}
    fn own(e:&Effect,gain:bool)->bool {match e {Effect::GainForce{target:Target::Name(n)} if gain=>n=="ownPlayer",Effect::UseForce{target:Target::Name(n)} if !gain=>n=="ownPlayer",_=>false}}
    fn unit(r:&TargetResolver)->bool {r.owner=="any"&&r.card_type=="unit"&&r.zone.as_deref()==Some("anyAttackable")&&r.arena.is_none()&&r.predicate.is_none()}
    if a.trigger=="event"&&!a.optional&&a.if_you_do.is_none() {if let Effect::Simultaneous{effects}=&a.effect {if effects.len()==2&&own(&effects[1],true) {if let Effect::Select{target_resolver:r,effect}=&effects[0] {if unit(r)&&!r.optional&&r.mode.is_none()&&r.count.is_none()&&r.prompt_title.as_ref().is_some_and(|t|!t.is_empty())&&matches!(effect.as_ref(),Effect::Damage{target:Target::Name(n),amount:Expr::Const{value}} if n=="chosen"&&value.as_u64()==Some(2)){return Some(ForceContinuation::Drain);}}}}}
    if a.trigger!="whenPlayed"||!a.optional||!own(&a.effect,false){return None;}
    let next=a.if_you_do.as_ref()?;if !plain_success(next){return None;}
    if let Some(r)=&next.target_resolver {if unit(r)&&r.optional&&r.mode.as_deref()==Some("upTo")&&matches!(&r.count,Some(Expr::Const{value}) if value.as_u64()==Some(5))&&r.prompt_title.is_none()&&matches!(&next.effect,Effect::GiveShield{target:Target::Name(n),amount:Expr::Const{value}} if n=="chosen"&&value.as_u64()==Some(1)){return Some(ForceContinuation::Shields);}}
    else if let Effect::Defeat{target:Target::ArenaList(list)}=&next.effect {if matches!(&list.receiver,Expr::Ref{path} if path==&["game".to_string()])&&force_wave_predicate(&list.predicate){return Some(ForceContinuation::Wave);}}
    None
}
fn force_wave_predicate(e:&Expr)->bool {
    fn part(e:&Expr)->Option<u8>{match e {
      Expr::Const{value} if value==&Value::Bool(true)=>Some(0),
      Expr::Query{name,receiver,args} if name=="isUnit"&&args.is_empty()&&matches!(receiver.as_ref(),Expr::Ref{path} if path==&["card".to_string()])=>Some(1),
      Expr::Unary{op,value} if op=="!"&&matches!(value.as_ref(),Expr::Query{name,receiver,args} if name=="hasSomeTrait"&&matches!(receiver.as_ref(),Expr::Ref{path} if path==&["card".to_string()])&&matches!(args.as_slice(),[Expr::Const{value:Value::String(s)}] if !s.is_empty()))=>Some(2),
      Expr::Binary{op,left,right} if op=="<="&&matches!(left.as_ref(),Expr::Ref{path} if path==&["card".to_string(),"remainingHp".to_string()])&&matches!(right.as_ref(),Expr::Const{value} if value.as_u64()==Some(3))=>Some(4),
      Expr::Binary{op,left,right} if op=="&&"=>Some(part(left)?|part(right)?),_=>None}}
    part(e)==Some(7)
}
fn contains_force(effect:&Effect)->bool {
    match effect {Effect::UseForce{..}|Effect::GainForce{..}=>true,Effect::Conditional{on_true,on_false,..}=>contains_force(on_true)||on_false.as_deref().is_some_and(contains_force),Effect::Select{effect,..}=>contains_force(effect),Effect::Simultaneous{effects}|Effect::Sequence{effects}|Effect::PhaseModifiers{effects}=>effects.iter().any(contains_force),_=>false}
}
fn contains_weakness(effect:&Effect)->bool {
    match effect {Effect::GiveWeakness{..}=>true,Effect::Conditional{on_true,on_false,..}=>contains_weakness(on_true)||on_false.as_deref().is_some_and(contains_weakness),Effect::Select{effect,..}=>contains_weakness(effect),Effect::Simultaneous{effects}|Effect::Sequence{effects}|Effect::PhaseModifiers{effects}=>effects.iter().any(contains_weakness),_=>false}
}
fn attachment_defeat_marker(e:&Effect)->bool {
    match e {
        Effect::Defeat{target:Target::Expression(e)}=>matches!(e.original(),Expr::Collection{..}),
        Effect::Simultaneous{effects}|Effect::Sequence{effects}|Effect::PhaseModifiers{effects}=>effects.iter().any(attachment_defeat_marker),
        Effect::Conditional{on_true,on_false,..}=>attachment_defeat_marker(on_true)||on_false.as_deref().is_some_and(attachment_defeat_marker),
        Effect::Select{effect,..}=>attachment_defeat_marker(effect),_=>false,
    }
}
fn attachment_defeat_registration(a:&Ability)->bool {
    if a.requires_confirmation||a.coordinate||a.attached_context.is_some()||!a.costs.is_empty()||!a.context_title.is_empty()||!a.target_resolvers.is_empty()||a.condition.is_some()||a.then.is_some()||a.if_you_do.is_some(){return false;}
    let Some(r)=&a.target_resolver else{return false;};
    if r.owner!="any"||r.card_type!="unit"||r.zone.as_deref()!=Some("anyAttackable")||r.arena.is_some()||r.optional||r.mode.is_some()||r.count.is_some()||r.predicate.is_some()||r.prompt_title.is_some()||!r.prompt_title_parts.is_empty(){return false;}
    let defeat=|e:&Effect,shield:bool|matches!(e,Effect::Defeat{target:Target::Expression(e)} if e.attachment_defeat_filter()==Some(shield));
    (a.trigger=="whenPlayed"&&a.optional&&defeat(&a.effect,false)) ||
    (a.trigger=="event"&&!a.optional&&matches!(&a.effect,Effect::Sequence{effects} if effects.len()==2&&defeat(&effects[0],true)&&matches!(&effects[1],Effect::Damage{target:Target::Name(n),amount:Expr::Const{value}} if n=="chosen"&&value.as_u64()==Some(3))))
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum NamedChooser { Own, Opponent }

#[derive(Debug, Clone, Serialize)]
pub enum Instruction {
    /// All then/sequence subwindows pass registrations to this initiating ability window.
    EndStep,
    Commit,
    Select(TargetResolver),
    CaptureMaximum {aspect:String},
    Conditional { condition: Expr, skip: usize },
    IfResult { skip: usize },
    Jump { to: usize },
    Apply(Effect),
    Group(Effect),
    /// Independent named selectors prepare before commitment and share one event window.
    NamedGroup(Effect),
    /// Independent mandatory selectors with separate prompt actors; original actor owns commitment.
    NamedChooserGroup { effect: Effect, choosers: Vec<NamedChooser> },
}
impl Instruction {
    pub fn group_effect(&self)->Option<&Effect> {match self {Self::Group(e)|Self::NamedGroup(e)|Self::NamedChooserGroup{effect:e,..}=>Some(e),_=>None}}
    pub fn is_named_group(&self)->bool {matches!(self,Self::NamedGroup(_)|Self::NamedChooserGroup{..})}
}

#[derive(Debug, Clone, Serialize)]
pub struct Program {
    #[serde(default, skip_serializing_if="std::ops::Not::not")]
    pub attached_host: bool,

    #[serde(default, skip_serializing_if="Vec::is_empty")]
    pub hooks: Vec<crate::death_hooks::Hook>,
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub gained_host: Option<bool>,
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub attached_context: Option<String>,
    pub title: String,
    pub context_title: Vec<Expr>,
    pub trigger: String,
    #[serde(default, skip_serializing_if="std::ops::Not::not")]
    pub attacker_must_survive: bool,
    pub coordinate: bool,
    pub costs: Vec<Cost>,
    pub optional: bool,
    pub requires_confirmation: bool,
    pub condition: Option<Expr>,
    pub instructions: Vec<Instruction>,
    pub event_end: u8,
    #[serde(skip)]
    pub executable: Result<usize, &'static str>,
    /// (pc, end) of each selection made inside an effect (selectCard): instructions pc+1..end are its card's own effect.
    /// Derived from `instructions`, so it stays out of the content hash.
    #[serde(skip)]
    pub select_ends: Vec<(usize, usize)>,
}
pub(crate) fn gained_host_marker(v:&Value)->bool {match v {Value::Object(o)=>o.contains_key("gainedHost")||o.values().any(gained_host_marker),Value::Array(a)=>a.iter().any(gained_host_marker),_=>false}}
fn gained_host_templates()->Vec<Ability> {serde_json::from_str(r##"[{"trigger":"onAttack","title":"You may heal 3 damage from another unit.","costs":[],"optional":true,"targetResolver":{"owner":"any","type":"unit","zone":"anyAttackable","predicate":{"kind":"binary","op":"!==","left":{"kind":"ref","path":["card"]},"right":{"kind":"ref","path":["source"]}}},"effect":{"kind":"heal","target":"chosen","amount":{"kind":"const","value":3}},"gainedHost":true},{"trigger":"onAttack","title":"Give another unit +3/+0 for this phase","costs":[],"optional":true,"targetResolver":{"owner":"any","type":"any","zone":"anyAttackable","predicate":{"kind":"binary","op":"&&","left":{"kind":"binary","op":"!==","left":{"kind":"ref","path":["card"]},"right":{"kind":"ref","path":["source"]}},"right":{"kind":"query","name":"isUnit","receiver":{"kind":"ref","path":["card"]},"args":[]}}},"effect":{"kind":"modifyStats","target":"chosen","duration":"phase","power":{"kind":"const","value":3},"hp":{"kind":"const","value":0}},"gainedHost":true},{"trigger":"onAttack","title":"Discard a card from your hand","costs":[],"effect":{"kind":"discardOwnHand","amount":{"kind":"const","value":1},"target":"ownPlayer"},"gainedHost":true}]"##).expect("static reviewed gained-host templates")}
pub(crate) fn gained_host_kind(a:&Ability)->Option<usize> {
 let expected=gained_host_templates();
 let actual=serde_json::to_value(a).ok()?;expected.iter().position(|x|serde_json::to_value(x).ok().as_ref()==Some(&actual))
}
pub(crate) fn gained_host_upgrade_kind(entry:&Value)->Option<usize> {
 fn markers(v:&Value)->usize {match v {Value::Object(o)=>usize::from(o.contains_key("gainedHost"))+o.values().map(markers).sum::<usize>(),Value::Array(a)=>a.iter().map(markers).sum(),_=>0}}
 if markers(entry)!=1 {return None;}
 let sides:Sides=serde_json::from_value(entry["abilities"].clone()).ok()?;
 if entry["baseClass"]!="UpgradeCard"||entry["card"]["types"]!=serde_json::json!(["upgrade"])||sides.card.len()!=1||!sides.leader.is_empty()||!sides.unit.is_empty()||!entry["piloting"].is_null()||entry["keywordValues"]!=serde_json::json!({})||entry["card"]["keywords"]!=serde_json::json!([])||["pilotText","pilotBox","pilotingBox","deployBox","deployCost"].iter().any(|k|!entry["card"][k].is_null())||!(entry["card"]["epicAction"].is_null()||entry["card"]["epicAction"]=="")||![("upgradePower","power"),("upgradeHp","hp")].iter().all(|(derived,printed)|entry["card"][*derived].as_i64().is_some_and(|n|(-1000..=1000).contains(&n))&&entry["card"][*derived]==entry["card"][*printed]) {return None;}
 let kind=gained_host_kind(&sides.card[0])?;
 let trait_query=|name:&str|serde_json::json!({"kind":"query","name":"hasSomeTrait","receiver":{"kind":"ref","path":["card"]},"args":[{"kind":"const","value":name}]});
 let attach=match kind {0=>trait_query("force"),1=>serde_json::json!({"kind":"unary","op":"!","value":trait_query("vehicle")}),2=>Value::Null,_=>return None};
 (entry["attachCondition"]==attach&&entry["card"]["unique"]==serde_json::json!(kind==1)).then_some(kind)
}

pub(crate) fn basezone_source_kind(entry:&Value)->Option<bool> {
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
fn trait_constant_registration(a:&Ability)->bool {
    a.trigger=="constant"&&!a.optional&&!a.requires_confirmation&&!a.coordinate&&a.context_title.is_empty()&&a.attached_context.is_none()&&a.costs.is_empty()&&a.target_resolvers.is_empty()&&a.then.is_none()&&a.if_you_do.is_none()&&
    matches!(&a.condition,Some(Expr::Query{name,..}) if name=="hasControlledInPlayTrait")&&
    a.target_resolver.as_ref().is_some_and(|r|r.owner=="self"&&r.card_type=="unit"&&r.zone.as_deref()==Some("arena")&&!r.optional&&r.arena.is_none()&&r.mode.is_none()&&r.count.is_none()&&r.prompt_title.is_none()&&r.prompt_title_parts.is_empty()&&r.predicate.as_ref().is_some_and(|p|serde_json::to_value(p).unwrap()==serde_json::json!({"kind":"binary","op":"===","left":{"kind":"ref","path":["card"]},"right":{"kind":"ref","path":["source"]}})))&&
    matches!(&a.effect,Effect::GainKeyword{target:Target::Name(n),keyword,amount:Expr::Const{value},duration} if n=="chosen"&&keyword=="sentinel"&&value.as_u64()==Some(1)&&duration=="persistent")
}
// Reviewed conditional title consumers reuse existing scalar event primitives.
fn title_registration(a:&Ability)->bool {
    fn test(e:&Expr,depth:u8)->bool {
        match e {
            Expr::Query{name,receiver,args} => name=="hasControlledInPlayTitle" && matches!(receiver.as_ref(),Expr::Ref{path} if path==&["player"]||path==&["opponent"]) && args.len()==1 && matches!(&args[0],Expr::Const{value} if value.as_str().is_some_and(|t|!t.is_empty()&&t.len()<=128)),
            Expr::Binary{op,left,right} if op=="||"&&depth==0 => test(left,1)&&test(right,1),
            _=>false,
        }
    }
    fn mentions(v:&Value)->bool {(v["kind"]=="query"&&v["name"]=="hasControlledInPlayTitle")||match v {Value::Array(a)=>a.iter().any(mentions),Value::Object(o)=>o.values().any(mentions),_=>false}}
    let mut remaining=serde_json::to_value(a).expect("ability serializes");
    if a.trigger=="constant" {remaining["condition"]=Value::Null;}else{remaining["effect"]["condition"]=Value::Null;}
    if mentions(&remaining) {return false;}
    let header=!a.requires_confirmation&&!a.coordinate&&a.context_title.is_empty()&&a.attached_context.is_none()&&a.costs.is_empty()&&a.target_resolvers.is_empty()&&a.then.is_none()&&a.if_you_do.is_none();
    if !header {return false;}
    if a.trigger=="constant" {
        return !a.optional&&a.target_resolver.is_none()&&matches!(&a.condition,Some(e) if test(e,1))&&matches!(&a.effect,Effect::ModifyStats{target:Target::Name(n),duration,..}|Effect::GainKeyword{target:Target::Name(n),duration,..} if n=="self"&&duration=="persistent");
    }
    if a.condition.is_some() {return false;}
    let Effect::Conditional{condition,on_true,on_false}=&a.effect else{return false;};
    if !test(condition,0)||!on_false.as_deref().is_some_and(|e|matches!(e,Effect::NoAction)){return false;}
    let unit=|r:&TargetResolver| r.owner=="any"&&r.card_type=="unit"&&r.zone.as_deref()==Some("anyAttackable")&&r.arena.is_none()&&!r.optional&&r.predicate.is_none()&&r.mode.is_none()&&r.count.is_none()&&r.prompt_title.is_none()&&r.prompt_title_parts.is_empty();
    match (a.trigger.as_str(),on_true.as_ref()) {
        ("whenPlayed",Effect::Ready{target:Target::Name(n)})=>!a.optional&&a.target_resolver.is_none()&&n=="self",
        ("onAttack",Effect::Damage{target:Target::Name(n),amount:Expr::Const{value}})=>a.optional&&a.target_resolver.as_ref().is_some_and(unit)&&n=="chosen"&&value.as_u64()==Some(1),
        ("whenDefeated",Effect::Select{target_resolver,effect})=>a.optional&&a.target_resolver.is_none()&&unit(target_resolver)&&matches!(effect.as_ref(),Effect::GiveExperience{target:Target::Name(n),amount:Expr::Const{value}} if n=="chosen"&&value.as_u64()==Some(1)),
        _=>false,
    }
}
fn contains_arena_tokens(a:&Ability)->bool {fn walk(e:&Effect)->bool {match e {Effect::GiveShield{target:Target::ArenaList(_),..}|Effect::GiveExperience{target:Target::ArenaList(_),..}=>true,Effect::Conditional{on_true,on_false,..}=>walk(on_true)||on_false.as_deref().is_some_and(walk),Effect::Simultaneous{effects}|Effect::Sequence{effects}|Effect::PhaseModifiers{effects}=>effects.iter().any(walk),Effect::Select{effect,..}=>walk(effect),_=>false}}walk(&a.effect)||a.then.as_deref().is_some_and(contains_arena_tokens)||a.if_you_do.as_deref().is_some_and(contains_arena_tokens)||a.target_resolvers.iter().any(|d|walk(&d.effect))}
fn arena_token_kind(a:&Ability)->Option<u8> {
 if !["whenPlayed","whenDefeated"].contains(&a.trigger.as_str())||a.optional||!a.costs.is_empty()||a.condition.is_some()||a.target_resolver.is_some()||!a.target_resolvers.is_empty()||a.then.is_some()||a.if_you_do.is_some()||a.coordinate||!a.context_title.is_empty(){return None;}
 let (list,shield)=match &a.effect {Effect::GiveShield{target:Target::ArenaList(l),amount:Expr::Const{value}} if value.as_u64()==Some(1)=>(l,true),Effect::GiveExperience{target:Target::ArenaList(l),amount:Expr::Const{value}} if value.as_u64()==Some(1)=>(l,false),_=>return None};
 if list.controller_order.is_some()||!matches!(&list.receiver,Expr::Ref{path} if path==&["player"])||list.predicate.validate().is_err(){return None;}
 // Exact friendly ground unit snapshot excluding physical native Shields.
 let ground=serde_json::json!({"kind":"binary","op":"&&","left":{"kind":"const","value":true},"right":{"kind":"binary","op":"||","left":{"kind":"unary","op":"!","value":{"kind":"const","value":"groundArena"}},"right":{"kind":"binary","op":"||","left":{"kind":"binary","op":"===","left":{"kind":"const","value":"groundArena"},"right":{"kind":"const","value":"anyArena"}},"right":{"kind":"binary","op":"===","left":{"kind":"ref","path":["card","zoneName"]},"right":{"kind":"const","value":"groundArena"}}}}});
 let unshielded=serde_json::json!({"kind":"binary","op":"&&","left":{"kind":"query","name":"isUnit","receiver":{"kind":"ref","path":["card"]},"args":[]},"right":{"kind":"unary","op":"!","value":{"kind":"query","name":"hasAttachedNativeToken","receiver":{"kind":"ref","path":["card"]},"args":[{"kind":"const","value":"shield"}]}}});
 if shield && a.trigger=="whenPlayed" && !a.requires_confirmation && a.attached_context.is_none() && serde_json::to_value(&list.predicate).ok()==Some(serde_json::json!({"kind":"binary","op":"&&","left":ground,"right":unshielded})) {return Some(4);}
 let Expr::Binary{op,left,right}=&list.predicate else{return None};if op!="&&"||!matches!(left.as_ref(),Expr::Const{value} if value.as_bool()==Some(true)){return None;}
 let reference=|e:&Expr,name:&str|matches!(e,Expr::Ref{path} if path==&[name]);
 if shield {if a.trigger=="whenPlayed"&&matches!(right.as_ref(),Expr::Query{name,receiver,args} if name=="hasSomeKeyword"&&reference(receiver,"card")&&args.len()==1&&matches!(&args[0],Expr::Const{value} if value.as_str()==Some("shielded"))){return Some(0);}return None;}
 if matches!(right.as_ref(),Expr::Binary{op,left,right} if op=="||"&&matches!(left.as_ref(),Expr::Unary{op,value} if op=="!"&&reference(value,"source"))&&matches!(right.as_ref(),Expr::Binary{op,left,right} if op=="!=="&&reference(left,"card")&&reference(right,"source"))){return Some(1);}
 if a.trigger=="whenDefeated"&&matches!(right.as_ref(),Expr::Query{name,receiver,args} if name=="hasSomeTrait"&&reference(receiver,"card")&&args.len()==1&&matches!(&args[0],Expr::Const{value} if value.as_str()==Some("rebel"))){return Some(2);}
 if a.trigger=="whenPlayed"&&matches!(right.as_ref(),Expr::Binary{op,left,right} if op==">"&&matches!(left.as_ref(),Expr::Ref{path} if path==&["card","damage"])&&matches!(right.as_ref(),Expr::Const{value} if value.as_u64()==Some(0))){return Some(3);}None
}
fn contains_arena_damage(ability:&Ability)->bool {
    fn walk(v:&Value)->bool {match v {Value::Object(o)=>o.get("kind").and_then(Value::as_str)==Some("damage")&&o.get("target").is_some_and(|t|t["kind"]=="arenaList")||o.values().any(walk),Value::Array(a)=>a.iter().any(walk),_=>false}}
    walk(&serde_json::to_value(ability).unwrap())
}
fn ordered_ground_registration(a:&Ability)->bool {
    if a.optional||a.requires_confirmation||a.coordinate||a.attached_context.is_some()||!a.context_title.is_empty()||!a.costs.is_empty()||a.target_resolver.is_some()||!a.target_resolvers.is_empty()||a.condition.is_some()||a.then.is_some()||a.if_you_do.is_some(){return false;}
    match &a.effect {
        Effect::Damage{target:Target::ArenaList(l),amount:Expr::Const{value}}=>["event","whenDefeated"].contains(&a.trigger.as_str())&&l.validate_ordered_ground().is_ok()&&value.as_u64().is_some_and(|n|(1..=3).contains(&n)),
        Effect::Exhaust{target:Target::ArenaList(l)}=>a.trigger=="whenPlayed"&&l.validate_ordered_ground().is_ok(),_=>false
    }
}
fn arena_damage_header(a:&Ability)->bool {
    ["whenPlayed","onAttack","whenDefeated"].contains(&a.trigger.as_str())&&!a.optional&&!a.requires_confirmation&&!a.coordinate&&a.context_title.is_empty()&&a.costs.is_empty()&&a.target_resolver.is_none()&&a.target_resolvers.is_empty()&&a.condition.is_none()&&a.then.is_none()&&a.if_you_do.is_none()
}
fn conditional_arena_damage_ability(a:&Ability)->bool {
    if !arena_damage_header(a){return false;}
    let Effect::Conditional{condition,on_true,on_false}=&a.effect else{return false;};
    if !on_false.as_deref().is_some_and(|e|matches!(e,Effect::NoAction)){return false;}
    let Effect::Damage{target:Target::ArenaList(list),amount:Expr::Const{value}}=on_true.as_ref() else{return false;};
    if list.controller_order.is_some()||!matches!(&list.receiver,Expr::Ref{path} if path==&["opponent"]){return false;}
    let Expr::Binary{op,left,right}=condition else{return false;};if op!=">="{return false;}
    if a.trigger=="whenPlayed"&&matches!(right.as_ref(),Expr::Const{value} if value.as_u64()==Some(4))&&value.as_u64()==Some(3)&&matches!(&list.predicate,Expr::Const{value} if value.as_bool()==Some(true))&&matches!(left.as_ref(),Expr::Query{name,receiver,args} if name=="arenaUnitCount"&&matches!(receiver.as_ref(),Expr::Ref{path} if path==&["player"])&&args.len()==1&&matches!(&args[0],Expr::Const{value} if value.as_bool()==Some(true))) {return true;}
    let ground=serde_json::json!({"kind":"binary","op":"&&","left":{"kind":"const","value":true},"right":{"kind":"binary","op":"||","left":{"kind":"unary","op":"!","value":{"kind":"const","value":"groundArena"}},"right":{"kind":"binary","op":"||","left":{"kind":"binary","op":"===","left":{"kind":"const","value":"groundArena"},"right":{"kind":"const","value":"anyArena"}},"right":{"kind":"binary","op":"===","left":{"kind":"ref","path":["card","zoneName"]},"right":{"kind":"const","value":"groundArena"}}}}});
    a.trigger=="onAttack"&&matches!(right.as_ref(),Expr::Const{value} if value.as_u64()==Some(15))&&value.as_u64()==Some(1)&&matches!(left.as_ref(),Expr::Ref{path} if path==&["player","base","damage"])&&serde_json::to_value(&list.predicate).ok()==Some(ground)
}
fn arena_damage_ability(a:&Ability)->bool {
    arena_damage_header(a)&&(matches!(&a.effect,Effect::Damage{target:Target::ArenaList(_),amount:Expr::Const{value}} if value.as_u64().is_some_and(|n|(1..=3).contains(&n)))||conditional_arena_damage_ability(a))
}
fn contains_new_arena_keywords(a:&Ability)->bool {
    fn walk(v:&Value)->bool {(matches!(v["kind"].as_str(),Some("phaseModifiers"|"simultaneous"))&&v["effects"].as_array().is_some_and(|items|items.iter().any(|e|matches!(e["kind"].as_str(),Some("gainKeyword"|"loseKeyword"))&&e["target"]["kind"]=="arenaList"))) || (v["trigger"]=="onAttack"&&v["effect"]["kind"]=="gainKeyword"&&v["effect"]["target"]["kind"]=="arenaList") || match v {Value::Array(a)=>a.iter().any(walk),Value::Object(o)=>o.values().any(walk),_=>false}}
    walk(&serde_json::to_value(a).expect("Ability serializes"))
}
fn validate_keyword_list(list:&ArenaListTarget)->Result<(),String> {
    if list.controller_order.is_some() {return Err("ordered keyword list unsupported".into());}
    fn reference(e:&Expr,n:&str)->bool {matches!(e,Expr::Ref{path} if path==&[n])}
    fn predicate(e:&Expr)->bool {match e {
        Expr::Const{value}=>value.as_bool()==Some(true),
        Expr::Binary{op,left,right} if op=="&&"=>predicate(left)&&predicate(right),
        Expr::Query{name,receiver,args}=>name=="hasSomeTrait"&&reference(receiver,"card")&&args.len()==1&&matches!(&args[0],Expr::Const{value} if value.as_str().is_some_and(|s|!s.is_empty())),
        Expr::Binary{op,left,right} if op=="||"=>matches!(left.as_ref(),Expr::Unary{op,value} if op=="!"&&reference(value,"source"))&&matches!(right.as_ref(),Expr::Binary{op,left,right} if op=="!=="&&reference(left,"card")&&reference(right,"source")),
        _=>false}}
    if !matches!(&list.receiver,Expr::Ref{path} if path.len()==1&&["game","player","opponent"].contains(&path[0].as_str())) || !predicate(&list.predicate) {return Err("arena keyword list requires static receiver/trait/source exclusion".into());}
    list.predicate.validate()
}
fn validate_arena_keyword_group(effect:&Effect)->Result<(),String> {
    fn leaf(e:&Effect,gain_only:bool)->Result<(),String> {
        match e {
            Effect::GainKeyword{target:Target::ArenaList(list),keyword,amount:Expr::Const{value},duration} if duration=="phase"=>{validate_keyword(keyword)?;validate_keyword_list(list)?;if !value.as_u64().is_some_and(|n|if ["raid","restore"].contains(&keyword.as_str()) {(1..=65535).contains(&n)}else {n==1}) {return Err("arena keyword gain requires positive bounded literal".into());}Ok(())},
            Effect::LoseKeyword{target:Target::ArenaList(list),keyword,duration} if !gain_only&&duration=="phase"=>{validate_keyword(keyword)?;validate_keyword_list(list)},
            _=>Err("arena keyword group supports only literal phase keyword factories".into())}
    }
    match effect {
        Effect::PhaseModifiers{effects} if effects.len()==2=>{
            for e in effects {leaf(e,false)?;}
            let target=|e:&Effect|match e {Effect::GainKeyword{target,..}|Effect::LoseKeyword{target,..}=>serde_json::to_value(target).unwrap(),_=>unreachable!()};
            if target(&effects[0])!=target(&effects[1]) {return Err("one phase event requires identical target snapshots for all factories".into());}Ok(())
        },
        Effect::Simultaneous{effects} if effects.len()==2=>{for e in effects {leaf(e,true)?;}Ok(())},
        _=>Err("arena keyword group requires exactly two independent factories".into())}
}
fn keyword_trait_and_exclusion(e:&Expr)->(bool,bool) {
    match e {
        Expr::Binary{op,left,right} if op=="&&"=>{let a=keyword_trait_and_exclusion(left);let b=keyword_trait_and_exclusion(right);(a.0||b.0,a.1||b.1)},
        Expr::Query{name,..} if name=="hasSomeTrait"=>(true,false),
        Expr::Binary{op,..} if op=="||"=>(false,true),
        _=>(false,false),
    }
}
fn arena_keyword_registration(a:&Ability)->bool {
    let header=!a.optional&&!a.requires_confirmation&&!a.coordinate&&a.context_title.is_empty()&&a.costs.is_empty()&&a.target_resolver.is_none()&&a.target_resolvers.is_empty()&&a.condition.is_none()&&a.then.is_none()&&a.if_you_do.is_none();
    header&&((a.trigger=="event"&&validate_arena_keyword_group(&a.effect).is_ok())||(a.trigger=="onAttack"&&matches!(&a.effect,Effect::GainKeyword{target:Target::ArenaList(list),keyword,amount:Expr::Const{value},duration} if keyword=="restore"&&value.as_u64()==Some(1)&&duration=="phase"&&validate_keyword_list(list).is_ok()&&matches!(&list.receiver,Expr::Ref{path} if path==&["player"])&&keyword_trait_and_exclusion(&list.predicate)==(true,true))))
}
#[derive(Clone,Copy,PartialEq,Eq)]
enum WeaknessExtension { Torrent, Picador, Mixed, Sequence, LeaderHeal }
// A scalar action and optional attack trigger reuse the existing Weakness
// lifecycle. Leader side topology is checked against the complete card below.
fn scalar_weakness_registration(a:&Ability)->bool {
    if a.requires_confirmation || a.coordinate || !a.context_title.is_empty() || a.attached_context.is_some() || a.gained_host.is_some()
        || a.condition.is_some() || a.then.is_some() || a.if_you_do.is_some() || !a.target_resolvers.is_empty() || !selected_weakness(&a.effect) {return false;}
    let Some(r)=&a.target_resolver else{return false;};
    if r.owner!="any" || !matches!(r.zone.as_deref(),Some("arena"|"anyAttackable")) || r.arena.is_some() || r.optional || r.mode.is_some() || r.count.is_some() || r.prompt_title.is_some() {return false;}
    match a.trigger.as_str() {
        "onAttack"=>a.optional && a.costs.is_empty() && r.card_type=="unit" && r.predicate.is_none(),
        "action"=>!a.optional && matches!(a.costs.as_slice(),[Cost::Resources{amount:Expr::Const{value}},Cost::ExhaustSelf] if value.as_u64()==Some(1)) && r.card_type=="any"
            && r.predicate.as_ref().is_some_and(|p|matches!(p,Expr::Binary{op,left,right} if op=="&&"
                && matches!(left.as_ref(),Expr::Query{name,receiver,args} if name=="isUnit" && args.is_empty() && matches!(receiver.as_ref(),Expr::Ref{path} if path==&["card"]))
                && matches!(right.as_ref(),Expr::Unary{op,value} if op=="!" && matches!(value.as_ref(),Expr::NativeTokenPredicate{operation,receiver,domain,token_type,optional_receiver:false,optional_call:false} if operation=="hasWeakness" && domain=="physicalAttachments" && token_type=="weakness" && matches!(receiver.as_ref(),Expr::Ref{path} if path==&["card"]))))),
        _=>false,
    }
}
fn selected_weakness(e:&Effect)->bool {matches!(e,Effect::GiveWeakness{target:Target::Name(n),amount:Expr::Const{value}} if n=="chosen"&&value.as_u64()==Some(1))}
fn weakness_selector(r:&TargetResolver,owner:&str)->bool {r.owner==owner&&r.card_type=="unit"&&r.zone.as_deref()==Some("anyAttackable")&&r.arena.is_none()&&!r.optional&&r.mode.is_none()&&r.count.is_none()&&r.predicate.is_none()&&r.prompt_title.is_none()}
fn own_base_trait(e:&Expr)->bool {matches!(e,Expr::Query{name,receiver,args} if name=="hasSomeTrait"&&matches!(receiver.as_ref(),Expr::Ref{path} if path==&["player","base"])&&args.len()==1&&matches!(&args[0],Expr::Const{value} if value.as_str().is_some_and(|n|!n.is_empty())))}
fn torrent_amount(e:&Expr)->bool {matches!(e,Expr::Conditional{condition,then,r#else} if own_base_trait(condition)&&matches!(then.as_ref(),Expr::Const{value} if value.as_u64()==Some(2))&&matches!(r#else.as_ref(),Expr::Const{value} if value.as_u64()==Some(1)))}
fn torrent_title(a:&Ability)->bool {
    let Effect::GiveWeakness{amount:Expr::Conditional{condition,..},..}=&a.effect else{return false;};
    a.context_title.len()==5&&matches!(&a.context_title[0],Expr::Const{value} if value.as_str()==Some("Give "))&&matches!(&a.context_title[1],Expr::Conditional{condition:c,then,r#else} if serde_json::to_value(c).unwrap()==serde_json::to_value(condition).unwrap()&&matches!(then.as_ref(),Expr::Const{value} if value.as_str()==Some("2 Weakness tokens"))&&matches!(r#else.as_ref(),Expr::Const{value} if value.as_str()==Some("a Weakness token")))&&matches!(&a.context_title[2],Expr::Const{value} if value.as_str()==Some(" to "))&&matches!(&a.context_title[3],Expr::Conditional{condition,then,r#else} if matches!(condition.as_ref(),Expr::Ref{path} if path==&["target"])&&matches!(then.as_ref(),Expr::Ref{path} if path==&["target","title"])&&matches!(r#else.as_ref(),Expr::Const{value} if value.as_str()==Some("a unit")))&&matches!(&a.context_title[4],Expr::Const{value} if value.as_str()==Some(""))
}
fn leader_heal_weakness_registration(a:&Ability)->bool {
    if a.trigger!="action" || a.optional || a.requires_confirmation || a.coordinate || !a.context_title.is_empty() || a.gained_host.is_some() || a.attached_context.is_some()
        || !matches!(a.costs.as_slice(),[Cost::ExhaustSelf]) || !a.target_resolvers.is_empty() || a.condition.is_some() || a.then.is_some() || a.if_you_do.is_some() {return false;}
    let Some(r)=&a.target_resolver else{return false;};
    if r.owner!="opponent" || r.card_type!="any" || r.zone.as_deref()!=Some("anyAttackable") || r.arena.is_some() || r.optional || r.mode.is_some() || r.count.is_some() || r.prompt_title.is_some() {return false;}
    let damaged=r.predicate.as_ref().is_some_and(|p|matches!(p,Expr::Binary{op,left,right} if op=="&&"
        && matches!(left.as_ref(),Expr::Query{name,receiver,args} if name=="isUnit"&&args.is_empty()&&matches!(receiver.as_ref(),Expr::Ref{path} if path==&["card"]))
        && matches!(right.as_ref(),Expr::Binary{op,left,right} if op=="!=="&&matches!(left.as_ref(),Expr::Ref{path} if path==&["card","damage"])&&matches!(right.as_ref(),Expr::Const{value} if value.as_u64()==Some(0)))));
    damaged && matches!(&a.effect,Effect::Simultaneous{effects} if effects.len()==2 && matches!(&effects[0],Effect::Heal{target:Target::Name(n),amount:Expr::Const{value}} if n=="chosen"&&value.as_u64()==Some(1)) && selected_weakness(&effects[1]))
}
fn token_upgrade_attack_registration(a:&Ability)->bool {
    let query=|name:&str|serde_json::json!({"kind":"query","name":name,"receiver":{"kind":"ref","path":["card"]},"args":[]});
    let predicate=serde_json::json!({"kind":"binary","op":"&&","left":{"kind":"binary","op":"&&","left":query("isUnit"),"right":query("isUpgraded")},"right":{"kind":"collection","name":"some","receiver":{"kind":"ref","path":["card","upgrades"]},"args":[query("isTokenUpgrade")],"limit":256}});
    a.trigger=="onAttack" && a.optional && !a.requires_confirmation && !a.coordinate && a.costs.is_empty() && a.context_title.is_empty() && a.gained_host.is_none() && a.attached_context.is_none()
        && a.target_resolvers.is_empty() && a.condition.is_none() && a.then.is_none() && a.if_you_do.is_none()
        && matches!(&a.effect,Effect::Damage{target:Target::Name(n),amount:Expr::Const{value}} if n=="chosen"&&value.as_u64()==Some(2))
        && a.target_resolver.as_ref().is_some_and(|r|r.owner=="any" && r.card_type=="unit" && r.zone.as_deref()==Some("anyAttackable") && r.arena.is_none() && !r.optional && r.mode.is_none() && r.count.is_none() && r.prompt_title.is_none() && r.predicate.as_ref().is_some_and(|p|serde_json::to_value(p).ok().as_ref()==Some(&predicate)))
}
fn weakness_extension(a:&Ability)->Option<WeaknessExtension> {
    if leader_heal_weakness_registration(a) {return Some(WeaknessExtension::LeaderHeal);}
    let header=!a.requires_confirmation&&!a.coordinate&&a.costs.is_empty()&&a.target_resolvers.is_empty()&&a.condition.is_none()&&a.if_you_do.is_none();
    if !header{return None;}
    if a.trigger=="event"&&!a.optional&&a.then.is_none()&&a.target_resolver.as_ref().is_some_and(|r|weakness_selector(r,"any"))&&matches!(&a.effect,Effect::GiveWeakness{target:Target::Name(n),amount} if n=="chosen"&&torrent_amount(amount))&&torrent_title(a){return Some(WeaknessExtension::Torrent);}
    if !a.context_title.is_empty(){return None;}
    if a.trigger=="whenPlayed"&&!a.optional&&a.target_resolver.is_none()&&matches!(&a.effect,Effect::CreateTokenUnit{token_type,target:Target::Name(n),amount:Expr::Const{value},enters_ready} if token_type=="beast"&&n=="ownPlayer"&&value.as_u64()==Some(1)&&!*enters_ready)&&a.then.as_ref().is_some_and(|next|next.trigger.is_empty()&&!next.optional&&!next.requires_confirmation&&!next.coordinate&&next.costs.is_empty()&&next.context_title.is_empty()&&next.target_resolvers.is_empty()&&next.condition.is_none()&&next.then.is_none()&&next.if_you_do.is_none()&&next.target_resolver.as_ref().is_some_and(|r|weakness_selector(r,"self"))&&selected_weakness(&next.effect)){return Some(WeaknessExtension::Picador);}
    if !a.optional||a.then.is_some()||!a.target_resolver.as_ref().is_some_and(|r|weakness_selector(r,"any")){return None;}
    if ["whenPlayed","whenDefeated"].contains(&a.trigger.as_str())&&matches!(&a.effect,Effect::Simultaneous{effects} if effects.len()==2&&matches!(&effects[0],Effect::Damage{target:Target::Name(n),amount:Expr::Const{value}} if n=="chosen"&&value.as_u64()==Some(1))&&selected_weakness(&effects[1])){return Some(WeaknessExtension::Mixed);}
    if a.trigger=="whenPlayed"&&matches!(&a.effect,Effect::Sequence{effects} if effects.len()==2&&selected_weakness(&effects[0])&&matches!(&effects[1],Effect::Conditional{condition:Expr::Ref{path},on_true,on_false} if path==&["target","unique"]&&matches!(on_true.as_ref(),Effect::Exhaust{target:Target::Name(n)} if n=="chosen")&&on_false.as_deref().is_some_and(|e|matches!(e,Effect::NoAction)))){return Some(WeaknessExtension::Sequence);}
    None
}
fn named_unit_base_damage(ability: &Ability) -> bool {
    ability.trigger == "onAttack" && ability.target_resolvers.len() == 2 &&
    ability.target_resolvers.iter().enumerate().all(|(index, descriptor)| {
        let r = &descriptor.target_resolver;
        let domain = if index == 0 {
            r.owner == "opponent" && r.card_type == "any" && r.zone.as_deref() == Some("arena") &&
                r.arena.as_deref() == Some("ground")
        } else {
            r.owner == "any" && r.card_type == "base" && r.zone.as_deref() == Some("anyAttackable") && r.arena.is_none()
        };
        domain && !r.optional && r.mode.is_none() && r.count.is_none() && r.predicate.is_none() &&
            r.prompt_title.as_ref().is_some_and(|title| !title.is_empty()) &&
            matches!(&descriptor.effect, Effect::Damage { target: Target::Name(name), amount: Expr::Const { value } }
                if name == "chosen" && value.as_u64() == Some(2))
    })
}

fn contains_keyword_inventory(v:&Value)->bool {
    (v["kind"]=="ref"&&v["path"]==serde_json::json!(["card","keywords","length"])) || match v {Value::Object(o)=>o.values().any(contains_keyword_inventory),Value::Array(a)=>a.iter().any(contains_keyword_inventory),_=>false}
}
fn keyword_inventory_registration(a:&Ability)->bool {
    if a.requires_confirmation||a.coordinate||!a.costs.is_empty()||!a.context_title.is_empty()||!a.target_resolvers.is_empty()||a.condition.is_some()||a.then.is_some()||a.if_you_do.is_some(){return false;}
    let Some(r)=&a.target_resolver else{return false;};
    if r.card_type!="unit"||r.optional||r.mode.is_some()||r.count.is_some()||r.arena.is_some()||r.prompt_title.is_some(){return false;}
    let length=serde_json::json!({"kind":"ref","path":["card","keywords","length"]});
    let more=|n:u64|serde_json::json!({"kind":"binary","op":">","left":length,"right":{"kind":"const","value":n}});
    let predicate=r.predicate.as_ref().and_then(|e|serde_json::to_value(e).ok());
    if a.trigger=="whenPlayed"&&a.optional&&r.owner=="any"&&r.zone.as_deref()==Some("anyAttackable")&&matches!(&a.effect,Effect::Exhaust{target:Target::Name(n)} if n=="chosen") {return predicate==Some(more(0));}
    let excluded=serde_json::json!({"kind":"binary","op":"!==","left":{"kind":"ref","path":["card"]},"right":{"kind":"ref","path":["source"]}});
    let unit=serde_json::json!({"kind":"query","name":"isUnit","receiver":{"kind":"ref","path":["card"]},"args":[]});
    let other_friendly=|threshold:u64|serde_json::json!({"kind":"binary","op":"&&","left":{"kind":"binary","op":"&&","left":excluded,"right":unit},"right":more(threshold)});
    // ASH_100-family: other friendly units with 2+ keywords get +2/+2.
    if a.trigger=="constant"&&!a.optional&&r.owner=="self"&&r.zone.as_deref()==Some("arena")&&predicate==Some(other_friendly(1))&&matches!(&a.effect,Effect::ModifyStats{target:Target::Name(n),power:Expr::Const{value:p},hp:Expr::Const{value:h},duration} if n=="chosen"&&duration=="persistent"&&p.as_i64()==Some(2)&&h.as_i64()==Some(2)) {return true;}
    // Boba Fett Daimyo unit side: other friendly units with 1+ keywords get +1/+0.
    a.trigger=="constant"&&!a.optional&&r.owner=="self"&&r.zone.as_deref()==Some("arena")&&predicate==Some(other_friendly(0))&&matches!(&a.effect,Effect::ModifyStats{target:Target::Name(n),power:Expr::Const{value:p},hp:Expr::Const{value:h},duration} if n=="chosen"&&duration=="persistent"&&p.as_i64()==Some(1)&&h.as_i64()==Some(0))
}

// Complete ordinary upgrade registrations only; parent getters remain asserting.
fn parent_upgrade_registration(a:&Ability)->Option<bool> {
    if a.trigger!="whenPlayed"||a.optional||a.requires_confirmation||a.coordinate||!a.costs.is_empty()||a.target_resolver.is_some()||!a.target_resolvers.is_empty()||a.condition.is_some()||a.then.is_some()||a.if_you_do.is_some(){return None;}
    let Effect::Conditional{condition,on_true,on_false}=&a.effect else{return None;};
    if !on_false.as_ref().is_some_and(|e|matches!(e.as_ref(),Effect::NoAction)){return None;}
    let parent=|e:&Expr,name:&str|matches!(e,Expr::Ref{path} if path.iter().map(String::as_str).collect::<Vec<_>>()==["source",name]);
    if a.context_title.is_empty() && matches!(on_true.as_ref(),Effect::CreateTokenUnit{token_type,amount:Expr::Const{value},target:Target::Name(n),enters_ready} if token_type=="beast"&&value.as_u64()==Some(1)&&n=="ownPlayer"&&!*enters_ready) {
        if let Expr::Binary{op,left,right}=condition {
            if op=="&&" && matches!(left.as_ref(),Expr::Binary{op,left,right} if ["!=","!=="].contains(&op.as_str())&&parent(left,"parentCard")&&matches!(right.as_ref(),Expr::Const{value} if value.is_null())) && matches!(right.as_ref(),Expr::Query{name,receiver,args} if name=="hasSomeTrait"&&parent(receiver,"parentCard")&&args.len()==1&&matches!(&args[0],Expr::Array{items} if !items.is_empty()&&items.iter().all(|e|matches!(e,Expr::Const{value} if value.as_str().is_some_and(|s|!s.is_empty()))))) {return Some(false);}
        }
    }
    if matches!(condition,Expr::Query{name,receiver,args} if name=="hasSomeAspect"&&parent(receiver,"parentUnit")&&args.len()==1&&matches!(&args[0],Expr::Const{value} if value.as_str().is_some_and(|s|!s.is_empty()))) && matches!(on_true.as_ref(),Effect::GiveShield{target:Target::Expression(e),amount:Expr::Const{value}} if parent(e,"parentUnit")&&value.as_u64()==Some(1)) && a.context_title.len()==3 && matches!(&a.context_title[0],Expr::Const{value} if value.is_string())&&matches!(&a.context_title[2],Expr::Const{value} if value.is_string())&&matches!(&a.context_title[1],Expr::Binary{op,left,right} if op=="??"&&matches!(left.as_ref(),Expr::Ref{path} if path.iter().map(String::as_str).collect::<Vec<_>>()==["source","parentUnit","title"])&&matches!(right.as_ref(),Expr::Const{value} if value.is_string())) {return Some(true);}
    None
}

fn contains_event_ref(v:&Value)->bool {(v["kind"]=="ref"&&v["path"][0]=="event")||match v {Value::Array(a)=>a.iter().any(contains_event_ref),Value::Object(o)=>o.values().any(contains_event_ref),_=>false}}
fn plain_result_child(a:&Ability)->bool {a.trigger.is_empty()&&!a.optional&&!a.coordinate&&!a.requires_confirmation&&a.costs.is_empty()&&a.context_title.is_empty()&&a.condition.is_none()&&a.target_resolver.is_none()&&a.target_resolvers.is_empty()&&a.then.is_none()&&a.if_you_do.is_none()}
fn resolved_defeat_continuation(a:&Ability)->bool {
    let Some(next)=a.then.as_deref() else{return false;};
    a.trigger=="event"&&!a.optional&&!a.coordinate&&!a.requires_confirmation&&a.costs.is_empty()&&a.context_title.is_empty()&&a.condition.is_none()&&a.target_resolver.is_none()&&a.target_resolvers.is_empty()&&a.if_you_do.is_none()&&plain_result_child(next)&&
    matches!(&a.effect,Effect::Defeat{target:Target::ArenaList(list)} if matches!(&list.receiver,Expr::Ref{path} if path==&["game"])&&matches!(&list.predicate,Expr::Const{value} if value.as_bool()==Some(true)))&&
    matches!(&next.effect,Effect::Damage{target:Target::Name(n),amount:Expr::Query{name,receiver,args}} if n=="enemyBase"&&name=="resolvedDefeatedUnitCount"&&args.is_empty()&&matches!(receiver.as_ref(),Expr::Ref{path} if path==&["opponent"]))
}
fn damage_result_continuation(a:&Ability)->bool {
    let Some(next)=a.then.as_deref() else{return false;};
    ["event","whenPlayed","whenDefeated","action"].contains(&a.trigger.as_str())&&!a.optional&&!a.coordinate&&!a.requires_confirmation&&a.costs.is_empty()&&a.context_title.is_empty()&&a.condition.is_none()&&a.target_resolvers.is_empty()&&a.target_resolver.as_ref().is_none_or(|r|!contains_event_ref(&serde_json::to_value(r).unwrap()))&&a.if_you_do.is_none()&&plain_result_child(next)&&
    matches!(&a.effect,Effect::Damage{target:Target::Name(n),amount:Expr::Const{value}} if ["chosen","self","enemyBase","ownBase"].contains(&n.as_str())&&value.as_u64().is_some_and(|n|n<=65535))&&
    matches!(&next.effect,Effect::Damage{target:Target::Name(n),amount:Expr::Ref{path}} if ["enemyBase","ownBase"].contains(&n.as_str())&&path==&["event","damageDealt"])
}
pub(crate) fn native_single_weakness_group(effect:&Effect)->bool {
    matches!(effect,Effect::Select{target_resolver,effect} if target_resolver.native.as_ref().is_some_and(|n|n.is_opponent()&&n.is_root_binding()) && selected_weakness(effect))
}
fn native_opponent_registration(a:&Ability)->bool {
    let Some(r)=&a.target_resolver else{return false;};
    let Some(native)=&r.native else{return false;};
    if !native.is_opponent()||!native.is_root_binding()||r.optional||r.predicate.is_some()||!native.types().is_ok_and(|types|types.iter().all(|t|matches!(t,selector_contract::CardTypeFilter::Unit|selector_contract::CardTypeFilter::NonLeaderUnit))) {return false;}
    if !["event","whenPlayed","onAttack"].contains(&a.trigger.as_str())||a.optional||a.requires_confirmation||a.coordinate||a.gained_host.is_some()||a.attached_context.is_some()||!a.context_title.is_empty()||!a.costs.is_empty()||a.condition.is_some()||!a.target_resolvers.is_empty()||a.then.is_some()||a.if_you_do.is_some(){return false;}
    matches!(&a.effect,Effect::Defeat{target:Target::Name(n)} if n=="chosen")
        || matches!(&a.effect,Effect::Damage{target:Target::Name(n),amount:Expr::Const{value}} if n=="chosen"&&value.as_u64().is_some_and(|n|(1..=65535).contains(&n)))
        || a.trigger=="whenPlayed"&&selected_weakness(&a.effect)
}

impl Program {
    /// Where the effect of the selection at `pc` ends. A selection inside an effect covers its own effect, the way
    /// forceteki's SelectCardSystem checks only its immediate effect; an ability's target covers the rest of the program.
    pub fn select_end(&self, pc: usize) -> usize {
        self.select_ends.iter().find(|(at, _)| *at == pc).map_or(self.instructions.len(), |(_, end)| *end)
    }
    pub(crate) fn visit_expression_roots_mut(&mut self, visit: &mut impl FnMut(&mut Expr)) {
        for fragment in &mut self.context_title { visit(fragment); }
        for cost in &mut self.costs {
            if let Cost::Resources { amount } = cost { visit(amount); }
        }
        if let Some(condition) = &mut self.condition { visit(condition); }
        for instruction in &mut self.instructions {
            match instruction {
                Instruction::Select(resolver) => resolver.visit_expression_roots_mut(visit),
                Instruction::Conditional { condition, .. } => visit(condition),
                Instruction::Apply(effect) | Instruction::Group(effect) => effect.visit_expression_roots_mut(visit),
                // Named selector groups stay portable, like the named resolvers they prepare.
                Instruction::NamedGroup(_) | Instruction::NamedChooserGroup { .. } => {}
                Instruction::CaptureMaximum{..}|Instruction::EndStep | Instruction::Commit | Instruction::IfResult { .. } | Instruction::Jump { .. } => {}
            }
        }
    }
    pub fn compile(ability: &Ability) -> Result<Self, String> {
        if matches!(ability.effect,Effect::InitiateAttack{..})&&!crate::native_attack::ability(ability) {return Err("native attack requires a standalone bounded ordinary registration".into());}

        if crate::death_hooks::marker(&serde_json::to_value(ability).map_err(|e|e.to_string())?) && ability.hooks.is_empty() {return Err("defeat hooks cannot be nested in another registration or effect".into());}
        if !ability.hooks.is_empty() || ability.trigger=="eventHooks" {
            let smuggle=crate::death_hooks::smuggle_play_registration(ability);
            let leader_deploy=crate::death_hooks::leader_deploy_base_registration(ability);
            let card_played=crate::death_hooks::card_played_registration(ability);
            if !crate::death_hooks::registration(ability) && !smuggle && !leader_deploy && !card_played {
                return Err(crate::death_hooks::reject_reason(ability));
            }
            let mut normalized=ability.clone();normalized.hooks.clear();normalized.trigger="whenPlayed".into();
            let mut program=Self::compile(&normalized)?;
            program.hooks=ability.hooks.clone();
            if smuggle {
                program.trigger="whenPlayedUsingSmuggle".into();
            } else if leader_deploy {
                program.trigger="onLeaderDeployed".into();
            } else if card_played {
                program.trigger="onCardPlayed".into();
            } else if !ability.hooks.iter().any(|h|matches!(h,crate::death_hooks::Hook::Alias(v) if matches!(v.alias,crate::death_hooks::AliasName::WhenPlayed))) {
                program.trigger="eventHooks".into();
            }
            program.executable=crate::program_backend(&serde_json::to_vec(&program).map_err(|e|e.to_string())?);
            return Ok(program);
        }
        if crate::phase_history::marker(&serde_json::to_value(ability).map_err(|e|e.to_string())?)&&!crate::phase_history::ability(ability){return Err("phase history requires a bounded ordinary root registration".into());}
        fn native_opponent(v:&Value)->bool {(v["kind"]=="cardSelection"&&v["chooser"]=="opponent")||match v {Value::Object(o)=>o.values().any(native_opponent),Value::Array(a)=>a.iter().any(native_opponent),_=>false}}
        if native_opponent(&serde_json::to_value(&ability.effect).map_err(|e|e.to_string())?) {return Err("opponent selection inside an effect requires its own chooser/context protocol".into());}
        if ability.target_resolver.as_ref().and_then(|r|r.native.as_ref()).is_some_and(|n|!n.is_root_binding()) {return Err("root native selector requires the target result binding".into());}
        for descriptor in &ability.target_resolvers {
            if let Some(native)=&descriptor.target_resolver.native {
                if !native.matches_named_binding(&descriptor.name,&descriptor.chooser) {return Err("native named selector binding/chooser must match its descriptor".into());}
            }
        }
        if crate::static_ongoing::marker(&serde_json::to_value(ability).map_err(|e|e.to_string())?) && !crate::static_ongoing::registration(ability) {return Err("static self ongoing contracts require a sole unconditional root constant".into());}
        if gained_host_marker(&serde_json::to_value(ability).map_err(|e|e.to_string())?) && gained_host_kind(ability).is_none() {return Err("gained host requires one exact reviewed root attack template".into());}
        if let Some(context)=&ability.attached_context {
            if !["upgrade","host"].contains(&context.as_str()) || ability.trigger!="constant" || ability.coordinate || ability.optional || ability.requires_confirmation || !ability.costs.is_empty() || !ability.context_title.is_empty() || !ability.target_resolvers.is_empty() || ability.then.is_some() || ability.if_you_do.is_some() {
                return Err("attached contexts require bounded constant keyword registrations".into());
            }
            fn keywords(e:&Effect,selected:bool,context:&str)->bool {match e {
                Effect::GainTrait{target:Target::Name(n),trait_name,duration}|Effect::LoseTrait{target:Target::Name(n),trait_name,duration}=>duration=="persistent" && context=="upgrade" && !selected && n=="parent" && !trait_name.is_empty() && trait_name.len()<=64,
                Effect::GainKeyword {target:Target::Name(n),keyword,amount:Expr::Const{value},duration}=>duration=="persistent" && n==if selected {"chosen"}else if context=="upgrade" {"parent"}else{"self"} && ["grit","sentinel","restore","raid","overwhelm"].contains(&keyword.as_str()) && value.as_u64().is_some_and(|n|(1..=2).contains(&n)),
                Effect::Simultaneous{effects}=>!effects.is_empty()&&effects.len()<=8&&effects.iter().all(|e|keywords(e,selected,context)),_=>false}}
            if !keywords(&ability.effect,ability.target_resolver.is_some(),context) && !crate::attached_constants::host_trait_aura(ability) || (context=="upgrade" && ability.target_resolver.is_some()) {return Err("attached context supports only persistent literal keyword grants".into());}
            if context=="host" {
                fn parent_ref(e:&Expr)->bool {match e {Expr::Ref{path}=>path.get(1).is_some_and(|p|p=="parentCard"||p=="parentUnit"),Expr::Collection{receiver,args,..}|Expr::Query{receiver,args,..}=>parent_ref(receiver)||args.iter().any(parent_ref),Expr::Binary{left,right,..}=>parent_ref(left)||parent_ref(right),Expr::Unary{value,..}=>parent_ref(value),Expr::Conditional{condition,then,r#else}=>parent_ref(condition)||parent_ref(then)||parent_ref(r#else),Expr::Array{items}|Expr::Math{args:items,..}=>items.iter().any(parent_ref),Expr::ArenaList{receiver,predicate}|Expr::ArenaFilter{receiver,predicate}|Expr::ArenaSome{receiver,predicate}=>parent_ref(receiver)||parent_ref(predicate),Expr::ArenaLength{receiver}=>parent_ref(receiver),_=>false}}
                if ability.condition.as_ref().is_some_and(parent_ref) {return Err("delegated host condition cannot read an upgrade parent getter".into());}
                if let Some(r)=&ability.target_resolver {
                    let expected=serde_json::json!({"kind":"binary","op":"&&","left":{"kind":"binary","op":"!==","left":{"kind":"ref","path":["card"]},"right":{"kind":"ref","path":["source"]}},"right":{"kind":"binary","op":"===","left":{"kind":"ref","path":["card","controller"]},"right":{"kind":"ref","path":["player"]}}});
                    if !crate::attached_constants::host_trait_aura(ability) && (r.owner!="self" || r.card_type!="unit" || r.zone.as_deref()!=Some("arena") || r.optional || r.count.is_some() || r.mode.is_some() || r.arena.is_some() || r.prompt_title.is_some() || r.predicate.as_ref().and_then(|p|serde_json::to_value(p).ok())!=Some(expected)) {return Err("delegated host aura requires other friendly arena units by host identity and controller".into());}
                }
            }
        }
        if contains_arena_scalar(&serde_json::to_value(ability).map_err(|e|e.to_string())?) && !arena_query_ability(ability) {return Err("arena scalar queries require exact reviewed registration".into());}
        if attachment_defeat_marker(&ability.effect)&&!attachment_defeat_registration(ability){return Err("attachment list defeat requires complete reviewed scalar-unit selector and event window".into());}
        if parent_upgrade_registration(ability)==Some(false) {
            let mut normalized=ability.clone();
            if let Effect::Conditional{condition:Expr::Binary{left,..},..}=&mut normalized.effect {
                if let Expr::Binary{op,..}=left.as_mut(){if op=="!="{*op="!==".into();return Self::compile(&normalized);}}
            }
        }
        // Deck discard has per-card events plus an aggregate event. The scalar
        // result is not a modeled engine event payload; reject payload captures.
        fn contains(value:&Value, predicate:fn(&Value)->bool)->bool {
            predicate(value) || match value {Value::Array(a)=>a.iter().any(|v|contains(v,predicate)),Value::Object(o)=>o.values().any(|v|contains(v,predicate)),_=>false}
        }
        let encoded=serde_json::to_value(ability).map_err(|e|e.to_string())?;
        if contains(&encoded,|v|v["kind"]=="ref"&&v["path"][0]=="snapshot") {return Err("capture references are reserved for verified maximum instructions".into());}
        if contains(&encoded,|v|v["kind"]=="ref"&&v["path"]==serde_json::json!(["source","parentCard","title"])) && !crate::attached_constants::parent_title_play(ability) {
            let condition=matches!(&ability.condition,Some(Expr::Binary{op,left,right}) if op=="==="&&matches!(left.as_ref(),Expr::Ref{path} if path==&["source","parentCard","title"])&&matches!(right.as_ref(),Expr::Const{value} if value.as_str().is_some_and(|title|!title.is_empty()&&title.len()<=128)));
            if !condition || ability.trigger!="constant" || ability.coordinate || ability.optional || ability.requires_confirmation || !ability.costs.is_empty() || !ability.context_title.is_empty() || ability.target_resolver.is_some() || !ability.target_resolvers.is_empty() || ability.then.is_some() || ability.if_you_do.is_some() || !matches!(&ability.effect,Effect::GainKeyword{target:Target::Name(n),amount:Expr::Const{value},duration,..} if n=="parent"&&duration=="persistent"&&value.as_u64()==Some(1)) {return Err("parent printed title requires a standalone live attached keyword equality condition".into());}
        }
        if ability.target_resolver.as_ref().is_some_and(|r|r.card_type=="any") && matches!(&ability.effect,Effect::ModifyStats{power,..} if contains(&serde_json::to_value(power).expect("typed amount serializes"),|v|v["kind"]=="query"&&v["name"]=="hasSomeArenaUnit")) {return Err("Any-target aspect-scaled phase stats require inert base/front-leader menu support".into());}
        if contains(&encoded,|v|v["kind"]=="query"&&v["name"]=="leaderTraits") {
            let r=ability.target_resolver.as_ref();
            fn pred(e:&Expr)->bool {serde_json::to_value(e).ok()==Some(serde_json::json!({"kind":"binary","op":"&&","left":{"kind":"binary","op":"&&","left":{"kind":"binary","op":"!==","left":{"kind":"ref","path":["card"]},"right":{"kind":"ref","path":["source"]}},"right":{"kind":"query","name":"isNonLeaderUnit","receiver":{"kind":"ref","path":["card"]},"args":[]}},"right":{"kind":"query","name":"hasSomeTrait","receiver":{"kind":"ref","path":["card"]},"args":[{"kind":"query","name":"leaderTraits","receiver":{"kind":"ref","path":["player"]},"args":[]}]}}))}
            if ability.trigger!="onAttack"||!ability.optional||ability.coordinate||ability.requires_confirmation||!ability.costs.is_empty()||!ability.context_title.is_empty()||ability.attached_context.is_some()||!ability.target_resolvers.is_empty()||ability.then.is_some()||ability.if_you_do.is_some()||ability.condition.is_some()||!r.is_some_and(|r|r.owner=="any"&&r.card_type=="any"&&r.zone.as_deref()==Some("anyAttackable")&&!r.optional&&r.count.is_none()&&r.mode.is_none()&&r.arena.is_none()&&r.prompt_title.is_none()&&r.predicate.as_ref().is_some_and(pred))||!matches!(&ability.effect,Effect::GiveExperience{target:Target::Name(n),amount:Expr::Const{value}} if n=="chosen"&&value.as_u64()==Some(1)) {return Err("leader trait union requires bounded optional ordinary On Attack Experience selector".into());}
        }
        if contains_keyword_inventory(&encoded)&&!keyword_inventory_registration(ability){return Err("keyword inventory requires bounded optional unit Exhaust or other-friendly persistent stat registration".into());}
        fn binder_context(value:&Value,live:bool)->Result<(),String> {
            if value["kind"]=="ref"&&value["path"][0]=="modifierTarget"&&!live {return Err("modifierTarget only exists in persistent stat expressions".into());}
            if value["kind"]=="query"&&value["name"]=="resourcePairs"&&!live {return Err("resourcePairs only exists in persistent stat expressions".into());}
            match value {Value::Object(object)=>{for(k,v)in object {let stat=value["kind"]=="modifyStats"&&value["duration"]=="persistent"&&["power","hp"].contains(&k.as_str());binder_context(v,live||stat)?;}},Value::Array(items)=>for v in items {binder_context(v,live)?;},_=>{}}Ok(())
        }
        binder_context(&encoded,false)?;

        if contains(&encoded,|v|v["kind"]=="discardDeck") && contains(&encoded,|v|v["kind"]=="ref" && v["path"][0]=="event") {
            return Err("deck discard event payload captures are not modeled".into());
        }
        fn validate_title(e: &Expr) -> Result<(), String> {
            match e {
                Expr::Ref {path} if path.iter().map(String::as_str).collect::<Vec<_>>() == ["source", "title"] => Ok(()),
                Expr::Conditional {condition, then, r#else} => { condition.validate()?; validate_title(then)?; validate_title(r#else) },
                Expr::Array { .. } => Err("context titles require scalar fragments".into()),
                Expr::Ref {path} if path.len() == 1 || matches!(path.last().map(String::as_str), Some("controller" | "base")) => Err("context titles require scalar reference values".into()),
                Expr::Binary {op, ..} if op == "+" => e.validate_number(),
                _ => e.validate(),
            }
        }
        fn result_query(v:&Value)->bool {v["kind"]=="query"&&v["name"]=="resolvedDefeatedUnitCount"}
        fn lki_power(v:&Value)->bool {v["kind"]=="ref"&&v["path"]==serde_json::json!(["event","lastKnownInformation","power"])}
        fn damage_result(v:&Value)->bool {v["kind"]=="ref"&&v["path"]==serde_json::json!(["event","damageDealt"])}
        let resolved_defeat=resolved_defeat_continuation(ability);
        if contains(&encoded,result_query)&&!resolved_defeat {return Err("resolved defeat count requires direct arena defeat then opponent-base damage".into());}
        if contains(&encoded,lki_power) {
            if ability.trigger!="whenDefeated"||ability.optional||ability.coordinate||ability.requires_confirmation||!ability.costs.is_empty()||ability.condition.is_some()||ability.then.is_some()||ability.if_you_do.is_some()||!ability.target_resolvers.is_empty()||ability.target_resolver.as_ref().is_none_or(|r|r.card_type!="unit"||r.optional||r.is_multiple())||!matches!(&ability.effect,Effect::Damage{target:Target::Name(n),amount:Expr::Ref{path}} if n=="chosen"&&path==&["event","lastKnownInformation","power"]) {return Err("last-known power requires scalar mandatory defeat damage registration".into());}
        }
        if contains(&encoded,damage_result) && !damage_result_continuation(ability) {return Err("actual damage result requires scalar damage then bounded base damage continuation".into());}
        if let Effect::Defeat {target:Target::ArenaList(list)}=&ability.effect {
            if ability.trigger!="event" || ability.target_resolver.is_some() || (ability.then.is_some()&&!resolved_defeat) || ability.if_you_do.is_some() || ability.condition.is_some() || ability.coordinate {return Err("arena-list defeat requires a standalone event or reviewed resolved-event continuation".into());}
            list.validate_scope(true)?;
        }
        if let Effect::ModifyStats {target:Target::ArenaList(list),power,hp,duration}=&ability.effect {
            let token_attack=ability.trigger=="onAttack" && list.token_unit_attack_scope() && matches!(power,Expr::Const{value} if value.as_i64()==Some(1)) && matches!(hp,Expr::Const{value} if value.as_i64()==Some(1));
            if !(["event","whenPlayed"].contains(&ability.trigger.as_str()) || token_attack) || ability.target_resolver.is_some() || ability.then.is_some() || ability.if_you_do.is_some() || ability.condition.is_some() || ability.coordinate || ability.optional || ability.requires_confirmation || !ability.costs.is_empty() || !ability.context_title.is_empty() || !ability.target_resolvers.is_empty() || duration!="phase" {return Err("arena-list phase stats require a standalone event, WhenPlayed or bounded friendly token-unit attack without selectors, costs or continuations".into());}
            if token_attack {list.receiver.validate()?;list.predicate.validate()?;} else {list.validate_phase_stats()?;}
            fn literal(e:&Expr)->bool {match e {Expr::Const {value}=>value.as_i64().is_some_and(|n|(-65535..=65535).contains(&n)),Expr::Unary {op,value} if ["+","-"].contains(&op.as_str())=>matches!(value.as_ref(),Expr::Const {value} if value.as_i64().is_some_and(|n|(0..=65535).contains(&n))),_=>false}}
            if !literal(power)||!literal(hp) {return Err("arena-list phase stats require bounded signed literals".into());}
        }
        if ability.coordinate && !["constant","onAttack"].contains(&ability.trigger.as_str()) {return Err("Coordinate supports only constant or onAttack registrations".into());}
        if ability.then.as_ref().is_some_and(|a|a.coordinate) || ability.if_you_do.as_ref().is_some_and(|a|a.coordinate) {return Err("Coordinate cannot register on a continuation".into());}
        if ability.context_title.len() > 32 { return Err("context title fragment count exceeds capacity".into()); }
        fn title_literal_bytes(e: &Expr) -> usize {
            match e {
                Expr::Compiled(compiled) => title_literal_bytes(&compiled.original),
                Expr::NativeTokenPredicate{receiver,..}=>title_literal_bytes(receiver),
 Expr::ArenaList{receiver,predicate}|Expr::ArenaFilter{receiver,predicate}|Expr::ArenaSome{receiver,predicate}=>title_literal_bytes(receiver)+title_literal_bytes(predicate),Expr::ArenaLength{receiver}=>title_literal_bytes(receiver),
                Expr::Const {value} => value.as_str().map_or(0, str::len),
                Expr::Array {items} => items.iter().map(title_literal_bytes).sum(),
                Expr::Math {args, ..} => args.iter().map(title_literal_bytes).sum(),
                Expr::HistoryAttackCheck {..} | Expr::HistoryQuery(_) | Expr::DataMember {..} | Expr::Ref { .. } => 0,
                Expr::Unary {value, ..} => title_literal_bytes(value),
                Expr::Binary {left, right, ..} => title_literal_bytes(left) + title_literal_bytes(right),
                Expr::Conditional {condition, then, r#else} => title_literal_bytes(condition) + title_literal_bytes(then) + title_literal_bytes(r#else),
                Expr::Collection {receiver,args,..}|Expr::Query {receiver, args, ..} => title_literal_bytes(receiver) + args.iter().map(title_literal_bytes).sum::<usize>(),
            }
        }
        let mut literal_bytes = 0;
        for fragment in &ability.context_title {
            if parent_upgrade_registration(ability)==Some(true) && matches!(fragment,Expr::Binary{op,..} if op=="??") {
                // This exact title-only form never broadens general Expr validation.
            } else { validate_title(fragment)?; }
            literal_bytes += title_literal_bytes(fragment);
            if literal_bytes > 1024 { return Err("context title aggregate literal bytes exceed capacity".into()); }
        }

        if !["action", "whenPlayed", "whenDefeated", "onAttack", "onDefense", "whenAttackEnds", "event", "constant", "epicAction"]
            .contains(&ability.trigger.as_str())
        {
            return Err(format!("unsupported runtime trigger {}", ability.trigger));
        }
        if ability.trigger=="whenAttackEnds" && ability.condition.is_some() {return Err("conditional attack-end registration capture is not implemented".into());}
        if crate::attack_history::contains(&encoded) && (ability.trigger!="whenAttackEnds" || !matches!(&ability.effect,Effect::Conditional{condition:Expr::HistoryAttackCheck{..},..}) || ability.condition.is_some() || ability.then.is_some() || ability.if_you_do.is_some() || ability.target_resolver.as_ref().is_some_and(|r|crate::attack_history::contains(&serde_json::to_value(r).unwrap()))) {return Err("attack history requires a standalone attack-end conditional effect".into());}
        if ability.attacker_must_survive && ability.trigger!="whenAttackEnds" {return Err("attacker survival gate requires attack-end registration".into());}
        fn trait_leaf(e:&Effect)->bool {match e {Effect::GainTrait{..}|Effect::LoseTrait{..}=>true,Effect::Simultaneous{effects}=>effects.iter().any(trait_leaf),_=>false}}
        if trait_leaf(&ability.effect) && (ability.trigger!="constant" || ability.condition.is_some() || ability.target_resolver.is_some() || !ability.target_resolvers.is_empty() || ability.coordinate || ability.gained_host.is_some()) {
            return Err("trait leaves require unconditional source/parent constant registrations without dependency cycles".into());
        }
        if ability.trigger == "constant" {
            if ability.optional
                || !ability.costs.is_empty()
                || ability.then.is_some()
                || ability.if_you_do.is_some()
            {
                return Err(
                    "constant modifiers cannot have costs, choices, or continuations".into(),
                );
            }
            fn validate_constant(effect: &Effect, selected: bool) -> Result<(), String> {
                let expected = if selected { "chosen" } else { "self" };
                let valid_target = |target: &Target, duration: &str| duration == "persistent" && matches!(target, Target::Name(n) if n == expected || (!selected && n == "parent"));
                match effect {
                    Effect::ConstantEffectRegistration(r) if !selected => r.validate(),
                    Effect::GainTrait{target,trait_name,duration}|Effect::LoseTrait{target,trait_name,duration}
                        if valid_target(target,duration) && !trait_name.is_empty() && trait_name.len()<=64 => Ok(()),
                    Effect::ModifyStats {power,hp,duration,target} if valid_target(target,duration)=>{power.live_stat_bounds()?;hp.live_stat_bounds()?;Ok(())},
                    Effect::CopyKeywords {target,duration,domain} if valid_target(target,duration) && domain.iter().map(String::as_str).collect::<Vec<_>>()==COPY_KEYWORDS=>Ok(()),
                    Effect::GainKeyword { target, amount, duration, .. } if valid_target(target, duration) && !amount.depends_on_keywords() => Ok(()),
                    Effect::LoseKeyword { target, duration, .. } if valid_target(target, duration) => Ok(()),
                    Effect::Simultaneous { effects } if !effects.is_empty() => { for e in effects { validate_constant(e, selected)?; } Ok(()) }
                    _ => Err("unsupported constant modifier or recursive effective-stat/keyword dependency".into()),
                }
            }
            validate_constant(&ability.effect, ability.target_resolver.is_some())?;
            fn modifies_keywords(e: &Effect) -> bool { match e { Effect::CopyKeywords{..}|Effect::GainKeyword { .. } | Effect::LoseKeyword { .. } => true, Effect::Simultaneous { effects } => effects.iter().any(modifies_keywords), _ => false } }
            if !crate::bounded_reductions::power_maximum_grant(ability) && modifies_keywords(&ability.effect) && (ability.condition.as_ref().is_some_and(Expr::depends_on_keywords) || ability.target_resolver.as_ref().and_then(|r| r.predicate.as_ref()).is_some_and(Expr::depends_on_keywords)) { return Err("constant keyword-dependent conditions would form evaluation cycles".into()); }

            if !crate::bounded_reductions::power_maximum_grant(ability) && (ability
                .condition
                .as_ref()
                .is_some_and(Expr::depends_on_stats)
                || ability
                    .target_resolver
                    .as_ref()
                    .and_then(|r| r.predicate.as_ref())
                    .is_some_and(Expr::depends_on_stats))
            {
                return Err(
                    "constant stat-dependent conditions would form evaluation cycles".into(),
                );
            }
            if ability.target_resolver.as_ref().is_some_and(|r| {
                r.optional || r.mode.is_some() || r.count.is_some()
                    || !matches!(
                        r.zone.as_deref(),
                        None | Some("arena") | Some("anyAttackable")
                    )
            }) {
                return Err(
                    "constant target filter must be arena-only without optional choice".into(),
                );
            }
        }
        if ability.requires_confirmation {
            return Err("explicit confirmation semantics are not implemented".into());
        }
        if ability.trigger != "action" && !ability.costs.is_empty() {
            return Err("triggered or event additional costs are not implemented".into());
        }
        let mut p = Self {
            attached_host: ability.attached_host,

            hooks: ability.hooks.clone(),
            gained_host: ability.gained_host,
            attached_context: ability.attached_context.clone(),
            title: ability.title.clone(),
            context_title: ability.context_title.clone(),
            trigger: ability.trigger.clone(),
            attacker_must_survive: ability.attacker_must_survive,
            coordinate: ability.coordinate,
            costs: ability.costs.clone(),
            optional: ability.optional,
            requires_confirmation: ability.requires_confirmation,
            condition: ability.condition.clone(),
            instructions: vec![],
            event_end:0,
            executable: Err("program has not been compiled"),
            select_ends: vec![],
        };
        if let Some(e) = &p.condition {
            e.validate()?;
        }
        if p.costs
            .iter()
            .filter(|c| matches!(c, Cost::ExhaustSelf))
            .count()
            > 1
        {
            return Err("duplicate exhaust cost cannot be paid".into());
        }
        let mut resource_cost = 0u64;
        let mut force_cost = false;
        for cost in &p.costs {
            if let Cost::UseForceCost { target, timing, must_fully_resolve } = cost {
                if target != "ownPlayer" || timing != "abilityCostPayment" || !must_fully_resolve {
                    return Err("Force cost requires own-player mandatory ability payment".into());
                }
                if force_cost { return Err("duplicate Force cost cannot be paid".into()); }
                force_cost = true;
            }
            if let Cost::Resources { amount } = cost {
                let Expr::Const { value } = amount else {
                    return Err("dynamic resource costs require typed bounded evaluation".into());
                };
                let amount = value
                    .as_u64()
                    .filter(|n| *n <= u16::MAX as u64)
                    .ok_or("resource cost must be a nonnegative bounded integer")?;
                resource_cost = resource_cost
                    .checked_add(amount)
                    .filter(|n| *n <= u16::MAX as u64)
                    .ok_or("total resource cost exceeds capacity")?;
            }
        }
        p.compile_ability(ability, 0)?;
        if p.instructions.len()>u8::MAX as usize {return Err("ability event boundary exceeds capacity".into());}
        p.event_end=p.instructions.len() as u8;
        p.instructions.push(Instruction::EndStep);
        if p.instructions.len() > MAX_PROGRAM_INSTRUCTIONS {
            return Err("ability program exceeds instruction capacity".into());
        }
        let bytes=serde_json::to_vec(&p).map_err(|e|e.to_string())?;
        p.executable=crate::program_backend(&bytes);
        Ok(p)
    }
    fn compile_ability(&mut self, ability: &Ability, depth: usize) -> Result<(), String> {
        if !ability.hooks.is_empty() || ability.trigger=="eventHooks" {return Err("event hooks cannot appear inside effect continuations".into());}
        // One captured target, then one simultaneous two-attachment window. This
        // preserves both token identities before the resulting stat-death sweep.
        if depth==0 && crate::phase_history::ability(ability) && contains_weakness(&ability.effect) {
            let Effect::Conditional{condition,on_true,..}=&ability.effect else{unreachable!()};
            let resolver=ability.target_resolver.as_ref().expect("validated history Weakness resolver");resolver.validate()?;condition.validate()?;
            let base=self.instructions.len();self.instructions.extend([Instruction::Select(resolver.clone()),Instruction::Conditional{condition:condition.clone(),skip:base+4},Instruction::Group(*on_true.clone()),Instruction::Jump{to:base+5},Instruction::Apply(Effect::NoAction)]);return Ok(());
        }
        if depth!=0 && (ability.attacker_must_survive || ability.trigger=="whenAttackEnds") {return Err("attack-end registration metadata cannot appear in a continuation".into());}
        if ability.target_resolver.as_ref().and_then(|r|r.native.as_ref()).is_some_and(|n|n.is_opponent()) {
            if depth!=0||!native_opponent_registration(ability){return Err("opponent single-card selector requires a bounded standalone ordinary damage/defeat/Weakness registration".into());}
            let r=ability.target_resolver.as_ref().unwrap();r.validate()?;
            let effect=Effect::Select{target_resolver:r.clone(),effect:Box::new(ability.effect.clone())};
            self.validate_group(&effect,1)?;
            self.instructions.push(Instruction::NamedChooserGroup{effect,choosers:vec![NamedChooser::Opponent]});
            return Ok(());
        }
        if let Effect::MaximumPowerExperience{aspect}=&ability.effect {
            if depth!=0||!maximum_registration(ability){return Err("maximum capture requires a standalone bounded leader action/attack".into());}
            let base=self.instructions.len();
            let r:TargetResolver=serde_json::from_value(serde_json::json!({"owner":"self","type":"unit","zone":"anyAttackable","predicate":{"kind":"binary","op":"&&","left":{"kind":"binary","op":"&&","left":{"kind":"query","name":"isUnit","receiver":{"kind":"ref","path":["card"]},"args":[]},"right":{"kind":"query","name":"hasSomeAspect","receiver":{"kind":"ref","path":["card"]},"args":[{"kind":"const","value":aspect}]}},"right":{"kind":"binary","op":"===","left":{"kind":"query","name":"getPower","receiver":{"kind":"ref","path":["card"]},"args":[]},"right":{"kind":"ref","path":["snapshot","power"]}}}})).map_err(|e|e.to_string())?;
            self.instructions.extend([
                Instruction::Commit,Instruction::CaptureMaximum{aspect:aspect.clone()},
                Instruction::Conditional{condition:Expr::Binary{op:">".into(),left:Box::new(Expr::Ref{path:vec!["snapshot".into(),"count".into()]}),right:Box::new(Expr::Const{value:Value::from(1)})},skip:base+6},
                Instruction::Select(r),Instruction::Apply(Effect::GiveExperience{target:Target::Name("chosen".into()),amount:one()}),Instruction::Jump{to:base+7},
                Instruction::Apply(Effect::GiveExperience{target:Target::Expression(Expr::Ref{path:vec!["snapshot".into(),"sole".into()]}),amount:one()})]);
            self.select_ends.push((base+3,base+5));return Ok(());
        }
        if gained_host_marker(&serde_json::to_value(ability).map_err(|e|e.to_string())?) && (depth!=0||gained_host_kind(ability).is_none()) {return Err("gained host marker cannot appear in nested or other registrations".into());}
        if parent_defense_marker(&serde_json::to_value(ability).map_err(|e|e.to_string())?)&&(depth!=0||!parent_defense_registration(ability)){return Err("parent defense requires a sole bounded upgrade constant registration".into());}
        fn oversized(v:&Value)->bool {v["mode"]=="upTo"&&v["count"]["value"].as_u64().is_some_and(|n|n>3) || match v {Value::Object(o)=>o.values().any(oversized),Value::Array(a)=>a.iter().any(oversized),_=>false}}
        if depth==0&&oversized(&serde_json::to_value(ability).map_err(|e|e.to_string())?)&&force_continuation(ability)!=Some(ForceContinuation::Shields){return Err("selectors above three require exact Force Shield continuation".into());}

        if depth==0 {if let Some(form)=force_continuation(ability) {
            match form {
              ForceContinuation::Drain=>{if let Effect::Simultaneous{effects}=&ability.effect {if let Effect::Select{target_resolver,..}=&effects[0]{target_resolver.validate_inline()?;}}self.instructions.push(Instruction::Group(ability.effect.clone()));},
              ForceContinuation::Wave|ForceContinuation::Shields=>{
                self.instructions.push(Instruction::Apply(ability.effect.clone()));
                let skip=self.instructions.len();self.instructions.push(Instruction::IfResult{skip:0});self.instructions.push(Instruction::EndStep);
                let next=ability.if_you_do.as_ref().unwrap();
                if let Some(r)=&next.target_resolver {r.validate()?;self.instructions.push(Instruction::Select(r.clone()));self.instructions.push(Instruction::Group(next.effect.clone()));}
                else {if let Effect::Defeat{target:Target::ArenaList(list)}=&next.effect {list.predicate.validate()?;}self.instructions.push(Instruction::Apply(next.effect.clone()));}
                self.instructions[skip]=Instruction::IfResult{skip:self.instructions.len()};
              }
            }return Ok(());
        }}
        if depth==0 {if let Some(form)=weakness_extension(ability) {
            if let Some(r)=&ability.target_resolver {r.validate()?;self.instructions.push(Instruction::Select(r.clone()));}
            match form {
                WeaknessExtension::Torrent|WeaknessExtension::Mixed|WeaknessExtension::LeaderHeal=>self.instructions.push(Instruction::Group(ability.effect.clone())),
                WeaknessExtension::Sequence=>self.compile_effect(&ability.effect,0)?,
                WeaknessExtension::Picador=>{self.compile_effect(&ability.effect,0)?;let next=ability.then.as_ref().unwrap();let r=next.target_resolver.as_ref().unwrap();r.validate()?;self.instructions.push(Instruction::Select(r.clone()));self.compile_effect(&next.effect,1)?;},
            }
            return Ok(());
        }}

        if contains_arena_tokens(ability)&&(depth!=0||arena_token_kind(ability).is_none()){return Err("arena tokens require verified mandatory standalone unit registration".into());}
        if contains_new_arena_keywords(ability) {
            if depth!=0||!arena_keyword_registration(ability) {return Err("arena keyword batches require standalone event or ordinary OnAttack Restore grant".into());}
            self.instructions.push(Instruction::Apply(ability.effect.clone()));return Ok(());
        }

        if contains_arena_damage(ability) && (depth!=0 || !(arena_damage_ability(ability)||ordered_ground_registration(ability))) {return Err("arena damage requires a mandatory standalone ordinary-unit play/attack/death registration".into());}
        if depth==0&&imperial_resource_continuation(ability) {
            // The source EventWindow resolves ifYouDo before ordinary death
            // triggers. Retain one program event_end; do not insert EndStep.
            let r=ability.target_resolver.as_ref().unwrap();r.validate_inline()?;
            self.instructions.push(Instruction::Select(r.clone()));self.compile_effect(&ability.effect,depth)?;
            let at=self.instructions.len();self.instructions.push(Instruction::IfResult{skip:0});
            self.compile_effect(&ability.if_you_do.as_ref().unwrap().effect,depth+1)?;
            self.instructions[at]=Instruction::IfResult{skip:self.instructions.len()};return Ok(());
        }
        if top_deck_marker(&serde_json::to_value(ability).unwrap()) && (depth!=0||!reviewed_top_deck_registration(ability)){return Err("top-deck resource requires a sole bounded optional unit entry registration".into());}
        if depth==0 && contains_resource(ability) && !reviewed_top_deck_registration(ability) {
            let header=!ability.requires_confirmation && !ability.coordinate && ability.context_title.is_empty() && ability.costs.is_empty() && ability.target_resolver.is_none() && ability.target_resolvers.is_empty() && ability.condition.is_none() && ability.then.is_none();
            let event=ability.trigger=="event" && !ability.optional && resource_self(&ability.effect) && ability.if_you_do.is_none();
            let death=ability.trigger=="whenDefeated" && ability.optional && ability.if_you_do.is_none() && matches!(&ability.effect,Effect::Conditional{condition:Expr::Binary{op,left,right},on_true,on_false} if op=="===" && matches!(left.as_ref(),Expr::Ref{path} if path.iter().map(String::as_str).collect::<Vec<_>>()==["source","zoneName"]) && matches!(right.as_ref(),Expr::Const{value} if value.as_str()==Some("discard")) && resource_self(on_true) && on_false.as_deref().is_some_and(|e|matches!(e,Effect::NoAction)));
            let force=ability.trigger=="whenDefeated" && ability.optional && matches!(&ability.effect,Effect::UseForce{target:Target::Name(n)} if n=="ownPlayer") && ability.if_you_do.as_ref().is_some_and(|n|resource_self(&n.effect));
            if !header || !(event||death||force) {return Err("resourceCard requires bounded self event/death registration".into());}
        }
        if contains_force_presence(ability) {
            let header=depth==0 && ability.trigger=="constant" && !ability.optional && !ability.requires_confirmation && !ability.coordinate && ability.context_title.is_empty() && ability.costs.is_empty() && ability.target_resolvers.is_empty() && ability.then.is_none() && ability.if_you_do.is_none() && matches!(&ability.condition,Some(Expr::Ref{path}) if path.iter().map(String::as_str).collect::<Vec<_>>()==["player","hasTheForce"]);
            let keyword=ability.target_resolver.is_none() && matches!(&ability.effect,Effect::GainKeyword{target:Target::Name(n),duration,keyword,amount:Expr::Const{value}} if n=="self" && duration=="persistent" && ["grit","sentinel","ambush"].contains(&keyword.as_str()) && value.as_u64()==Some(1));
            let aura=ability.target_resolver.as_ref().is_some_and(|r|r.owner=="self" && r.card_type=="unit" && r.zone.as_deref()==Some("arena") && r.arena.is_none() && !r.optional && r.mode.is_none() && r.count.is_none() && r.prompt_title.is_none() && matches!(&r.predicate,Some(Expr::Query{name,receiver,args}) if name=="isUnit" && args.is_empty() && matches!(receiver.as_ref(),Expr::Ref{path} if path.len()==1&&path[0]=="card"))) && matches!(&ability.effect,Effect::ModifyStats{target:Target::Name(n),duration,power:Expr::Const{value:p},hp:Expr::Const{value:h}} if n=="chosen" && duration=="persistent" && p.as_i64().is_some_and(|n|(-65535..=65535).contains(&n)) && h.as_i64()==Some(0));
            if !header || !(keyword||aura){return Err("Force presence supports bounded ordinary constant own-player keyword or friendly literal stat aura".into());}
        }
        if contains_force(&ability.effect) {
            let root=depth==0 && ["whenPlayed","whenDefeated","onAttack","event"].contains(&ability.trigger.as_str()) && !ability.requires_confirmation && !ability.coordinate && ability.context_title.is_empty() && ability.costs.is_empty() && ability.target_resolver.is_none() && ability.target_resolvers.is_empty() && ability.condition.is_none() && ability.then.is_none();
            let gain=["whenPlayed","onAttack","whenDefeated"].contains(&ability.trigger.as_str()) && matches!(&ability.effect,Effect::GainForce{target:Target::Name(n)} if n=="ownPlayer") && !ability.optional && ability.if_you_do.is_none();
            let use_force=["whenPlayed","whenDefeated","event"].contains(&ability.trigger.as_str()) && matches!(&ability.effect,Effect::UseForce{target:Target::Name(n)} if n=="ownPlayer") && ability.optional==(ability.trigger!="event") && ability.if_you_do.as_ref().is_some_and(|next| {
                if !next.trigger.is_empty() || next.coordinate || !next.context_title.is_empty() || !next.costs.is_empty() || next.optional || next.requires_confirmation || !next.target_resolvers.is_empty() || next.condition.is_some() || next.then.is_some() || next.if_you_do.is_some() {return false;}
                fn bounded(e:&Expr)->bool {matches!(e,Expr::Const{value} if value.as_i64().is_some_and(|n|(-65535..=65535).contains(&n))) || matches!(e,Expr::Unary{op,value} if op=="-" && matches!(value.as_ref(),Expr::Const{value} if value.as_u64().is_some_and(|n|n<=65535)))}
                match (&next.target_resolver,&next.effect) {
                    (None,Effect::ResourceCard{target:Target::Name(n)})=>ability.trigger=="whenDefeated" && n=="self",
                    (None,Effect::Draw{target:Target::Name(n),amount:Expr::Const{value}})=>ability.trigger=="whenPlayed" && n=="ownPlayer"&&value.as_u64().is_some_and(|n|n==1||n==2),
                    (None,Effect::Ready{target:Target::Name(n)})=>ability.trigger=="whenPlayed" && n=="self",
                    (None,Effect::DiscardOwnHand{target:Target::Name(n),amount:Expr::Const{value}})=>ability.trigger=="whenDefeated" && n=="enemyPlayer"&&value.as_u64()==Some(1),
                    (None,Effect::Simultaneous{effects})=>ability.trigger=="whenPlayed" && effects.len()==2 && {
                        let one=|e:&Expr|matches!(e,Expr::Const{value} if value.as_u64()==Some(1));
                        let exp=effects.iter().any(|e|matches!(e,Effect::GiveExperience{target:Target::Name(n),amount} if n=="self"&&one(amount)));
                        let shield=effects.iter().any(|e|matches!(e,Effect::GiveShield{target:Target::Name(n),amount} if n=="self"&&one(amount)));
                        exp&&shield
                    },
                    (Some(r),effect)=> {
                        let domain=(["any","self"].contains(&r.owner.as_str()) && ["unit","base"].contains(&r.card_type.as_str()) && matches!(r.zone.as_deref(),Some("arena"|"anyAttackable"))) || (r.owner=="any" && r.card_type=="any" && r.zone.as_deref()==Some("base") && matches!(effect,Effect::Heal{..}));
                        let selector=domain && r.arena.is_none() && !r.optional && r.mode.is_none() && r.count.is_none() && r.prompt_title.is_none() && r.predicate.is_none();
                        selector && match effect {
                            Effect::ModifyStats{target:Target::Name(n),duration,power,hp}=>r.card_type=="unit"&&n=="chosen"&&duration=="phase"&&bounded(power)&&bounded(hp),
                            Effect::Heal{target:Target::Name(n),amount:Expr::Const{value}}=>n=="chosen"&&value.as_u64().is_some_and(|n|(1..=65535).contains(&n)),
                            Effect::Damage{target:Target::Name(n),amount:Expr::Const{value}}=>r.card_type=="unit"&&n=="chosen"&&value.as_u64().is_some_and(|n|(1..=65535).contains(&n)),
                            _=>false,
                        }
                    },
                    _=>false,
                }
            });
            if !root || !(gain||use_force) {return Err("Force requires standalone ordinary play/death/event use and one bounded independent success continuation".into());}
        }

        if contains_weakness(&ability.effect) && !(depth==0 && scalar_weakness_registration(ability)) {
            fn primitive(e:&Effect)->bool {matches!(e,Effect::GiveWeakness{amount:Expr::Const{value},target:Target::Name(n)} if value.as_u64()==Some(1)&&n=="chosen")}
            let shape=primitive(&ability.effect) || matches!(&ability.effect,Effect::Conditional{condition:Expr::Binary{op,left,right},on_true,on_false} if op==">=" && matches!(left.as_ref(),Expr::Ref{path} if path.iter().map(String::as_str).collect::<Vec<_>>()==["player","resources","length"]) && matches!(right.as_ref(),Expr::Const{value} if value.as_u64().is_some_and(|n|n<=1000)) && primitive(on_true) && on_false.as_deref().is_some_and(|e|matches!(e,Effect::NoAction)));
            fn damaged_predicate(e:&Expr)->bool {matches!(e,Expr::Binary{op,left,right} if op=="&&" && matches!(left.as_ref(),Expr::Query{name,receiver,args} if name=="isUnit"&&args.is_empty()&&matches!(receiver.as_ref(),Expr::Ref{path} if path.len()==1&&path[0]=="card")) && matches!(right.as_ref(),Expr::Binary{op,left,right} if op=="!=="&&matches!(left.as_ref(),Expr::Ref{path} if path.iter().map(String::as_str).collect::<Vec<_>>()==["card","damage"])&&matches!(right.as_ref(),Expr::Const{value} if value.as_u64()==Some(0))))}
            let selector=ability.target_resolver.as_ref().is_some_and(|r| r.owner=="any" && r.card_type=="unit" && matches!(r.zone.as_deref(),Some("arena"|"anyAttackable")) && r.arena.is_none() && !r.optional && r.mode.is_none() && r.count.is_none() && r.prompt_title.is_none() && r.predicate.as_ref().map_or(true,damaged_predicate));
            if depth!=0 || !["whenPlayed","whenDefeated"].contains(&ability.trigger.as_str()) || !ability.optional || ability.requires_confirmation || ability.coordinate || !ability.context_title.is_empty() || !ability.costs.is_empty() || ability.condition.is_some() || ability.then.is_some() || ability.if_you_do.is_some() || !ability.target_resolvers.is_empty() || !shape || !selector {return Err("Weakness requires an optional standalone scalar unit play/death registration with amount one".into());}
        }

        if !ability.target_resolvers.is_empty() {
            if depth!=0 || !["event","whenPlayed","onAttack"].contains(&ability.trigger.as_str()) || ability.target_resolvers.len()!=2 || ability.target_resolver.is_some() || ability.optional || ability.requires_confirmation || ability.coordinate || !ability.context_title.is_empty() || !ability.costs.is_empty() || ability.condition.is_some() || ability.then.is_some() || ability.if_you_do.is_some() || !matches!(ability.effect,Effect::NoAction) {
                return Err("named selectors require two independent standalone event/WhenPlayed/OnAttack stat systems".into());
            }
            if ability.target_resolvers.iter().any(|d|d.chooser!="self") {
                let damage=ability.target_resolvers.iter().all(|d|matches!(&d.effect,Effect::Damage{target:Target::Name(n),amount:Expr::Const{value}} if n=="chosen"&&value.as_u64()==Some(3)));
                let defeat=ability.target_resolvers.iter().all(|d|matches!(&d.effect,Effect::Defeat{target:Target::Name(n)} if n=="chosen"));
                if !(damage&&ability.trigger=="whenPlayed" || defeat&&["whenPlayed","event"].contains(&ability.trigger.as_str())) {return Err("opponent named chooser requires ordinary play damage3 or play/event defeat pair".into());}
                let expected_type=if ability.trigger=="event" {"nonLeaderUnit"}else{"unit"};
                let expected_zone=if damage {"anyAttackable"}else{"arena"};
                let mut names=std::collections::HashSet::new();let mut effects=Vec::new();
                for (index,d) in ability.target_resolvers.iter().enumerate() {
                    let owner=if index==0 {"opponent"}else{"self"};let r=&d.target_resolver;
                    if d.name.is_empty() || ["target","constructor","prototype","__proto__"].contains(&d.name.as_str()) || !d.name.chars().next().is_some_and(|c|c.is_ascii_alphabetic()||c=='_') || !d.name.chars().all(|c|c.is_ascii_alphanumeric()||c=='_') || !names.insert(d.name.as_str()) || d.chooser!=owner || r.owner!=owner || r.card_type!=expected_type || r.zone.as_deref()!=Some(expected_zone) || r.arena.is_some() || r.optional || r.mode.is_some() || r.count.is_some() || r.predicate.is_some() || r.prompt_title.is_some() {return Err("opponent named chooser requires mandatory opposite/self scalar domains and default prompts".into());}
                    r.validate()?;effects.push(Effect::Select{target_resolver:r.clone(),effect:Box::new(d.effect.clone())});
                }
                let effect=Effect::Simultaneous{effects};self.validate_group(&effect,1)?;
                self.instructions.push(Instruction::NamedChooserGroup{effect,choosers:vec![NamedChooser::Opponent,NamedChooser::Own]});return Ok(());
            }
            let fervor=matches!(&ability.target_resolvers[0].effect,Effect::Ready{target:Target::Name(n)} if n=="chosen") && matches!(&ability.target_resolvers[1].effect,Effect::Damage{target:Target::Name(n),amount:Expr::Const{value}} if n=="chosen"&&value.as_u64()==Some(3));
            let sanctuary=matches!(&ability.target_resolvers[0].effect,Effect::Exhaust{target:Target::Name(n)} if n=="chosen") && matches!(&ability.target_resolvers[1].effect,Effect::GiveShield{target:Target::Name(n),amount:Expr::Const{value}} if n=="chosen"&&value.as_u64()==Some(1));
            fn minus(e:&Expr,n:u64)->bool {matches!(e,Expr::Unary{op,value} if op=="-" && matches!(value.as_ref(),Expr::Const{value} if value.as_u64()==Some(n)))}
            let mislead=matches!(&ability.target_resolvers[0].effect,Effect::GiveShield{target:Target::Name(n),amount:Expr::Const{value}} if n=="chosen"&&value.as_u64()==Some(1)) && matches!(&ability.target_resolvers[1].effect,Effect::ModifyStats{target:Target::Name(n),duration,power,hp} if n=="chosen"&&duration=="phase"&&minus(power,3)&&minus(hp,0));
            if (fervor||sanctuary||mislead) && ability.trigger!="event" {return Err("named mixed systems require standalone event".into());}
            let damage=ability.target_resolvers.iter().all(|d|matches!(d.effect,Effect::Damage{..}));
            if damage && !["whenPlayed","onAttack"].contains(&ability.trigger.as_str()) {return Err("named damage requires ordinary play/attack registration".into());}
            let unit_base_damage = named_unit_base_damage(ability);
            let mut names=std::collections::HashSet::new();
            let mut effects=Vec::new();
            for (index,descriptor) in ability.target_resolvers.iter().enumerate() {
                if descriptor.name.is_empty() || descriptor.name=="target" || !descriptor.name.chars().all(|c|c.is_ascii_alphanumeric()||c=='_') || !names.insert(descriptor.name.as_str()) || descriptor.chooser!="self" {return Err("named selectors require distinct ordinary names and own chooser".into());}
                let mut r=descriptor.target_resolver.clone();
                r.prompt_title=None;
                r.validate()?;
                if fervor||sanctuary||mislead {
                    let common=!r.optional && r.mode.is_none() && r.count.is_none() && matches!(r.zone.as_deref(),Some("arena"|"anyAttackable"));
                    let domain=if fervor||mislead {r.owner=="any" && r.card_type=="unit" && r.arena.is_none() && r.predicate.is_none()} else if index==0 {r.owner=="opponent" && r.card_type=="any" && r.zone.as_deref()==Some("arena") && r.arena.is_none() && r.predicate.is_none()} else {
                        r.owner=="self" && r.card_type=="any" && r.zone.as_deref()==Some("anyAttackable") && r.arena.is_none() && matches!(&r.predicate,Some(Expr::Binary{op,left,right}) if op=="&&" && matches!(left.as_ref(),Expr::Query{name,receiver,args} if name=="isUnit"&&args.is_empty()&&matches!(receiver.as_ref(),Expr::Ref{path} if path.len()==1&&path[0]=="card")) && matches!(right.as_ref(),Expr::Binary{op,left,right} if op=="<="&&matches!(left.as_ref(),Expr::Ref{path} if path.iter().map(String::as_str).collect::<Vec<_>>()==["card","cost"])&&matches!(right.as_ref(),Expr::Const{value} if value.as_u64()==Some(3))))
                    };
                    if !common||!domain || (sanctuary && descriptor.target_resolver.prompt_title.is_some()) {return Err("named mixed selector domain is unverified".into());}
                    let effect=if mislead && index==1 {Effect::ModifyStats{target:Target::Name("chosen".into()),duration:"phase".into(),power:Expr::number(-3),hp:Expr::number(0)}} else {descriptor.effect.clone()};
                    effects.push(Effect::Select{target_resolver:r,effect:Box::new(effect)});continue;
                }
                if unit_base_damage {
                    effects.push(Effect::Select { target_resolver: r, effect: Box::new(descriptor.effect.clone()) });
                    continue;
                }
                if damage {
                    if r.owner!=if index==0 {"self"}else{"opponent"} || r.card_type!="unit" || r.optional || r.mode.is_some() || r.count.is_some() || !matches!(r.zone.as_deref(),Some("arena"|"anyAttackable")) || r.arena.as_deref().is_some_and(|a|!["ground","space"].contains(&a)) || r.predicate.is_some() || descriptor.target_resolver.prompt_title.is_some() {return Err("named damage requires mandatory own/opponent scalar arena units with default prompts".into());}
                    let Effect::Damage{target:Target::Name(n),amount:Expr::Const{value}}=&descriptor.effect else {return Err("named damage requires literal chosen amounts".into());};
                    if n!="chosen" || !value.as_u64().is_some_and(|n|(1..=2).contains(&n)) {return Err("named damage requires one or two ability damage".into());}
                    effects.push(Effect::Select{target_resolver:r,effect:Box::new(descriptor.effect.clone())});
                    continue;
                }
                if r.owner!="any" || r.card_type!="unit" || (!r.optional && !(ability.trigger=="onAttack" && index==0)) || r.mode.is_some() || r.count.is_some() || !matches!(r.zone.as_deref(),Some("arena"|"anyAttackable")) || r.arena.as_deref().is_some_and(|a|!["ground","space"].contains(&a)) {return Err("named selectors require optional scalar any-owner arena units".into());}
                if ability.trigger=="onAttack" && r.predicate.is_some() {return Err("named attack selectors require unconditional arena domains".into());}
                if let Some(predicate)=&r.predicate {
                    if !matches!(predicate,Expr::Query{name,receiver,args} if name=="hasSomeTrait" && matches!(receiver.as_ref(),Expr::Ref{path} if path.len()==1&&path[0]=="card") && args.len()==1 && matches!(&args[0],Expr::Const{value} if value.as_str().is_some_and(|v|!v.is_empty()))) {return Err("named selector predicates require one literal unit trait".into());}
                }
                fn literal(e:&Expr,negative:bool)->Option<Expr> {
                    match e {Expr::Const{value} if value.as_i64().is_some_and(|v|(-65535..=65535).contains(&v))=>Some(e.clone()),Expr::Unary{op,value} if negative && op=="-"=>match value.as_ref(){Expr::Const{value}=>value.as_u64().filter(|v|*v<=65535).map(|v|Expr::Const{value:Value::from(-(v as i64))}),_=>None},_=>None}
                }
                let effect=match &descriptor.effect {
                    Effect::ModifyStats{target:Target::Name(n),duration,power,hp} if n=="chosen" && duration=="phase"=>Effect::ModifyStats{target:Target::Name(n.clone()),duration:duration.clone(),power:literal(power,ability.trigger=="onAttack").ok_or("named selectors require literal scalar phase power")?,hp:literal(hp,ability.trigger=="onAttack").ok_or("named selectors require literal scalar phase HP")?},
                    _=>return Err("named selectors require literal scalar phase stats".into()),
                };
                effects.push(Effect::Select{target_resolver:r,effect:Box::new(effect)});
            }
            let effect=Effect::Simultaneous{effects};
            self.validate_group(&effect,1)?;
            self.instructions.push(Instruction::NamedGroup(effect));
            return Ok(());
        }

        if matches!(&ability.effect,Effect::GainKeyword {target:Target::ArenaList(_),..}|Effect::LoseKeyword {target:Target::ArenaList(_),..}) {
            if depth!=0 || !["event","whenPlayed"].contains(&ability.trigger.as_str()) || ability.optional || ability.target_resolver.is_some() || ability.condition.is_some() || ability.then.is_some() || ability.if_you_do.is_some() || ability.coordinate || !ability.context_title.is_empty() {
                return Err("arena-list keywords require a standalone event or WhenPlayed registration".into());
            }
        }
        if ability.target_resolver.as_ref().is_some_and(|r|r.is_upgrade_domain() && (r.is_multiple() || !scalar_upgrade_operation(&ability.effect,r.zone.as_deref()==Some("discard")))) {return Err("upgrade selection supports only scalar defeat or return to hand".into());}
        if depth > MAX_ABILITY_NESTING {
            return Err("ability continuation nesting exceeds capacity".into());
        }
        if depth > 0
            && (ability.coordinate || !ability.context_title.is_empty()
                || !ability.costs.is_empty()
                || ability.optional
                || !ability.trigger.is_empty()
                || ability.condition.is_some()
                || ability.requires_confirmation)
        {
            return Err(
                "nested continuation costs, optional triggers and registrations are unsupported"
                    .into(),
            );
        }
        fn uses_group(effect: &Effect) -> bool {
            match effect { Effect::Simultaneous { .. } | Effect::PhaseModifiers { .. } => true,
                Effect::Select {target_resolver,effect} => target_resolver.is_multiple() || uses_group(effect),
                Effect::Conditional {on_true,on_false,..} => uses_group(on_true) || on_false.as_deref().is_some_and(uses_group),
                Effect::Sequence {effects} => effects.iter().any(uses_group), _ => false }
        }
        let multiple = ability.target_resolver.as_ref().is_some_and(TargetResolver::is_multiple);
        if (multiple || uses_group(&ability.effect)) && (ability.then.is_some() || ability.if_you_do.is_some()) { return Err("event groups and array selections cannot have unverified continuations".into()); }
        if let Some(target) = &ability.target_resolver {
            target.validate()?;
            self.instructions.push(Instruction::Select(target.clone()));
        }
        if multiple {
            self.validate_group(&ability.effect, ability.target_resolver.as_ref().unwrap().max_targets()?)?;
            self.instructions.push(Instruction::Group(ability.effect.clone()));
        } else if arena_token_kind(ability).is_some() {
            self.instructions.push(Instruction::Apply(ability.effect.clone()));
        } else if conditional_arena_damage_ability(ability) {
            // The complete root shape proves this one leaf is standalone.
            // Do not relax the general nested-arena depth guard.
            let Effect::Conditional{condition,on_true,on_false}=&ability.effect else{unreachable!()};
            condition.validate()?;
            let start=self.instructions.len();self.instructions.push(Instruction::Conditional{condition:condition.clone(),skip:0});
            self.compile_effect(on_true,0)?;
            let jump=self.instructions.len();self.instructions.push(Instruction::Jump{to:0});
            self.instructions[start]=Instruction::Conditional{condition:condition.clone(),skip:self.instructions.len()};
            self.compile_effect(on_false.as_deref().expect("validated NoAction"),depth+1)?;
            self.instructions[jump]=Instruction::Jump{to:self.instructions.len()};
        } else { self.compile_effect(&ability.effect, depth)?; }
        if let Some(next) = &ability.then {
            self.compile_ability(next, depth + 1)?;
        }
        if let Some(next) = &ability.if_you_do {
            let start = self.instructions.len();
            self.instructions.push(Instruction::IfResult { skip: 0 });
            self.compile_ability(next, depth + 1)?;
            self.instructions[start] = Instruction::IfResult {
                skip: self.instructions.len(),
            };
        }
        Ok(())
    }
    fn scalar_hand_amount(amount:&Expr,target:&Target)->Result<usize,String> {
        if !matches!(target,Target::Name(name) if ["ownPlayer","enemyPlayer","chosenPlayer"].contains(&name.as_str())) {return Err("hand effects require a scalar player target".into());}
        match amount.original() {Expr::Const {value}=>value.as_u64().filter(|n|(1..=2).contains(n)).map(|n|n as usize).ok_or("hand effect amount must be 1..2".into()),_=>Err("hand effects require literal bounded amounts".into())}
    }
    fn scalar_deck_amount(amount:&Expr,target:&Target)->Result<usize,String> {
        if !matches!(target,Target::Name(name) if ["ownPlayer","enemyPlayer"].contains(&name.as_str())) {return Err("deck discard requires a scalar player target".into());}
        match amount.original() {Expr::Const {value}=>value.as_u64().filter(|n|*n<=5).map(|n|n as usize).ok_or("deck discard amount must be 0..5".into()),_=>Err("deck discard requires a literal bounded amount".into())}
    }
    fn token_unit_amount(token_type:&str,amount:&Expr,target:&Target)->Result<usize,String> {
        if !["battleDroid","cloneTrooper","beast","spy","tieFighter","xwing","mandalorian"].contains(&token_type) {return Err("unsupported created token unit type".into());}
        if !matches!(target,Target::Name(name) if ["ownPlayer","enemyPlayer"].contains(&name.as_str())) {return Err("token creation requires a scalar player target".into());}
        match amount.original() {Expr::Const {value}=>value.as_u64().filter(|n|(1..=5).contains(n)).map(|n|n as usize).ok_or("token creation amount must be 1..5".into()),_=>Err("token creation requires a literal bounded amount".into())}
    }
    fn validate_phase_modifiers(&self, effects: &[Effect]) -> Result<(), String> {
        if effects.is_empty() || effects.len() > 8 { return Err("phase modifier list exceeds bounded capacity".into()); }
        let mut shared_target = None;
        for effect in effects {
            let (target, duration) = match effect {
                Effect::ModifyStats {target, duration, ..} | Effect::GainKeyword {target, duration, ..} | Effect::LoseKeyword {target, duration, ..} => (target, duration),
                _ => return Err("phase modifier lists contain only primitive lasting modifiers".into()),
            };
            if duration != "phase" { return Err("phase modifier list requires phase duration".into()); }
            let encoded = serde_json::to_value(target).map_err(|e| e.to_string())?;
            if shared_target.as_ref().is_some_and(|prior| prior != &encoded) { return Err("phase modifier list must share one target".into()); }
            shared_target = Some(encoded);
            let mut checker = self.clone(); checker.instructions.clear(); checker.compile_effect(effect, 0)?;
        }
        Ok(())
    }
    fn validate_group(&self, effect: &Effect, targets: usize) -> Result<(), String> {
        // Only the reviewed root pair captures non-arena ResourceCard events.
        if resource_pair_effect(effect) { return Ok(()); }
        fn dependent_in_scope(e: &Expr, card_bound: bool) -> bool {
            match e {
                Expr::Compiled(compiled) => dependent_in_scope(&compiled.original,card_bound),
                Expr::NativeTokenPredicate{receiver,..}=>dependent_in_scope(receiver,card_bound),
 Expr::Collection{name,receiver,args,..}=>dependent_in_scope(receiver,card_bound)||args.iter().any(|arg|dependent_in_scope(arg,name!="includes"||card_bound)),Expr::ArenaList{receiver,predicate}|Expr::ArenaFilter{receiver,predicate}|Expr::ArenaSome{receiver,predicate}=>dependent_in_scope(receiver,card_bound)||dependent_in_scope(predicate,true),Expr::ArenaLength{receiver}=>dependent_in_scope(receiver,card_bound),
                Expr::Ref {path} => matches!(path.first().map(String::as_str), Some("target" | "targets")) || (!card_bound && path.first().is_some_and(|p|p=="card")),
                Expr::Array {items} => items.iter().any(|e|dependent_in_scope(e,card_bound)),
                Expr::Math {args, ..} => args.iter().any(|e|dependent_in_scope(e,card_bound)),
                Expr::Unary {value, ..} => dependent_in_scope(value,card_bound),
                Expr::Binary {left, right, ..} => dependent_in_scope(left,card_bound) || dependent_in_scope(right,card_bound),
                Expr::Conditional {condition, then, r#else} => dependent_in_scope(condition,card_bound) || dependent_in_scope(then,card_bound) || dependent_in_scope(r#else,card_bound),
                Expr::Query {name,receiver,args} => dependent_in_scope(receiver,card_bound) || args.iter().enumerate().any(|(i,e)|dependent_in_scope(e,card_bound || (i==0 && ["hasSomeArenaUnit","arenaUnitCount"].contains(&name.as_str())))),
                Expr::DataMember {..} => false,
                Expr::HistoryQuery(q) => !card_bound && q.kind()==Ok(crate::phase_history::QueryKind::EnteredCard),
                Expr::HistoryAttackCheck {..} | Expr::Const { .. } => false,
            }
        }
        fn target_dependent(e: &Expr) -> bool { dependent_in_scope(e,false) }
        fn multiplicity(target: &Target, selected: usize, array: bool) -> Result<usize, String> {
            fn card_expr(e: &Expr, array: bool) -> Result<(), String> {
                match e {
                    Expr::Ref {path} => match path.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
                        ["source"] | ["player" | "opponent", "base"] => Ok(()),
                        ["target"] if !array => Ok(()),
                        _ => Err("group expression targets require scalar card references; array target and player references are unverified".into()),
                    },
                    Expr::Conditional {condition,then,r#else} => { if array && target_dependent(condition) { return Err("array-dependent expression targets are unsupported".into()); } card_expr(then,array)?; card_expr(r#else,array) },
                    _ => Err("unsupported group card expression target".into()),
                }
            }
            match target {
                Target::Name(name) => match name.as_str() {
                    "chosen" => Ok(selected), "bothBases" => Ok(2), "self" | "ownBase" | "enemyBase" => Ok(1),
                    _ => Err("card event groups do not support player targets".into()),
                },
                Target::Expression(e) => { card_expr(e,array)?; Ok(1) },
                Target::ArenaList(_)=>Err("grouped arena lists require separate event-capacity witnesses".into()),
            }
        }
        fn walk(p: &Program, effect: &Effect, targets: usize, array: bool, depth: usize) -> Result<(usize, bool, bool), String> {
            if depth > MAX_ABILITY_NESTING { return Err("event group nesting exceeds capacity".into()); }
            let (records, damage, newly_attached) = match effect {
                Effect::Simultaneous {effects} => {
                    if effects.is_empty() || effects.len() > 8 { return Err("simultaneous group requires one to eight children".into()); }
                    let mut records = 0usize; let mut damage = false; let mut new = false;
                    for child in effects { let (n,d,a) = walk(p, child, targets, array, depth + 1)?; records = records.checked_add(n).ok_or("event group capacity overflow")?; damage |= d; new |= a; }
                    (records, damage, new)
                },
                Effect::Conditional {condition, on_true, on_false} => {
                    condition.validate()?;
                    if array && target_dependent(condition) { return Err("array-dependent group conditional is unsupported".into()); }
                    let a = walk(p,on_true,targets,array,depth+1)?;
                    let b = if let Some(other)=on_false { walk(p,other,targets,array,depth+1)? } else { (0,false,false) };
                    (a.0.max(b.0),a.1||b.1,a.2||b.2)
                },
                Effect::Select {target_resolver, effect} => {
                    if target_resolver.is_upgrade_domain() {return Err("grouped upgrade selection is not modeled".into());}
                    target_resolver.validate_inline()?;
                    if array && target_resolver.predicate.as_ref().is_some_and(|e|dependent_in_scope(e,true)) { return Err("array-dependent nested selection is unsupported".into()); }
                    // Nested selection replaces the outer selected value, not its multiplicity.
                    walk(p,effect,target_resolver.max_targets()?,target_resolver.is_multiple(),depth+1)?
                },
                Effect::PhaseModifiers {effects} => {
                    p.validate_phase_modifiers(effects)?;
                    if array && effects.iter().any(|e| match e { Effect::ModifyStats {power,hp,..} => target_dependent(power)||target_dependent(hp), Effect::GainKeyword {amount,..} => target_dependent(amount), _ => false }) { return Err("array-dependent phase modifier values are unsupported".into()); }
                    let mut records=0usize;
                    for e in effects { let target=match e { Effect::ModifyStats {target,..}|Effect::GainKeyword {target,..}|Effect::LoseKeyword {target,..}=>target,_=>unreachable!() }; records+=multiplicity(target,targets,array)?; }
                    (records,false,true)
                },
                Effect::DiscardOwnHand {amount,target} => (Program::scalar_hand_amount(amount,target)?+1,false,false),
                Effect::DiscardDeck {amount,target} => (Program::scalar_deck_amount(amount,target)?+1,false,false),
                Effect::Draw {amount,target} => {Program::scalar_hand_amount(amount,target)?;(2,false,false)},
                Effect::CreateTokenUnit {token_type,amount,target,..} => (Program::token_unit_amount(token_type,amount,target)?+1,false,true),
                Effect::ResourceCard { .. } | Effect::Sequence { .. } | Effect::ReturnToHand { .. } | Effect::ReadyResources { .. } | Effect::DistributeHealing { .. } | Effect::LookAt { .. } | Effect::LookAtAndSelectCard { .. } => return Err("event group contains an unverified operation".into()),
                Effect::NoAction => (0,false,false),
                primitive => {
                    let mut checker=p.clone(); checker.instructions.clear(); checker.compile_effect(primitive,depth+1)?;
                    let amount = match primitive { Effect::Damage {amount,..} | Effect::Heal {amount,..} | Effect::GiveShield {amount,..} | Effect::GiveExperience {amount,..} | Effect::GiveWeakness {amount,..} | Effect::GiveAdvantage {amount,..} | Effect::GainKeyword {amount,..} => Some(amount), _ => None };
                    if array && amount.is_some_and(target_dependent) { return Err("array-dependent group amount is unsupported".into()); }
                    if array && matches!(primitive, Effect::ModifyStats {power,hp,..} if target_dependent(power)||target_dependent(hp)) { return Err("array-dependent phase modifier values are unsupported".into()); }
                    let destination=match primitive { Effect::Damage {target,..}|Effect::Heal {target,..}|Effect::GiveShield {target,..}|Effect::GiveExperience {target,..}|Effect::GiveWeakness {target,..}|Effect::GiveAdvantage {target,..}|Effect::Ready {target}|Effect::Exhaust {target}|Effect::Defeat {target}|Effect::ModifyStats {target,..}|Effect::GainKeyword {target,..}|Effect::LoseKeyword {target,..} => target, _ => unreachable!() };
                    let expanded=multiplicity(destination,targets,array)?;
                    match primitive {
                        Effect::GiveShield {amount,..} | Effect::GiveExperience {amount,..} | Effect::GiveWeakness {amount,..} | Effect::GiveAdvantage {amount,..} => {
                            let Expr::Const {value} = amount else { return Err("group tokens require literal bounded amounts".into()); };
                            let n=value.as_u64().filter(|n| (1..=2).contains(n)).ok_or("group tokens require one or two per target")? as usize;
                            (expanded * n * 2,false,true)
                        },
                        Effect::Damage { .. } => (expanded,true,false),
                        Effect::ModifyStats { .. } | Effect::GainKeyword { .. } | Effect::LoseKeyword { .. } => (expanded,false,true),
                        _ => (expanded,false,false),
                    }
                }
            };
            if records > MAX_GROUP_RECORDS { return Err("event group record budget exceeds capacity".into()); }
            if damage && newly_attached { return Err("mixed damage and newly attached tokens/lasting modifiers require dedicated engine witnesses".into()); }
            fn has_hand_effect(e:&Effect)->bool {match e {Effect::Draw{..}|Effect::DiscardOwnHand{..}|Effect::DiscardDeck{..}=>true,Effect::Simultaneous{effects}|Effect::PhaseModifiers{effects}=>effects.iter().any(has_hand_effect),Effect::Conditional{on_true,on_false,..}=>has_hand_effect(on_true)||on_false.as_deref().is_some_and(has_hand_effect),Effect::Select{effect,..}=>has_hand_effect(effect),_=>false}}
            if damage && has_hand_effect(effect) {return Err("mixed damage and hand effects require dedicated engine witnesses".into());}
            Ok((records,damage,newly_attached))
        }
        fn operation_budget(e: &Effect, depth: usize) -> Result<usize,String> {
            if depth > MAX_ABILITY_NESTING { return Err("event group nesting exceeds capacity".into()); }
            let n=match e {
                Effect::Simultaneous {effects}|Effect::PhaseModifiers {effects} => { let mut sum=0usize;for e in effects {sum+=operation_budget(e,depth+1)?;if sum>MAX_GROUP_OPS {return Err("event group operation budget exceeds capacity".into());}}sum },
                Effect::Conditional {on_true,on_false,..} => 2+operation_budget(on_true,depth+1)?+if let Some(e)=on_false {operation_budget(e,depth+1)?}else{0},
                Effect::Select {effect,..} => 2+operation_budget(effect,depth+1)?,
                Effect::NoAction=>0,_=>1,
            };
            if n>MAX_GROUP_OPS {return Err("event group operation budget exceeds capacity".into());}Ok(n)
        }
        fn contains_select(e: &Effect) -> bool { match e { Effect::Select {..}=>true,Effect::Simultaneous {effects}|Effect::PhaseModifiers {effects}|Effect::Sequence {effects}=>effects.iter().any(contains_select),Effect::Conditional {on_true,on_false,..}=>contains_select(on_true)||on_false.as_deref().is_some_and(contains_select),_=>false } }
        if contains_weakness(effect)&&contains_select(effect)&&!native_single_weakness_group(effect) {return Err("selected Weakness group requires the bounded native opponent single-card contract".into());}
        fn primitive_captures_target(e: &Effect) -> bool {
            fn target(t: &Target) -> bool { match t { Target::Name(n)=>n=="chosen",Target::Expression(e)=>target_dependent(e),Target::ArenaList(_)=>true } }
            match e {
                Effect::Damage {target:t,amount}|Effect::Heal {target:t,amount}|Effect::Draw {target:t,amount}|Effect::GiveShield {target:t,amount}|Effect::GiveExperience {target:t,amount}|Effect::ReadyResources {target:t,amount}|Effect::GainKeyword {target:t,amount,..}=>target(t)||target_dependent(amount),
                Effect::ModifyStats {target:t,power,hp,..}=>target(t)||target_dependent(power)||target_dependent(hp),
                Effect::Ready {target:t}|Effect::Exhaust {target:t}|Effect::Defeat {target:t}|Effect::ReturnToHand {target:t}|Effect::LoseKeyword {target:t,..}=>target(t),
                _=>false,
            }
        }
        fn validate_shared_target(e: &Effect, changed: bool) -> Result<(),String> {
            match e {
                // The new selection supplies a fresh target for its child; lexical factory captures are rejected by the compiler.
                Effect::Select {effect,..}=>validate_shared_target(effect,false),
                Effect::Simultaneous {effects}|Effect::PhaseModifiers {effects}=>{let mut changed=changed;for e in effects {validate_shared_target(e,changed)?;changed|=contains_select(e);}Ok(())},
                Effect::Conditional {condition,on_true,on_false}=>{
                    if changed&&target_dependent(condition) {return Err("target-dependent sibling after nested selection requires capture semantics".into());}
                    validate_shared_target(on_true,changed)?;if let Some(e)=on_false {validate_shared_target(e,changed)?;}Ok(())
                },
                primitive=>{
                    if changed {
                        if primitive_captures_target(primitive) {return Err("target-dependent sibling after nested selection requires capture semantics".into());}
                    }Ok(())
                }
            }
        }
        fn unique_draw_players(e:&Effect,seen:&mut Vec<String>)->Result<(),String> {
            match e {
                Effect::Draw {target:Target::Name(player),..}=>{if seen.contains(player) {return Err("multiple grouped draws for one player require separate contingent witnesses".into());}seen.push(player.clone());}
                Effect::Simultaneous {effects}|Effect::PhaseModifiers {effects}=>for e in effects {unique_draw_players(e,seen)?;},
                Effect::Conditional {on_true,on_false,..}=>{unique_draw_players(on_true,seen)?;if let Some(e)=on_false {unique_draw_players(e,seen)?;}},
                Effect::Select {effect,..}=>unique_draw_players(effect,seen)?,_=>{},
            }Ok(())
        }
        unique_draw_players(effect,&mut vec![])?;
        operation_budget(effect,0)?;
        validate_shared_target(effect,false)?;
        let array=self.instructions.iter().rev().find_map(|i|if let Instruction::Select(r)=i {Some(r.is_multiple())}else{None}).unwrap_or(false);
        walk(self,effect,targets,array,0)?;
        #[derive(Clone)]
        enum Domain { Source, Base(Option<bool>), Unit(Option<bool>, bool), Unknown }
        fn source_excluded(e: &Expr) -> bool {
            fn path(e: &Expr, root: &str) -> bool { matches!(e,Expr::Ref {path} if path.len()==1 && path[0]==root) }
            match e {
                Expr::Binary {op,left,right} if op == "!==" => (path(left,"card")&&path(right,"source"))||(path(right,"card")&&path(left,"source")),
                Expr::Binary {op,left,right} if op == "&&" => source_excluded(left)||source_excluded(right),
                _ => false,
            }
        }
        fn selected_domain(r: Option<&TargetResolver>) -> Domain {
            let Some(r)=r else { return Domain::Unknown; };
            let owner=match r.owner.as_str() { "self"=>Some(false),"opponent"=>Some(true),_=>None };
            if r.card_type=="base" || r.zone.as_deref()==Some("base") { Domain::Base(owner) }
            else if ["unit","nonLeaderUnit","leaderUnit"].contains(&r.card_type.as_str()) || r.zone.as_deref()==Some("arena") { Domain::Unit(owner,r.predicate.as_ref().is_some_and(source_excluded)) }
            else { Domain::Unknown }
        }
        fn domains(t: &Target, selected: &Domain) -> Vec<Domain> {
            match t {
                Target::Name(n) => match n.as_str() { "self"=>vec![Domain::Source],"ownBase"=>vec![Domain::Base(Some(false))],"enemyBase"=>vec![Domain::Base(Some(true))],"bothBases"=>vec![Domain::Base(Some(false)),Domain::Base(Some(true))],"chosen"=>vec![selected.clone()],_=>vec![Domain::Unknown] },
                Target::Expression(Expr::Ref {path}) => match path.iter().map(String::as_str).collect::<Vec<_>>().as_slice() { ["source"]=>vec![Domain::Source],["player","base"]=>vec![Domain::Base(Some(false))],["opponent","base"]=>vec![Domain::Base(Some(true))],["target"]=>vec![selected.clone()],_=>vec![Domain::Unknown] },
                Target::Expression(Expr::Conditional {then,r#else,..}) => { let mut result=domains(&Target::Expression(*then.clone()),selected);result.extend(domains(&Target::Expression(*r#else.clone()),selected));result },
                _=>vec![Domain::Unknown],
            }
        }
        fn disjoint(a: &Domain,b: &Domain) -> bool {
            match (a,b) {
                (Domain::Base(_),Domain::Unit(..)) | (Domain::Unit(..),Domain::Base(_)) => true,
                (Domain::Base(Some(a)),Domain::Base(Some(b))) | (Domain::Unit(Some(a),_),Domain::Unit(Some(b),_)) => a!=b,
                (Domain::Source,Domain::Unit(owner,excluded)) | (Domain::Unit(owner,excluded),Domain::Source) => *excluded || *owner==Some(true),
                (Domain::Source,Domain::Base(Some(true))) | (Domain::Base(Some(true)),Domain::Source) => true,
                _=>false,
            }
        }
        fn damage_sets(e: &Effect,selected:&Domain,out:&mut Vec<Vec<Domain>>,defeat:&mut bool) {
            match e {
                Effect::Damage {target,..}=>out.push(domains(target,selected)),
                Effect::Defeat {..}=>*defeat=true,
                Effect::Simultaneous {effects}|Effect::PhaseModifiers {effects}=>for e in effects {damage_sets(e,selected,out,defeat)},
                Effect::Select {target_resolver,effect}=>damage_sets(effect,&selected_domain(Some(target_resolver)),out,defeat),
                Effect::Conditional {on_true,on_false,..}=>{damage_sets(on_true,selected,out,defeat);if let Some(e)=on_false {damage_sets(e,selected,out,defeat)}},
                _=>{},
            }
        }
        let selected=self.instructions.iter().rev().find_map(|i|if let Instruction::Select(r)=i {Some(r)}else{None});
        let mut sets=vec![];let mut defeat=false;damage_sets(effect,&selected_domain(selected),&mut sets,&mut defeat);
        if !sets.is_empty()&&defeat { return Err("mixed damage and explicit defeat groups require dedicated witnesses".into()); }
        for i in 0..sets.len() { for j in i+1..sets.len() {
            if sets[i].iter().any(|a|sets[j].iter().any(|b|!disjoint(a,b))) { return Err("group damage destinations must be provably disjoint until repeated-damage replacement semantics are witnessed".into()); }
        } }
        Ok(())
    }
    fn compile_effect(&mut self, effect: &Effect, depth: usize) -> Result<(), String> {
        if depth > MAX_ABILITY_NESTING {
            return Err("effect nesting exceeds capacity".into());
        }
        match effect {
            Effect::InitiateAttack{properties,..} => {if depth!=0 || !["event","whenPlayed","action"].contains(&self.trigger.as_str()) {return Err("native attack requires a standalone ordinary registration".into());}properties.validate()?;self.instructions.push(Instruction::Apply(effect.clone()));}

            Effect::GainTrait{target,trait_name,duration}|Effect::LoseTrait{target,trait_name,duration} => {
                if self.trigger!="constant" || duration!="persistent" || !matches!(target,Target::Name(n) if ["self","parent"].contains(&n.as_str())) || trait_name.is_empty() || trait_name.len()>64 {
                    return Err("trait leaves require bounded persistent self/parent constant registrations".into());
                }
                self.instructions.push(Instruction::Apply(effect.clone()));
            }
            Effect::ConstantEffectRegistration(r) => {if depth!=0 || self.trigger!="constant" {return Err("static self ongoing contract must be a root constant".into());}r.validate()?;self.instructions.push(Instruction::Apply(effect.clone()));}
            Effect::DiscardDeck {amount,target} => {Self::scalar_deck_amount(amount,target)?;self.instructions.push(Instruction::Group(effect.clone()));}
            Effect::DiscardOwnHand {amount,target} => {Self::scalar_hand_amount(amount,target)?;self.instructions.push(Instruction::Group(effect.clone()));}
            Effect::CreateTokenUnit {token_type,amount,target,..} => {Self::token_unit_amount(token_type,amount,target)?;self.instructions.push(Instruction::Group(effect.clone()));}
            Effect::Sequence { effects } => { for effect in effects { self.compile_effect(effect, depth + 1)?; } }
            Effect::Conditional { condition, on_true, on_false } => {
                condition.validate()?;let start=self.instructions.len();self.instructions.push(Instruction::Conditional {condition:condition.clone(),skip:0});
                self.compile_effect(on_true,depth+1)?;let jump=self.instructions.len();self.instructions.push(Instruction::Jump{to:0});
                self.instructions[start]=Instruction::Conditional{condition:condition.clone(),skip:self.instructions.len()};
                if let Some(other)=on_false{self.compile_effect(other,depth+1)?;}else{self.instructions.push(Instruction::Apply(Effect::NoAction));}
                self.instructions[jump]=Instruction::Jump{to:self.instructions.len()};
            }
            Effect::Simultaneous { effects } if self.trigger == "constant" => { for effect in effects { self.compile_effect(effect, depth + 1)?; } }
            Effect::Simultaneous { .. } => { self.validate_group(effect,1)?; self.instructions.push(Instruction::Group(effect.clone())); }
            Effect::PhaseModifiers {effects:_} => { self.validate_group(effect,1)?; self.instructions.push(Instruction::Apply(effect.clone())); }
            Effect::DistributeHealing { amount, max_targets, target_resolver, .. } => {
                amount.validate_number()?;target_resolver.validate()?;
                if *max_targets != 1 { return Err("distribution runtime currently supports exactly one selected target".into()); }
                if !matches!(amount, Expr::Const{value} if value.as_u64().is_some_and(|n|n<=3)){return Err("distribution runtime currently supports up to three healing".into());}
                self.instructions.push(Instruction::Apply(effect.clone()));
            }
            Effect::ReadyResources {amount,target} => {
                amount.validate_number()?;target.validate()?;
                if matches!(target,Target::Name(name) if name=="bothPlayers") {return Err("multi-player resource ready requires verified event batching".into());}
                if matches!(target, Target::Name(name) if name == "chosenPlayer")
                    && !matches!(
                        amount.original(),
                        Expr::Const { value } if value.as_u64().is_some_and(|n| (1..=2).contains(&n))
                    )
                {
                    return Err("chosen-player resource ready amount must be literal 1..2".into());
                }
                self.instructions.push(Instruction::Apply(effect.clone()));
            }
            Effect::SelectedPlayerSystem {
                system,
                target,
                properties,
                defaults,
                default_target_override,
            } => {
                let lowered = crate::selected_player::lower_fields(
                    system,
                    target,
                    properties,
                    defaults,
                    default_target_override,
                )?;
                self.compile_effect(&lowered, depth)?;
            }
            Effect::LookAt { .. } => {
                if depth != 0 || !crate::view_cards::reviewed_look_at(effect) {
                    return Err(
                        "lookAt supports only reviewed view-only opponent-hand display prompts"
                            .into(),
                    );
                }
                self.instructions.push(Instruction::Apply(effect.clone()));
            }
            Effect::LookAtAndSelectCard { .. } => {
                if depth != 0 || !crate::view_cards::reviewed_look_at_select(effect) {
                    return Err(
                        "lookAtAndSelectCard supports only reviewed opponent-hand discard prompts"
                            .into(),
                    );
                }
                self.instructions.push(Instruction::Apply(effect.clone()));
            }
            Effect::Damage{target:Target::ArenaList(list),amount:Expr::Const{value}}=>{
                if depth!=0 || !(if list.controller_order.is_some() {["event","whenDefeated"].contains(&self.trigger.as_str())}else{["whenPlayed","onAttack","whenDefeated"].contains(&self.trigger.as_str())}) || !value.as_u64().is_some_and(|n|(1..=3).contains(&n)) {return Err("arena damage requires literal standalone unit damage1..3".into());}
                if list.controller_order.is_some(){list.validate_ordered_ground()?;}else{list.validate_damage()?;}self.instructions.push(Instruction::Group(effect.clone()));
            }
            Effect::Damage { amount, target } | Effect::Heal { amount, target } | Effect::Draw { amount, target } | Effect::GiveShield { amount, target } | Effect::GiveExperience { amount, target } => { amount.validate()?; if amount.contains_typed_query() {amount.validate_number()?;} if amount.references_title() { return Err("card title metadata is not an effect amount".into()); } target.validate()?;self.instructions.push(Instruction::Apply(effect.clone())); }
            Effect::Select { target_resolver, effect } => {
                if target_resolver.is_upgrade_domain() && (target_resolver.is_multiple() || !scalar_upgrade_operation(effect,target_resolver.zone.as_deref()==Some("discard"))) {return Err("upgrade selection supports only scalar defeat or return to hand".into());}
                target_resolver.validate_inline()?; self.instructions.push(Instruction::Commit); let at=self.instructions.len(); self.instructions.push(Instruction::Select(target_resolver.clone()));
                if target_resolver.is_multiple() { self.validate_group(effect,target_resolver.max_targets()?)?; self.instructions.push(Instruction::Group(*effect.clone())); }
                else { self.compile_effect(effect,depth+1)?; }
                self.select_ends.push((at, self.instructions.len()));
            }
            Effect::UseForce {target}|Effect::GainForce{target} => {if !matches!(target,Target::Name(n) if n=="ownPlayer") {return Err("Force requires default own player target".into());}self.instructions.push(Instruction::Apply(effect.clone()));}
            Effect::GiveAdvantage {amount,target,..} => {crate::advantage::validate_amount(amount)?;if !matches!(target,Target::Name(n) if n=="self"||n=="chosen") {return Err("Advantage supports self or selected unit targets".into());}self.instructions.push(Instruction::Apply(effect.clone()));}
            Effect::GiveWeakness {amount,target} => {if !matches!(amount,Expr::Const{value} if value.as_u64()==Some(1)) || !matches!(target,Target::Name(n) if n=="chosen") {return Err("Weakness supports one selected unit token".into());}self.instructions.push(Instruction::Apply(effect.clone()));}
            Effect::NoAction => self.instructions.push(Instruction::Apply(effect.clone())),
            Effect::Exhaust {target:Target::ArenaList(list)} => {
                if depth!=0 || self.trigger!="whenPlayed" {return Err("ordered arena exhaust requires standalone unit entry".into());}
                list.validate_ordered_ground()?;self.instructions.push(Instruction::Apply(effect.clone()));
            }
            Effect::Defeat {target:Target::Expression(e)} if e.attachment_defeat_filter().is_some() => {
                e.validate()?;self.instructions.push(Instruction::Apply(effect.clone()));
            }
            Effect::Defeat {target:Target::ArenaList(list)} => {
                if depth!=0 {return Err("nested arena-list defeat requires explicit timing witnesses".into());}
                list.validate_scope(true)?;self.instructions.push(Instruction::Apply(effect.clone()));
            }
            Effect::ResourceCard {target} => {if !matches!(target,Target::Name(n) if n=="self"||n=="ownTopDeck") {return Err("resourceCard supports self or the reviewed own top-deck target only".into());} self.instructions.push(Instruction::Apply(effect.clone()));}
            Effect::ReturnToHand { target } => { target.validate()?; self.instructions.push(Instruction::Apply(effect.clone())); }
            Effect::ModifyStats { target, power, hp, duration } => {
                if matches!(target,Target::ArenaList(_)) && depth!=0 {return Err("nested arena-list modifiers require explicit continuation witnesses".into());}
                if duration != "phase" && !(self.trigger=="constant" && duration=="persistent") { return Err("only phase stat modifications execute".into()); }
                if let Target::ArenaList(list)=target {
                    if self.trigger=="onAttack" && list.token_unit_attack_scope() {list.receiver.validate()?;list.predicate.validate()?;} else {list.validate_phase_stats()?;}
                }else{target.validate()?;}
                if matches!(target,Target::ArenaList(_)) {
                    // Unary plus is emitted by signed source literals. Normalize only this
                    // bounded stat scope; general expression unary-plus remains rejected.
                    fn normalized(e:&Expr)->Expr {match e {Expr::Unary{op,value} if op=="+" && matches!(value.as_ref(),Expr::Const{value} if value.as_i64().is_some_and(|n|(0..=65535).contains(&n)))=>*value.clone(),_=>e.clone()}}
                    let power=normalized(power);let hp=normalized(hp);power.validate_number()?;hp.validate_number()?;
                    self.instructions.push(Instruction::Apply(Effect::ModifyStats{target:target.clone(),power,hp,duration:duration.clone()}));
                }else {
                    if duration=="persistent" {power.live_stat_bounds()?;hp.live_stat_bounds()?;}else{power.validate_number()?;hp.validate_number()?;}
                    self.instructions.push(Instruction::Apply(effect.clone()));
                }
            }
            Effect::MaximumPowerExperience{..}=>return Err("maximum capture cannot be nested or used as a primitive".into()),
            Effect::CopyKeywords{target,duration,domain}=>{
                if self.trigger!="constant"||duration!="persistent"||domain.iter().map(String::as_str).collect::<Vec<_>>()!=COPY_KEYWORDS||!matches!(target,Target::Name(n) if n=="chosen") {return Err("keyword copy requires persistent selected core keyword inventory".into());}
                self.instructions.push(Instruction::Apply(effect.clone()));
            }
            Effect::GainKeyword { target, keyword, amount, duration } => {
                validate_keyword(keyword)?;
                if duration != "phase" && !(self.trigger == "constant" && duration == "persistent") { return Err("only phase or constant keywords execute".into()); }
                if let Target::ArenaList(list)=target {
                    if depth!=0 || duration!="phase" {return Err("nested or constant arena-list keywords are unsupported".into());}
                    list.validate_scope(true)?;
                }else{target.validate()?;} amount.validate_number()?;
                let bounded_shape = matches!(amount, Expr::Const {value} if value.as_u64().is_some_and(|n| n <= 65535))
                    || (duration == "persistent" && matches!(amount, Expr::Ref {path} if matches!(path.last().map(String::as_str), Some("damage" | "readyResourceCount" | "exhaustedResourceCount"))));
                if !bounded_shape { return Err("keyword amounts require nonnegative literals or verified live damage/resource counts".into()); }
                if keyword == "restore" && !matches!(amount, Expr::Const {value} if value.as_u64().is_some_and(|n| n > 0 && n <= 65535)) { return Err("Restore grants require positive literals until nullable zero-trigger titles are verified".into()); }


                if let Expr::Const { value } = amount { if !value.as_u64().is_some_and(|n| n <= 65535) { return Err("keyword amount must be bounded nonnegative integer".into()); } }
                if !["raid", "restore"].contains(&keyword.as_str()) && !matches!(amount, Expr::Const {value} if value.as_u64() == Some(1)) { return Err("boolean keyword amount must be one".into()); }
                self.instructions.push(Instruction::Apply(effect.clone()));
            }
            Effect::LoseKeyword { target, keyword, duration } => {
                validate_keyword(keyword)?;
                if duration != "phase" && !(self.trigger == "constant" && duration == "persistent") { return Err("only phase or constant keyword losses execute".into()); }
                if let Target::ArenaList(list)=target {
                    if depth!=0 || duration!="phase" {return Err("nested or constant arena-list keyword loss is unsupported".into());}
                    list.validate_scope(true)?;
                }else{target.validate()?;} self.instructions.push(Instruction::Apply(effect.clone()));
            }
            Effect::Ready { target } | Effect::Exhaust { target } | Effect::Defeat { target } => { target.validate()?;self.instructions.push(Instruction::Apply(effect.clone())); }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct CardPrograms {
    pub leader: Vec<u16>,
    pub unit: Vec<u16>,
    pub card: Vec<u16>,
    pub attach_condition: Option<Expr>,
}

fn query_shape(value:&Value)->Value {match value {Value::Object(o)=>Value::Object(o.iter().filter(|(k,_)|k.as_str()!="title").map(|(k,v)|(k.clone(),query_shape(v))).collect()),Value::Array(a)=>Value::Array(a.iter().map(query_shape).collect()),_=>value.clone()}}
pub(crate) fn contains_arena_query(v:&Value)->bool {v["kind"].as_str().is_some_and(|s|["arenaList","arenaFilter","arenaLength","arenaSome"].contains(&s)) && (v.get("receiver").is_some()) || match v {Value::Array(a)=>a.iter().any(contains_arena_query),Value::Object(o)=>o.values().any(contains_arena_query),_=>false}}
fn contains_arena_scalar(v:&Value)->bool {fn walk(v:&Value,target_container:bool)->bool {if !target_container&&v["kind"].as_str().is_some_and(|s|["arenaList","arenaFilter","arenaLength","arenaSome"].contains(&s)){return true;}match v{Value::Array(a)=>a.iter().any(|v|walk(v,false)),Value::Object(o)=>o.iter().any(|(k,v)|walk(v,k=="target"&&v["kind"]=="arenaList")),_=>false}}walk(v,false)}
pub(crate) fn arena_query_source_kind(entry:&Value)->Option<usize> {
 if !entry["card"]["unique"].is_boolean() || !entry["attachCondition"].is_null() || !entry["piloting"].is_null() || ["pilotText","pilotBox","pilotingBox"].iter().any(|k|!entry["card"][k].is_null()) {return None;}
 let shape=query_shape(&entry["abilities"]);let shapes:Vec<Value>=vec![
serde_json::from_str(r##"{"leader":[],"unit":[],"card":[{"trigger":"constant","costs":[],"effect":{"kind":"gainKeyword","target":"self","duration":"persistent","keyword":"ambush","amount":{"kind":"const","value":1}},"condition":{"kind":"arenaSome","receiver":{"kind":"arenaList","receiver":{"kind":"ref","path":["player"]},"predicate":{"kind":"binary","op":"&&","left":{"kind":"const","value":true},"right":{"kind":"binary","op":"||","left":{"kind":"unary","op":"!","value":{"kind":"ref","path":["source"]}},"right":{"kind":"binary","op":"!==","left":{"kind":"ref","path":["card"]},"right":{"kind":"ref","path":["source"]}}}}},"predicate":{"kind":"unary","op":"!","value":{"kind":"ref","path":["card","unique"]}}}}]}"##).ok()?,
serde_json::from_str(r##"{"leader":[],"unit":[],"card":[{"trigger":"whenPlayed","costs":[],"optional":true,"targetResolver":{"owner":"any","type":"unit","zone":"anyAttackable"},"effect":{"kind":"conditional","condition":{"kind":"binary","op":">","left":{"kind":"arenaLength","receiver":{"kind":"arenaFilter","receiver":{"kind":"arenaList","receiver":{"kind":"ref","path":["player"]},"predicate":{"kind":"binary","op":"&&","left":{"kind":"const","value":true},"right":{"kind":"binary","op":"||","left":{"kind":"unary","op":"!","value":{"kind":"const","value":"spaceArena"}},"right":{"kind":"binary","op":"||","left":{"kind":"binary","op":"===","left":{"kind":"const","value":"spaceArena"},"right":{"kind":"const","value":"anyArena"}},"right":{"kind":"binary","op":"===","left":{"kind":"ref","path":["card","zoneName"]},"right":{"kind":"const","value":"spaceArena"}}}}}},"predicate":{"kind":"binary","op":"!==","left":{"kind":"ref","path":["card"]},"right":{"kind":"ref","path":["source"]}}}},"right":{"kind":"const","value":0}},"onTrue":{"kind":"exhaust","target":"chosen"},"onFalse":{"kind":"noAction"}}}]}"##).ok()?,
serde_json::from_str(r##"{"leader":[{"trigger":"action","costs":[{"kind":"exhaustSelf"},{"kind":"resources","amount":{"kind":"const","value":1}}],"effect":{"kind":"conditional","condition":{"kind":"binary","op":">=","left":{"kind":"arenaLength","receiver":{"kind":"arenaList","receiver":{"kind":"ref","path":["game"]},"predicate":{"kind":"binary","op":"&&","left":{"kind":"const","value":true},"right":{"kind":"binary","op":"&&","left":{"kind":"query","name":"isUnit","receiver":{"kind":"ref","path":["card"]},"args":[]},"right":{"kind":"ref","path":["card","exhausted"]}}}}},"right":{"kind":"const","value":4}},"onTrue":{"kind":"createTokenUnit","tokenType":"spy","amount":{"kind":"const","value":1},"target":"ownPlayer","entersReady":false},"onFalse":{"kind":"noAction"}}}],"unit":[{"trigger":"onAttack","costs":[],"optional":true,"targetResolver":{"owner":"any","type":"unit","zone":"anyAttackable","predicate":{"kind":"binary","op":"&&","left":{"kind":"query","name":"isUnit","receiver":{"kind":"ref","path":["card"]},"args":[]},"right":{"kind":"ref","path":["card","exhausted"]}}},"effect":{"kind":"damage","target":"chosen","amount":{"kind":"const","value":2}}}],"card":[]}"##).ok()?];
 let kind=shapes.iter().position(|s|s==&shape)?;let metadata:Vec<Value>=serde_json::from_str(r##"[{"cost":4,"hp":5,"power":4,"arena":"ground","text":"While you control another non-<uq> unit, this unit gains Ambush. (When you play this unit, it may attack an enemy unit.)","deployBox":null,"epicAction":"","rules":"Ambush is an ability that triggers at the same time as \u201cWhen Played\u201d abilities. The unit does not have to be ready in order to attack.\n","upgradePower":null,"upgradeHp":null,"unique":false,"keywords":["ambush"]},{"cost":3,"hp":3,"power":3,"arena":"space","text":"When Played: If you control another space unit, you may exhaust a unit.","deployBox":null,"epicAction":"","rules":null,"upgradePower":null,"upgradeHp":null,"unique":false,"keywords":[]},{"cost":5,"hp":6,"power":3,"arena":"ground","text":"Action [1 resource, Exhaust]: If there are 4 or more exhausted units in play, create a Spy token.","deployBox":"On Attack: You may deal 2 damage to an exhausted unit.","epicAction":"Epic Action: If you control 5 or more resources, deploy this leader.","rules":"Sly\u2019s leader ability still can be used as an action even if fewer than 4 exhausted units are in play (but no Spy token is created).\n","upgradePower":null,"upgradeHp":null,"unique":true,"keywords":[]}]"##).ok()?;if !metadata[kind].as_object()?.iter().all(|(k,v)|entry["card"][k]==*v)||entry["keywordValues"]!=serde_json::json!({}){return None;}let ty=if kind==2 {"leader"}else{"unit"};if entry["card"]["types"]!=serde_json::json!([ty]) || entry["baseClass"]!=if kind==2 {"LeaderUnitCard"}else{"NonLeaderUnitCard"} {return None;}Some(kind)
}
fn arena_query_ability(a:&Ability)->bool {let v=query_shape(&serde_json::to_value(a).unwrap());let templates:Vec<Value>=vec![
query_shape(&serde_json::to_value(serde_json::from_str::<Ability>(r##"{"trigger":"constant","title":"Gain Ambush while you control another non-unique unit","costs":[],"effect":{"kind":"gainKeyword","target":"self","duration":"persistent","keyword":"ambush","amount":{"kind":"const","value":1}},"condition":{"kind":"arenaSome","receiver":{"kind":"arenaList","receiver":{"kind":"ref","path":["player"]},"predicate":{"kind":"binary","op":"&&","left":{"kind":"const","value":true},"right":{"kind":"binary","op":"||","left":{"kind":"unary","op":"!","value":{"kind":"ref","path":["source"]}},"right":{"kind":"binary","op":"!==","left":{"kind":"ref","path":["card"]},"right":{"kind":"ref","path":["source"]}}}}},"predicate":{"kind":"unary","op":"!","value":{"kind":"ref","path":["card","unique"]}}}}"##).unwrap()).unwrap()),
query_shape(&serde_json::to_value(serde_json::from_str::<Ability>(r##"{"trigger":"whenPlayed","title":"Exhaust a unit.","costs":[],"optional":true,"targetResolver":{"owner":"any","type":"unit","zone":"anyAttackable"},"effect":{"kind":"conditional","condition":{"kind":"binary","op":">","left":{"kind":"arenaLength","receiver":{"kind":"arenaFilter","receiver":{"kind":"arenaList","receiver":{"kind":"ref","path":["player"]},"predicate":{"kind":"binary","op":"&&","left":{"kind":"const","value":true},"right":{"kind":"binary","op":"||","left":{"kind":"unary","op":"!","value":{"kind":"const","value":"spaceArena"}},"right":{"kind":"binary","op":"||","left":{"kind":"binary","op":"===","left":{"kind":"const","value":"spaceArena"},"right":{"kind":"const","value":"anyArena"}},"right":{"kind":"binary","op":"===","left":{"kind":"ref","path":["card","zoneName"]},"right":{"kind":"const","value":"spaceArena"}}}}}},"predicate":{"kind":"binary","op":"!==","left":{"kind":"ref","path":["card"]},"right":{"kind":"ref","path":["source"]}}}},"right":{"kind":"const","value":0}},"onTrue":{"kind":"exhaust","target":"chosen"},"onFalse":{"kind":"noAction"}}}"##).unwrap()).unwrap()),
query_shape(&serde_json::to_value(serde_json::from_str::<Ability>(r##"{"trigger":"action","title":"If there are 4 or more exhausted units in play, create a Spy token","costs":[{"kind":"exhaustSelf"},{"kind":"resources","amount":{"kind":"const","value":1}}],"effect":{"kind":"conditional","condition":{"kind":"binary","op":">=","left":{"kind":"arenaLength","receiver":{"kind":"arenaList","receiver":{"kind":"ref","path":["game"]},"predicate":{"kind":"binary","op":"&&","left":{"kind":"const","value":true},"right":{"kind":"binary","op":"&&","left":{"kind":"query","name":"isUnit","receiver":{"kind":"ref","path":["card"]},"args":[]},"right":{"kind":"ref","path":["card","exhausted"]}}}}},"right":{"kind":"const","value":4}},"onTrue":{"kind":"createTokenUnit","tokenType":"spy","amount":{"kind":"const","value":1},"target":"ownPlayer","entersReady":false},"onFalse":{"kind":"noAction"}}}"##).unwrap()).unwrap())];templates.iter().any(|t|t==&v)}
#[derive(Debug, Default)]
pub struct Catalogue {
    pub programs: Vec<Program>,
    pub cards: std::collections::HashMap<String, CardPrograms>,
    pub metadata: std::collections::HashMap<String, Value>,
    pub rejected: std::collections::BTreeMap<String, String>,
    /// Alternate printings (`setCodes`) mapped to the catalogue's canonical setCode.
    pub reprint_of: std::collections::HashMap<String, String>,
}
impl Catalogue {
    /// Converts every compiler entry into executable programs or a named, explicit
    /// rejection. A syntactically compiled card is never assumed executable.
    pub fn load(value: &Value) -> Result<Self, String> {
        if value["schema"] != SCHEMA {
            return Err(format!("unsupported ability IR schema {}", value["schema"]));
        }
        let entries = value["cards"]
            .as_array()
            .ok_or("IR catalogue needs cards")?;
        let power_feedback=entries.iter().any(crate::bounded_reductions::power_feedback);
        let trait_edges=entries.iter().flat_map(crate::static_ongoing::trait_dependencies).collect::<Vec<_>>();
        fn trait_path(edges:&[(String,String)],from:&str,to:&str,seen:&mut std::collections::BTreeSet<String>)->bool {
            if from==to {return true;}if !seen.insert(from.into()){return false;}
            edges.iter().filter(|(a,_)|a==from).any(|(_,b)|trait_path(edges,b,to,seen))
        }
        let cyclic_traits=trait_edges.iter().filter(|(a,b)|trait_path(&trait_edges,b,a,&mut Default::default())).cloned().collect::<std::collections::BTreeSet<_>>();
        let mut catalogue = Self::default();
        for entry in entries {
            let code = entry["setCode"]
                .as_str()
                .ok_or("IR card missing setCode")?
                .to_owned();
            if catalogue.cards.contains_key(&code) || catalogue.rejected.contains_key(&code) {
                return Err(format!("duplicate IR card {code}"));
            }
            let result = || -> Result<(CardPrograms, Vec<Program>), String> {
                if entry["status"] != "supported" {
                    return Err(format!(
                        "compiler rejected {}: {}",
                        entry["internalName"], entry["diagnostics"]
                    ));
                }
                for key in ["abilities", "attachCondition", "piloting"] { crate::deck_search::reject(&entry[key])?; }
                // Upgrade grant/host refs use source.parent* or target "parent".
                // Selector predicates on an upgrade candidate use card.parent* and must not
                // force the watching unit/event into the UpgradeCard source class.
                fn parent_reference(value:&Value)->bool {
                    (value["kind"]=="ref"
                        && value["path"][0]=="source"
                        && value["path"][1].as_str().is_some_and(|p|p=="parentCard"||p=="parentUnit"))
                        || (value["target"]=="parent")
                        || match value {
                            Value::Array(a)=>a.iter().any(parent_reference),
                            Value::Object(o)=>o.values().any(parent_reference),
                            _=>false,
                        }
                }
                if crate::native_attack::marker(&entry["abilities"])&&!crate::native_attack::entry(entry){return Err("native attacks require a complete sole ordinary unit/event registration".into());}
                // historyHelpers/privatePrograms may embed historyQuery AST that is not part of
                // executable abilities; do not treat those leftovers as admitted phase history.
                if crate::phase_history::marker(&entry["historyHelpers"])||crate::phase_history::marker(&entry["privatePrograms"]){return Err("unresolved phase history helpers require reviewed registration".into());}
                if crate::phase_history::marker(&entry["abilities"])&&!crate::phase_history::entry(entry){return Err("phase history requires complete ordinary unit/event source metadata and registration".into());}
                if crate::view_cards::marker(&entry["abilities"])&&!crate::view_cards::entry(entry){return Err("view-only lookAt/lookAtAndSelectCard requires complete ordinary unit/event registration".into());}
                if crate::static_ongoing::trait_dependencies(entry).iter().any(|edge|cyclic_traits.contains(edge)){return Err("trait producer dependency cycle is unsupported".into());}
                if crate::static_ongoing::marker(entry) && !crate::static_ongoing::entry(entry) && !crate::attached_constants::entry(entry) {return Err("static self ongoing contracts require complete ordinary nonleader unit metadata and one registration".into());}
                if parent_reference(&entry["abilities"]) && entry["baseClass"]!="UpgradeCard" && !crate::piloting_attached::entry(entry) {
                    return Err("parent references require an ordinary attached upgrade".into());
                }
                let mut entry_owned = entry.clone();
                let piloting_keyword_grant = crate::piloting_attached::marker(&entry_owned["abilities"]);
                if piloting_keyword_grant {
                    crate::piloting_attached::normalize(&mut entry_owned)?;
                }
                let entry = &entry_owned;
                if crate::live_power_maximum::marker(entry)&&!crate::live_power_maximum::entry(entry){return Err("live maximum requires complete two-sided leader metadata and registrations".into());}
                if power_feedback&&crate::bounded_reductions::leader_entry(entry){return Err("power maximum keyword grant conflicts with keyword-dependent stat source".into());}
                if crate::bounded_reductions::marker(entry)&&!crate::bounded_reductions::entry(entry){return Err("bounded arena reductions require complete reviewed registration and source metadata".into());}
                let basezone_relevant=entry["abilities"]["card"].as_array().is_some_and(|a|(entry["baseClass"]=="BaseCard"||entry["card"]["types"].as_array().is_some_and(|t|t.iter().any(|v|v=="base")))&&!a.is_empty()) || entry["abilities"]["leader"].as_array().is_some_and(|a|a.iter().any(|a|a["trigger"]=="constant"));
                if basezone_relevant&&basezone_source_kind(entry).is_none(){return Err("base-zone ongoing source requires exact reviewed base or two-side leader shape".into());}
                if contains_arena_scalar(&entry["abilities"]) && arena_query_source_kind(entry).is_none() {return Err("arena scalar query requires exact reviewed complete source shape".into());}
                let sides: Sides = crate::attached_registration::sides(entry)?;
                if piloting_keyword_grant
                    && (sides.card.len() != 1 || !crate::piloting_attached::registration(&sides.card[0]))
                {
                    return Err("piloting attached keyword requires a sole persistent parent grant".into());
                }

                if crate::advantage::marker(&entry["abilities"]) && !crate::advantage::source_entry(entry) {return Err("Advantage requires complete ordinary unit/event sources and bounded native token contracts".into());}

                if sides.card.iter().any(crate::attached_constants::host_trait_aura)&&!crate::attached_constants::entry(entry) {return Err("host trait auras require the complete attached constant wrapper source contract".into());}
                if sides.card.iter().chain(&sides.leader).chain(&sides.unit).any(|a|!a.hooks.is_empty()||a.trigger=="eventHooks") {
                    let hooks:Vec<_>=sides.card.iter().chain(&sides.leader).chain(&sides.unit).filter(|a|!a.hooks.is_empty()||a.trigger=="eventHooks").collect();
                    if hooks.iter().all(|a|crate::death_hooks::registration(a)) {
                        if !crate::death_hooks::entry(entry,&sides) {return Err("defeat hooks require complete ordinary unit source metadata and bounded registrations".into());}
                    } else if hooks.iter().all(|a|crate::death_hooks::smuggle_play_registration(a)) {
                        if !crate::death_hooks::smuggle_play_entry(entry,&sides) {return Err("smuggle when-played requires complete ordinary smuggle unit source metadata and registration".into());}
                    } else if hooks.iter().all(|a|crate::death_hooks::leader_deploy_base_registration(a)) {
                        if !crate::death_hooks::leader_deploy_base_entry(entry,&sides) {return Err("leader-deploy base watchers require exact reviewed base shape".into());}
                    } else if hooks.iter().all(|a|crate::death_hooks::card_played_registration(a)) {
                        if !crate::death_hooks::card_played_entry(entry,&sides) {return Err("onCardPlayed requires complete ordinary unit source metadata and bounded registrations".into());}
                    } else {
                        return Err(crate::death_hooks::reject_reason(hooks[0]));
                    }
                }
                if selector_contract::expanded_selector_marker(&entry["abilities"]) && !selector_contract::native_source_entry(entry,&sides) {return Err("expanded selectors require complete ordinary unit/event card-side source metadata without unimplemented registration contexts".into());}
                if ["abilities","attachCondition","piloting"].iter().any(|k|parent_defense_marker(&entry[*k]))&&!parent_defense_upgrade_entry(entry){return Err("parent defense requires complete nonunique non-Vehicle-attached upgrade metadata".into());}

                if sides.card.iter().any(crate::attached_constants::parent_title_play)&&!crate::attached_registration::plain_upgrade(entry) {return Err("parent-title play requires complete ordinary upgrade source metadata".into());}
                let exact_type=|expected:&str|entry["card"]["types"].as_array().is_some_and(|types|types.len()==1&&types[0]==expected);
                let source_class=entry["baseClass"].as_str();
                if gained_host_marker(entry) && gained_host_upgrade_kind(entry).is_none() {return Err("gained host attack templates require complete reviewed upgrade metadata".into());}
                let source_types:Option<&[&str]>=match source_class {
                    Some("NonLeaderUnitCard")=>Some(&["unit"]), Some("LeaderUnitCard")=>Some(&["leader"]),
                    Some("EventCard")=>Some(&["event"]), Some("BaseCard")=>Some(&["base"]),
                    Some("UpgradeCard")=>Some(&["upgrade"]), Some("TokenUnitCard")=>Some(&["token","unit"]),
                    Some("TokenUpgradeCard")=>Some(&["token","upgrade"]), Some("TokenCard")=>Some(&["token"]), _=>None,
                };
                if let Some(expected)=source_types {
                    if !entry["card"]["types"].as_array().is_some_and(|types|types.len()==expected.len()&&expected.iter().all(|kind|types.iter().any(|value|value.as_str()==Some(*kind)))) {return Err("source class and card type metadata disagree".into());}
                    if source_class==Some("LeaderUnitCard") {
                        if !sides.card.is_empty() {return Err("leader sources require explicit leader/unit side registrations".into());}
                    } else if !sides.leader.is_empty()||!sides.unit.is_empty() {return Err("ordinary sources require card-side registrations".into());}
                }
                fn trait_leaf_marker(v:&Value)->bool {matches!(v["kind"].as_str(),Some("gainTrait"|"loseTrait"))||match v {Value::Object(o)=>o.values().any(trait_leaf_marker),Value::Array(a)=>a.iter().any(trait_leaf_marker),_=>false}}
                if trait_leaf_marker(&entry["abilities"]) {
                    let unit=source_class==Some("NonLeaderUnitCard")&&exact_type("unit");
                    let upgrade=source_class==Some("UpgradeCard")&&exact_type("upgrade");
                    fn valid_leaf_target(e:&Effect,expected:&str)->bool {match e {Effect::GainTrait{target:Target::Name(n),..}|Effect::LoseTrait{target:Target::Name(n),..}=>n==expected,Effect::Simultaneous{effects}=>effects.iter().all(|e|valid_leaf_target(e,expected)),_=>true}}
                    if !crate::static_ongoing::closed_source_context(entry) || !(unit||upgrade) || !sides.unit.is_empty() || !sides.leader.is_empty() || !entry["card"]["unique"].is_boolean() || !entry["piloting"].is_null() || ["pilotText","pilotBox","pilotingBox","deployBox","deployCost"].iter().any(|k|!entry["card"][*k].is_null()) || !(entry["card"]["epicAction"].is_null()||entry["card"]["epicAction"]=="") || (unit && (!entry["attachCondition"].is_null() || !entry["card"]["upgradePower"].is_null() || !entry["card"]["upgradeHp"].is_null())) || !sides.card.iter().all(|a|valid_leaf_target(&a.effect,if unit {"self"}else{"parent"})) {
                        return Err("trait leaves require complete ordinary unit or attached upgrade source metadata".into());
                    }
                }
                fn contains_presence_query(value:&Value)->bool {
                    (value["kind"]=="query" && matches!(value["name"].as_str(),Some("hasSomeArenaCard"|"isDeployed"))) || match value {Value::Array(items)=>items.iter().any(contains_presence_query),Value::Object(object)=>object.values().any(contains_presence_query),_=>false}
                }
                fn contains_query_named(value:&Value,name:&str)->bool {
                    (value["kind"]=="query"&&value["name"]==name) || match value {Value::Array(items)=>items.iter().any(|v|contains_query_named(v,name)),Value::Object(object)=>object.values().any(|v|contains_query_named(v,name)),_=>false}
                }
                fn contains_arena_leader_predicate(value:&Value)->bool {
                    (value["kind"]=="query"&&matches!(value["name"].as_str(),Some("hasSomeArenaUnit"|"arenaUnitCount"))&&contains_query_named(&value["args"],"isLeader")) || match value {Value::Array(items)=>items.iter().any(contains_arena_leader_predicate),Value::Object(object)=>object.values().any(contains_arena_leader_predicate),_=>false}
                }
                if ["abilities","attachCondition","piloting"].iter().any(|key|contains_arena_leader_predicate(&entry[*key])) && (source_class!=Some("NonLeaderUnitCard")||!exact_type("unit")||!sides.unit.is_empty()||!sides.leader.is_empty()||!entry["card"]["unique"].is_boolean()||!entry["attachCondition"].is_null()||!entry["piloting"].is_null()||["pilotText","pilotBox","pilotingBox","deployBox","deployCost","upgradePower","upgradeHp"].iter().any(|k|!entry["card"][k].is_null())||!(entry["card"]["epicAction"].is_null()||entry["card"]["epicAction"]=="")||entry["keywordValues"]!=serde_json::json!({})) {return Err("arena leader predicates require complete ordinary unit metadata without side, pilot, attachment, deploy or epic mechanics".into());}
                fn complete_plain_source_metadata(entry:&Value)->bool {
                    entry["card"]["unique"].is_boolean() && entry["attachCondition"].is_null() && entry["piloting"].is_null() &&
                    ["pilotText","pilotBox","pilotingBox","deployBox","deployCost","upgradePower","upgradeHp"].iter().all(|key|entry["card"][*key].is_null()) &&
                    (entry["card"]["epicAction"].is_null()||entry["card"]["epicAction"]=="")
                }
                if sides.card.iter().chain(&sides.leader).chain(&sides.unit).any(|a|attachment_defeat_marker(&a.effect)) &&
                    (sides.card.len()!=1||!sides.leader.is_empty()||!sides.unit.is_empty()||!attachment_defeat_registration(&sides.card[0])||!complete_plain_source_metadata(entry)||entry["keywordValues"]!=serde_json::json!({})||entry["card"]["keywords"]!=serde_json::json!([])||
                    !((entry["baseClass"]=="EventCard"&&exact_type("event")&&sides.card[0].trigger=="event")||(entry["baseClass"]=="NonLeaderUnitCard"&&exact_type("unit")&&sides.card[0].trigger=="whenPlayed"))) {return Err("attachment list defeat requires sole ordinary event/unit source without unimplemented intrinsic mechanics".into());}
                if sides.leader.iter().chain(&sides.unit).any(|a|a.trigger=="whenAttackEnds" || a.attacker_must_survive) || sides.card.iter().any(|a|a.trigger=="whenAttackEnds" || a.attacker_must_survive) && (source_class!=Some("NonLeaderUnitCard") || !exact_type("unit") || !complete_plain_source_metadata(entry) || !crate::attack_history::declarations(entry)) {return Err("attack-end registrations require complete ordinary unit sources and exact watcher declarations".into());}
                fn has_maximum(value:&Value)->bool {value["kind"]=="maximumPowerExperience"||match value {Value::Array(a)=>a.iter().any(has_maximum),Value::Object(o)=>o.values().any(has_maximum),_=>false}}
                if has_maximum(&entry["abilities"]) && (source_class!=Some("LeaderUnitCard")||!exact_type("leader")||!sides.card.is_empty()||sides.leader.len()!=1||sides.unit.len()!=1||sides.leader[0].trigger!="action"||sides.unit[0].trigger!="onAttack"||!maximum_registration(&sides.leader[0])||!maximum_registration(&sides.unit[0])||serde_json::to_value(&sides.leader[0].effect).ok()!=serde_json::to_value(&sides.unit[0].effect).ok()||!entry["attachCondition"].is_null()||!entry["piloting"].is_null()||entry["card"]["unique"]!=true||entry["keywordValues"]!=serde_json::json!({})||["pilotText","pilotBox","pilotingBox","upgradePower","upgradeHp"].iter().any(|k|!entry["card"][*k].is_null())) {return Err("maximum capture requires complete matching leader action and unit attack registrations".into());}
                fn has_copy(value:&Value)->bool {value["kind"]=="copyKeywords"||match value{Value::Array(a)=>a.iter().any(has_copy),Value::Object(o)=>o.values().any(has_copy),_=>false}}
                if has_copy(&entry["abilities"]) {
                    let expected:Value=serde_json::json!({"trigger":"constant","costs":[],"effect":{"kind":"copyKeywords","target":"chosen","duration":"persistent","domain":COPY_KEYWORDS},"targetResolver":{"owner":"self","type":"unit","zone":"arena","predicate":{"kind":"binary","op":"&&","left":{"kind":"binary","op":"&&","left":{"kind":"binary","op":"!==","left":{"kind":"ref","path":["card"]},"right":{"kind":"ref","path":["source"]}},"right":{"kind":"query","name":"isUnit","receiver":{"kind":"ref","path":["card"]},"args":[]}},"right":{"kind":"query","name":"hasSomeTrait","receiver":{"kind":"ref","path":["card"]},"args":[{"kind":"const","value":"spectre"}]}}}});
                    let mut copy=entry["abilities"]["card"][0].clone();if let Some(o)=copy.as_object_mut(){o.remove("title");}
                    let mut companion=entry["abilities"]["card"][1].clone();if let Some(o)=companion.as_object_mut(){o.remove("title");}
                    let sentinel=serde_json::json!({"trigger":"constant","costs":[],"condition":{"kind":"query","name":"isUpgraded","receiver":{"kind":"ref","path":["source"]},"args":[]},"effect":{"kind":"gainKeyword","target":"self","duration":"persistent","keyword":"sentinel","amount":{"kind":"const","value":1}}});
                    if source_class!=Some("NonLeaderUnitCard")||!exact_type("unit")||!complete_plain_source_metadata(entry)||sides.card.len()!=2||!sides.unit.is_empty()||!sides.leader.is_empty()||copy!=expected||companion!=sentinel||entry["keywordValues"].as_object().is_none_or(|o|o.keys().any(|k|!CORE_KEYWORDS.contains(&k.as_str()))) {return Err("keyword copy requires complete bounded other-friendly trait aura and self upgraded Sentinel source".into());}
                }
                if contains_query_named(&entry["abilities"],"leaderTraits") && (source_class!=Some("NonLeaderUnitCard")||!exact_type("unit")||!complete_plain_source_metadata(entry)||!sides.unit.is_empty()||!sides.leader.is_empty()||sides.card.len()!=1||entry["keywordValues"]!=serde_json::json!({})) {return Err("leader trait union requires complete ordinary unit source".into());}
                if contains_query_named(&entry["abilities"],"hasControlledInPlayTitle") || contains_query_named(&entry["abilities"],"hasControlledInPlayTrait") {
                    let reviewed=|a:&Ability| {
                        let value=serde_json::to_value(a).unwrap();
                        if contains_query_named(&value,"hasControlledInPlayTrait") {return trait_constant_registration(a)&&!contains_query_named(&value,"hasControlledInPlayTitle");}
                        if !contains_query_named(&value,"hasControlledInPlayTitle") {return true;}
                        title_registration(a)
                    };
                    if source_class!=Some("NonLeaderUnitCard") || !exact_type("unit") || !complete_plain_source_metadata(entry) || !sides.unit.is_empty() || !sides.leader.is_empty() || !sides.card.iter().all(reviewed) {return Err("controlled title requires complete ordinary unit reviewed scalar registration".into());}
                }
                fn contains_ref_path(value:&Value,path:&[&str])->bool {
                    (value["kind"]=="ref"&&value["path"].as_array().is_some_and(|p|p.len()==path.len()&&p.iter().zip(path).all(|(v,k)|v==*k))) || match value {Value::Array(items)=>items.iter().any(|v|contains_ref_path(v,path)),Value::Object(object)=>object.values().any(|v|contains_ref_path(v,path)),_=>false}
                }
                fn other_base_heal(value:&Value)->bool {
                    (value["kind"]=="heal"&&value["target"]["kind"]=="conditional"&&contains_ref_path(&value["target"],&["target"])&&contains_ref_path(&value["target"],&["player","base"])&&contains_ref_path(&value["target"],&["opponent","base"])) || match value {Value::Array(items)=>items.iter().any(other_base_heal),Value::Object(object)=>object.values().any(other_base_heal),_=>false}
                }
                fn recipient_remaining_hp_damage(value:&Value)->bool {
                    (value["kind"]=="damage"&&contains_ref_path(&value["amount"],&["target","remainingHp"])) || match value {Value::Array(items)=>items.iter().any(recipient_remaining_hp_damage),Value::Object(object)=>object.values().any(recipient_remaining_hp_damage),_=>false}
                }
                let other_base=["abilities","attachCondition","piloting"].iter().any(|key|other_base_heal(&entry[*key]));
                let recipient_hp=["abilities","attachCondition","piloting"].iter().any(|key|recipient_remaining_hp_damage(&entry[*key]));
                if other_base || recipient_hp {
                    let card=&entry["abilities"]["card"][0];
                    let (class,kind,trigger,effect,target_type)=if other_base {("NonLeaderUnitCard","unit","whenPlayed","heal","base")} else {("EventCard","event","event","damage","nonLeaderUnit")};
                    if source_class!=Some(class)||!exact_type(kind)||sides.card.len()!=1||!sides.unit.is_empty()||!sides.leader.is_empty()||!complete_plain_source_metadata(entry)||entry["keywordValues"]!=serde_json::json!({})||card["trigger"]!=trigger||card["effect"]["kind"]!=effect||card["targetResolver"]!=serde_json::json!({"owner":"any","type":target_type,"zone":"anyAttackable"})||card["costs"]!=serde_json::json!([])||!card["condition"].is_null()||card["optional"]==true||card["coordinate"]==true||card["requiresConfirmation"]==true||!(card["contextTitle"].is_null()||card["contextTitle"]==serde_json::json!([]))||!(card["targetResolvers"].is_null()||card["targetResolvers"]==serde_json::json!([]))||!card["then"].is_null()||!card["ifYouDo"].is_null() {return Err("other-base healing and recipient remaining-HP damage require complete bounded ordinary source metadata and registration topology".into());}
                    if recipient_hp && (card["effect"]["target"]!="chosen"||entry["card"]["unique"]!=false) {return Err("recipient remaining-HP damage requires a sole ordinary event using its chosen nonleader recipient".into());}
                    if other_base && card["effect"]["target"]!=serde_json::json!({"kind":"conditional","condition":{"kind":"binary","op":"===","left":{"kind":"ref","path":["target"]},"right":{"kind":"ref","path":["player","base"]}},"then":{"kind":"ref","path":["opponent","base"]},"else":{"kind":"ref","path":["player","base"]}}) {return Err("other-base healing requires an exact selected-base complement target".into());}
                }
                // These source forms use existing execution primitives, but their
                // complete source metadata and registration scope remain bounded.
                fn target_arena_count(value:&Value)->bool {
                    (value["kind"]=="query"&&value["name"]=="arenaUnitCount"&&contains_ref_path(&value["args"],&["target","zoneName"])) || match value {Value::Array(items)=>items.iter().any(target_arena_count),Value::Object(object)=>object.values().any(target_arena_count),_=>false}
                }
                fn resource_hand_group(value:&Value)->bool {
                    (value["kind"]=="simultaneous"&&value["effects"].as_array().is_some_and(|effects|effects.iter().any(|effect|effect["kind"]=="draw")&&effects.iter().any(|effect|effect["kind"]=="conditional"&&effect["onTrue"]["kind"]=="discardOwnHand"))) || match value {Value::Array(items)=>items.iter().any(resource_hand_group),Value::Object(object)=>object.values().any(resource_hand_group),_=>false}
                }
                let arena_damage=["abilities","attachCondition","piloting"].iter().any(|key|target_arena_count(&entry[*key]));
                let hand_group=["abilities","attachCondition","piloting"].iter().any(|key|resource_hand_group(&entry[*key]));
                if arena_damage || hand_group {
                    let registration=&entry["abilities"]["card"][0];let effect=&registration["effect"];
                    let ordinary_unit_arena=arena_damage&&!hand_group&&source_class==Some("NonLeaderUnitCard")&&exact_type("unit");
                    if ordinary_unit_arena {
                        if sides.card.len()!=1||!sides.unit.is_empty()||!sides.leader.is_empty()||!complete_plain_source_metadata(entry)||entry["keywordValues"]!=serde_json::json!({"restore":1})||entry["card"]["keywords"]!=serde_json::json!(["restore"])||registration["trigger"]!="whenPlayed"||registration["costs"]!=serde_json::json!([])||!registration["condition"].is_null()||registration["coordinate"]==true||registration["requiresConfirmation"]==true||!(registration["contextTitle"].is_null()||registration["contextTitle"]==serde_json::json!([]))||!(registration["targetResolvers"].is_null()||registration["targetResolvers"]==serde_json::json!([]))||!registration["then"].is_null()||!registration["ifYouDo"].is_null() {return Err("unit target-arena damage requires a sole complete ordinary play registration with reviewed Restore".into());}
                    } else if source_class!=Some("EventCard")||!exact_type("event")||sides.card.len()!=1||!sides.unit.is_empty()||!sides.leader.is_empty()||!complete_plain_source_metadata(entry)||entry["card"]["unique"]!=false||entry["card"]["keywords"]!=serde_json::json!([])||entry["keywordValues"]!=serde_json::json!({})||registration["trigger"]!="event"||registration["costs"]!=serde_json::json!([])||!registration["condition"].is_null()||registration["optional"]==true||registration["coordinate"]==true||registration["requiresConfirmation"]==true||!(registration["contextTitle"].is_null()||registration["contextTitle"]==serde_json::json!([]))||!(registration["targetResolvers"].is_null()||registration["targetResolvers"]==serde_json::json!([]))||!registration["then"].is_null()||!registration["ifYouDo"].is_null() {
                        return Err("target-arena damage and resource hand groups require a sole complete ordinary event registration".into());
                    }
                    if arena_damage && (hand_group||effect["kind"]!="damage"||effect["target"]!="chosen"||!target_arena_count(&effect["amount"])||!["any","self","opponent"].iter().any(|owner|registration["targetResolver"]==serde_json::json!({"owner":owner,"type":"unit","zone":"anyAttackable"}))) {return Err("target-arena damage requires its scalar chosen-unit resolver".into());}
                    if hand_group {
                        let effects=effect["effects"].as_array().ok_or("resource hand group requires finite effects")?;
                        if arena_damage||!registration["targetResolver"].is_null()||effect["kind"]!="simultaneous"||effects.len()!=2||effects[0]["kind"]!="draw"||!matches!(effects[0]["target"].as_str(),Some("ownPlayer"|"enemyPlayer"))||effects[1]["kind"]!="conditional"||effects[1]["onTrue"]["kind"]!="discardOwnHand"||!matches!(effects[1]["onTrue"]["target"].as_str(),Some("ownPlayer"|"enemyPlayer"))||effects[1]["onFalse"]["kind"]!="noAction" {return Err("resource hand group requires own draw then conditional opponent hand discard".into());}
                    }
                }
                fn healing_above_two(value:&Value)->bool {
                    (value["kind"]=="distributeHealing"&&value["amount"]["kind"]=="const"&&value["amount"]["value"].as_u64().is_some_and(|n|n>2)) || match value {Value::Array(items)=>items.iter().any(healing_above_two),Value::Object(object)=>object.values().any(healing_above_two),_=>false}
                }
                if ["abilities","attachCondition","piloting"].iter().any(|key|healing_above_two(&entry[*key])) {
                    let a=&entry["abilities"]["card"][0];let effect=&a["effect"];
                    if source_class!=Some("NonLeaderUnitCard")||!exact_type("unit")||sides.card.len()!=1||!sides.unit.is_empty()||!sides.leader.is_empty()||!complete_plain_source_metadata(entry)||entry["keywordValues"]!=serde_json::json!({"restore":2})||entry["card"]["keywords"]!=serde_json::json!(["restore"])||a["trigger"]!="whenDefeated"||a["costs"]!=serde_json::json!([])||a["optional"]==true||!a["condition"].is_null()||a["coordinate"]==true||a["requiresConfirmation"]==true||!(a["contextTitle"].is_null()||a["contextTitle"]==serde_json::json!([]))||!a["targetResolver"].is_null()||!(a["targetResolvers"].is_null()||a["targetResolvers"]==serde_json::json!([]))||!a["then"].is_null()||!a["ifYouDo"].is_null()||effect["kind"]!="distributeHealing"||effect["maxTargets"]!=1||effect["amount"]!=serde_json::json!({"kind":"const","value":3})||effect["canChooseNoTargets"]!=true||effect["canDistributeLess"]!=true||!["any","self","opponent"].iter().any(|owner|["any","unit","base"].iter().any(|kind|effect["targetResolver"]==serde_json::json!({"owner":owner,"type":kind,"zone":"anyAttackable"}))) {return Err("three-point distribution requires complete ordinary death healing with one recipient and reviewed Restore".into());}
                }
                fn space_ready(value:&Value)->bool {
                    fn space(v:&Value)->bool {(v["kind"]=="const"&&v["value"]=="spaceArena")||match v {Value::Array(items)=>items.iter().any(space),Value::Object(object)=>object.values().any(space),_=>false}}
                    (value["kind"]=="conditional"&&value["onTrue"]==serde_json::json!({"kind":"ready","target":"self"})&&((contains_query_named(&value["condition"],"arenaUnitCount")&&space(&value["condition"]))||(value["condition"]["kind"]=="const"&&value["condition"]["value"].is_boolean()))) || match value {Value::Array(items)=>items.iter().any(space_ready),Value::Object(object)=>object.values().any(space_ready),_=>false}
                }
                if ["abilities","attachCondition","piloting"].iter().any(|key|space_ready(&entry[*key])) {
                    let a=&entry["abilities"]["card"][0];
                    if source_class!=Some("NonLeaderUnitCard")||!exact_type("unit")||sides.card.len()!=1||!sides.unit.is_empty()||!sides.leader.is_empty()||!complete_plain_source_metadata(entry)||entry["keywordValues"]!=serde_json::json!({})||entry["card"]["keywords"]!=serde_json::json!([])||a["trigger"]!="whenPlayed"||a["costs"]!=serde_json::json!([])||a["optional"]==true||!a["condition"].is_null()||a["coordinate"]==true||a["requiresConfirmation"]==true||!(a["contextTitle"].is_null()||a["contextTitle"]==serde_json::json!([]))||!a["targetResolver"].is_null()||!(a["targetResolvers"].is_null()||a["targetResolvers"]==serde_json::json!([]))||!a["then"].is_null()||!a["ifYouDo"].is_null()||a["effect"]["kind"]!="conditional"||a["effect"]["onTrue"]!=serde_json::json!({"kind":"ready","target":"self"})||a["effect"]["onFalse"]!=serde_json::json!({"kind":"noAction"}) {return Err("space-count readiness requires complete sole ordinary unit play metadata".into());}
                }
                fn plain_controls(a:&Value,trigger:&str)->bool {
                    (a["trigger"]==trigger||(trigger.is_empty()&&a["trigger"].is_null())) && a["costs"]==serde_json::json!([]) && !a["optional"].as_bool().unwrap_or(false) && a["condition"].is_null() && a["coordinate"]!=true && a["requiresConfirmation"]!=true && a["attachedContext"].is_null() && (a["contextTitle"].is_null()||a["contextTitle"]==serde_json::json!([])) && (a["targetResolvers"].is_null()||a["targetResolvers"]==serde_json::json!([])) && a["then"].is_null()
                }
                fn remaining_hp_draw(value:&Value)->bool {
                    (value["kind"]=="draw"&&contains_query_named(&value["amount"],"arenaUnitCount")&&contains_ref_path(&value["amount"],&["card","remainingHp"])) || match value {Value::Array(items)=>items.iter().any(remaining_hp_draw),Value::Object(object)=>object.values().any(remaining_hp_draw),_=>false}
                }
                fn conditional_shield_selection(value:&Value)->bool {
                    (value["kind"]=="conditional"&&value["onTrue"]["kind"]=="select"&&value["onTrue"]["effect"]["kind"]=="giveShield") || match value {Value::Array(items)=>items.iter().any(conditional_shield_selection),Value::Object(object)=>object.values().any(conditional_shield_selection),_=>false}
                }
                fn exhaust_token_continuation(value:&Value)->bool {
                    (value["effect"]["kind"]=="exhaust"&&value["ifYouDo"]["effect"]["kind"]=="simultaneous"&&value["ifYouDo"]["effect"]["effects"].as_array().is_some_and(|effects|effects.iter().any(|e|matches!(e["kind"].as_str(),Some("giveExperience"|"giveShield"))))) || match value {Value::Array(items)=>items.iter().any(exhaust_token_continuation),Value::Object(object)=>object.values().any(exhaust_token_continuation),_=>false}
                }
                let hp_draw=["abilities","attachCondition","piloting"].iter().any(|key|remaining_hp_draw(&entry[*key]));
                let count_shield=["abilities","attachCondition","piloting"].iter().any(|key|conditional_shield_selection(&entry[*key])) && !(crate::attached_constants::entry(entry)&&sides.card.iter().any(crate::attached_constants::parent_title_play));
                let exhaust_tokens=["abilities","attachCondition","piloting"].iter().any(|key|exhaust_token_continuation(&entry[*key]));
                if hp_draw || count_shield || exhaust_tokens {
                    let a=&entry["abilities"]["card"][0];let effect=&a["effect"];
                    if sides.card.len()!=1||!sides.unit.is_empty()||!sides.leader.is_empty()||!complete_plain_source_metadata(entry) {return Err("filtered count and captured exhaustion families require complete ordinary card-side metadata".into());}
                    if hp_draw {
                        if count_shield||exhaust_tokens||source_class!=Some("NonLeaderUnitCard")||!exact_type("unit")||entry["keywordValues"]!=serde_json::json!({"restore":4})||entry["card"]["keywords"]!=serde_json::json!(["restore"])||!plain_controls(a,"whenPlayed")||!a["targetResolver"].is_null()||!a["ifYouDo"].is_null()||effect["kind"]!="draw"||!matches!(effect["target"].as_str(),Some("ownPlayer"|"enemyPlayer"))||!remaining_hp_draw(effect) {return Err("remaining-HP draw requires sole ordinary play draw with reviewed Restore".into());}
                    } else if count_shield {
                        let select=&effect["onTrue"];
                        if exhaust_tokens||source_class!=Some("NonLeaderUnitCard")||!exact_type("unit")||entry["keywordValues"]!=serde_json::json!({})||entry["card"]["keywords"]!=serde_json::json!([])||!plain_controls(a,"whenPlayed")||!a["targetResolver"].is_null()||!a["ifYouDo"].is_null()||effect["kind"]!="conditional"||effect["onFalse"]!=serde_json::json!({"kind":"noAction"})||select["kind"]!="select"||select["effect"]["kind"]!="giveShield"||select["effect"]["target"]!="chosen"||!["any","self","opponent"].iter().any(|owner|select["targetResolver"]==serde_json::json!({"owner":owner,"type":"unit","zone":"anyAttackable"})) {return Err("conditional Shield selection requires sole complete ordinary unit play metadata".into());}
                    } else {
                        let next=&a["ifYouDo"];let effects=next["effect"]["effects"].as_array().ok_or("captured token continuation requires finite leaves")?;
                        if source_class!=Some("EventCard")||!exact_type("event")||entry["card"]["unique"]!=false||entry["keywordValues"]!=serde_json::json!({})||entry["card"]["keywords"]!=serde_json::json!([])||!plain_controls(a,"event")||effect!=&serde_json::json!({"kind":"exhaust","target":"chosen"})||!["any","self","opponent"].iter().any(|owner|a["targetResolver"]==serde_json::json!({"owner":owner,"type":"unit","zone":"anyAttackable"}))||!plain_controls(next,"")||!next["targetResolver"].is_null()||!next["ifYouDo"].is_null()||effects.len()!=2||!effects.iter().all(|e|matches!(e["kind"].as_str(),Some("giveExperience"|"giveShield"))&&e["target"]=="chosen") {return Err("captured exhaustion requires a sole ordinary event and same-target token continuation".into());}
                    }
                }
                if ["abilities","attachCondition","piloting"].iter().any(|key|contains_presence_query(&entry[*key])) && (source_class!=Some("NonLeaderUnitCard")||!exact_type("unit")||!sides.unit.is_empty()||!sides.leader.is_empty()||!complete_plain_source_metadata(entry)) {return Err("arena-card/deployed-leader queries require ordinary unit card-side metadata".into());}
                fn parent_printed_title(v:&Value)->bool {v["kind"]=="ref"&&v["path"]==serde_json::json!(["source","parentCard","title"]) || match v {Value::Object(o)=>o.values().any(parent_printed_title),Value::Array(a)=>a.iter().any(parent_printed_title),_=>false}}
                let attached_context=sides.card.iter().chain(&sides.unit).chain(&sides.leader).any(|a|a.attached_context.is_some());
                let parent_title=parent_printed_title(&entry["abilities"]);
                if (attached_context || parent_title) && !crate::attached_constants::entry(entry) {
                    if entry["baseClass"]!="UpgradeCard" || !exact_type("upgrade") || sides.card.len()!=1 || !sides.unit.is_empty() || !sides.leader.is_empty() || !entry["card"]["unique"].is_boolean() || !entry["piloting"].is_null() || ["pilotText","pilotBox","pilotingBox"].iter().any(|k|!entry["card"][k].is_null()) || entry["keywordValues"]!=serde_json::json!({}) || !entry["card"]["keywords"].as_array().is_some_and(|keywords|keywords.iter().all(|k|k.as_str().is_some_and(|k|CORE_KEYWORDS.contains(&k)))) {return Err("attached context or parent-title constant requires sole ordinary upgrade metadata".into());}
                }
                if contains_keyword_inventory(&entry["abilities"]) {
                    let ordinary=entry["baseClass"]=="NonLeaderUnitCard"&&exact_type("unit")&&sides.card.len()==1&&sides.card.iter().all(keyword_inventory_registration)&&sides.unit.is_empty()&&sides.leader.is_empty()&&entry["piloting"].is_null()&&entry["attachCondition"].is_null()&&entry["card"]["unique"].is_boolean()&&entry["keywordValues"]==serde_json::json!({})&&entry["card"]["keywords"]==serde_json::json!([])&&["pilotText","pilotBox","pilotingBox"].iter().all(|k|entry["card"][k].is_null());
                    // Boba Fett Daimyo: leader onCardPlayed exhaust→stats beside unit-side keyword aura.
                    let leader_boba=entry["baseClass"]=="LeaderUnitCard"&&exact_type("leader")&&sides.card.is_empty()&&sides.leader.len()==1&&sides.unit.len()==1&&crate::death_hooks::card_played_registration(&sides.leader[0])&&keyword_inventory_registration(&sides.unit[0])&&entry["piloting"].is_null()&&entry["attachCondition"].is_null()&&entry["card"]["unique"]==true&&entry["keywordValues"]==serde_json::json!({})&&entry["card"]["keywords"]==serde_json::json!([])&&["pilotText","pilotBox","pilotingBox"].iter().all(|k|entry["card"][k].is_null());
                    if !ordinary && !leader_boba {return Err("keyword inventory requires sole ordinary nonleader unit metadata".into());}
                }
                if sides.card.iter().chain(&sides.unit).chain(&sides.leader).any(|a|parent_upgrade_registration(a).is_some()) {
                    if entry["baseClass"]!="UpgradeCard"||!exact_type("upgrade")||sides.card.len()!=1||!sides.unit.is_empty()||!sides.leader.is_empty()||entry["card"]["unique"]!=false||!entry["card"]["title"].is_string()||!entry["attachCondition"].is_null()||!entry["piloting"].is_null()||["pilotText","pilotBox","pilotingBox"].iter().any(|k|!entry["card"][k].is_null())||entry["keywordValues"]!=serde_json::json!({})||entry["card"]["keywords"]!=serde_json::json!([]) {return Err("parent token play requires sole ordinary nonunique attached upgrade metadata".into());}
                }
                if sides.card.iter().any(|a|a.target_resolvers.iter().any(|d|d.chooser!="self")) {
                    let no_keywords=entry["keywordValues"].as_object().is_some_and(|v|v.is_empty())&&entry["card"]["keywords"].as_array().is_some_and(|v|v.is_empty());
                    let no_pilot=["pilotText","pilotBox","pilotingBox"].iter().all(|k|entry["card"][k].is_null())&&entry["piloting"].is_null();
                    if sides.card.len()!=1 || !sides.leader.is_empty() || !sides.unit.is_empty() || !no_keywords || !no_pilot || !entry["attachCondition"].is_null() || !entry["card"]["unique"].is_boolean() || !((entry["baseClass"]=="NonLeaderUnitCard"&&exact_type("unit")&&sides.card[0].trigger=="whenPlayed")||(entry["baseClass"]=="EventCard"&&exact_type("event")&&sides.card[0].trigger=="event")) {return Err("opponent named chooser requires a sole ordinary play/event source without side, intrinsic or pilot mechanics".into());}
                }
                if sides.card.iter().any(named_unit_base_damage) {
                    let no_keywords = entry["keywordValues"].as_object().is_some_and(|values| values.is_empty()) &&
                        entry["card"]["keywords"].as_array().is_some_and(|values| values.is_empty());
                    let no_pilot = ["pilotText", "pilotBox", "pilotingBox"].iter().all(|key| entry["card"][key].is_null()) &&
                        entry["piloting"].is_null();
                    if sides.card.len() != 1 || !sides.leader.is_empty() || !sides.unit.is_empty() ||
                        entry["baseClass"] != "NonLeaderUnitCard" || !exact_type("unit") || !no_keywords || !no_pilot ||
                        !entry["attachCondition"].is_null() {
                        return Err("named unit/base damage requires a sole ordinary nonleader card-side source without intrinsic keywords or piloting".into());
                    }
                }
                if sides.leader.iter().chain(&sides.unit).any(|a|!a.target_resolvers.is_empty()) || sides.card.iter().any(|a|!a.target_resolvers.is_empty() && !((a.trigger=="event" && entry["baseClass"]=="EventCard" && exact_type("event")) || (["whenPlayed","onAttack"].contains(&a.trigger.as_str()) && entry["baseClass"]=="NonLeaderUnitCard" && exact_type("unit")))) {return Err("named selectors support ordinary unit WhenPlayed or event card-side sources".into());}
                let upgrade_weakness = entry["baseClass"] == "UpgradeCard" && exact_type("upgrade") &&
                    sides.card.len() == 1 && sides.leader.is_empty() && sides.unit.is_empty() &&
                    entry["keywordValues"].as_object().is_some_and(|v|v.is_empty()) &&
                    entry["card"]["keywords"].as_array().is_some_and(|v|v.is_empty()) &&
                    ["pilotText","pilotBox","pilotingBox"].iter().all(|key|entry["card"][key].is_null()) &&
                    entry["piloting"].is_null() && entry["attachCondition"].is_null() &&
                    sides.card.first().is_some_and(|a|a.trigger == "whenPlayed" && a.optional &&
                        matches!(&a.effect,Effect::GiveWeakness{amount:Expr::Const{value},target:Target::Name(n)} if value.as_u64()==Some(1)&&n=="chosen") &&
                        a.target_resolver.as_ref().is_some_and(|r|r.owner=="any"&&r.card_type=="unit"&&r.zone.as_deref()==Some("anyAttackable")&&r.predicate.is_none()));
                let extension=sides.card.iter().chain(&sides.unit).chain(&sides.leader).any(|a|weakness_extension(a).is_some());
                let leader_heal=sides.leader.iter().any(leader_heal_weakness_registration);
                if leader_heal && (source_class!=Some("LeaderUnitCard") || !exact_type("leader") || !sides.card.is_empty() || sides.leader.len()!=1 || sides.unit.len()!=1
                    || !token_upgrade_attack_registration(&sides.unit[0]) || entry["keywordValues"]!=serde_json::json!({}) || entry["card"]["keywords"]!=serde_json::json!([])
                    || !entry["card"]["unique"].is_boolean() || !entry["piloting"].is_null() || !entry["attachCondition"].is_null()
                    || ["pilotText","pilotBox","pilotingBox"].iter().any(|k|entry["card"].get(*k).is_some_and(|v|!v.is_null()))) {return Err("leader heal/Weakness requires complete scalar action and bounded optional attack sides".into());}
                if extension && !leader_heal {
                    let source=if sides.card.iter().any(|a|weakness_extension(a)==Some(WeaknessExtension::Mixed)) {
                        sides.card.len()==2&&sides.card.iter().all(|a|weakness_extension(a)==Some(WeaknessExtension::Mixed))&&sides.card.iter().any(|a|a.trigger=="whenPlayed")&&sides.card.iter().any(|a|a.trigger=="whenDefeated")&&entry["baseClass"]=="NonLeaderUnitCard"&&exact_type("unit")
                    }else {sides.card.len()==1&&((weakness_extension(&sides.card[0])==Some(WeaknessExtension::Torrent)&&entry["baseClass"]=="EventCard"&&exact_type("event"))||(matches!(weakness_extension(&sides.card[0]),Some(WeaknessExtension::Picador|WeaknessExtension::Sequence))&&entry["baseClass"]=="NonLeaderUnitCard"&&exact_type("unit")))};
                    if !source||!sides.leader.is_empty()||!sides.unit.is_empty()||!entry["piloting"].is_null()||!entry["attachCondition"].is_null()||!entry["card"]["unique"].is_boolean()||!entry["keywordValues"].as_object().is_some_and(|v|v.is_empty())||["pilotText","pilotBox","pilotingBox"].iter().any(|k|entry["card"].get(*k).is_some_and(|v|!v.is_null())) {return Err("extended Weakness requires complete ordinary event/unit metadata and bounded registration topology".into());}
                }
                let leader_weakness=sides.leader.iter().chain(&sides.unit).any(scalar_weakness_registration);
                if leader_weakness && (source_class!=Some("LeaderUnitCard") || !exact_type("leader") || !sides.card.is_empty() || sides.leader.len()!=1 || sides.unit.len()!=1
                    || sides.leader[0].trigger!="action" || sides.unit[0].trigger!="onAttack" || !sides.leader.iter().chain(&sides.unit).all(scalar_weakness_registration)
                    || !entry["piloting"].is_null() || !entry["attachCondition"].is_null() || !entry["card"]["unique"].is_boolean()
                    || entry["keywordValues"]!=serde_json::json!({}) || entry["card"]["keywords"]!=serde_json::json!([])
                    || ["pilotText","pilotBox","pilotingBox"].iter().any(|k|entry["card"].get(*k).is_some_and(|v|!v.is_null()))) {return Err("leader Weakness requires complete bounded action and optional attack sides".into());}
                if !extension && !leader_weakness && (sides.leader.iter().chain(&sides.unit).any(|a|contains_weakness(&a.effect)) || sides.card.iter().any(|a|contains_weakness(&a.effect) && !(entry["baseClass"]=="NonLeaderUnitCard" && exact_type("unit") || upgrade_weakness)) ) {return Err("Weakness source requires ordinary unit or bounded ordinary upgrade play metadata".into());}
                if sides.card.iter().chain(&sides.unit).chain(&sides.leader).any(contains_arena_tokens) {
                    let kinds=sides.card.iter().map(arena_token_kind).collect::<Option<Vec<_>>>().ok_or("unsupported arena token registration")?;
                    if kinds.contains(&4) && (!complete_plain_source_metadata(entry)||entry["keywordValues"]!=serde_json::json!({})||entry["card"]["keywords"]!=serde_json::json!([])||entry["card"]["unique"]!=false) {return Err("unshielded ground snapshot requires complete ordinary nonunique unit metadata".into());}
                    let pair=kinds==[1,1]&&sides.card[0].trigger=="whenPlayed"&&sides.card[1].trigger=="whenDefeated";
                    if !entry["card"]["unique"].is_boolean()||entry["baseClass"]!="NonLeaderUnitCard"||entry["card"]["types"]!=serde_json::json!(["unit"])||!sides.unit.is_empty()||!sides.leader.is_empty()||!(pair||(kinds.len()==1&&kinds[0]!=1))||!entry["attachCondition"].is_null()||!entry["piloting"].is_null()||["pilotText","pilotBox","pilotingBox"].iter().any(|k|entry["card"].get(*k).is_some_and(|v|!v.is_null())) {return Err("arena token source requires complete ordinary unit shape".into());}
                }
                fn ordered_marker(v:&Value)->bool {match v {Value::Object(o)=>o.contains_key("controllerOrder")||o.values().any(ordered_marker),Value::Array(a)=>a.iter().any(ordered_marker),_=>false}}
                let ordered=["abilities","attachCondition","piloting"].iter().any(|k|ordered_marker(&entry[*k]));
                if ordered {
                    let a=sides.card.first().ok_or("ordered ground family needs sole card registration")?;
                    let plain=!a.optional&&!a.requires_confirmation&&!a.coordinate&&a.attached_context.is_none()&&a.costs.is_empty()&&a.context_title.is_empty()&&a.target_resolver.is_none()&&a.target_resolvers.is_empty()&&a.condition.is_none()&&a.then.is_none()&&a.if_you_do.is_none();
                    let operation=match &a.effect {
                        Effect::Damage{target:Target::ArenaList(l),amount:Expr::Const{value}}=>l.validate_ordered_ground().is_ok()&&value.as_u64().is_some_and(|n|(1..=3).contains(&n))&&((a.trigger=="event"&&source_class==Some("EventCard")&&exact_type("event"))||(a.trigger=="whenDefeated"&&source_class==Some("NonLeaderUnitCard")&&exact_type("unit"))),
                        Effect::Exhaust{target:Target::ArenaList(l)}=>l.validate_ordered_ground().is_ok()&&a.trigger=="whenPlayed"&&source_class==Some("NonLeaderUnitCard")&&exact_type("unit"),_=>false};
                    if !plain||!operation||sides.card.len()!=1||!sides.unit.is_empty()||!sides.leader.is_empty()||!complete_plain_source_metadata(entry)||entry["keywordValues"]!=serde_json::json!({})||entry["card"]["keywords"]!=serde_json::json!([])||entry["card"]["unique"]!=false {return Err("ordered ground effects require complete sole mandatory ordinary source metadata".into());}
                }
                if !ordered && sides.card.iter().chain(&sides.unit).chain(&sides.leader).any(contains_arena_damage) {
                    if sides.card.iter().any(conditional_arena_damage_ability)&&sides.card.len()!=1 {return Err("conditional arena damage requires a sole ordinary unit registration".into());}
                    let companion=|a:&Ability|a.trigger=="constant"&&a.coordinate&&!a.optional&&!a.requires_confirmation&&a.costs.is_empty()&&a.context_title.is_empty()&&a.target_resolver.is_none()&&a.target_resolvers.is_empty()&&a.condition.is_none()&&a.then.is_none()&&a.if_you_do.is_none()&&matches!(&a.effect,Effect::GainKeyword{target:Target::Name(n),keyword,amount:Expr::Const{value},duration} if n=="self"&&keyword=="raid"&&duration=="persistent"&&value.as_u64().is_some_and(|n|n<=65535));
                    if entry.get("baseClass").and_then(Value::as_str)!=Some("NonLeaderUnitCard") || entry["card"]["types"]!=serde_json::json!(["unit"]) || !sides.unit.is_empty() || !sides.leader.is_empty() || sides.card.iter().filter(|a|contains_arena_damage(a)).count()!=1 || !sides.card.iter().all(|a|arena_damage_ability(a)||companion(a)) || entry.get("piloting").is_some_and(|v|!v.is_null()) || ["pilotText","pilotBox","pilotingBox"].iter().any(|k|entry["card"].get(*k).is_some_and(|v|!v.is_null())) {
                        return Err("arena damage requires ordinary unit metadata and only a bounded Coordinate Raid companion".into());
                    }
                }
                if sides.card.iter().chain(&sides.unit).chain(&sides.leader).any(contains_new_arena_keywords) {
                    let ordinary=(sides.card.first().is_some_and(|a|a.trigger=="event")&&entry["baseClass"]=="EventCard"&&exact_type("event"))||(sides.card.first().is_some_and(|a|a.trigger=="onAttack")&&entry["baseClass"]=="NonLeaderUnitCard"&&exact_type("unit"));
                    if !ordinary||sides.card.len()!=1||!sides.unit.is_empty()||!sides.leader.is_empty()||!arena_keyword_registration(&sides.card[0])||!entry["piloting"].is_null()||!entry["attachCondition"].is_null()||["pilotText","pilotBox","pilotingBox"].iter().any(|k|entry["card"].get(*k).is_some_and(|v|!v.is_null())) {return Err("arena keyword batch requires sole ordinary event/unit registration without side mechanics".into());}
                }
                let arena_stats=|a:&Ability|matches!(&a.effect,Effect::ModifyStats{target:Target::ArenaList(_),..});
                if sides.leader.iter().chain(&sides.unit).any(arena_stats) || sides.card.iter().any(|a|arena_stats(a) && !((a.trigger=="event"&&entry["baseClass"]=="EventCard"&&exact_type("event")) || (["whenPlayed","onAttack"].contains(&a.trigger.as_str())&&entry["baseClass"]=="NonLeaderUnitCard"&&exact_type("unit")))) {return Err("arena phase stats require ordinary unit play/attack or event card-side metadata".into());}
                if sides.card.iter().any(|a|arena_stats(a)&&a.trigger=="onAttack") && (!sides.leader.is_empty() || !sides.unit.is_empty() || !entry["piloting"].is_null() || !entry["attachCondition"].is_null() || ["pilotText","pilotBox","pilotingBox"].iter().any(|k|!entry["card"][k].is_null())) {return Err("arena phase-stat attacks require only ordinary card-side abilities without piloting or attachment mechanics".into());}
                if sides.card.iter().chain(&sides.unit).chain(&sides.leader).any(|a|force_continuation(a).is_some()) {
                    let ordinary=sides.card.first().is_some_and(|a|if a.trigger=="event" {entry["baseClass"]=="EventCard"&&exact_type("event")}else{entry["baseClass"]=="NonLeaderUnitCard"&&exact_type("unit")});
                    if !ordinary||sides.card.len()!=1||!sides.unit.is_empty()||!sides.leader.is_empty()||!entry["attachCondition"].is_null()||!entry["piloting"].is_null()||["pilotText","pilotBox","pilotingBox"].iter().any(|k|entry["card"].get(*k).is_some_and(|v|!v.is_null())) {return Err("Force continuation requires sole ordinary source registration without side mechanics".into());}
                }
                let upgrade_force=|a:&Ability|entry["baseClass"]=="UpgradeCard" && exact_type("upgrade") && a.trigger=="whenPlayed" && a.optional && matches!(&a.effect,Effect::UseForce{target:Target::Name(n)} if n=="ownPlayer") && a.if_you_do.as_ref().is_some_and(|next|next.target_resolver.as_ref().is_some_and(|r|r.owner=="any" && r.card_type=="any" && r.zone.as_deref()==Some("base")) && matches!(&next.effect,Effect::Heal{target:Target::Name(n),amount:Expr::Const{value}} if n=="chosen" && value.as_u64().is_some_and(|n|(1..=65535).contains(&n))));
                if sides.leader.iter().chain(&sides.unit).any(|a|contains_force(&a.effect)) || sides.card.iter().any(|a|contains_force(&a.effect) && !((entry["baseClass"]=="NonLeaderUnitCard" && exact_type("unit") && (["whenPlayed","whenDefeated","onAttack"].contains(&a.trigger.as_str()) || (a.trigger=="eventHooks" && crate::death_hooks::card_played_registration(a)))) || (entry["baseClass"]=="EventCard" && exact_type("event") && a.trigger=="event") || upgrade_force(a))) {return Err("Force registration requires ordinary unit/event or bounded upgrade play-heal metadata".into());}
                if sides.leader.iter().chain(&sides.unit).any(contains_force_presence) || sides.card.iter().any(|a|contains_force_presence(a) && (entry["baseClass"]!="NonLeaderUnitCard" || !exact_type("unit"))) {return Err("Force presence requires ordinary unit card-side metadata".into());}
                let top_deck=["abilities","attachCondition","piloting"].iter().any(|key|top_deck_marker(&entry[*key]));
                if top_deck && (sides.card.len()!=1||!sides.unit.is_empty()||!sides.leader.is_empty()||!reviewed_top_deck_registration(&sides.card[0])||!(if imperial_resource_continuation(&sides.card[0])||simultaneous_resource_registration(&sides.card[0]) {entry["baseClass"]=="EventCard"&&exact_type("event")&&entry["card"]["unique"]==false}else{entry["baseClass"]=="NonLeaderUnitCard"&&exact_type("unit")})||!complete_plain_source_metadata(entry)||entry["keywordValues"]!=serde_json::json!({})||entry["card"]["keywords"]!=serde_json::json!([])) {return Err("top-deck resource requires complete sole optional ordinary unit metadata".into());}
                if !top_deck && (sides.leader.iter().chain(&sides.unit).any(contains_resource) || sides.card.iter().any(|a|contains_resource(a) && !((entry["baseClass"]=="NonLeaderUnitCard" && exact_type("unit") && a.trigger=="whenDefeated") || (entry["baseClass"]=="EventCard" && exact_type("event") && a.trigger=="event")))) {return Err("resourceCard requires ordinary unit death/event metadata".into());}
                let coordinates=sides.card.iter().filter(|a|a.coordinate).count();
                if sides.leader.iter().chain(&sides.unit).any(|a|a.coordinate) || coordinates>1 || (coordinates==1 && (entry["baseClass"]!="NonLeaderUnitCard" || entry["keywordValues"]["coordinate"].as_u64()!=Some(1))) {return Err("Coordinate requires exactly one printed nonleader card-side registration".into());}
                if entry["keywordValues"].get("coordinate").is_some() && coordinates!=1 {return Err("printed Coordinate must have one implemented registration".into());}
                let mut out = CardPrograms::default();
                if !entry["attachCondition"].is_null() {
                    if entry["baseClass"] != "UpgradeCard" {return Err("attachCondition requires an ordinary upgrade".into());}
                    let condition:Expr=serde_json::from_value(entry["attachCondition"].clone()).map_err(|e|e.to_string())?;
                    if condition.live_only() {return Err("attachment conditions cannot capture modifierTarget/resourcePairs".into());}
                    condition.validate()?;out.attach_condition=Some(condition);
                }

                let mut programs = Vec::new();
                for (src, dst) in [
                    (&sides.leader, &mut out.leader),
                    (&sides.unit, &mut out.unit),
                    (&sides.card, &mut out.card),
                ] {
                    for ability in src {
                        let id = catalogue.programs.len() + programs.len();
                        if id >= u16::MAX as usize {
                            return Err("ability program capacity exceeded".into());
                        }
                        dst.push(id as u16);
                        let program=Program::compile(ability)?;
                        if entry["baseClass"]=="UpgradeCard" {
                            fn source_target(target:&Target)->bool {
                                match target {
                                    Target::Name(name)=>name=="self",
                                    Target::Expression(Expr::Ref {path})=>path.iter().map(String::as_str).collect::<Vec<_>>()==["source"],
                                    Target::Expression(Expr::Conditional {then,r#else,..})=>source_target(&Target::Expression(*then.clone()))||source_target(&Target::Expression(*r#else.clone())),
                                    _=>false,
                                }
                            }
                            fn attachment_defeat(effect:&Effect)->bool {
                                match effect {
                                    Effect::Defeat {target}=>source_target(target),
                                    Effect::Simultaneous {effects}|Effect::PhaseModifiers {effects}|Effect::Sequence {effects}=>effects.iter().any(attachment_defeat),
                                    Effect::Select {effect,..}=>attachment_defeat(effect),
                                    Effect::Conditional {on_true,on_false,..}=>attachment_defeat(on_true)||on_false.as_deref().is_some_and(attachment_defeat),
                                    _=>false,
                                }
                            }
                            if program.instructions.iter().any(|i|matches!(i,Instruction::Group(effect) if attachment_defeat(effect))) {return Err("grouped upgrade source defeat is not modeled".into());}
                        }
                        programs.push(program);
                    }
                }
                Ok((out, programs))
            };
            match result() {
                Ok((card, programs)) => {
                    catalogue.programs.extend(programs);
                    catalogue.cards.insert(code.clone(), card);
                    catalogue.metadata.insert(code, entry["card"].clone());
                }
                Err(e) => {
                    catalogue.rejected.insert(
                        code,
                        format!(
                            "{}: {e}",
                            entry["internalName"].as_str().unwrap_or("unnamed card")
                        ),
                    );
                }
            }
        }
        // Preserve structural admission and canonical program hashes before
        // adding runtime attachments. Unsupported expression roots stay portable.
        for program in &mut catalogue.programs {
            program.visit_expression_roots_mut(&mut Expr::bind_compiled);
        }
        for card in catalogue.cards.values_mut() {
            if let Some(condition) = &mut card.attach_condition { condition.bind_compiled(); }
        }
        Ok(catalogue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn attach_test_executable(expression: &mut Expr) {
        let original = Box::new(std::mem::replace(expression, Expr::number(0)));
        *expression = Expr::Compiled(CompiledExpr { original, index: usize::MAX });
    }
    #[test]
    fn compiled_expression_preserves_program_canonical_bytes_and_clone() {
        let ability: Ability = serde_json::from_value(json!({
            "trigger":"action", "title":"Identity witness", "contextTitle":[{"kind":"const","value":"Witness"}],
            "condition":{"kind":"const","value":true},
            "costs":[{"kind":"resources","amount":{"kind":"const","value":1}}],
            "targetResolver":{"type":"unit","predicate":{"kind":"query","name":"isUnit","receiver":{"kind":"ref","path":["card"]}}},
            "effect":{"kind":"damage","target":"chosen","amount":{"kind":"const","value":1}}
        })).unwrap();
        let mut program = Program::compile(&ability).unwrap();
        let canonical = serde_json::to_vec(&program).unwrap();
        let executable = program.executable;
        let mut roots = 0;
        program.visit_expression_roots_mut(&mut |expression| {
            let valid = expression.validate();
            attach_test_executable(expression);
            assert_eq!(expression.validate(), valid);
            roots += 1;
        });
        assert_eq!(roots, 5);
        assert_eq!(serde_json::to_vec(&program).unwrap(), canonical);
        assert_eq!(serde_json::to_vec(&program.clone()).unwrap(), canonical);
        assert_eq!(program.executable, executable);
    }
    #[test]
    fn compiled_expression_preserves_named_portable_rejections() {
        for kind in ["arenaCount", "unimplementedCollection", "compiled"] {
            let expression = json!({"kind":kind});
            let expected = format!("unknown variant `{kind}`, expected one of `nativeTokenPredicate`, `historyQuery`, `dataMember`, `historyAttackCheck`, `arenaList`, `arenaFilter`, `arenaSome`, `arenaLength`, `collection`, `array`, `math`, `const`, `ref`, `binary`, `unary`, `conditional`, `query`");
            assert_eq!(serde_json::from_value::<Expr>(expression.clone()).unwrap_err().to_string(), expected);
            let nested = json!({"kind":"unary", "op":"!", "value":expression});
            assert_eq!(serde_json::from_value::<Expr>(nested).unwrap_err().to_string(), expected);
        }
        for (kind, field) in [("collection", "name"), ("nativeTokenPredicate", "operation")] {
            assert_eq!(serde_json::from_value::<Expr>(json!({"kind":kind})).unwrap_err().to_string(),
                format!("missing field `{field}`"));
        }
        let forged = json!({"kind":"const", "value":1, "index":0});
        assert_eq!(serde_json::from_value::<Expr>(forged).unwrap_err().to_string(), "unknown field `index`, expected `value`");
    }
    #[test]
    fn compiled_expression_is_not_a_raw_ir_variant_or_field() {
        for value in [
            json!({"kind":"compiled","original":{"kind":"const","value":1},"index":0}),
            json!({"kind":"const","value":1,"index":0}),
            json!({"original":{"kind":"const","value":1},"index":0}),
        ] { assert!(serde_json::from_value::<Expr>(value).is_err()); }
        let mut expression = Expr::number(3);
        attach_test_executable(&mut expression);
        let bytes = serde_json::to_vec(&expression).unwrap();
        let portable: Expr = serde_json::from_slice(&bytes).unwrap();
        assert!(matches!(portable, Expr::Const { .. }));
        assert_eq!(bytes, serde_json::to_vec(&portable).unwrap());
        let resolver = TargetResolver { native:None, owner: "any".into(), card_type:"unit".into(), arena:None,
            zone:None, optional:false, predicate:None, mode:Some("upTo".into()), count:Some(expression), prompt_title:None, prompt_title_parts:vec![], player_selection:false };
        assert_eq!(resolver.max_targets(), Ok(3));
    }
    #[test]
    fn rejects_unknown_effects_instead_of_ignoring_them() {
        assert!(
            serde_json::from_value::<Effect>(json!({"kind":"capture","target":"chosen"})).is_err()
        );
    }
    #[test]
    fn rejects_unknown_fields_that_could_hide_additional_rules() {
        assert!(serde_json::from_value::<Effect>(json!({"kind":"damage","amount":{"kind":"const","value":1},"target":"chosen","lastingEffect":true})).is_err());
    }
    #[test]
    fn references_are_explicitly_bounded() {
        assert!(Expr::Ref {
            path: vec!["game".into(), "randomGenerator".into()]
        }
        .validate()
        .is_err());
        assert!(Expr::Ref {
            path: vec!["event".into(), "totalDistributed".into()]
        }
        .validate()
        .is_ok());
    }
    #[test]
    fn conditional_and_if_you_do_compile_real_continuations() {
        let a:Ability=serde_json::from_value(json!({"trigger":"action","title":"heal and pay","effect":{"kind":"heal","amount":{"kind":"const","value":2},"target":"self"},"ifYouDo":{"title":"pay","effect":{"kind":"damage","amount":{"kind":"ref","path":["event","totalDistributed"]},"target":"ownBase"}}})).unwrap();
        let p = Program::compile(&a).unwrap();
        assert!(matches!(
            p.instructions[1],
            Instruction::IfResult { skip: 3 }
        ));
    }
    fn compiled_group(value: Value) -> Result<Program, String> {
        let a: Ability=serde_json::from_value(value).map_err(|e|e.to_string())?;
        Program::compile(&a)
    }
    #[test]
    fn groups_preserve_nested_selection_and_phase_event_identity() {
        let primitive=json!({"kind":"damage","target":"chosen","amount":{"kind":"const","value":1}});
        let tree=json!({"kind":"simultaneous","effects":[{"kind":"damage","target":"self","amount":{"kind":"const","value":1}},{"kind":"select","targetResolver":{"type":"unit","owner":"opponent"},"effect":primitive}]});
        let p=compiled_group(json!({"trigger":"whenPlayed","title":"group","effect":tree})).unwrap();
        assert!(matches!(&p.instructions[0],Instruction::Group(Effect::Simultaneous {effects}) if matches!(effects[1],Effect::Select{..})));
        let phase=json!({"kind":"phaseModifiers","effects":[{"kind":"modifyStats","target":"chosen","duration":"phase","power":{"kind":"const","value":1},"hp":{"kind":"const","value":0}},{"kind":"gainKeyword","target":"chosen","duration":"phase","keyword":"sentinel","amount":{"kind":"const","value":1}}]});
        let p=compiled_group(json!({"trigger":"event","title":"phase","effect":phase})).unwrap();
        assert!(matches!(p.instructions[0],Instruction::Apply(Effect::PhaseModifiers{..})));
    }
    #[test]
    fn bounded_selectors_compile_to_group_even_for_one_selected_card() {
        for mode in [json!({"mode":"upTo","count":{"kind":"const","value":1},"optional":true,"type":"unit"}),json!({"mode":"unlimited","optional":true,"type":"base"})] {
            let p=compiled_group(json!({"trigger":"event","title":"multi","targetResolver":mode,"effect":{"kind":"heal","target":"chosen","amount":{"kind":"const","value":2}}})).unwrap();
            assert!(matches!(p.instructions[0],Instruction::Select(_)));assert!(matches!(p.instructions[1],Instruction::Group(_)));
        }
        for resolver in [json!({"mode":"upTo","count":{"kind":"const","value":4}}),json!({"mode":"upTo","count":{"kind":"ref","path":["player","readyResourceCount"]}}),json!({"mode":"unlimited","optional":true,"type":"unit"})] {
            assert!(compiled_group(json!({"trigger":"event","title":"bad","targetResolver":resolver,"effect":{"kind":"ready","target":"chosen"}})).is_err());
        }
    }
    #[test]
    fn unverified_group_shapes_and_array_captures_reject() {
        let damage=json!({"kind":"damage","target":"chosen","amount":{"kind":"const","value":1}});
        let shield=json!({"kind":"giveShield","target":"chosen","amount":{"kind":"const","value":1}});
        let draw=json!({"kind":"draw","target":"ownPlayer","amount":{"kind":"const","value":1}});
        for children in [vec![damage.clone(),shield],vec![damage.clone(),draw]] {
            assert!(compiled_group(json!({"trigger":"event","title":"bad","effect":{"kind":"simultaneous","effects":children}})).is_err());
        }
        assert!(compiled_group(json!({"trigger":"event","title":"array capture","targetResolver":{"mode":"upTo","count":{"kind":"const","value":2},"optional":true},"effect":{"kind":"damage","target":"chosen","amount":{"kind":"ref","path":["target","damage"]}}})).is_err());
        assert!(compiled_group(json!({"trigger":"event","title":"continuation","effect":{"kind":"simultaneous","effects":[damage]},"then":{"title":"later","effect":{"kind":"noAction"}}})).is_err());
    }
    #[test]
    fn phase_modifier_lists_validate_shared_target_and_duration() {
        for bad in [json!({"kind":"gainKeyword","target":"self","duration":"phase","keyword":"sentinel","amount":{"kind":"const","value":1}}),json!({"kind":"gainKeyword","target":"chosen","duration":"persistent","keyword":"sentinel","amount":{"kind":"const","value":1}})] {
            assert!(compiled_group(json!({"trigger":"event","title":"bad phase","effect":{"kind":"phaseModifiers","effects":[{"kind":"modifyStats","target":"chosen","duration":"phase","power":{"kind":"const","value":1},"hp":{"kind":"const","value":0}},bad]}})).is_err());
        }
    }
    #[test]
    fn token_contingents_count_toward_group_capacity() {
        let shields=vec![json!({"kind":"giveShield","target":"chosen","amount":{"kind":"const","value":2}});8];
        assert!(compiled_group(json!({"trigger":"event","title":"oversized","targetResolver":{"mode":"upTo","count":{"kind":"const","value":3},"optional":true},"effect":{"kind":"simultaneous","effects":shields}})).is_err());
    }

    #[test]
    fn expanded_named_targets_count_toward_physical_record_budget() {
        let eight=vec![json!({"kind":"damage","target":"bothBases","amount":{"kind":"const","value":1}});8];
        let four=vec![json!({"kind":"simultaneous","effects":eight});4];
        let err=compiled_group(json!({"trigger":"event","title":"too many","effect":{"kind":"simultaneous","effects":four}})).unwrap_err();
        assert!(err.contains("record budget"));
        let phase=vec![json!({"kind":"gainKeyword","target":"chosen","duration":"phase","keyword":"sentinel","amount":{"kind":"const","value":1}});8];
        assert!(compiled_group(json!({"trigger":"event","title":"too many phase leaves","targetResolver":{"mode":"upTo","count":{"kind":"const","value":3},"optional":true,"type":"unit"},"effect":{"kind":"simultaneous","effects":[{"kind":"phaseModifiers","effects":phase},{"kind":"phaseModifiers","effects":phase}]}})).unwrap_err().contains("record budget"));
    }
    #[test]
    fn groups_reject_player_targets_and_scalar_target_expressions_under_arrays() {
        for target in [json!("bothPlayers"),json!({"kind":"ref","path":["player"]}),json!({"kind":"ref","path":["target","controller"]})] {
            assert!(compiled_group(json!({"trigger":"event","title":"player target","effect":{"kind":"simultaneous","effects":[{"kind":"heal","target":target,"amount":{"kind":"const","value":1}}]}})).is_err());
        }
        for count in [1,2] { for target in [json!({"kind":"ref","path":["target"]}),json!({"kind":"conditional","condition":{"kind":"const","value":true},"then":{"kind":"ref","path":["target"]},"else":{"kind":"ref","path":["source"]}})] {
            assert!(compiled_group(json!({"trigger":"event","title":"array target","targetResolver":{"mode":"upTo","count":{"kind":"const","value":count},"optional":true,"type":"unit"},"effect":{"kind":"heal","target":target,"amount":{"kind":"const","value":1}}})).is_err());
        } }
    }
    #[test]
    fn damage_groups_require_disjoint_destinations_and_no_unverified_defeat_mix() {
        let damage=json!({"kind":"damage","target":"chosen","amount":{"kind":"const","value":1}});
        assert!(compiled_group(json!({"trigger":"event","title":"repeat","effect":{"kind":"simultaneous","effects":[damage,damage]}})).is_err());
        assert!(compiled_group(json!({"trigger":"event","title":"mixed","effect":{"kind":"simultaneous","effects":[{"kind":"damage","target":"self","amount":{"kind":"const","value":1}},{"kind":"defeat","target":"chosen"}]}})).is_err());
        assert!(compiled_group(json!({"trigger":"event","title":"distinct bases","effect":{"kind":"simultaneous","effects":[{"kind":"damage","target":"ownBase","amount":{"kind":"const","value":1}},{"kind":"damage","target":"enemyBase","amount":{"kind":"const","value":1}}]}})).is_ok());
        let p=compiled_group(json!({"trigger":"onAttack","title":"other unit","effect":{"kind":"simultaneous","effects":[{"kind":"damage","target":"self","amount":{"kind":"const","value":1}},{"kind":"select","targetResolver":{"type":"unit","predicate":{"kind":"binary","op":"!==","left":{"kind":"ref","path":["card"]},"right":{"kind":"ref","path":["source"]}}},"effect":{"kind":"damage","target":"chosen","amount":{"kind":"const","value":1}}}]}}));assert!(p.is_ok());
    }

    #[test]
    fn standalone_phase_lists_reject_noncard_destinations() {
        assert!(compiled_group(json!({"trigger":"event","title":"player phase","effect":{"kind":"phaseModifiers","effects":[{"kind":"gainKeyword","target":"ownPlayer","duration":"phase","keyword":"sentinel","amount":{"kind":"const","value":1}}]}})).is_err());
    }
    #[test]
    fn zero_record_trees_still_require_bounded_group_operations() {
        let leaf=json!({"kind":"conditional","condition":{"kind":"const","value":true},"onTrue":{"kind":"noAction"},"onFalse":{"kind":"noAction"}});
        let mut tree=leaf;for _ in 0..5 {tree=json!({"kind":"simultaneous","effects":vec![tree;8]});}
        let err=compiled_group(json!({"trigger":"event","title":"op overflow","effect":tree})).unwrap_err();assert!(err.contains("operation budget"));
    }
    #[test]
    fn sibling_target_capture_after_select_rejects_but_fresh_selections_remain_legal() {
        let select=json!({"kind":"select","targetResolver":{"type":"unit"},"effect":{"kind":"heal","target":"chosen","amount":{"kind":"const","value":1}}});
        for next in [json!({"kind":"ready","target":"chosen"}),json!({"kind":"heal","target":"ownBase","amount":{"kind":"ref","path":["target","damage"]}}),json!({"kind":"conditional","condition":{"kind":"query","name":"isUnit","receiver":{"kind":"ref","path":["target"]},"args":[]},"onTrue":{"kind":"noAction"}})] {
            assert!(compiled_group(json!({"trigger":"event","title":"shared target","effect":{"kind":"simultaneous","effects":[select,next]}})).is_err());
        }
        assert!(compiled_group(json!({"trigger":"event","title":"fresh target","effect":{"kind":"simultaneous","effects":[select,select]}})).is_ok());
    }

    #[test]
    fn exact_card_title_predicates_support_named_friendly_auras() {
        for (title,keyword) in [("Zuckuss","ambush"),("4-LOM","saboteur")] {
            assert!(compiled_group(json!({"trigger":"constant","title":"named aura","targetResolver":{"owner":"self","type":"unit","zone":"arena","predicate":{"kind":"binary","op":"===","left":{"kind":"ref","path":["card","title"]},"right":{"kind":"const","value":title}}},"effect":{"kind":"simultaneous","effects":[{"kind":"modifyStats","target":"chosen","duration":"persistent","power":{"kind":"const","value":1},"hp":{"kind":"const","value":1}},{"kind":"gainKeyword","target":"chosen","duration":"persistent","keyword":keyword,"amount":{"kind":"const","value":1}}]}})).is_ok());
        }
        assert!(Expr::Ref {path:vec!["card".into(),"title".into()]}.validate().is_ok());
        assert!(Expr::Ref {path:vec!["card".into(),"title".into()]}.validate_number().is_err());
        assert!(Expr::Ref {path:vec!["source".into(),"parentCard".into(),"title".into()]}.validate().is_ok());
        assert!(Expr::Ref {path:vec!["source".into(),"parentUnit".into(),"title".into()]}.validate().is_err());
        assert!(compiled_group(json!({"trigger":"event","title":"text amount","effect":{"kind":"damage","target":"chosen","amount":{"kind":"ref","path":["card","title"]}}})).is_err());
    }

    #[test]
    fn array_group_values_reject_unbound_card_alias_but_keep_local_arena_predicates() {
        let resolver=json!({"mode":"upTo","count":{"kind":"const","value":2},"optional":true,"type":"unit"});
        for amount in [json!({"kind":"ref","path":["card","damage"]}),json!({"kind":"query","name":"getPower","receiver":{"kind":"ref","path":["card"]},"args":[]})] {
            assert!(compiled_group(json!({"trigger":"event","title":"first alias","targetResolver":resolver,"effect":{"kind":"heal","target":"chosen","amount":amount}})).is_err());
        }
        let count=json!({"kind":"query","name":"arenaUnitCount","receiver":{"kind":"ref","path":["player"]},"args":[{"kind":"binary","op":">","left":{"kind":"ref","path":["card","damage"]},"right":{"kind":"const","value":0}}]});
        assert!(compiled_group(json!({"trigger":"event","title":"local predicate","targetResolver":resolver,"effect":{"kind":"heal","target":"chosen","amount":count}})).is_ok());
    }

    #[test]
    fn changed_sibling_card_aliases_reject_without_rejecting_local_query_bindings() {
        let select=json!({"kind":"select","targetResolver":{"type":"unit"},"effect":{"kind":"heal","target":"chosen","amount":{"kind":"const","value":1}}});
        for amount in [json!({"kind":"ref","path":["card","damage"]}),json!({"kind":"query","name":"getPower","receiver":{"kind":"ref","path":["card"]},"args":[]})] {
            let err=compiled_group(json!({"trigger":"event","title":"changed alias","effect":{"kind":"simultaneous","effects":[select,{"kind":"heal","target":"ownBase","amount":amount}]}})).unwrap_err();assert!(err.contains("target-dependent sibling"));
        }
        let amount=json!({"kind":"query","name":"arenaUnitCount","receiver":{"kind":"ref","path":["player"]},"args":[{"kind":"binary","op":">","left":{"kind":"ref","path":["card","damage"]},"right":{"kind":"const","value":0}}]});
        assert!(compiled_group(json!({"trigger":"event","title":"local card binding","effect":{"kind":"simultaneous","effects":[select,{"kind":"heal","target":"ownBase","amount":amount}]}})).is_ok());
    }

}

// Inventory copy is intentionally independent of later gameplay keyword additions.
pub const COPY_KEYWORDS: [&str; 9] = ["sentinel", "overwhelm", "saboteur", "ambush", "grit", "shielded", "raid", "restore", "hidden"];
pub const CORE_KEYWORDS: [&str; 10] = ["sentinel", "overwhelm", "saboteur", "ambush", "grit", "shielded", "raid", "restore", "hidden", "support"];
fn validate_keyword(keyword: &str) -> Result<(), String> { if CORE_KEYWORDS.contains(&keyword) { Ok(()) } else { Err(format!("unsupported keyword {keyword}")) } }

#[cfg(test)]
mod arena_list_phase_tests {
    use super::*;
    use serde_json::json;
    fn ability()->serde_json::Value {json!({"trigger":"event","title":"Phase stats","effect":{"kind":"modifyStats","duration":"phase","target":{"kind":"arenaList","receiver":{"kind":"ref","path":["player"]},"predicate":{"kind":"const","value":true}},"power":{"kind":"const","value":1},"hp":{"kind":"const","value":1}}})}
    fn compile(v:serde_json::Value)->Result<Program,String>{Program::compile(&serde_json::from_value(v).unwrap())}
    #[test] fn accepts_only_witnessed_standalone_shape() {
        assert!(compile(ability()).is_ok());
        let mut v=ability();let mut predicate=json!({"kind":"const","value":true});
        for _ in 0..20 {predicate=json!({"kind":"unary","op":"!","value":predicate});}
        v["effect"]["target"]["predicate"]=predicate;assert!(compile(v).is_err(),"arena-list expressions retain the general depth bound");
        let mut v=ability();v["trigger"]=json!("whenPlayed");assert!(compile(v).is_ok(),"ordinary WhenPlayed list effects are now source-witnessed");
        let mut v=ability();v["then"]=json!({"title":"Continuation","effect":{"kind":"noAction"}});assert!(compile(v).is_err());
        let mut v=ability();v["targetResolver"]=json!({"type":"unit"});assert!(compile(v).is_err());
        let mut v=ability();v["effect"]["target"]["receiver"]=json!({"kind":"ref","path":["game"]});assert!(compile(v).is_err());
        let mut v=ability();v["effect"]["target"]["predicate"]=json!({"kind":"query","name":"hasSomeTrait","receiver":{"kind":"ref","path":["card"]},"args":[{"kind":"const","value":"creature"}]});assert!(compile(v).is_ok(),"one static card trait is now source-witnessed");
        let mut v=ability();v["effect"]["power"]=json!({"kind":"ref","path":["target","damage"]});assert!(compile(v).is_err());
        let mut v=ability();v["effect"]["kind"]=json!("damage");v["effect"].as_object_mut().unwrap().remove("duration");v["effect"].as_object_mut().unwrap().remove("power");v["effect"].as_object_mut().unwrap().remove("hp");v["effect"]["amount"]=json!({"kind":"const","value":1});assert!(compile(v).is_err());
        let mut v=ability();v["effect"]=json!({"kind":"sequence","effects":[v["effect"].clone()]});assert!(compile(v).is_err());
    }
}

#[cfg(test)]
mod arena_list_defeat_tests {
    use super::*;use serde_json::json;
    fn ability()->serde_json::Value {json!({"trigger":"event","title":"Defeat","effect":{"kind":"defeat","target":{"kind":"arenaList","receiver":{"kind":"ref","path":["game"]},"predicate":{"kind":"const","value":true}}}})}
    fn compile(v:serde_json::Value)->Result<Program,String>{Program::compile(&serde_json::from_value(v).unwrap())}
    #[test]fn defeat_list_rejects_unwitnessed_contexts_and_predicates(){
        assert!(compile(ability()).is_ok());
        let mut v=ability();v["trigger"]=json!("onAttack");assert!(compile(v).is_err());
        let mut v=ability();v["then"]=json!({"title":"Next","effect":{"kind":"noAction"}});assert!(compile(v).is_err());
        let mut v=ability();v["effect"]["target"]["predicate"]=json!({"kind":"query","name":"isUpgraded","receiver":{"kind":"ref","path":["card"]},"args":[]});assert!(compile(v.clone()).is_ok());v["effect"]["target"]["predicate"]["receiver"]=json!({"kind":"ref","path":["target"]});assert!(compile(v).is_err());
        let mut v=ability();let mut p=json!({"kind":"const","value":true});for _ in 0..20 {p=json!({"kind":"unary","op":"!","value":p});}v["effect"]["target"]["predicate"]=p;assert!(compile(v).is_err());
        let mut v=ability();v["effect"]=json!({"kind":"simultaneous","effects":[v["effect"].clone()]});assert!(compile(v).is_err());
        let mut v=ability();v["effect"]["target"]["receiver"]=json!({"kind":"ref","path":["source"]});assert!(compile(v).is_err());
    }
}

/// Numeric storage slots remain independent of keyword bit positions.
pub fn numeric_keyword_slot(index: usize) -> Option<usize> { match index { 6 => Some(0), 7 => Some(1), _ => None } }

#[cfg(test)]
mod event_helper_metadata_tests {
    use super::*;
    use serde_json::json;
    fn entry(id:&str)->Value {
        let catalog:Value=serde_json::from_str(include_str!("../../../compiler/generated/catalog.json")).unwrap();
        catalog["cards"].as_array().unwrap().iter().find(|entry|entry["setCode"]==id).unwrap().clone()
    }
    fn admitted(entry:Value)->bool {
        let code=entry["setCode"].as_str().unwrap().to_owned();
        Catalogue::load(&json!({"schema":SCHEMA,"cards":[entry]})).unwrap().cards.contains_key(&code)
    }
    #[test]
    fn event_helpers_require_complete_ordinary_metadata_and_registration_scope() {
        for id in ["SEC_130","SHD_156"] {
            let source=entry(id);assert!(admitted(source.clone()),"positive {id}");
            for (class,types) in [(json!(null),json!(["event"])),(json!("UnknownCard"),json!(["event"])),(json!("NonLeaderUnitCard"),json!(["unit"])),(json!("LeaderUnitCard"),json!(["leader"])),(json!("UpgradeCard"),json!(["upgrade"])),(json!("TokenUnitCard"),json!(["token","unit"]))] {
                let mut bad=source.clone();bad["baseClass"]=class;bad["card"]["types"]=types;assert!(!admitted(bad),"class/type {id}");
            }
            for key in ["pilotText","pilotBox","pilotingBox","deployBox","deployCost","upgradePower","upgradeHp","epicAction"] {
                let mut bad=source.clone();bad["card"][key]=json!("unsupported");assert!(!admitted(bad),"metadata {id}/{key}");
            }
            for key in ["attachCondition","piloting"] {let mut bad=source.clone();bad[key]=json!({});assert!(!admitted(bad),"root {id}/{key}");}
            let mut bad=source.clone();bad["card"]["unique"]=Value::Null;assert!(!admitted(bad));
            let mut bad=source.clone();bad["keywordValues"]=json!({"raid":1});assert!(!admitted(bad));
            for (key,value) in [("optional",json!(true)),("coordinate",json!(true)),("requiresConfirmation",json!(true)),("condition",json!({"kind":"const","value":true})),("costs",json!([{"kind":"exhaustSelf"}])),("contextTitle",json!([{"kind":"const","value":"other"}])),("then",json!({"title":"unreviewed","effect":{"kind":"noAction"}})),("ifYouDo",json!({"title":"unreviewed","effect":{"kind":"noAction"}}))] {
                let mut bad=source.clone();bad["abilities"]["card"][0][key]=value;assert!(!admitted(bad),"registration {id}/{key}");
            }
            let mut bad=source.clone();bad["card"]["unique"]=json!(true);assert!(!admitted(bad));
            let mut bad=source.clone();bad["card"]["keywords"]=json!(["raid"]);assert!(!admitted(bad));
            let mut bad=source.clone();bad["abilities"]["card"][0]["targetResolvers"]=json!([{ "name":"other","chooser":"self","targetResolver":{"owner":"any","type":"unit","zone":"anyAttackable"},"effect":{"kind":"damage","target":"chosen","amount":{"kind":"const","value":1}} }]);assert!(!admitted(bad));
            for side in ["unit","leader"] {let mut bad=source.clone();bad["abilities"][side]=bad["abilities"]["card"].clone();assert!(!admitted(bad));}
            let mut bad=source.clone();bad["abilities"]["card"].as_array_mut().unwrap().push(source["abilities"]["card"][0].clone());assert!(!admitted(bad));
        }
    }
    #[test]
    fn event_helpers_keep_amount_and_comparison_sensitivity_mutants_executable() {
        assert!(admitted(entry("TWI_099")),"preserve existing recipient-arena event");
        for owner in ["any","self","opponent"] {let mut ferrix=entry("SEC_130");ferrix["abilities"]["card"][0]["targetResolver"]["owner"]=json!(owner);assert!(admitted(ferrix));}
        let mut ferrix=entry("SEC_130");ferrix["abilities"]["card"][0]["effect"]["amount"]["right"]["value"]=json!(1);assert!(admitted(ferrix));
        let mut cripple=entry("SHD_156");cripple["abilities"]["card"][0]["effect"]["effects"][0]["amount"]["value"]=json!(2);assert!(admitted(cripple));
        for target in ["ownPlayer","enemyPlayer"] {let mut cripple=entry("SHD_156");cripple["abilities"]["card"][0]["effect"]["effects"][0]["target"]=json!(target);cripple["abilities"]["card"][0]["effect"]["effects"][1]["onTrue"]["target"]=json!(target);assert!(admitted(cripple));}
        for condition in [true,false] {
            let mut cripple=entry("SHD_156");cripple["abilities"]["card"][0]["effect"]["effects"][1]["condition"]=json!({"kind":"const","value":condition});assert!(admitted(cripple.clone()));cripple["baseClass"]=json!("UnknownCard");assert!(!admitted(cripple));
        }
        let mut cripple=entry("SHD_156");cripple["abilities"]["card"][0]["effect"]["effects"][1]["condition"]["op"]=json!(">=");assert!(admitted(cripple));
    }
}

#[cfg(test)]
mod bounded_unit_helper_metadata_tests {
    use super::*;use serde_json::json;
    fn entry(id:&str)->Value {let catalog:Value=serde_json::from_str(include_str!("../../../compiler/generated/catalog.json")).unwrap();catalog["cards"].as_array().unwrap().iter().find(|e|e["setCode"]==id).unwrap().clone()}
    fn admitted(e:Value)->bool {let id=e["setCode"].as_str().unwrap().to_owned();Catalogue::load(&json!({"schema":SCHEMA,"cards":[e]})).unwrap().cards.contains_key(&id)}
    #[test]
    fn complete_unit_helpers_reject_metadata_and_registration_scope_changes() {
        for id in ["SOR_097","JTL_135","JTL_071"] {
            let source=entry(id);assert!(admitted(source.clone()),"positive {id}");
            for (class,types) in [(json!(null),json!(["unit"])),(json!("UnknownCard"),json!(["unit"])),(json!("EventCard"),json!(["event"])),(json!("LeaderUnitCard"),json!(["leader"])),(json!("TokenUnitCard"),json!(["token","unit"])),(json!("UpgradeCard"),json!(["upgrade"]))] {let mut bad=source.clone();bad["baseClass"]=class;bad["card"]["types"]=types;assert!(!admitted(bad),"class/types {id}");}
            for key in ["pilotText","pilotBox","pilotingBox","deployBox","deployCost","upgradePower","upgradeHp","epicAction"] {let mut bad=source.clone();bad["card"][key]=json!("unsupported");assert!(!admitted(bad),"metadata {id}/{key}");}
            for key in ["attachCondition","piloting"] {let mut bad=source.clone();bad[key]=json!({});assert!(!admitted(bad));}
            let mut bad=source.clone();bad["card"]["unique"]=Value::Null;assert!(!admitted(bad));
            let mut bad=source.clone();bad["keywordValues"]=json!({"raid":1});assert!(!admitted(bad));
            for (key,value) in [("coordinate",json!(true)),("requiresConfirmation",json!(true)),("condition",json!({"kind":"const","value":true})),("costs",json!([{"kind":"exhaustSelf"}])),("contextTitle",json!([{"kind":"const","value":"other"}])),("then",json!({"title":"other","effect":{"kind":"noAction"}})),("ifYouDo",json!({"title":"other","effect":{"kind":"noAction"}}))] {let mut bad=source.clone();bad["abilities"]["card"][0][key]=value;assert!(!admitted(bad),"registration {id}/{key}");}
            let mut bad=source.clone();bad["card"]["keywords"]=json!(["raid"]);assert!(!admitted(bad));
            let mut bad=source.clone();bad["abilities"]["card"][0]["targetResolvers"]=json!([{ "name":"other","chooser":"self","targetResolver":{"owner":"any","type":"unit","zone":"anyAttackable"},"effect":{"kind":"damage","target":"chosen","amount":{"kind":"const","value":1}} }]);assert!(!admitted(bad));
            for side in ["unit","leader"] {let mut bad=source.clone();bad["abilities"][side]=bad["abilities"]["card"].clone();assert!(!admitted(bad));}
            let mut bad=source.clone();bad["abilities"]["card"].as_array_mut().unwrap().push(source["abilities"]["card"][0].clone());assert!(!admitted(bad));
        }
    }
    #[test]
    fn preserves_existing_forms_and_executable_scope_mutants() {
        for id in ["SEC_130","SHD_156","TWI_099","IC27_108","LOF_234","LOF_246","SEC_005","TWI_044"] {assert!(admitted(entry(id)),"preserve {id}");}
        for owner in ["any","self","opponent"] {let mut a=entry("SOR_097");a["abilities"]["card"][0]["targetResolver"]["owner"]=json!(owner);assert!(admitted(a));for kind in ["any","unit","base"] {let mut h=entry("JTL_071");h["abilities"]["card"][0]["effect"]["targetResolver"]=json!({"owner":owner,"type":kind,"zone":"anyAttackable"});assert!(admitted(h));}}
        for condition in [true,false] {let mut tie=entry("JTL_135");tie["abilities"]["card"][0]["effect"]["condition"]=json!({"kind":"const","value":condition});assert!(admitted(tie.clone()));tie["baseClass"]=json!("UnknownCard");assert!(!admitted(tie));}
        let mut h=entry("JTL_071");h["abilities"]["card"][0]["effect"]["amount"]["value"]=json!(4);assert!(!admitted(h));
        let mut h=entry("JTL_071");h["abilities"]["card"][0]["effect"]["maxTargets"]=json!(2);assert!(!admitted(h));
    }
}

#[cfg(test)]
mod filtered_count_singleton_metadata_tests {
    use super::*;use serde_json::json;
    fn entry(id:&str)->Value {let c:Value=serde_json::from_str(include_str!("../../../compiler/generated/catalog.json")).unwrap();c["cards"].as_array().unwrap().iter().find(|e|e["setCode"]==id).unwrap().clone()}
    fn admitted(e:Value)->bool {let id=e["setCode"].as_str().unwrap().to_owned();Catalogue::load(&json!({"schema":SCHEMA,"cards":[e]})).unwrap().cards.contains_key(&id)}
    #[test]
    fn filtered_counts_and_captured_exhaustion_require_complete_source_metadata() {
        for id in ["LOF_121","JTL_199","LOF_054"] {
            let source=entry(id);assert!(admitted(source.clone()),"positive {id}");
            for (class,types) in [(json!(null),json!(["unit"])),(json!("UnknownCard"),source["card"]["types"].clone()),(json!("LeaderUnitCard"),json!(["leader"])),(json!("TokenUnitCard"),json!(["token","unit"])),(json!("UpgradeCard"),json!(["upgrade"]))] {let mut bad=source.clone();bad["baseClass"]=class;bad["card"]["types"]=types;assert!(!admitted(bad),"class/types {id}");}
            let mut bad=source.clone();bad["baseClass"]=json!(if id=="LOF_054"{"NonLeaderUnitCard"}else{"EventCard"});bad["card"]["types"]=json!([if id=="LOF_054"{"unit"}else{"event"}]);assert!(!admitted(bad));
            for key in ["pilotText","pilotBox","pilotingBox","deployBox","deployCost","upgradePower","upgradeHp","epicAction"] {let mut bad=source.clone();bad["card"][key]=json!("unreviewed");assert!(!admitted(bad),"metadata {id}/{key}");}
            for key in ["attachCondition","piloting"] {let mut bad=source.clone();bad[key]=json!({});assert!(!admitted(bad));}
            let mut bad=source.clone();bad["card"]["unique"]=Value::Null;assert!(!admitted(bad));
            let mut bad=source.clone();bad["keywordValues"]=json!({"raid":1});assert!(!admitted(bad));
            let mut bad=source.clone();bad["card"]["keywords"]=json!(["raid"]);assert!(!admitted(bad));
            for (key,value) in [("optional",json!(true)),("coordinate",json!(true)),("requiresConfirmation",json!(true)),("condition",json!({"kind":"const","value":true})),("costs",json!([{"kind":"exhaustSelf"}])),("contextTitle",json!([{"kind":"const","value":"other"}])),("then",json!({"title":"other","effect":{"kind":"noAction"}}))] {let mut bad=source.clone();bad["abilities"]["card"][0][key]=value;assert!(!admitted(bad),"registration {id}/{key}");}
            for side in ["unit","leader"] {let mut bad=source.clone();bad["abilities"][side]=bad["abilities"]["card"].clone();assert!(!admitted(bad));}
            let mut bad=source.clone();bad["abilities"]["card"].as_array_mut().unwrap().push(source["abilities"]["card"][0].clone());assert!(!admitted(bad));
            let mut bad=source.clone();bad["abilities"]["card"][0]["targetResolvers"]=json!([{ "name":"other","chooser":"self","targetResolver":{"owner":"any","type":"unit","zone":"anyAttackable"},"effect":{"kind":"damage","target":"chosen","amount":{"kind":"const","value":1}} }]);assert!(!admitted(bad));
        }
    }
    #[test]
    fn scalar_scope_mutants_stay_executable_and_extra_continuation_leaves_reject() {
        for target in ["ownPlayer","enemyPlayer"] {let mut p=entry("LOF_121");p["abilities"]["card"][0]["effect"]["target"]=json!(target);assert!(admitted(p));}
        let mut p=entry("LOF_121");p["abilities"]["card"][0]["effect"]["amount"]["args"][0]["right"]["right"]["value"]=json!(8);assert!(admitted(p));
        for condition in [true,false] {let mut b=entry("JTL_199");b["abilities"]["card"][0]["effect"]["condition"]=json!({"kind":"const","value":condition});assert!(admitted(b.clone()));b["baseClass"]=json!("UnknownCard");assert!(!admitted(b));}
        for owner in ["any","self","opponent"] {let mut b=entry("JTL_199");b["abilities"]["card"][0]["effect"]["onTrue"]["targetResolver"]["owner"]=json!(owner);assert!(admitted(b));let mut c=entry("LOF_054");c["abilities"]["card"][0]["targetResolver"]["owner"]=json!(owner);assert!(admitted(c));}
        let mut c=entry("LOF_054");c["abilities"]["card"][0]["ifYouDo"]["effect"]["effects"][0]["amount"]["value"]=json!(1);assert!(admitted(c));
        let mut c=entry("LOF_054");c["abilities"]["card"][0]["ifYouDo"]["effect"]["effects"][0]["amount"]["value"]=json!(3);assert!(!admitted(c),"existing group token bound remains one or two per leaf");
        let mut c=entry("LOF_054");c["abilities"]["card"][0]["ifYouDo"]["effect"]["effects"][0]["kind"]=json!("giveShield");assert!(admitted(c));
        let mut c=entry("LOF_054");let leaf=c["abilities"]["card"][0]["ifYouDo"]["effect"]["effects"][0].clone();c["abilities"]["card"][0]["ifYouDo"]["effect"]["effects"].as_array_mut().unwrap().push(leaf);assert!(!admitted(c));
        for key in ["then","ifYouDo"] {let mut c=entry("LOF_054");c["abilities"]["card"][0]["ifYouDo"][key]=json!({"title":"unreviewed","effect":{"kind":"noAction"}});assert!(!admitted(c));}
        for id in ["LAW_165","TWI_221","LOF_246","SOR_097","JTL_071","JTL_135"] {assert!(admitted(entry(id)),"prior {id}");}
    }
}

#[cfg(test)]
mod ordered_ground_metadata_tests {
 use super::*;use serde_json::json;
 fn entry(code:&str)->Value {let c:Value=serde_json::from_str(include_str!("../../../compiler/generated/catalog.json")).unwrap();c["cards"].as_array().unwrap().iter().find(|c|c["setCode"]==code).unwrap().clone()}
 fn admitted(e:Value)->bool {let id=e["setCode"].as_str().unwrap().to_string();Catalogue::load(&json!({"schema":SCHEMA,"cards":[e]})).unwrap().cards.contains_key(&id)}
 #[test]fn ordered_ground_metadata_is_complete_and_operation_specific(){
  for id in ["IC27_182","IC27_027","TWI_173"] {
   let e=entry(id);assert!(admitted(e.clone()),"original {id}");
   for order in ["playerFirst","opponentFirst","seatOrder"] {let mut x=e.clone();x["abilities"]["card"][0]["effect"]["target"]["controllerOrder"]=json!(order);assert!(admitted(x),"order sensitivity {id} {order}");}
   for (path,value) in [("/baseClass",json!("MysteryCard")),("/card/types",json!(["leader"])),("/card/unique",json!(true)),("/card/keywords",json!(["shielded"])),("/keywordValues",json!({"restore":4})),("/card/deployBox",json!("deploy")),("/card/upgradeHp",json!(1)),("/piloting",json!({})),("/attachCondition",json!({"kind":"const","value":true})),("/abilities/card/0/optional",json!(true)),("/abilities/card/0/requiresConfirmation",json!(true)),("/abilities/card/0/coordinate",json!(true)),("/abilities/card/0/condition",json!({"kind":"const","value":false})),("/abilities/card/0/attachedContext",json!("host")),("/abilities/card/0/targetResolver",json!({"owner":"any","type":"unit","zone":"anyAttackable"})),("/abilities/card/0/then",json!({"title":"extra","trigger":"then","effect":{"kind":"noAction"}})),("/abilities/card/0/effect/target/receiver",json!({"kind":"ref","path":["player"]})),("/abilities/card/0/effect/target/predicate",json!({"kind":"const","value":true}))] {
    let mut x=e.clone();let tokens=path.split('/').skip(1).collect::<Vec<_>>();let mut dst=&mut x;for key in &tokens[..tokens.len()-1] {dst=if dst.is_array(){&mut dst[key.parse::<usize>().unwrap()]}else{&mut dst[*key]};}dst[tokens[tokens.len()-1]]=value;assert!(!admitted(x),"must reject {id} {path}");
   }
   let mut x=e.clone();x["abilities"]["card"].as_array_mut().unwrap().push(json!({"trigger":"whenPlayed","title":"extra","effect":{"kind":"noAction"}}));assert!(!admitted(x));
   let mut x=e.clone();x["abilities"]["card"][0]["effect"]["target"]["controllerOrder"]=json!("arbitrary");assert!(!admitted(x));
   let mut x=e.clone();x["abilities"]["card"][0]["effect"]["kind"]=json!("defeat");x["abilities"]["card"][0]["effect"].as_object_mut().unwrap().remove("amount");assert!(!admitted(x));
  }
 }
 #[test]fn omitted_order_serializes_identically_and_old_families_reject_new_order(){
  let raw=json!({"kind":"arenaList","receiver":{"kind":"ref","path":["game"]},"predicate":{"kind":"const","value":true}});let list:ArenaListTarget=serde_json::from_value(raw.clone()).unwrap();assert_eq!(serde_json::to_value(list).unwrap(),raw);
  for kind in ["modifyStats","defeat","giveShield","giveExperience","gainKeyword"] {let mut x=entry("TWI_173");let e=&mut x["abilities"]["card"][0]["effect"];e["kind"]=json!(kind);if kind=="defeat"{e.as_object_mut().unwrap().remove("amount");}assert!(!admitted(x),"new order cannot leak to {kind}");}
  let mut event=entry("TWI_173");event["abilities"]["card"][0]["effect"]["target"].as_object_mut().unwrap().remove("controllerOrder");assert!(!admitted(event),"old unordered event damage remains rejected");
 }
}

#[cfg(test)]
mod top_deck_resource_metadata_tests {
 use super::*;use serde_json::json;
 fn entry(code:&str)->Value {let c:Value=serde_json::from_str(include_str!("../../../compiler/generated/catalog.json")).unwrap();c["cards"].as_array().unwrap().iter().find(|x|x["setCode"]==code).unwrap().clone()}
 fn admitted(e:Value)->bool {let id=e["setCode"].as_str().unwrap().to_string();Catalogue::load(&json!({"schema":SCHEMA,"cards":[e]})).unwrap().cards.contains_key(&id)}
 #[test]fn top_deck_resource_requires_complete_optional_ordinary_unit_registration(){
  for id in ["JTL_119","JTL_164","HMW_136"] {let e=entry(id);assert!(admitted(e.clone()),"original {id}");
   for(path,value)in[("/baseClass",json!("MysteryCard")),("/card/types",json!(["leader"])),("/card/unique",json!("false")),("/keywordValues",json!({"restore":1})),("/card/keywords",json!(["shielded"])),("/card/epicAction",json!("extra")),("/card/deployCost",json!(6)),("/piloting",json!({})),("/attachCondition",json!({"kind":"const","value":true})),("/abilities/card/0/optional",json!(false)),("/abilities/card/0/requiresConfirmation",json!(true)),("/abilities/card/0/coordinate",json!(true)),("/abilities/card/0/attachedContext",json!("host")),("/abilities/card/0/condition",json!({"kind":"const","value":true})),("/abilities/card/0/targetResolver",json!({"owner":"any","type":"unit","zone":"anyAttackable"})),("/abilities/card/0/then",json!({"title":"extra","trigger":"then","effect":{"kind":"noAction"}}))] {let mut x=e.clone();let parts=path.split('/').skip(1).collect::<Vec<_>>();let mut dst=&mut x;for key in &parts[..parts.len()-1]{dst=if dst.is_array(){&mut dst[key.parse::<usize>().unwrap()]}else{&mut dst[*key]};}dst[parts[parts.len()-1]]=value;assert!(!admitted(x),"must reject {id} {path}");}
   for kind in ["draw","damage","heal","ready","exhaust","defeat","returnToHand"]{let mut x=e.clone();x["abilities"]["card"][0]["effect"]=json!({"kind":kind,"target":"ownTopDeck"});if ["draw","damage","heal"].contains(&kind){x["abilities"]["card"][0]["effect"]["amount"]=json!({"kind":"const","value":1});}assert!(!admitted(x),"unreviewed deck operation {kind}");}
   let mut x=e.clone();x["abilities"]["card"].as_array_mut().unwrap().push(json!({"trigger":"whenPlayed","title":"extra","effect":{"kind":"noAction"}}));assert!(!admitted(x));
   for value in [true,false]{let mut x=e.clone();x["abilities"]["card"][0]["effect"]=json!({"kind":"conditional","condition":{"kind":"const","value":value},"onTrue":{"kind":"resourceCard","target":"ownTopDeck"},"onFalse":{"kind":"noAction"}});assert!(admitted(x),"bounded boolean sensitivity");}
  }
 }
 #[test]fn top_deck_name_is_resource_only_and_existing_self_scope_is_not_broadened(){
  assert!(Target::Name("ownTopDeck".into()).validate().is_err());
  let mut x=entry("JTL_119");x["abilities"]["card"][0]["effect"]["target"]=json!("self");assert!(!admitted(x));
  for n in [0,2,3,4,64]{let mut x=entry("HMW_136");x["abilities"]["card"][0]["effect"]["condition"]["right"]["value"]=json!(n);assert!(admitted(x));}let mut x=entry("HMW_136");x["abilities"]["card"][0]["effect"]["condition"]["right"]["value"]=json!(65);assert!(!admitted(x));
  for op in ["<","<=",">",">=","===","!=="]{let mut x=entry("JTL_164");x["abilities"]["card"][0]["effect"]["condition"]["op"]=json!(op);assert!(admitted(x));}
 }
}

#[cfg(test)]
mod resource_continuation_metadata_tests {
 use super::*;use serde_json::json;
 fn entry(code:&str)->Value {let c:Value=serde_json::from_str(include_str!("../../../compiler/generated/catalog.json")).unwrap();c["cards"].as_array().unwrap().iter().find(|x|x["setCode"]==code).unwrap().clone()}
 fn admitted(e:Value)->bool {let id=e["setCode"].as_str().unwrap().to_string();Catalogue::load(&json!({"schema":SCHEMA,"cards":[e]})).unwrap().cards.contains_key(&id)}
 #[test]fn continuations_require_complete_original_source_and_exact_topology(){
  for id in ["ASH_103","LAW_083"]{let e=entry(id);assert!(admitted(e.clone()));for(path,value)in[("/baseClass",json!("MysteryCard")),("/card/types",json!(["leader"])),("/keywordValues",json!({"restore":1})),("/card/keywords",json!(["shielded"])),("/card/epicAction",json!("extra")),("/piloting",json!({})),("/attachCondition",json!({"kind":"const","value":true})),("/abilities/card/0/optional",json!(true)),("/abilities/card/0/requiresConfirmation",json!(true)),("/abilities/card/0/coordinate",json!(true)),("/abilities/card/0/condition",json!({"kind":"const","value":true})),("/abilities/card/0/attachedContext",json!("host")),("/abilities/card/0/then",json!({"title":"extra","trigger":"then","effect":{"kind":"noAction"}}))]{let mut x=e.clone();let parts=path.split('/').skip(1).collect::<Vec<_>>();let mut dst=&mut x;for key in &parts[..parts.len()-1]{dst=if dst.is_array(){&mut dst[key.parse::<usize>().unwrap()]}else{&mut dst[*key]};}dst[parts[parts.len()-1]]=value;assert!(!admitted(x),"{id} {path}");}let mut x=e.clone();x["abilities"]["card"].as_array_mut().unwrap().push(json!({"trigger":"whenPlayed","title":"extra","effect":{"kind":"noAction"}}));assert!(!admitted(x));}
  let e=entry("ASH_103");for (key,val)in [("owner",json!("opponent")),("type",json!("upgrade")),("zone",json!("hand")),("optional",json!(true)),("promptTitle",json!("")),("predicate",json!({"kind":"const","value":true}))]{let mut x=e.clone();x["abilities"]["card"][0]["targetResolver"][key]=val;assert!(!admitted(x),"ASH resolver {key}");}let mut x=e.clone();x["abilities"]["card"][0]["ifYouDo"]["optional"]=json!(true);assert!(!admitted(x));
  let e=entry("LAW_083");let mut x=e.clone();x["abilities"]["card"][0]["effect"]["effects"].as_array_mut().unwrap().swap(0,1);assert!(!admitted(x));let mut x=e.clone();x["abilities"]["card"][0]["effect"]["effects"].as_array_mut().unwrap().push(json!({"kind":"noAction"}));assert!(!admitted(x));let mut x=e.clone();x["abilities"]["card"][0]["effect"]["effects"][0]["onTrue"]["amount"]["value"]=json!(3);assert!(!admitted(x));
 }
 #[test]fn emitted_program_preserves_source_continuation_order_and_bounded_sensitivity(){
  let a:Ability=serde_json::from_value(entry("ASH_103")["abilities"]["card"][0].clone()).unwrap();let p=Program::compile(&a).unwrap();assert!(matches!(&p.instructions[0],Instruction::Select(r) if r.prompt_title.as_deref()==Some("Defeat a friendly Imperial unit")));assert!(matches!(&p.instructions[1],Instruction::Apply(Effect::Defeat{..})));assert!(matches!(&p.instructions[2],Instruction::IfResult{..}));assert!(matches!(&p.instructions[3],Instruction::Apply(Effect::ResourceCard{target:Target::Name(n)}) if n=="ownTopDeck"));assert!(!p.instructions[1..4].iter().any(|i|matches!(i,Instruction::EndStep)));
  let a:Ability=serde_json::from_value(entry("LAW_083")["abilities"]["card"][0].clone()).unwrap();let p=Program::compile(&a).unwrap();let draw=p.instructions.iter().position(|i|matches!(i,Instruction::Apply(Effect::Draw{..}))).unwrap();let resource=p.instructions.iter().position(|i|matches!(i,Instruction::Apply(Effect::ResourceCard{..}))).unwrap();assert!(draw<resource);
  for value in [true,false]{let mut x=entry("LAW_083");x["abilities"]["card"][0]["effect"]["effects"][0]["condition"]=json!({"kind":"const","value":value});assert!(admitted(x));}let mut x=entry("LAW_083");x["abilities"]["card"][0]["effect"]["effects"][0]["onTrue"]["amount"]["value"]=json!(2);assert!(admitted(x));
 }
}

#[cfg(test)]
mod simultaneous_resource_pair_metadata_tests {
 use super::*;use serde_json::json;
 fn entry()->Value {let c:Value=serde_json::from_str(include_str!("../../../compiler/generated/catalog.json")).unwrap();let mut e=c["cards"].as_array().unwrap().iter().find(|e|e["setCode"]=="LAW_171").unwrap().clone();e["status"]=json!("supported");e["diagnostics"]=json!([]);e["abilities"]=json!({"leader":[],"unit":[],"card":[{"trigger":"event","title":"Resource this event and the top card of your deck","effect":{"kind":"simultaneous","effects":[{"kind":"resourceCard","target":"self"},{"kind":"resourceCard","target":"ownTopDeck"}]}}]});e}
 fn admitted(e:Value)->bool {Catalogue::load(&json!({"schema":SCHEMA,"cards":[e]})).unwrap().cards.contains_key("LAW_171")}
 #[test]fn resource_pair_requires_exact_complete_ordinary_event_source(){let e=entry();assert!(admitted(e.clone()));for(path,value)in[("/baseClass",json!("NonLeaderUnitCard")),("/card/types",json!(["unit"])),("/card/types",json!(["event","leader"])),("/card/unique",json!(true)),("/card/unique",json!("false")),("/keywordValues",json!({"restore":1})),("/card/keywords",json!(["shielded"])),("/card/epicAction",json!("extra")),("/card/deployCost",json!(6)),("/piloting",json!({})),("/attachCondition",json!({"kind":"const","value":true})),("/abilities/card/0/trigger",json!("whenPlayed")),("/abilities/card/0/optional",json!(true)),("/abilities/card/0/requiresConfirmation",json!(true)),("/abilities/card/0/coordinate",json!(true)),("/abilities/card/0/attachedContext",json!("host")),("/abilities/card/0/condition",json!({"kind":"const","value":true})),("/abilities/card/0/targetResolver",json!({"owner":"any","type":"unit","zone":"anyAttackable"})),("/abilities/card/0/then",json!({"trigger":"then","title":"extra","effect":{"kind":"noAction"}})),("/abilities/card/0/ifYouDo",json!({"title":"extra","effect":{"kind":"noAction"}}))]{let mut x=e.clone();let parts=path.split('/').skip(1).collect::<Vec<_>>();let mut at=&mut x;for k in &parts[..parts.len()-1]{at=if at.is_array(){&mut at[k.parse::<usize>().unwrap()]}else{&mut at[*k]};}at[parts[parts.len()-1]]=value;assert!(!admitted(x),"{path}");}for side in ["card","unit","leader"]{let mut x=e.clone();x["abilities"][side].as_array_mut().unwrap().push(json!({"trigger":"whenPlayed","title":"extra","effect":{"kind":"noAction"}}));assert!(!admitted(x),"extra {side}");}}
 #[test]fn resource_pair_program_is_one_group_and_all_other_resource_groups_fail_closed(){let original=entry();let a:Ability=serde_json::from_value(original["abilities"]["card"][0].clone()).unwrap();let p=Program::compile(&a).unwrap();assert!(matches!(&p.instructions[0],Instruction::Group(e) if resource_pair_effect(e)));assert_eq!(p.instructions.iter().filter(|i|matches!(i,Instruction::Group(_))).count(),1);assert!(p.instructions.iter().all(|i|!matches!(i,Instruction::Apply(Effect::ResourceCard{..}))));let pair=original["abilities"]["card"][0]["effect"].clone();let mut variants=vec![json!({"kind":"simultaneous","effects":[]}),json!({"kind":"simultaneous","effects":[pair["effects"][0].clone()]}),json!({"kind":"simultaneous","effects":[pair["effects"][1].clone(),pair["effects"][0].clone()]}),json!({"kind":"simultaneous","effects":[pair["effects"][0].clone(),pair["effects"][0].clone()]}),json!({"kind":"sequence","effects":pair["effects"].clone()}),json!({"kind":"simultaneous","effects":[pair.clone()]}),json!({"kind":"conditional","condition":{"kind":"const","value":true},"onTrue":pair.clone(),"onFalse":{"kind":"noAction"}})];for target in ["enemyTopDeck","ownPlayer","chosen","ownBase"]{variants.push(json!({"kind":"simultaneous","effects":[{"kind":"resourceCard","target":"self"},{"kind":"resourceCard","target":target}]}));}let mut three=pair.clone();three["effects"].as_array_mut().unwrap().push(json!({"kind":"noAction"}));variants.push(three);for effect in variants{let mut x=original.clone();x["abilities"]["card"][0]["effect"]=effect.clone();assert!(!admitted(x),"unreviewed {effect}");}}
}

#[cfg(test)]
pub(crate) mod parent_defense_upgrade_metadata_tests {
 use super::*;use serde_json::json;
 pub fn entry()->Value {let c:Value=serde_json::from_str(include_str!("../../../compiler/generated/catalog.json")).unwrap();let mut e=c["cards"].as_array().unwrap().iter().find(|e|e["setCode"]=="SOR_071").unwrap().clone();e["status"]=json!("supported");e["diagnostics"]=json!([]);let q=|name:&str,path:Value|json!({"kind":"query","name":name,"receiver":{"kind":"ref","path":path},"args":[]});e["abilities"]=json!({"leader":[],"unit":[],"card":[{"trigger":"constant","title":"While attached unit is defending, the attacker gets -1/-0","costs":[],"condition":q("isDefending",json!(["source","parentUnit"])),"targetResolver":{"owner":"opponent","type":"unit","zone":"arena","predicate":{"kind":"binary","op":"&&","left":{"kind":"binary","op":"&&","left":q("isUnit",json!(["card"])),"right":q("isInPlay",json!(["card"]))},"right":q("isAttacking",json!(["card"]))}},"effect":{"kind":"modifyStats","target":"chosen","duration":"persistent","power":{"kind":"const","value":-1},"hp":{"kind":"const","value":0}}}]});e}
 fn admitted(e:Value)->bool {Catalogue::load(&json!({"schema":SCHEMA,"cards":[e]})).unwrap().cards.contains_key("SOR_071")}
 #[test]fn parent_defense_requires_exact_registration_and_complete_upgrade_metadata(){let original=entry();assert!(admitted(original.clone()));assert!(parent_defense_upgrade_entry(&original));for key in ["power","hp"] {let mut valid=original.clone();let derived=if key=="power" {"upgradePower"} else {"upgradeHp"};valid["card"][key]=json!(3);valid["card"][derived]=json!(3);assert!(admitted(valid),"coordinated {key}");for (field,value) in [(key,json!(99)),(key,Value::Null),(derived,json!(3))] {let mut bad=original.clone();bad["card"][field]=value;assert!(!admitted(bad),"inconsistent {field}");}}for(path,value)in[("/baseClass",json!("MysteryCard")),("/baseClass",json!("NonLeaderUnitCard")),("/card/types",json!(["unit"])),("/card/types",json!(["upgrade","leader"])),("/card/unique",json!(true)),("/keywordValues",json!({"sentinel":1})),("/card/keywords",json!(["sentinel"])),("/card/pilotText",json!("pilot")),("/piloting",json!({})),("/card/deployBox",json!("extra")),("/card/epicAction",json!("extra")),("/card/upgradeHp",json!(1001)),("/attachCondition",json!({"kind":"const","value":true})),("/abilities/card/0/trigger",json!("event")),("/abilities/card/0/optional",json!(true)),("/abilities/card/0/requiresConfirmation",json!(true)),("/abilities/card/0/coordinate",json!(true)),("/abilities/card/0/attachedContext",json!("host")),("/abilities/card/0/targetResolver/owner",json!("self")),("/abilities/card/0/targetResolver/predicate",json!({"kind":"const","value":true})),("/abilities/card/0/effect/hp/value",json!(1)),("/abilities/card/0/effect/power/value",json!(-2)),("/abilities/card/0/condition/name",json!("isAttacking")),("/abilities/card/0/condition/receiver/path",json!(["source","parentCard"]))]{let mut e=original.clone();let keys=path.split('/').skip(1).collect::<Vec<_>>();let mut at=&mut e;for k in &keys[..keys.len()-1]{at=if at.is_array(){&mut at[k.parse::<usize>().unwrap()]}else{&mut at[*k]};}at[keys[keys.len()-1]]=value;assert!(!admitted(e),"{path}");}for side in ["card","unit","leader"]{let mut e=original.clone();e["abilities"][side].as_array_mut().unwrap().push(json!({"trigger":"whenPlayed","title":"extra","effect":{"kind":"noAction"}}));assert!(!admitted(e),"extra {side}");}}
 #[test]fn parent_defense_marker_cannot_hide_in_other_expression_or_child_slots(){let original=entry();let query=original["abilities"]["card"][0]["condition"].clone();for key in["then","ifYouDo"]{let mut e=original.clone();e["abilities"]["card"][0][key]=json!({"title":"nested","condition":query,"effect":{"kind":"noAction"}});assert!(!admitted(e),"{key}");}let mut e=original.clone();e["abilities"]["card"][0]["contextTitle"]=json!([query]);assert!(!admitted(e));let mut e=original.clone();e["abilities"]["card"][0]["costs"]=json!([{"kind":"resources","amount":query}]);assert!(!admitted(e));let mut e=original.clone();e["abilities"]["card"][0]["effect"]=json!({"kind":"conditional","condition":query,"onTrue":original["abilities"]["card"][0]["effect"].clone(),"onFalse":{"kind":"noAction"}});assert!(!admitted(e));let a:Ability=serde_json::from_value(original["abilities"]["card"][0].clone()).unwrap();assert!(Program::compile(&a).is_ok());let mut a=a;a.effect=Effect::NoAction;assert!(Program::compile(&a).is_err());let bad:Expr=serde_json::from_value(json!({"kind":"query","name":"isAttacking","receiver":{"kind":"ref","path":["source","parentUnit"]},"args":[]})).unwrap();assert!(bad.validate().is_err());}
 #[test]fn parent_defense_direct_program_guard_rejects_nested_and_misplaced_queries(){
  let value=entry()["abilities"]["card"][0].clone();let valid:Ability=serde_json::from_value(value.clone()).unwrap();assert!(Program::compile(&valid).is_ok());let q=value["condition"].clone();let mut variants=vec![];let mut v=value.clone();v["trigger"]=json!("event");variants.push(v);let mut v=value.clone();v["targetResolver"]["predicate"]=q.clone();variants.push(v);let mut v=value.clone();v["effect"]["power"]=q.clone();variants.push(v);let mut v=value.clone();v["contextTitle"]=json!([q]);variants.push(v);for field in["then","ifYouDo"]{let mut outer=json!({"trigger":"event","title":"outer","effect":{"kind":"noAction"}});outer[field]=value.clone();variants.push(outer);}for v in variants{let a:Ability=serde_json::from_value(v.clone()).unwrap();assert!(Program::compile(&a).is_err(),"direct {v}");}
 }

}

#[cfg(test)]
pub(crate) mod gained_host_tests {
 use super::*;use serde_json::json;
 pub fn catalogue()->Value {let mut c:Value=serde_json::from_str(include_str!("../../../compiler/generated/catalog.json")).unwrap();for (i,code) in ["LOF_051","LAW_186","LOF_139"].iter().enumerate(){let entry=c["cards"].as_array_mut().unwrap().iter_mut().find(|e|e["setCode"]==*code).unwrap();entry["status"]=json!("supported");entry["diagnostics"]=json!([]);entry["abilities"]=json!({"leader":[],"unit":[],"card":[gained_host_templates().remove(i)]});let trait_q=|name:&str|json!({"kind":"query","name":"hasSomeTrait","receiver":{"kind":"ref","path":["card"]},"args":[{"kind":"const","value":name}]});entry["attachCondition"]=match i {0=>trait_q("force"),1=>json!({"kind":"unary","op":"!","value":trait_q("vehicle")}),_=>Value::Null};}c}
 #[test]fn gained_host_exact_templates_and_metadata_reject_misplaced_or_changed_markers(){let c=catalogue();let compiled=Catalogue::load(&c).unwrap();for (i,code)in["LOF_051","LAW_186","LOF_139"].iter().enumerate(){let original=c["cards"].as_array().unwrap().iter().find(|e|e["setCode"]==*code).unwrap();assert_eq!(gained_host_upgrade_kind(original),Some(i));assert!(compiled.cards.contains_key(*code));for(path,value)in[("/baseClass",json!("EventCard")),("/card/unique",json!(i!=1)),("/card/types",json!(["unit"])),("/card/power",json!(99)),("/card/hp",Value::Null),("/card/gainedHost",json!(true)),("/piloting",json!({})),("/keywordValues",json!({"raid":1})),("/abilities/card/0/gainedHost",json!(false)),("/abilities/card/0/gainedHost",Value::Null),("/abilities/card/0/attachedContext",json!("host")),("/abilities/card/0/trigger",json!("whenPlayed")),("/abilities/card/0/condition",json!({"kind":"const","value":true}))]{let mut bad=original.clone();let parts=path.split('/').skip(1).collect::<Vec<_>>();let mut at=&mut bad;for k in &parts[..parts.len()-1]{at=if at.is_array(){&mut at[k.parse::<usize>().unwrap()]}else{&mut at[*k]};}at[parts[parts.len()-1]]=value;assert!(gained_host_upgrade_kind(&bad).is_none(),"{code}/{path}");let loaded=Catalogue::load(&json!({"schema":SCHEMA,"cards":[bad]})).unwrap();assert!(!loaded.cards.contains_key(*code),"admitted {code}/{path}");}}}
 #[test]fn gained_host_direct_program_rejects_nested_or_other_body_and_omits_absent_marker(){for template in gained_host_templates(){let p=Program::compile(&template).unwrap();assert_eq!(p.gained_host,Some(true));let mut invalid=template.clone();invalid.effect=Effect::NoAction;assert!(Program::compile(&invalid).is_err());let outer:Ability=serde_json::from_value(json!({"trigger":"event","title":"outer","effect":{"kind":"noAction"},"then":template})).unwrap();assert!(Program::compile(&outer).is_err());}let a:Ability=serde_json::from_value(json!({"trigger":"event","title":"old","effect":{"kind":"noAction"}})).unwrap();assert!(serde_json::to_value(&a).unwrap().get("gainedHost").is_none());assert!(serde_json::to_value(Program::compile(&a).unwrap()).unwrap().get("gainedHost").is_none());}
}
