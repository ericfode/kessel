//! Closed modal target resolvers from the pinned SelectTargetResolver.
//!
//! Only a sole ordinary event/unit registration is normalized. Every branch is
//! checked and compiled before the containing card can be admitted. Menus reuse
//! the existing program counter and target decision; no state is allocated.
use crate::ability_ir::{Ability, Effect, Expr, Instruction, Program, Target};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Chooser {
    Own,
    Opponent,
    SelectedOwner,
}
#[derive(Debug, Clone, Serialize)]
pub struct Branch {
    pub label: String,
    pub start: usize,
    /// Exclusive branch end, before its jump to the common continuation.
    pub end: usize,
    /// NoAction({ hasLegalTarget: true }) is legal without an effective event.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub always_legal: bool,
}
#[derive(Debug, Clone, Serialize)]
pub struct Menu {
    pub chooser: Chooser,
    pub prompt_title: String,
    pub branches: Vec<Branch>,
    pub end: usize,
    #[serde(skip_serializing_if="std::ops::Not::not")]
    pub preserve_target:bool,
    #[serde(skip_serializing_if="Option::is_none")]
    pub unless_branch:Option<usize>,
}
#[derive(Debug)]
pub struct Prepared {
    pub normalized: Value,
    chooser: Chooser,
    prompt_title: String,
    branches: Vec<(String, Program)>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Resolver {
    kind: String,
    mode: String,
    result_binding: String,
    named_result_binding: String,
    defaults: String,
    chooser: Expr,
    #[serde(default)]
    prompt_title_parts: Vec<Expr>,
    choices: Choices,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Choices {
    entries: Vec<Entry>,
    context_binding: String,
    key_order: String,
    duplicate_policy: String,
    // A factory is safe only when its entire emitted body is in our closed
    // effect vocabulary. Callback/data program nodes fail deserialization.
    factory: bool,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Entry {
    label_parts: Vec<Expr>,
    effect: Effect,
}

fn marker(v: &Value) -> bool {
    matches!(
        v["kind"].as_str(),
        Some("choiceSelection" | "resolveChoice")
    ) || match v {
        Value::Object(o) => o.values().any(marker),
        Value::Array(a) => a.iter().any(marker),
        _ => false,
    }
}
fn text(parts: &[Expr]) -> Result<String, String> {
    if parts.len() > 32 {
        return Err("modal label exceeds fragment capacity".into());
    }
    let mut out = String::new();
    for part in parts {
        let Expr::Const { value } = part else {
            return Err("modal labels and titles require literal strings".into());
        };
        out.push_str(value.as_str().ok_or("modal text requires strings")?);
    }
    if out.len() > 1024 {
        return Err("modal text exceeds capacity".into());
    }
    Ok(out)
}
fn amount(e: &Expr, max: u64) -> bool {
    matches!(e, Expr::Const { value } if value.as_u64().is_some_and(|n| (1..=max).contains(&n)))
}
fn named(t: &Target, names: &[&str]) -> bool {
    matches!(t, Target::Name(n) if names.contains(&n.as_str()))
}
fn branch_effect(e: &Effect) -> bool {
    match e {
        Effect::Damage { target, amount: n } => named(target, &["enemyBase"]) && amount(n, 5),
        Effect::Draw { target, amount: n } => named(target, &["ownPlayer"]) && amount(n, 2),
        Effect::DiscardOwnHand { target, amount: n } => {
            named(target, &["enemyPlayer"]) && amount(n, 2)
        }
        Effect::ReadyResources { target, amount: n } => {
            named(target, &["ownPlayer", "enemyPlayer"]) && amount(n, 2)
        }
        Effect::CreateTokenUnit {
            target,
            amount: n,
            token_type,
            enters_ready,
        } => {
            named(target, &["ownPlayer"])
                && amount(n, 5)
                && !*enters_ready
                && ["cloneTrooper", "battleDroid"].contains(&token_type.as_str())
        }
        Effect::Select {
            target_resolver: r,
            effect,
        } => {
            r.native.is_none()
                && !r.player_selection
                && r.card_type == "unit"
                && ["self", "opponent", "any"].contains(&r.owner.as_str())
                && matches!(r.zone.as_deref(), Some("anyAttackable" | "arena"))
                && r.arena
                    .as_deref()
                    .is_none_or(|a| ["ground", "space"].contains(&a))
                && !r.optional
                && r.mode.is_none()
                && r.count.is_none()
                && r.prompt_title_parts.is_empty()
                && r.validate_inline().is_ok()
                && match effect.as_ref() {
                    Effect::Exhaust { target } => named(target, &["chosen"]),
                    Effect::GiveExperience { target, amount: n } => {
                        named(target, &["chosen"]) && amount(n, 1)
                    }
                    Effect::Damage { target, amount: n } => {
                        named(target, &["chosen"]) && amount(n, 5)
                    }
                    _ => false,
                }
        }
        _ => false,
    }
}
// ECMAScript array-index keys enumerate before other string keys. Duplicate
// assignments replace the value while retaining the first insertion position.
fn index_key(s: &str) -> Option<u32> {
    let n: u32 = s.parse().ok()?;
    (n < u32::MAX && n.to_string() == s).then_some(n)
}

pub fn prepare(entry: &Value) -> Result<Option<Prepared>, String> {
    if !marker(&entry["abilities"]) {
        return Ok(None);
    }
    let cards = entry["abilities"]["card"]
        .as_array()
        .ok_or("modal card abilities missing")?;
    let source = (entry["baseClass"] == "NonLeaderUnitCard"
        && entry["card"]["types"] == json!(["unit"]))
        || (entry["baseClass"] == "EventCard" && entry["card"]["types"] == json!(["event"]));
    if !source
        || !crate::admission::SourceContextPolicy::Closed.accepts(entry)
        || cards.len() != 1
        || ["unit", "leader"].iter().any(|side| {
            !entry["abilities"][side]
                .as_array()
                .is_some_and(Vec::is_empty)
        })
        || !entry["attachCondition"].is_null()
        || !entry["piloting"].is_null()
        || !entry["card"]["unique"].is_boolean()
        || [
            "pilotText",
            "pilotBox",
            "pilotingBox",
            "deployBox",
            "deployCost",
            "upgradePower",
            "upgradeHp",
        ]
        .iter()
        .any(|key| !entry["card"][key].is_null())
        || !(entry["card"]["epicAction"].is_null() || entry["card"]["epicAction"] == "")
    {
        return Err(
            "modal choices require a complete sole ordinary unit/event registration".into(),
        );
    }
    let raw = &cards[0];
    let trigger = raw["trigger"].as_str().unwrap_or("");
    if raw.as_object().is_none_or(|object| {
        object.keys().any(|key| {
            !["trigger", "title", "costs", "targetResolver", "effect"].contains(&key.as_str())
        })
    }) {
        return Err("modal registration contains an unreviewed header field".into());
    }
    if !(if entry["baseClass"] == "EventCard" {
        trigger == "event"
    } else {
        ["whenPlayed", "onAttack", "whenDefeated"].contains(&trigger)
    }) || raw["optional"] == true
        || raw["coordinate"] == true
        || raw["requiresConfirmation"] == true
        || !raw["costs"].as_array().is_some_and(Vec::is_empty)
        || [
            "condition",
            "then",
            "ifYouDo",
            "attachedContext",
            "gainedHost",
            "hooks",
        ]
        .iter()
        .any(|key| !raw[key].is_null())
        || !raw["contextTitle"].as_array().is_none_or(Vec::is_empty)
        || !raw["targetResolvers"].as_array().is_none_or(Vec::is_empty)
        || raw["effect"] != json!({"kind":"resolveChoice", "resultBinding":"select"})
    {
        return Err(
            "modal choices require an unmodified root resolver without continuation or costs"
                .into(),
        );
    }
    let resolver: Resolver =
        serde_json::from_value(raw["targetResolver"].clone()).map_err(|e| e.to_string())?;
    let chooser = match resolver.chooser {
        Expr::Const { value } if value == "self" => Chooser::Own,
        Expr::Const { value } if value == "opponent" => Chooser::Opponent,
        _ => return Err("modal chooser must be literal self or opponent".into()),
    };
    // Opponent event targeting is delayed until after event payment. That is
    // a separate resolver stage; this lane admits only self-chooser events.
    if entry["baseClass"] == "EventCard" && chooser != Chooser::Own {
        return Err("opponent modal event selection requires post-payment targeting".into());
    }
    if resolver.kind != "choiceSelection"
        || resolver.mode != "select"
        || resolver.result_binding != "select"
        || resolver.named_result_binding != "selects.target.choice"
        || resolver.defaults != "pinnedTargetResolver"
        || resolver.choices.context_binding != "resolverContextByReference"
        || resolver.choices.key_order != "javascriptPropertyOrder"
        || resolver.choices.duplicate_policy != "lastAssignmentWins"
        || !(1..=8).contains(&resolver.choices.entries.len())
    {
        return Err("modal choices require the pinned bounded select protocol".into());
    }
    let _factory = resolver.choices.factory;
    let title = text(&resolver.prompt_title_parts)?;
    let mut branches: Vec<(String, Program)> = Vec::new();
    let mut total_text = 0;
    for child in resolver.choices.entries {
        let label = text(&child.label_parts)?;
        total_text += label.len();
        if label.is_empty() || total_text > 4096 || !branch_effect(&child.effect) {
            return Err("modal branch exceeds the reviewed scalar/select/token vocabulary".into());
        }
        let ability: Ability = serde_json::from_value(json!({
            "trigger": trigger, "title": label, "costs": [], "effect": child.effect
        }))
        .map_err(|e| e.to_string())?;
        let program = Program::compile(&ability)?;
        if let Some(old) = branches.iter_mut().find(|(key, _)| key == &label) {
            old.1 = program;
        } else {
            branches.push((label, program));
        }
    }
    branches.sort_by_key(|(key, _)| index_key(key).map_or((1, 0), |n| (0, n)));
    let mut normalized = entry.clone();
    let object = normalized["abilities"]["card"][0].as_object_mut().unwrap();
    object.remove("targetResolver");
    object.insert("effect".into(), json!({"kind":"noAction"}));
    Ok(Some(Prepared {
        normalized,
        chooser,
        prompt_title: if title.is_empty() {
            "Select one".into()
        } else {
            title
        },
        branches,
    }))
}
impl Prepared {
    pub fn compile(&self, ability: &Ability) -> Result<Program, String> {
        let branches = self
            .branches
            .iter()
            .map(|(label, program)| (label.clone(), program.clone(), false))
            .collect::<Vec<_>>();
        compile_branches(ability, self.chooser, &self.prompt_title, &branches)
    }
}
/// Reuse the same bounded jump table for fully validated native modal leaves.
/// Explicit legality is separate from the selected branch's execution result.
pub(crate) fn compile_branches(
    ability: &Ability,
    chooser: Chooser,
    prompt_title: &str,
    branches: &[(String, Program, bool)],
) -> Result<Program, String> {
    let mut program = Program::compile(ability)?;
    program.instructions.clear();
    program.select_ends.clear();
    let mut menu = Menu {
        chooser,
        prompt_title: prompt_title.to_owned(),
        branches: Vec::new(),
        end: 0,
        preserve_target:false,
        unless_branch:None,
    };
    // Placeholder is replaced after the absolute branch boundaries are known.
    program.instructions.push(Instruction::EndStep);
    let mut jumps = Vec::new();
    for (label, child, always_legal) in branches {
        let start = program.instructions.len();
        for instruction in child.instructions.iter().take(child.instructions.len() - 1) {
            // Our closed effect vocabulary emits no conditional/jump subprograms.
            if !matches!(
                instruction,
                Instruction::Commit
                    | Instruction::Select(_)
                    | Instruction::Apply(_)
                    | Instruction::Group(_)
            ) {
                return Err("modal branch emitted unsupported control flow".into());
            }
            program.instructions.push(instruction.clone());
        }
        let end = program.instructions.len();
        for (at, stop) in &child.select_ends {
            program.select_ends.push((start + at, start + stop));
        }
        menu.branches.push(Branch {
            label: label.clone(),
            start,
            end,
            always_legal: *always_legal,
        });
        jumps.push(end);
        program.instructions.push(Instruction::EndStep);
    }
    menu.end = program.instructions.len();
    for pc in jumps {
        program.instructions[pc] = Instruction::Jump { to: menu.end };
    }
    program.instructions[0] = Instruction::ModalChoice(menu);
    program.event_end = program
        .instructions
        .len()
        .try_into()
        .map_err(|_| "modal instruction capacity exceeded")?;
    program.instructions.push(Instruction::EndStep);
    if program.instructions.len() > crate::ability_ir::MAX_PROGRAM_INSTRUCTIONS {
        return Err("modal instruction capacity exceeded".into());
    }
    program.executable = Err("modal choices use validated interpreter continuation");
    Ok(program)
}
