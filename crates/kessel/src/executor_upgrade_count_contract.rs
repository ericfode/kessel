//! Closed dynamic self-stat source contract. No callback evaluator or durable cache.
//! Capture/normalization happens at admission; evaluation scans live physical attachments.
use serde_json::{json, Value};
pub(crate) const QUERY: &str = "otherControlledArenaUpgradeCount";
fn source_abilities() -> Value {
    serde_json::from_str(
        r###"{
  "leader": [],
  "unit": [],
  "card": [
    {
      "trigger": "constant",
      "title": "This unit gets +1/+0 for each upgrade on other friendly units",
      "costs": [],
      "effect": {
        "kind": "constantEffectRegistration",
        "properties": {},
        "modifier": {
          "kind": "ongoingContract",
          "scope": "card",
          "effectType": "modifyStats",
          "target": "constantMatch",
          "duration": "persistent",
          "valueMode": "dynamic",
          "value": {
            "kind": "ongoingValueFactory",
            "callback": {
              "kind": "pureCallback",
              "parameters": [
                {
                  "slot": "data:callback:819:0",
                  "argument": "targetCard",
                  "bindingPattern": {
                    "name": "target"
                  }
                }
              ],
              "body": {
                "kind": "pureBlock",
                "statements": [
                  {
                    "kind": "pureBindings",
                    "bindings": [
                      {
                        "slot": "data:local:855",
                        "mutable": false,
                        "value": {
                          "kind": "ongoingMember",
                          "receiver": {
                            "kind": "pureCollectionCall",
                            "method": "flatMap",
                            "receiver": {
                              "kind": "ongoingQuery",
                              "name": "getArenaUnits",
                              "receiver": {
                                "kind": "dataPath",
                                "root": [
                                  "data:callback:819:0"
                                ],
                                "steps": [
                                  {
                                    "key": "controller",
                                    "optional": false
                                  }
                                ]
                              },
                              "args": [
                                {
                                  "kind": "record",
                                  "properties": {
                                    "otherThan": {
                                      "kind": "dataPath",
                                      "root": [
                                        "data:callback:819:0"
                                      ],
                                      "steps": []
                                    }
                                  }
                                }
                              ],
                              "optionalReceiver": false,
                              "optionalCall": false,
                              "continuesOptionalChain": false
                            },
                            "args": [
                              {
                                "kind": "pureCallback",
                                "parameters": [
                                  {
                                    "slot": "data:callback:953:0",
                                    "argument": "item",
                                    "bindingPattern": {
                                      "name": "x"
                                    }
                                  }
                                ],
                                "body": {
                                  "kind": "pureReturn",
                                  "value": {
                                    "kind": "dataPath",
                                    "root": [
                                      "data:callback:953:0"
                                    ],
                                    "steps": [
                                      {
                                        "key": "upgrades",
                                        "optional": false
                                      }
                                    ]
                                  }
                                },
                                "capture": "lexicalReferences",
                                "invokeAt": "nativeCallbackInvocation"
                              }
                            ],
                            "optionalReceiver": false,
                            "optionalCall": false
                          },
                          "member": "length",
                          "optionalReceiver": false,
                          "continuesOptionalChain": false
                        },
                        "bindingPattern": {
                          "name": "upgradeCountOnOtherFriendlyUnits"
                        },
                        "evaluateAt": "declaration"
                      }
                    ]
                  },
                  {
                    "kind": "pureReturn",
                    "value": {
                      "kind": "dataRecord",
                      "fields": {
                        "power": {
                          "kind": "dataPath",
                          "root": [
                            "data:local:855"
                          ],
                          "steps": []
                        },
                        "hp": {
                          "kind": "const",
                          "value": 0
                        }
                      }
                    }
                  }
                ]
              },
              "capture": "lexicalReferences",
              "invokeAt": "nativeCallbackInvocation"
            }
          }
        },
        "duration": "persistent",
        "context": "refreshFromSourceController",
        "scopeDefaults": "pinnedOngoingCardOrPlayerEffect",
        "dependencyPolicy": "liveFixedPointRequired"
      }
    },
    {
      "trigger": "whenPlayed",
      "title": "Give an Advantage token to each other friendly unit",
      "costs": [],
      "effect": {
        "kind": "giveAdvantage",
        "target": {
          "kind": "arenaList",
          "receiver": {
            "kind": "ref",
            "path": [
              "player"
            ]
          },
          "predicate": {
            "kind": "binary",
            "op": "&&",
            "left": {
              "kind": "const",
              "value": true
            },
            "right": {
              "kind": "binary",
              "op": "||",
              "left": {
                "kind": "unary",
                "op": "!",
                "value": {
                  "kind": "ref",
                  "path": [
                    "source"
                  ]
                }
              },
              "right": {
                "kind": "binary",
                "op": "!==",
                "left": {
                  "kind": "ref",
                  "path": [
                    "card"
                  ]
                },
                "right": {
                  "kind": "ref",
                  "path": [
                    "source"
                  ]
                }
              }
            }
          }
        },
        "amount": {
          "kind": "const",
          "value": 1
        },
        "createdBy": "self",
        "batch": "singleCreationEvent",
        "tokenOwner": "recipientController"
      }
    }
  ]
}"###,
    )
    .expect("reviewed source IR")
}
fn profile() -> Value {
    serde_json::from_str(r###"{
  "cost": 8,
  "hp": 12,
  "power": 5,
  "text": "This unit gets +1/+0 for each upgrade on other friendly units.\nWhen Played: Give an Advantage token to each other friendly unit.",
  "deployBox": null,
  "epicAction": "",
  "unique": true,
  "rules": null,
  "upgradePower": null,
  "upgradeHp": null,
  "aspects": [
    "cunning",
    "villainy"
  ],
  "traits": [
    "imperial",
    "vehicle",
    "capital ship"
  ],
  "arena": "space",
  "keywords": [],
  "types": [
    "unit"
  ]
}"###).expect("reviewed metadata")
}
pub(crate) fn marker(e: &Value) -> bool {
    e["card"]["text"].as_str()==Some("This unit gets +1/+0 for each upgrade on other friendly units.\nWhen Played: Give an Advantage token to each other friendly unit.") || crate::ability_ir::contains(e,|v|(v["kind"]=="query"&&v["name"]==QUERY)||(v["kind"]=="giveAdvantage"&&v["target"]["kind"]=="arenaList"))
}
// Only lexical spelling and opaque compiler slot numbers may vary. The exact
// declaration/use graph, evaluation stage, callback roles and all flags remain.
fn alpha(v: &Value) -> Option<Value> {
    fn declarations(
        v: &Value,
        slots: &mut std::collections::BTreeMap<String, String>,
    ) -> Option<()> {
        match v {
            Value::Object(o) => {
                if let Some(slot) = o.get("slot") {
                    let key = slot.as_str()?;
                    if slots.contains_key(key) {
                        return None;
                    }
                    let n = slots.len();
                    slots.insert(key.into(), format!("binding{n}"));
                }
                for child in o.values() {
                    declarations(child, slots)?;
                }
            }
            Value::Array(a) => {
                for child in a {
                    declarations(child, slots)?;
                }
            }
            _ => {}
        }
        Some(())
    }
    fn rewrite(v: &Value, slots: &std::collections::BTreeMap<String, String>) -> Option<Value> {
        match v {
            Value::Array(a) => Some(Value::Array(
                a.iter().map(|v| rewrite(v, slots)).collect::<Option<_>>()?,
            )),
            Value::Object(o) => {
                let mut result = serde_json::Map::new();
                for (k, v) in o {
                    let value = if k == "slot" {
                        json!(slots.get(v.as_str()?)?)
                    } else if k == "root" && o.get("kind") == Some(&json!("dataPath")) {
                        let a = v.as_array()?;
                        if a.len() != 1 {
                            return None;
                        }
                        json!([slots.get(a[0].as_str()?)?])
                    } else if k == "bindingPattern" {
                        let p = v.as_object()?;
                        let name = p.get("name")?.as_str()?;
                        if p.len() != 1
                            || name.is_empty()
                            || !name
                                .chars()
                                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
                        {
                            return None;
                        }
                        json!({"name":"binding"})
                    } else {
                        rewrite(v, slots)?
                    };
                    result.insert(k.clone(), value);
                }
                Some(Value::Object(result))
            }
            _ => Some(v.clone()),
        }
    }
    let mut slots = Default::default();
    declarations(v, &mut slots)?;
    rewrite(v, &slots)
}
pub(crate) fn prepare(e: &Value) -> Result<Option<Value>, String> {
    if !marker(e) {
        return Ok(None);
    }
    let valid = crate::admission::SourceContextPolicy::Closed.accepts(e)
        && crate::admission::OrdinarySource::from_entry(e)
            == Some(crate::admission::OrdinarySource::Unit)
        && e["keywordValues"] == json!({})
        && e["attachCondition"].is_null()
        && e["piloting"].is_null()
        && ["pilotText", "pilotBox", "pilotingBox", "deployCost"]
            .iter()
            .all(|k| e["card"][*k].is_null())
        && profile()
            .as_object()
            .unwrap()
            .iter()
            .all(|(k, v)| e["card"][k] == *v)
        && alpha(&e["abilities"]).is_some_and(|a| Some(a) == alpha(&source_abilities()));
    if !valid {
        return Err("other-controlled-unit upgrade count requires complete closed dynamic source and paired batched Advantage registration".into());
    }
    let mut abilities = e["abilities"].clone();
    abilities["card"][0]["effect"] = json!({"kind":"modifyStats","target":"self","duration":"persistent","power":{"kind":"query","name":QUERY,"receiver":{"kind":"ref","path":["modifierTarget"]},"args":[]},"hp":{"kind":"const","value":0}});
    Ok(Some(abilities))
}

pub(crate) fn batch_target(target: &crate::ability_ir::Target) -> bool {
    serde_json::to_value(target)
        .is_ok_and(|t| t == source_abilities()["card"][1]["effect"]["target"])
}

/// Standalone compilation keeps the same closed registration boundary as loading.
pub(crate) fn program_marker(v:&Value)->bool {crate::ability_ir::contains(v,|v|(v["kind"]=="query"&&v["name"]==QUERY)||(v["kind"]=="giveAdvantage"&&v["target"]["kind"]=="arenaList"))}
pub(crate) fn ability(a:&crate::ability_ir::Ability)->bool {
 let mut expected=source_abilities();expected["card"][0]["effect"]=json!({"kind":"modifyStats","target":"self","duration":"persistent","power":{"kind":"query","name":QUERY,"receiver":{"kind":"ref","path":["modifierTarget"]},"args":[]},"hp":{"kind":"const","value":0}});
 expected["card"].as_array().unwrap().iter().any(|v|serde_json::from_value::<crate::ability_ir::Ability>(v.clone()).is_ok_and(|expected|serde_json::to_value(expected).ok()==serde_json::to_value(a).ok()))
}
