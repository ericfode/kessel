use super::*;
use serde_json::json;

fn boxed(e:Expr)->Box<Expr> {Box::new(e)}
fn reference(path:&[&str])->Expr {Expr::Ref{path:path.iter().map(|s|(*s).into()).collect()}}
fn native(receiver:Expr)->Expr {Expr::NativeTokenPredicate{operation:"hasWeakness".into(),receiver:boxed(receiver),domain:"physicalAttachments".into(),token_type:"weakness".into(),optional_receiver:false,optional_call:false}}
fn compiled(original:Expr)->Expr {Expr::Compiled(CompiledExpr{original:boxed(original),index:0})}
fn collection(receiver:Expr)->Expr {Expr::Collection{name:"some".into(),receiver:boxed(receiver),args:vec![],limit:256}}
fn flags(e:&Expr)->[bool;4] {[e.references_title(),e.depends_on_stats(),e.depends_on_keywords(),e.live_only()]}

#[test]
fn expr_search_dependency_guards_preserve_native_pruning_and_compiled_descent() {
    let payload=Expr::Array{items:vec![reference(&["card","title"]),reference(&["card","power"]),reference(&["modifierTarget"])]};
    assert_eq!(flags(&payload),[true;4]);
    assert_eq!(flags(&native(payload.clone())),[false,true,true,false]);
    assert_eq!(flags(&compiled(payload.clone())),[true;4]);
    assert_eq!(flags(&collection(payload.clone())),[true;4]);
    for e in [Expr::ArenaLength{receiver:boxed(payload.clone())},Expr::ArenaFilter{receiver:boxed(Expr::number(0)),predicate:boxed(payload)}] {
        assert_eq!(flags(&e),[true;4]);
    }
    let query=|name:&str|Expr::Query{name:name.into(),receiver:boxed(reference(&["player"])),args:vec![]};
    assert_eq!(flags(&query("arenaPowerMax")),[false,true,false,false]);
    assert_eq!(flags(&query("resourcePairs")),[false,false,false,true]);
    assert_eq!(flags(&reference(&["card","keywords","length"])),[false,false,true,false]);
}

#[test]
fn expr_search_upgrade_queries_keep_pruned_inventories_and_portable_compiled_queries() {
    let query=Expr::Query{name:"isUpgrade".into(),receiver:boxed(reference(&["card"])),args:vec![]};
    let resolver=|predicate|TargetResolver {prompt_title:None,prompt_title_parts:vec![],owner:"any".into(),card_type:"any".into(),arena:None,zone:None,optional:false,predicate:Some(predicate),mode:None,count:None,native:None};
    for e in [query.clone(),compiled(query.clone()),Expr::Array{items:vec![query.clone()]}] {
        assert!(resolver(e).mentions_upgrade_query());
    }
    for e in [native(query.clone()),collection(query.clone()),Expr::ArenaLength{receiver:boxed(query.clone())},Expr::ArenaSome{receiver:boxed(Expr::number(0)),predicate:boxed(query)}] {
        assert!(!resolver(e).mentions_upgrade_query());
    }
}

#[test]
fn expr_search_parent_condition_preserves_native_and_compiled_boundaries() {
    let condition=reference(&["source","parentCard"]);
    let ability=|condition| {
        let mut a:Ability=serde_json::from_value(json!({"title":"host witness","trigger":"constant","attachedContext":"host",
            "effect":{"kind":"gainKeyword","keyword":"raid","target":"self","amount":{"kind":"const","value":1},"duration":"persistent"}})).unwrap();
        a.condition=Some(condition);a
    };
    for e in [condition.clone(),collection(condition.clone()),Expr::ArenaLength{receiver:boxed(condition.clone())}] {
        assert_eq!(crate::attached_constants::validate_context(&ability(e)).unwrap_err(),"delegated host condition cannot read an upgrade parent getter");
    }
    for e in [native(condition.clone()),compiled(condition)] {
        assert!(crate::attached_constants::validate_context(&ability(e)).is_ok());
    }
}

#[test]
fn expr_search_can_prune_without_matching_and_checks_later_children() {
    let needle=reference(&["target"]);
    let test=|e:&Expr|match e {Expr::Ref{path}=>Some(path==&["target"]),_=>None};
    let wrappers=[
        Expr::Binary{op:"&&".into(),left:boxed(Expr::number(0)),right:boxed(needle.clone())},
        Expr::Conditional{condition:boxed(Expr::number(0)),then:boxed(Expr::number(0)),r#else:boxed(needle.clone())},
        Expr::Query{name:"test".into(),receiver:boxed(Expr::number(0)),args:vec![needle.clone()]},
        Expr::Math{name:"min".into(),args:vec![Expr::number(0),needle]},
    ];
    for e in wrappers {assert!(e.any(test));assert!(!e.any(|_|Some(false)));}
}
