use super::*;
use serde_json::json;

#[test]
fn json_scan_searches_values_preorder_and_short_circuits_without_visiting_keys() {
    let tree=json!({"a":[1,null],"b":{"c":"end"}});
    let visited=std::cell::RefCell::new(Vec::new());
    assert!(contains(&tree,|v| {
        visited.borrow_mut().push(v.clone());
        v==&json!(1)
    }));
    assert_eq!(*visited.borrow(),vec![tree.clone(),json!([1,null]),json!(1)]);
    assert!(!contains(&json!({"marker":null}),|v|v=="marker"));
    assert!(contains(&json!({"marker":null}),|v|v.get("marker").is_some()));
    assert!(contains(&json!([{},[],false,0,null]),Value::is_null));
}

#[test]
fn json_scan_markers_keep_nested_arrays_unknown_positions_and_null_key_presence() {
    let cases:[(fn(&Value)->bool,Value);10]=[
        (crate::static_ongoing::marker,json!({"kind":"constantEffectRegistration"})),
        (crate::attached_constants::marker,json!({"kind":"attachedConstantContract"})),
        (crate::native_attack::marker,json!({"kind":"initiateAttack"})),
        (crate::phase_history::marker,json!({"kind":"historyQuery"})),
        (crate::attack_history::contains,json!({"kind":"historyAttackCheck"})),
        (crate::death_hooks::marker,json!({"hooks":null})),
        (crate::death_hooks::marker,json!({"trigger":"eventHooks"})),
        (crate::advantage::marker,json!({"kind":"giveAdvantage"})),
        (crate::bounded_reductions::marker,json!({"kind":"query","name":"arenaPowerMax"})),
        (gained_host_marker,json!({"gainedHost":null})),
    ];
    for (marker,leaf) in cases {
        assert!(marker(&leaf),"{leaf}");
        assert!(marker(&json!({"unknown":[0,[null,{"unrecognized":leaf}]]})),"{leaf}");
        assert!(!marker(&json!({"unknown":[null,false,0,"kind",{},[]]})),"{leaf}");
    }
    let raw=json!({"abilities":{"unused":[{"attachedHost":null}]}});
    assert_eq!(crate::attached_registration::sides(&raw).unwrap_err(),"attachedHost is an internal program marker");
}

#[test]
fn json_scan_keeps_force_reference_suffix_and_resource_serialization_failure_policy() {
    assert!(contains_force_presence(&json!({"irrelevant":[{"kind":"ref","path":["source","hasTheForce"]}]})));
    assert!(contains_force_presence(&json!({"kind":"query","name":"hasTheForce"})));
    for v in [json!({"kind":"ref","path":["hasTheForce","source"]}),json!({"hasTheForce":true}),json!({"kind":"const","value":"hasTheForce"})] {
        assert!(!contains_force_presence(&v));
    }
    struct NotSerializable;
    impl Serialize for NotSerializable {
        fn serialize<S:serde::Serializer>(&self,_:S)->Result<S::Ok,S::Error> {
            Err(serde::ser::Error::custom("test serialization failure"))
        }
    }
    assert!(contains_resource(&NotSerializable),"serialization failure must fail closed");
    assert!(contains_resource(&json!({"then":[{"kind":"resourceCard"}]})));
    assert!(!contains_resource(&json!({"kind":"const","value":"resourceCard"})));
}

#[test]
fn json_scan_preserves_arena_target_boundary_and_nested_keyword_argument_scope() {
    let list=json!({"kind":"arenaList","receiver":{"kind":"ref","path":["player"]},"predicate":{"kind":"const","value":true}});
    assert!(!contains_arena_scalar(&json!({"kind":"damage","target":list})));
    assert!(contains_arena_scalar(&json!({"target":list,"amount":{"kind":"arenaLength","receiver":list}})));
    let feedback=|condition:Value|json!({"status":"supported","abilities":{"card":[{
        "trigger":"constant","condition":condition,"effect":{"kind":"modifyStats"}}]}});
    assert!(!crate::bounded_reductions::power_feedback(&feedback(json!({
        "kind":"query","name":"hasSomeKeyword","args":[],"unrelated":{"kind":"const","value":"overwhelm"}}))));
    assert!(crate::bounded_reductions::power_feedback(&feedback(json!({
        "kind":"query","name":"hasSomeKeyword","args":[{"kind":"const","value":"overwhelm"}]}))));
}
