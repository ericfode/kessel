//! Complete closed source recipes for mandatory deck inspection and discard recovery.
//! Pure admission: private callbacks are projected only after full provenance equality.
use crate::ability_ir::{Ability, CardPrograms, Effect, Instruction, Program, TargetResolver};
use crate::modal_choices::{Branch, Chooser, Menu};
use serde_json::Value;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Top,
    Recovery,
}
const TOP: &str = r####"{"abilities":{"card":[{"costs":[],"effect":{"kind":"resolveChoice","resultBinding":"select"},"targetResolver":{"choices":{"contextBinding":"resolverContextByReference","duplicatePolicy":"lastAssignmentWins","entries":[{"effect":{"args":[{"capability":"AbilityHelper","kind":"lexicalCapability","name":"AbilityHelper"},{"kind":"ref","path":["player"]}],"defaultTarget":"self","defaultTargetLookup":"nativePropertyGenerationAfterDefaultTargetFnOverrides","invocationPolicy":"sourceMethodCall","invokeAt":"expressionEvaluation","kind":"privateConstructionCall","name":"lookAtTopCardOfDeckAndChooseDiscard","resultType":"gameSystem","source":"physicalAbilitySource"},"labelParts":[{"kind":"const","value":"Your deck"}]},{"effect":{"args":[{"capability":"AbilityHelper","kind":"lexicalCapability","name":"AbilityHelper"},{"kind":"ref","path":["opponent"]}],"defaultTarget":"self","defaultTargetLookup":"nativePropertyGenerationAfterDefaultTargetFnOverrides","invocationPolicy":"sourceMethodCall","invokeAt":"expressionEvaluation","kind":"privateConstructionCall","name":"lookAtTopCardOfDeckAndChooseDiscard","resultType":"gameSystem","source":"physicalAbilitySource"},"labelParts":[{"kind":"const","value":"Opponent's deck"}]}],"factory":true,"keyOrder":"javascriptPropertyOrder"},"chooser":{"kind":"const","value":"self"},"defaults":"pinnedTargetResolver","kind":"choiceSelection","mode":"select","namedResultBinding":"selects.target.choice","promptTitleParts":[{"kind":"const","value":"Look at the top card of a deck"}],"resultBinding":"select","showUnresolvable":true},"title":"Look at the top card of a deck","trigger":"whenDefeated"}],"leader":[],"unit":[]},"baseClass":"NonLeaderUnitCard","card":{"arena":"ground","aspects":["vigilance","villainy"],"cost":1,"deployBox":null,"epicAction":"","hp":2,"id":"9790448244","internalName":"reanimated-night-trooper","keywords":[],"power":2,"rules":null,"setCodes":[{"number":45,"set":"ASH"}],"setId":{"number":45,"set":"ASH"},"subtitle":"","text":"When Defeated: Look at the top card of a deck. You may discard it.","title":"Reanimated Night Trooper","traits":["imperial","night","undead","trooper"],"types":["unit"],"unique":false,"upgradeHp":null,"upgradePower":null},"className":"ReanimatedNightTrooper","dataHash":"7e7f841fa06f6c874ef940221f5c58fbd8afe1d802662ae662520038041788a1","diagnostics":[],"id":"9790448244","internalName":"reanimated-night-trooper","keywordValues":{},"privatePrograms":[{"body":{"kind":"pureBlock","statements":[{"kind":"pureReturn","value":{"callbackEvaluation":"deferredWithLexicalBindings","captures":[],"effect":{"captureIdentity":"valueOrPhysicalObjectReference","captures":[{"evaluateAt":"propertyFactoryInvocation","name":"systemCapture1465_0","order":0,"role":"readonlyDeclaration","value":{"args":[],"empty":"null","kind":"deckQuery","operation":"topCard","order":"topFirst","receiver":{"kind":"dataPath","root":["data:private:lookAtTopCardOfDeckAndChooseDiscard:1"],"steps":[]}}},{"evaluateAt":"propertyFactoryInvocation","name":"systemCapture1465_1","order":1,"role":"staticProperty","value":{"kind":"ref","path":["systemFactoryCaptures","systemCapture1465_0"]}},{"evaluateAt":"propertyFactoryInvocation","name":"systemCapture1465_2","order":2,"role":"staticProperty","value":{"kind":"ref","path":["systemFactoryCaptures","systemCapture1465_0"]}}],"constructionOrder":"readonlyDeclarationsThenObjectLeftToRight","effect":{"buttonDefaultTargetOverride":"clickedPhysicalCard","buttons":{"buttons":[{"arg":"discard","effect":{"kind":"discardSpecificCard","target":{"kind":"ref","path":["systemFactoryCaptures","systemCapture1465_2"]}},"textParts":[{"kind":"const","value":"Discard it"}]},{"arg":"leave","effect":{"hasLegalTarget":{"kind":"const","value":true},"kind":"choiceNoAction"},"textParts":[{"kind":"const","value":"Leave it on top"}]}],"kind":"buttonList"},"captureStage":"propertyFactory","captures":[],"defaults":"pinnedLookAtSystem","interactMode":"perCardButtons","kind":"lookAtOptions","resolveWindow":"afterDisplayComplete","target":{"kind":"ref","path":["systemFactoryCaptures","systemCapture1465_1"]}},"invocationPolicy":"eachNativePropertyGeneration","kind":"systemPropertiesFactory","parameters":[]},"evaluation":"capturePropertiesThenConstruct","kind":"systemConstruction"}}]},"defaultTargetParameter":{"aggregatePropagation":"AggregateSystem.setDefaultTargetFn","binding":"returnedSystemMutableDefaultTargetFn","captureSelectedTargetEarly":false,"lookup":"nativePropertyGeneration","name":"targetDefault:lookAtTopCardOfDeckAndChooseDiscard"},"kind":"constructionProgram","name":"lookAtTopCardOfDeckAndChooseDiscard","parameters":[{"bindingName":"AbilityHelper","capability":"AbilityHelper","optional":false,"slot":"data:private:lookAtTopCardOfDeckAndChooseDiscard:0"},{"optional":false,"slot":"data:private:lookAtTopCardOfDeckAndChooseDiscard:1"}],"source":"physicalAbilitySource"}],"requiredCapabilities":["cardType:unit","ir:choiceSelection","ir:const","ir:lexicalCapability","ir:privateConstructionCall","ir:ref","ir:resolveChoice","ref:opponent","ref:player","targetMode:select","trigger:whenDefeated"],"setCode":"ASH_045","sourceHash":"93c1891214b79bcdafc71497c995c8aa42008a8c96610b4bdff1232553e24ab0","sourcePath":"game/cards/08_ASH/units/ReanimatedNightTrooper.ts","status":"supported"}"####;
const RECOVERY: &str = r####"{"abilities":{"card":[{"costs":[],"effect":{"amountOfChoices":{"kind":"const","value":1},"canChooseSameOptionMoreThanOnce":false,"choices":[{"effect":{"effects":[{"kind":"moveToBottomOfDeck","target":"chosen"},{"amount":{"kind":"const","value":3},"kind":"heal","target":"ownBase"}],"kind":"simultaneous"},"label":"Put on the bottom of your deck and heal 3 damage from your base"},{"effect":{"kind":"returnToHand","target":"chosen"},"label":"Return it to your hand"}],"kind":"chooseModalEffects","target":"chosen"},"targetResolver":{"owner":"self","predicate":{"kind":"binary","left":{"args":[],"kind":"query","name":"hasCost","receiver":{"kind":"ref","path":["card"]}},"op":"&&","right":{"kind":"binary","left":{"kind":"ref","path":["card","cost"]},"op":"<=","right":{"kind":"const","value":7}}},"type":"unit","zone":"discard"},"title":"Choose a unit in your discard pile that costs 7 or less","trigger":"whenPlayed"},{"costs":[],"effect":{"amountOfChoices":{"kind":"const","value":1},"canChooseSameOptionMoreThanOnce":false,"choices":[{"effect":{"effects":[{"kind":"moveToBottomOfDeck","target":"chosen"},{"amount":{"kind":"const","value":3},"kind":"heal","target":"ownBase"}],"kind":"simultaneous"},"label":"Put on the bottom of your deck and heal 3 damage from your base"},{"effect":{"kind":"returnToHand","target":"chosen"},"label":"Return it to your hand"}],"kind":"chooseModalEffects","target":"chosen"},"targetResolver":{"owner":"self","predicate":{"kind":"binary","left":{"args":[],"kind":"query","name":"hasCost","receiver":{"kind":"ref","path":["card"]}},"op":"&&","right":{"kind":"binary","left":{"kind":"ref","path":["card","cost"]},"op":"<=","right":{"kind":"const","value":7}}},"type":"unit","zone":"discard"},"title":"Choose a unit in your discard pile that costs 7 or less","trigger":"onAttack"}],"leader":[],"unit":[]},"baseClass":"NonLeaderUnitCard","card":{"arena":"ground","aspects":["command"],"cost":8,"deployBox":null,"epicAction":"","hp":9,"id":"6744545975","internalName":"trask-walker","keywords":[],"power":5,"rules":"Abilities that refer to a card\u2019s cost always refer to its printed cost, regardless of modifiers.\n","setCodes":[{"number":133,"set":"ASH"}],"setId":{"number":133,"set":"ASH"},"subtitle":"","text":"When Played/On Attack: Choose a unit in your discard pile that costs 7 or less. Either put that card on the bottom of your deck and heal 3 damage from your base or return it to your hand.","title":"Trask Walker","traits":["vehicle","walker"],"types":["unit"],"unique":false,"upgradeHp":null,"upgradePower":null},"className":"TraskWalker","dataHash":"2caed1d257602b592c83ea4f6c81ba37e2a999d5d5fd011901b4eec032f58ef8","diagnostics":[],"id":"6744545975","internalName":"trask-walker","keywordValues":{},"requiredCapabilities":["binary:&&","binary:<=","cardType:unit","ir:binary","ir:chooseModalEffects","ir:const","ir:heal","ir:moveToBottomOfDeck","ir:query","ir:ref","ir:returnToHand","ir:simultaneous","query:hasCost","ref:card","ref:card.cost","targetType:unit","targetZone:discard","trigger:onAttack","trigger:whenPlayed"],"setCode":"ASH_133","sourceHash":"3d59c9606adc0b1e5a2df8ada505ec82ca3487f0f948ddc0aa84479e6a5c965c","sourcePath":"game/cards/08_ASH/units/TraskWalker.ts","status":"supported"}"####;
pub(crate) fn profile(kind: Kind) -> Value {
    serde_json::from_str(match kind {
        Kind::Top => TOP,
        Kind::Recovery => RECOVERY,
    })
    .expect("closed modal deck source")
}
pub(crate) fn prepare(raw: &Value) -> Result<Option<Kind>, String> {
    let profiles = [
        (Kind::Top, profile(Kind::Top)),
        (Kind::Recovery, profile(Kind::Recovery)),
    ];
    let mut todo = vec![(raw, 0usize)];
    let mut n = 0;
    let mut marked = false;
    while let Some((v, d)) = todo.pop() {
        n += 1;
        if n > 65536 || d > 128 {
            return Err("modal deck source inspection capacity exceeded".into());
        }
        marked |= profiles.iter().any(|(_, p)| {
            v["setCode"] == p["setCode"]
                || v["sourcePath"] == p["sourcePath"]
                || v["id"] == p["id"]
                || v["sourceHash"] == p["sourceHash"]
                || v["className"] == p["className"]
                || v["internalName"] == p["internalName"]
        });
        marked |= v["name"] == "lookAtTopCardOfDeckAndChooseDiscard";
        match v {
            Value::Array(a) => todo.extend(a.iter().map(|v| (v, d + 1))),
            Value::Object(o) => todo.extend(o.values().map(|v| (v, d + 1))),
            _ => {}
        }
        if todo.len() + n > 65536 {
            return Err("modal deck source inspection capacity exceeded".into());
        }
    }
    if !marked {
        return Ok(None);
    }
    for (kind, p) in profiles {
        if raw == &p {
            return Ok(Some(kind));
        }
    }
    Err(
        "modal deck movement requires complete canonical source, helper and metadata profile"
            .into(),
    )
}
// The complete recipes have exactly two mandatory choices. Static branch
// availability preserves the source menus, including an empty chosen deck,
// without granting a generic private callback or unresolvable-system adapter.
fn menu(title: &str, a: &str, b: &str, show: bool, preserve: bool) -> Instruction {
    Instruction::ModalChoice(Menu {
        show_unresolvable: show,
        chooser: Chooser::Own,
        prompt_title: title.into(),
        branches: vec![
            Branch {
                label: a.into(),
                start: 1,
                end: 2,
                always_legal: preserve,
            },
            Branch {
                label: b.into(),
                start: 3,
                end: 4,
                always_legal: preserve,
            },
        ],
        end: 5,
        preserve_target: preserve,
        unless_branch: None,
    })
}
pub(crate) fn compile(kind: Kind, start: usize) -> Result<(CardPrograms, Vec<Program>), String> {
    let entry = profile(kind);
    let mut programs = vec![];
    for raw in entry["abilities"]["card"]
        .as_array()
        .ok_or("missing modal registrations")?
    {
        // Compile an ordinary neutral registration header, then install only the
        // source-proved private recipe. Standalone private construction stays rejected.
        let mut header = raw.clone();
        header.as_object_mut().unwrap().remove("targetResolver");
        header["effect"] = serde_json::json!({"kind":"heal","target":"ownBase","amount":{"kind":"const","value":0}});
        let ability: Ability = serde_json::from_value(header).map_err(|e| e.to_string())?;
        let mut p = Program::compile(&ability)?;
        match kind {
            Kind::Top => {
                p.instructions = vec![
                    menu(
                        "Look at the top card of a deck",
                        "Your deck",
                        "Opponent's deck",
                        false,
                        false,
                    ),
                    Instruction::LookTopDeckOptions { opponent: false },
                    Instruction::Jump { to: 5 },
                    Instruction::LookTopDeckOptions { opponent: true },
                    Instruction::Jump { to: 5 },
                    Instruction::EndStep,
                ];
                p.event_end = 5;
            }
            Kind::Recovery => {
                let select: TargetResolver = serde_json::from_value(raw["targetResolver"].clone())
                    .map_err(|e| e.to_string())?;
                let entries = raw["effect"]["choices"]
                    .as_array()
                    .ok_or("missing recovery choices")?;
                let bottom: Effect = serde_json::from_value(entries[0]["effect"].clone())
                    .map_err(|e| e.to_string())?;
                let hand: Effect = serde_json::from_value(entries[1]["effect"].clone())
                    .map_err(|e| e.to_string())?;
                let mut m = match menu(
                    "Choose 1 of the following",
                    entries[0]["label"].as_str().ok_or("recovery label")?,
                    entries[1]["label"].as_str().ok_or("recovery label")?,
                    false,
                    true,
                ) {
                    Instruction::ModalChoice(m) => m,
                    _ => unreachable!(),
                };
                for b in &mut m.branches {
                    b.start += 1;
                    b.end += 1;
                    b.always_legal = true;
                }
                m.end += 1;
                p.instructions = vec![
                    Instruction::Select(select),
                    Instruction::ModalChoice(m),
                    Instruction::Group(bottom),
                    Instruction::Jump { to: 6 },
                    Instruction::Apply(hand),
                    Instruction::Jump { to: 6 },
                    Instruction::EndStep,
                ];
                p.event_end = 6;
            }
        }
        p.executable = Err("closed modal deck recipe uses shared Rust interpreter");
        programs.push(p);
    }
    let ids = (start..start + programs.len())
        .map(|i| u16::try_from(i).map_err(|_| "ability program capacity exceeded".to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    Ok((
        CardPrograms {
            card: ids,
            ..CardPrograms::default()
        },
        programs,
    ))
}
pub(crate) fn requires(p: &Program) -> bool {
    p.instructions.iter().any(|i|matches!(i,Instruction::LookTopDeckOptions{..})) || p.instructions.iter().any(|i|matches!(i,Instruction::Group(Effect::Simultaneous{effects}) if effects.iter().any(|e|matches!(e,Effect::MoveToBottomOfDeck{..}))))
}
pub(crate) fn program(p: &Program) -> bool {
    [Kind::Top, Kind::Recovery].into_iter().any(|k| {
        compile(k, 0).is_ok_and(|(_, ps)| {
            ps.iter()
                .any(|q| serde_json::to_value(p).ok() == serde_json::to_value(q).ok())
        })
    })
}

/// The static source menu reveals both choices only after one real look system
/// has a legal top card. This is narrower than a generic unresolvable menu.
pub(crate) fn top_program(p: &Program) -> bool {
    if !p
        .instructions
        .iter()
        .any(|i| matches!(i, Instruction::LookTopDeckOptions { .. }))
    {
        return false;
    }
    compile(Kind::Top, 0)
        .is_ok_and(|(_, ps)| serde_json::to_value(p).ok() == serde_json::to_value(&ps[0]).ok())
}
