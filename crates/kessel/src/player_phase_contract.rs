//! Closed whole-source authorization for direct player-owned phase effects.
//! Pure source proof: no runtime state, engine or host callback execution.
use serde_json::{json, Value};
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Kind {
    ImperialDiscount,
    LowPrintedPowerReady,
}
const TEMPLATES: [&str; 2] = [
    r###"{"setCode":"ASH_237","sourcePath":"game/cards/08_ASH/units/MouseDroid.ts","sourceHash":"62deb9ca7463987af7ca1af68a4778562e0ed283edca5d0d19885703525eb85c","abilities":{"leader":[],"unit":[],"card":[{"trigger":"whenPlayed","title":"The next unit Imperial you play this phase costs 1 resource less","costs":[],"effect":{"kind":"phasePlayerCostEffect","target":"ownPlayer","duration":"phase","effects":[{"kind":"costAdjuster","adjustment":"decrease","stage":"standard_0","cardTypeFilter":"unit","matchAbilityCosts":false,"matchCardEffectResourcePayments":false,"limit":{"kind":"costAdjusterLimit","scope":"perPlayerPerGame","max":1,"clonePerEffectApplication":true},"match":{"kind":"boundCallback","parameters":["adjustedCard"],"timing":"costEligibility","value":{"kind":"query","name":"hasSomeTrait","receiver":{"kind":"ref","path":["adjustedCard"]},"args":[{"kind":"const","value":"imperial"}]}},"amount":{"kind":"const","value":1}}]}}]},"className":"MouseDroid","baseClass":"NonLeaderUnitCard","id":"2588977937","internalName":"mouse-droid","status":"supported","card":{"title":"Mouse Droid","subtitle":"","cost":1,"hp":1,"power":1,"text":"Raid 1 (This unit gets +1/+0 while attacking.)\nWhen Played: The next Imperial unit you play this phase costs 1 resource less.","deployBox":null,"epicAction":"","unique":false,"rules":"Once Mouse Droid's \u201cWhen Played\u201d ability resolves, it remains active even if it is defeated before you play your next Imperial unit.\n","upgradePower":null,"upgradeHp":null,"id":"2588977937","aspects":["villainy"],"traits":["imperial","droid"],"arena":"ground","keywords":["raid"],"types":["unit"],"setId":{"set":"ASH","number":237},"setCodes":[{"set":"ASH","number":237}],"internalName":"mouse-droid"},"dataHash":"0731dde5e7abc1d9cd9e134f203b36e98545f16642f717051611de0bd99d8d7c","keywordValues":{"raid":1},"requiredCapabilities":["cardType:unit","ir:boundCallback","ir:const","ir:costAdjuster","ir:costAdjusterLimit","ir:phasePlayerCostEffect","ir:query","ir:ref","keyword:raid","query:hasSomeTrait","ref:adjustedCard","trigger:whenPlayed"],"diagnostics":[]}"###,
    r###"{"setCode":"ASH_248","sourcePath":"game/cards/08_ASH/units/NeelTheCutestBoy.ts","sourceHash":"89edabd6669728db5736536685918f69d4a36de7c3152c11760f5c9c84dcc547","abilities":{"leader":[],"unit":[],"card":[{"trigger":"whenPlayed","title":"The next unit you play this phase with 1 or less power enters play ready","costs":[],"effect":{"kind":"phasePlayerEffect","target":"ownPlayer","duration":"phase","effects":[{"kind":"unitsEnterPlayReady","cardTypeFilter":"unit","match":{"kind":"attackCallback","parameters":[{"root":"attackBinding1_0","role":"enteringUnit"}],"body":{"kind":"binary","op":"<=","left":{"kind":"query","name":"getPrintedPower","receiver":{"kind":"ref","path":["attackBinding1_0"]},"args":[]},"right":{"kind":"const","value":1}}},"limit":{"kind":"playerModifierLimit","scope":"perPlayerPerGame","max":1,"clonePerApplication":true},"entryType":"played","matchTiming":"beforeUnitEntry","limitIncrementTiming":"successfulMatcherApplication"}]}},{"trigger":"onAttack","title":"The next unit you play this phase with 1 or less power enters play ready","costs":[],"effect":{"kind":"phasePlayerEffect","target":"ownPlayer","duration":"phase","effects":[{"kind":"unitsEnterPlayReady","cardTypeFilter":"unit","match":{"kind":"attackCallback","parameters":[{"root":"attackBinding1_0","role":"enteringUnit"}],"body":{"kind":"binary","op":"<=","left":{"kind":"query","name":"getPrintedPower","receiver":{"kind":"ref","path":["attackBinding1_0"]},"args":[]},"right":{"kind":"const","value":1}}},"limit":{"kind":"playerModifierLimit","scope":"perPlayerPerGame","max":1,"clonePerApplication":true},"entryType":"played","matchTiming":"beforeUnitEntry","limitIncrementTiming":"successfulMatcherApplication"}]}}]},"className":"NeelTheCutestBoy","baseClass":"NonLeaderUnitCard","id":"8371755043","internalName":"neel#the-cutest-boy","status":"supported","card":{"title":"Neel","subtitle":"The Cutest Boy","cost":1,"hp":4,"power":1,"text":"When Played/On Attack: The next unit you play this phase with 1 or less power enters play ready.","deployBox":null,"epicAction":"","unique":true,"rules":"(ERRATA) When Played/On Attack: The next unit you play this phase with 1 or less printed power enters play ready.\nOnce Neel's \u201cWhen Played/On Attack\u201d ability resolves, it remains active even if it is defeated before you play your next unit with 1 or less printed power.\n","upgradePower":null,"upgradeHp":null,"id":"8371755043","aspects":["heroism"],"traits":["fringe"],"arena":"ground","keywords":[],"types":["unit"],"setId":{"set":"ASH","number":248},"setCodes":[{"set":"ASH","number":248}],"internalName":"neel#the-cutest-boy"},"dataHash":"40ef64a9dfd8f239807c0233fd8baf878756f41ca4ec38c0c8471e91f93dafe3","keywordValues":{},"requiredCapabilities":["binary:<=","cardType:unit","ir:attackCallback","ir:binary","ir:const","ir:phasePlayerEffect","ir:playerModifierLimit","ir:query","ir:ref","ir:unitsEnterPlayReady","query:getPrintedPower","ref:attackBinding1_0","trigger:onAttack","trigger:whenPlayed"],"diagnostics":[]}"###,
];
pub(crate) fn marker(value: &Value) -> Result<bool, String> {
    let mut todo = vec![(value, 0usize)];
    let mut count = 0;
    let mut found = false;
    while let Some((v, depth)) = todo.pop() {
        count += 1;
        if count > 32768 || depth > 64 {
            return Err("player phase source inspection capacity exceeded".into());
        }
        match v {
            Value::Object(o) => {
                found |= o.get("kind").and_then(Value::as_str).is_some_and(|s| {
                    matches!(
                        s,
                        "phasePlayerCostEffect"
                            | "phasePlayerEffect"
                            | "costAdjuster"
                            | "costAdjusterLimit"
                            | "playerModifierLimit"
                            | "unitsEnterPlayReady"
                            | "installPlayerPhase"
                    )
                }) || o
                    .keys()
                    .any(|k| k.starts_with("playerPhase") || k.starts_with("player_phase"));
                for x in o.values() {
                    todo.push((x, depth + 1));
                }
            }
            Value::Array(a) => {
                for x in a {
                    todo.push((x, depth + 1));
                }
            }
            _ => {}
        }
    }
    Ok(found)
}
#[derive(Debug, Clone)]
pub(crate) struct Projection {
    pub kind: Kind,
    abilities: Value,
}
pub(crate) fn prepare(entry: &Value) -> Result<Option<Projection>, String> {
    if !marker(entry)? {
        return Ok(None);
    }
    for (index, template) in TEMPLATES.iter().enumerate() {
        let expected: Value = serde_json::from_str(template).expect("closed phase template");
        let closed_helpers = [
            "stateWatchers",
            "historyHelpers",
            "privatePrograms",
            "classMethods",
            "privateClassConstants",
            "inheritedSource",
            "deploymentOverride",
            "setupValidationOverrides",
            "piloting",
            "attachCondition",
        ];
        if !entry.as_object().is_some_and(|o| {
            o.keys()
                .all(|k| expected.get(k).is_some() || closed_helpers.contains(&k.as_str()))
        }) || [
            "internalName",
            "className",
            "requiredCapabilities",
            "diagnostics",
        ]
        .iter()
        .any(|k| entry[*k] != expected[*k])
        {
            continue;
        }
        let mut card = entry["card"].clone();
        let mut want = expected["card"].clone();
        for key in ["id", "setId", "setCodes"] {
            if let Some(o) = card.as_object_mut() {
                o.remove(key);
            }
            if let Some(o) = want.as_object_mut() {
                o.remove(key);
            }
        }
        if card != want
            || entry["abilities"] != expected["abilities"]
            || entry["baseClass"] != expected["baseClass"]
            || entry["keywordValues"] != expected["keywordValues"]
            || entry["sourcePath"] != expected["sourcePath"]
            || entry["sourceHash"] != expected["sourceHash"]
            || entry["status"] != "supported"
            || !entry["piloting"].is_null()
            || !entry["attachCondition"].is_null()
            || !crate::admission::SourceContextPolicy::Closed.accepts(entry)
        {
            continue;
        }
        return Ok(Some(Projection {
            kind: if index == 0 {
                Kind::ImperialDiscount
            } else {
                Kind::LowPrintedPowerReady
            },
            abilities: entry["abilities"].clone(),
        }));
    }
    Err("invalid player phase installer requires the exact closed ordinary direct-source profile".into())
}
impl Projection {
    pub(crate) fn compile(
        &self,
        start: usize,
    ) -> Result<
        (
            crate::ability_ir::CardPrograms,
            Vec<crate::ability_ir::Program>,
        ),
        String,
    > {
        use crate::ability_ir::{Ability, CardPrograms, Effect, Instruction, Program};
        let mut bindings = CardPrograms::default();
        let mut programs = vec![];
        for raw in self.abilities["card"]
            .as_array()
            .ok_or("phase card registrations missing")?
        {
            let mut harmless = raw.clone();
            harmless["effect"] = json!({"kind":"noAction"});
            let a: Ability = serde_json::from_value(harmless).map_err(|e| e.to_string())?;
            let mut p = Program::compile(&a)?;
            if p.instructions.len() != 2 {
                return Err("phase direct handler shape drift".into());
            }
            p.instructions[0] = Instruction::Apply(Effect::InstallPlayerPhase {
                phase_kind: self.kind,
            });
            p.executable = Err("direct player phase installer uses shared Rust interpreter");
            bindings.card.push(
                u16::try_from(start + programs.len()).map_err(|_| "program capacity exceeded")?,
            );
            programs.push(p);
        }
        Ok((bindings, programs))
    }
}
pub(crate) fn copied_program_allowed(p: &crate::ability_ir::Program) -> Result<(), String> {
    if program_operations(p)?.installers != 0
        || crate::ability_ir::contains(&serde_json::to_value(p).map_err(|e| e.to_string())?, |v| {
            v["kind"] == "installPlayerPhase"
        })
    {
        return Err("copied player phase installer lacks direct own-source authorization".into());
    }
    Ok(())
}

#[cfg(test)]
pub(crate) fn fixture(index: usize) -> Value {
    serde_json::from_str(TEMPLATES[index]).unwrap()
}

/// Exhaustive typed mutation inventory for the direct-only interval proof.
/// A future Effect or Instruction variant must receive an explicit review here.
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct Operations {
    pub installers: usize,
    pub attacks: usize,
}
impl Operations {
    fn add(&mut self, other: Self) {
        self.installers += other.installers;
        self.attacks += other.attacks;
    }
}
fn operations(e: &crate::ability_ir::Effect) -> Result<Operations, String> {
    use crate::ability_ir::Effect;
    let mut out = Operations::default();
    match e {
        Effect::InstallPlayerPhase { .. } => out.installers = 1,
        Effect::InitiateAttack { properties, .. } => {
            properties.validate()?;
            out.attacks = 1;
        }
        Effect::Optional { effect, .. } | Effect::Select { effect, .. } => {
            out.add(operations(effect)?)
        }
        Effect::SelectionSystem { inner_system, .. } => out.add(operations(inner_system)?),
        Effect::Sequence { effects }
        | Effect::Simultaneous { effects }
        | Effect::PhaseModifiers { effects }
        | Effect::AttackModifiers { effects } => {
            for e in effects {
                out.add(operations(e)?);
            }
        }
        Effect::Conditional {
            on_true, on_false, ..
        } => {
            out.add(operations(on_true)?);
            if let Some(e) = on_false {
                out.add(operations(e)?);
            }
        }
        Effect::ConstantEffectRegistration(r) => {
            r.native()?;
        }
        Effect::Disclose { .. }
        | Effect::AttackPermission { .. }
        | Effect::GainTrait { .. }
        | Effect::LoseTrait { .. }
        | Effect::DiscardOwnHand { .. }
        | Effect::DiscardDeck { .. }
        | Effect::CreateTokenUnit { .. }
        | Effect::Damage { .. }
        | Effect::RecipientHeal { .. }
        | Effect::Heal { .. }
        | Effect::Draw { .. }
        | Effect::CreateCreditToken { .. }
        | Effect::ReadyResources { .. }
        | Effect::ExhaustResources { .. }
        | Effect::PayResources { .. }
        | Effect::PayAdjustedResources { .. }
        | Effect::NoAction
        | Effect::ResourceCard { .. }
        | Effect::ResourceCardSystem { .. }
        | Effect::MoveToBottomOfDeck { .. }
        | Effect::MoveSelectedArenaUnitToOwnerDeck { .. }
        | Effect::DealsCombatDamageFirst { .. }
        | Effect::ReturnToHand { .. }
        | Effect::ModifyStats { .. }
        | Effect::MaximumPowerExperience { .. }
        | Effect::CopyKeywords { .. }
        | Effect::GainKeyword { .. }
        | Effect::LoseKeyword { .. }
        | Effect::Ready { .. }
        | Effect::Exhaust { .. }
        | Effect::Defeat { .. }
        | Effect::GiveShield { .. }
        | Effect::UseForce { .. }
        | Effect::GainForce { .. }
        | Effect::GiveWeakness { .. }
        | Effect::GiveAdvantage { .. }
        | Effect::GiveExperience { .. }
        | Effect::DiscardSpecificCard { .. }
        | Effect::DistributeTokenUpgrade { .. }
        | Effect::DistributeDamage { .. }
        | Effect::DistributeHealing { .. }
        | Effect::SelectedPlayerSystem { .. }
        | Effect::LookAt { .. }
        | Effect::CardRestriction { .. }
        | Effect::LookAtAndSelectCard { .. } => {}
    }
    Ok(out)
}
pub(crate) fn program_operations(p: &crate::ability_ir::Program) -> Result<Operations, String> {
    use crate::ability_ir::Instruction;
    let mut out = Operations::default();
    for i in &p.instructions {
        match i {
            Instruction::Apply(e)
            | Instruction::Group(e)
            | Instruction::NamedGroup(e)
            | Instruction::NamedExhaustContingent(e)
            | Instruction::NamedChooserGroup { effect: e, .. } => out.add(operations(e)?),
            Instruction::EndStep
            | Instruction::Commit
            | Instruction::Select(_)
            | Instruction::ModalChoice(_)
            | Instruction::HandCost { .. }
            | Instruction::GrantCreatedTokenPhaseKeyword { .. }
            | Instruction::NameOpponentPlay
            | Instruction::ChooseNumber
            | Instruction::PayUnadjustedSelectedResources
            | Instruction::GiveSelectedExperience
            | Instruction::CaptureMaximum { .. }
            | Instruction::Conditional { .. }
            | Instruction::IfResult { .. }
            | Instruction::IfWillDefeat { .. }
            | Instruction::IfDiscardedSharesAspect { .. }
            | Instruction::IfDiscardedIsUnit { .. }
            | Instruction::OptionalConfirm { .. }
            | Instruction::Jump { .. } => {}
        }
    }
    Ok(out)
}
pub(crate) fn validate_direct_only_programs(
    programs: &[crate::ability_ir::Program],
) -> Result<(), String> {
    use crate::ability_ir::{Effect, Instruction};
    for p in programs {
        let ops = program_operations(p)?;
        if ops.installers != 0
            && (ops.installers != 1
                || !matches!(
                    p.instructions.as_slice(),
                    [
                        Instruction::Apply(Effect::InstallPlayerPhase { .. }),
                        Instruction::EndStep
                    ]
                )
                || !matches!(p.trigger.as_str(), "whenPlayed" | "onAttack")
                || !p.costs.is_empty()
                || p.optional
                || p.requires_confirmation
                || p.condition.is_some()
                || !p.hooks.is_empty()
                || p.attached_host
                || p.gained_host.is_some()
                || p.attached_context.is_some()
                || p.coordinate)
        {
            return Err("player phase direct-only program origin exceeds closed lifecycle".into());
        }
        if ops.attacks > 64 {
            return Err("player phase parent attack batch exceeds application capacity".into());
        }
        if ops.attacks != 0
            && (!matches!(p.trigger.as_str(), "event" | "action" | "whenPlayed")
                || !p.hooks.is_empty()
                || p.attached_host
                || p.gained_host.is_some()
                || p.attached_context.is_some())
        {
            return Err("player phase direct-only proof forbids attack initiation from captured/copy/death hooks".into());
        }
    }
    Ok(())
}
