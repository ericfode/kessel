#[path="src/round_limit_contract.rs"]
mod round_limit_contract;
#[path = "src/collective_trigger_contract.rs"]
mod collective_trigger_contract;
#[allow(dead_code)]
#[path="src/defeat_cause_contract.rs"]
mod defeat_cause;
// Compile validated catalog programs into executable Rust, keyed by content.
#[path="src/executor_upgrade_count_contract.rs"]
mod executor_upgrade_count;
#[path = "src/cinder_contract.rs"]
mod cinder_contract;
#[path="src/blade_base_damage_contract.rs"]
mod blade_base_damage_contract;
#[path="src/variable_payment_contract.rs"]
mod variable_payment_contract;
#[path = "src/arena_defeat_attack_contract.rs"]
mod arena_defeat_attack_contract;
#[path = "src/piloting_lifecycle.rs"]
mod piloting_lifecycle;
#[path = "src/unit_entry_contract.rs"]
mod unit_entry_contract;
#[path = "src/cross_arena_contract.rs"]
mod cross_arena_contract;
#[path="src/recipient_heal_contract.rs"]
mod recipient_heal_contract;
#[path = "src/admission.rs"]
mod admission;
#[allow(dead_code)]
#[path = "src/attachment_play_cost.rs"]
mod attachment_play_cost;
#[allow(dead_code)]
#[path = "src/credits.rs"]
mod credits;
#[allow(dead_code)]
#[path = "src/migration_credit_contract.rs"]
mod migration_credit_contract;
#[allow(dead_code)]
#[path = "src/migration_look_contract.rs"]
mod migration_look_contract;
#[allow(dead_code)]
#[path = "src/self_play_cost.rs"]
mod self_play_cost;
#[allow(dead_code)]
#[path = "src/card_restrictions.rs"]
mod card_restrictions;
#[allow(dead_code)]
#[path = "src/pay_adjusted_resources.rs"]
mod pay_adjusted_resources;
#[path = "src/resource_exhaust.rs"]
mod resource_exhaust;
#[allow(dead_code)]
#[path = "src/migration_attack_contract.rs"]
mod migration_attack_contract;
#[allow(dead_code)]
#[path = "src/native_attack.rs"]
mod native_attack;

#[path = "src/death_hooks.rs"]
mod death_hooks;
#[path = "src/regroup_hooks.rs"]
mod regroup_hooks;
#[allow(dead_code)]
#[path = "src/ability_ir.rs"]
mod ability_ir;
#[allow(dead_code)]
#[path = "src/bounded_reductions.rs"]
mod bounded_reductions;
#[allow(dead_code)]
#[path = "src/live_power_maximum.rs"]
mod live_power_maximum;
#[allow(dead_code)]
#[path = "src/static_ongoing.rs"]
mod static_ongoing;
#[allow(dead_code)]
#[path = "src/migration_defeat_history_contract.rs"]
mod migration_defeat_history_contract;
#[allow(dead_code)]
#[path = "src/phase_history.rs"]
mod phase_history;
#[allow(dead_code)]
#[path = "src/attack_history.rs"]
mod attack_history;
#[allow(dead_code)]
#[path = "src/attached_registration.rs"]
mod attached_registration;

#[allow(dead_code)]
#[path = "src/piloting_attached.rs"]
mod piloting_attached;

#[path="src/advantage.rs"]
mod advantage;

#[allow(dead_code)]
#[path = "src/attached_constants.rs"]
mod attached_constants;

#[path = "src/deck_search.rs"]
mod deck_search;
#[allow(dead_code)]
#[path = "src/selected_player.rs"]
mod selected_player;

#[allow(dead_code)]
#[path = "src/migration_modal_contract.rs"]
mod migration_modal_contract;
#[path = "src/migration_modal_deck_contract.rs"]
mod migration_modal_deck_contract;
#[path = "src/named_play_contract.rs"]
mod named_play_contract;
#[path = "src/named_play.rs"]
mod named_play;
#[allow(dead_code)]
#[path = "src/modal_choices.rs"]
mod modal_choices;

#[allow(dead_code)]
#[path = "src/modal_arena_choices.rs"]
mod modal_arena_choices;
#[allow(dead_code)]
#[path = "src/discard_return.rs"]
mod discard_return;
#[allow(dead_code)]
#[path = "src/view_cards.rs"]
mod view_cards;
#[allow(dead_code)]
#[path = "src/disclose.rs"]
mod disclose;
#[allow(dead_code)]
#[path="src/current_played_unit.rs"]
mod current_played_unit;
#[path = "src/catalogue_io.rs"]
mod catalogue_io;
mod expression_codegen;
fn expression_backend(_bytes: &[u8]) -> Option<usize> { None }
use ability_ir::{Catalogue, Effect, Expr, Instruction, Program, Target};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, env, fs, path::PathBuf};
fn program_backend(_bytes: &[u8]) -> Result<usize, &'static str> {
    Err("build-time compiler")
}
fn number(e: &Expr) -> Result<String, String> {
    match e {
        Expr::Const { value } => value
            .as_i64()
            .map(|n| format!("{n}i64"))
            .ok_or("noninteger amount".into()),
        Expr::Unary { op, value } if op == "-" => {
            Ok(format!("({}).saturating_neg()", number(value)?))
        }
        Expr::Binary { op, left, right } if ["+", "-", "*"].contains(&op.as_str()) => Ok(format!(
            "({}).{}({})",
            number(left)?,
            match op.as_str() {
                "+" => "saturating_add",
                "-" => "saturating_sub",
                _ => "saturating_mul",
            },
            number(right)?
        )),
        _ => Err("amount expression requires interpreter".into()),
    }
}
fn targets(t: &Target) -> Result<String, String> {
    let Target::Name(n) = t else {
        return Err("expression target requires interpreter".into());
    };
    Ok(match n.as_str() {
        "self" => "vec![Eval::Card(s.source)]",
        "chosen" => "vec![Eval::Card(s.ability_target)]",
        "ownBase" => "vec![Eval::Card(s.player(s.card(s.source).owner).base)]",
        "enemyBase" => "vec![Eval::Card(s.player(s.card(s.source).owner.other()).base)]",
        "bothBases" => "vec![Eval::Card(s.players[0].base), Eval::Card(s.players[1].base)]",
        "ownPlayer" => "vec![Eval::Player(s.card(s.source).owner)]",
        "enemyPlayer" => "vec![Eval::Player(s.card(s.source).owner.other())]",
        "bothPlayers" => "vec![Eval::Player(Seat::P1), Eval::Player(Seat::P2)]",
        "chosenPlayer" => "if s.ability_target!=0 && self.def(s,s.ability_target).info.is(\"base\") {vec![Eval::Player(s.card(s.ability_target).owner)]}else{vec![]}",
        _ => return Err(format!("target {n} requires interpreter")),
    }
    .into())
}
fn primitive(e: &Effect) -> Result<String, String> {
    if matches!(e,Effect::PayAdjustedResources{..}) {return Ok("let Instruction::Apply(effect)=&p.instructions[s.ability_pc as usize] else {return Err(\"generated payment identity mismatch\".into())};if !self.run_adjusted_resource_payment(s,effect)? {return Ok(true);}".into());}
    if matches!(e,Effect::ExhaustResources{..}|Effect::CreateCreditToken{..}|Effect::Damage{..}|Effect::Defeat{..}|Effect::Ready{..}|Effect::Exhaust{..}|Effect::PayAdjustedResources{..}) {return Ok("let Instruction::Apply(effect)=&p.instructions[s.ability_pc as usize] else {return Err(\"generated shared effect identity mismatch\".into())};let (amount,resolved)=self.apply_effect(s,effect)?;s.ability_result=amount;s.ability_succeeded=resolved;".into());}
    if matches!(e, Effect::NoAction) {
        return Ok("s.event_result=StepEventResult::default();s.ability_result=0; s.ability_succeeded=false;".into());
    }
    if let Effect::ModifyStats {
        power,
        hp,
        duration,
        target,
    } = e
    {
        if duration != "phase" {
            return Err("persistent stat operation uses modifier backend".into());
        }
        let power = number(power)?;
        let hp = number(hp)?;
        let targets = targets(target)?;
        return Ok(format!(
            r#"let delta=PhaseStatDelta{{power:{power},hp:{hp}}};let mut result=0u16;for target in {targets} {{if let Eval::Card(id)=target {{if id!=0 && self.in_arena(s,id) {{self.modify_phase_stats(s,id,delta)?;result+=1;}}}}}}self.finish_scalar_effect(s)?;s.event_result=StepEventResult::default();s.ability_result=result;s.ability_succeeded=result>0;"#
        ));
    }
    // Damage/Defeat/Ready/Exhaust return through apply_effect above.
    let (target, amount, operation) = match e {
        Effect::Draw { target, amount } => (target, number(amount)?,
            r#"if let Eval::Player(seat)=target {let draw=self.draw_cards(s,seat,amount)?;result=result.saturating_add(draw.drawn);resolved|=draw.resolved;}"#),
        Effect::Heal { target, amount } => (target, number(amount)?,
            r#"if let Eval::Card(id)=target {if id!=0 && (self.in_arena(s,id)||self.def(s,id).info.is("base")) {result=result.saturating_add(self.heal_card(s,id,amount));}}"#),
        _ => return Err("effect operation requires interpreter".into()),
    };
    Ok(format!("let raw_amount={amount};let amount=u16::try_from(raw_amount).map_err(|_|format!(\"effect amount {{raw_amount}} exceeds bounded runtime\"))?;let mut result=0u16;let mut resolved=false;for target in {} {{ {operation} }} self.finish_scalar_effect(s)?; s.event_result=StepEventResult::default();s.ability_result=result;s.ability_succeeded=result>0||resolved;",targets(target)?))
}
fn lower(p: &Program) -> Result<String, String> {
    if round_limit_contract::Registration::from_trigger(&p.trigger).is_some() {return Err("registration limit uses shared Rust interpreter".into());}
    if p.trigger == "constant" {
        return Err("constant registration uses live modifier backend".into());
    }
    let mut body = String::from("match s.ability_pc {\n");
    for (pc, i) in p.instructions.iter().enumerate() {
        let step=match i {
            Instruction::ChooseNumber|Instruction::PayUnadjustedSelectedResources|Instruction::GiveSelectedExperience=>"if self.run_number_instruction(s)? {return Ok(true);}".into(),
            Instruction::EndStep=>"s.ability_pc+=1;".into(),
            Instruction::Commit=>"if !self.commit_program(s)? {return Ok(true);}s.ability_pc+=1;".into(),
            Instruction::Select(_)=>format!("let Instruction::Select(r)=&p.instructions[{pc}] else {{return Err(\"AOT selection identity mismatch\".into())}};if self.program_targets(s,s.source,s.ability_program-1,{pc},r,true).is_empty() {{s.ability_target=0;s.ability_selection=Selection::default();s.ability_pc+=1;}} else {{s.decision=Decision::AbilityTarget;return Ok(true);}}"),
            Instruction::Conditional {skip,..}=>format!("let Instruction::Conditional{{condition,..}}=&p.instructions[{pc}] else {{return Err(\"AOT condition identity mismatch\".into())}};s.ability_pc=if self.eval(s,s.source,s.ability_target,condition).truth() {{{}}}else{{{skip}}};",pc+1),
            Instruction::Apply(effect @ Effect::SelectionSystem { .. })=>{
                crate::resource_exhaust::selection(effect)?;
                "let Instruction::Apply(effect)=&p.instructions[s.ability_pc as usize] else {return Err(\"generated nested selection identity mismatch\".into())};self.begin_selection_exhaust(s,effect)?;return Ok(true);".into()
            },
            Instruction::Apply(Effect::DistributeHealing{..})=>"if !self.commit_program(s)? {return Ok(true);}self.distribution_actions(s)?;s.decision=Decision::AbilityDistribute;return Ok(true);".into(),
            Instruction::IfResult {skip}=>format!("s.ability_pc=if s.ability_succeeded {{{}}}else{{{skip}}};",pc+1),
            Instruction::IfWillDefeat {skip}=>format!("s.ability_pc=if s.ability_succeeded&&s.event_result.will_defeat {{{}}}else{{{skip}}};",pc+1),
            Instruction::Jump {to}=>format!("s.ability_pc={to};"),
            Instruction::Apply(e)=>format!("if !self.commit_program(s)? {{return Ok(true);}}{}s.ability_pc+=1;",primitive(e)?),
            _=>return Err("instruction control/expression requires interpreter".into()),
        };
        body.push_str(&format!("{pc}=>{{{step}}},\n"));
    }
    body.push_str("_=>{self.end_program(s)?;return Ok(true);}} Ok(false)");
    Ok(body)
}
fn main() {
    println!("cargo:rerun-if-changed=src/executor_upgrade_count_contract.rs");
    println!("cargo:rerun-if-changed=src/variable_payment_contract.rs");
    println!("cargo:rerun-if-changed=src/modal_choices.rs");
    println!("cargo:rerun-if-changed=src/migration_modal_contract.rs");
    println!("cargo:rerun-if-changed=src/migration_modal_deck_contract.rs");
    println!("cargo:rerun-if-changed=src/named_play_contract.rs");
    println!("cargo:rerun-if-changed=src/named_play.rs");
    println!("cargo:rerun-if-changed=src/modal_arena_choices.rs");
    println!("cargo:rerun-if-changed=src/native_attack.rs");
    println!("cargo:rerun-if-changed=src/migration_attack_contract.rs");
    println!("cargo:rerun-if-changed=src/pay_adjusted_resources.rs");
    println!("cargo:rerun-if-changed=src/resource_exhaust.rs");
    println!("cargo:rerun-if-changed=src/cross_arena_contract.rs");
    println!("cargo:rerun-if-changed=src/recipient_heal_contract.rs");
    println!("cargo:rerun-if-changed=src/ambush_context_contract.rs");
    println!("cargo:rerun-if-changed=src/cinder_contract.rs");
    println!("cargo:rerun-if-changed=src/blade_base_damage_contract.rs");
    println!("cargo:rerun-if-changed=src/credits.rs");
    println!("cargo:rerun-if-changed=src/migration_credit_contract.rs");
    println!("cargo:rerun-if-changed=src/regroup_hooks.rs");
    println!("cargo:rerun-if-changed=src/migration_look_contract.rs");
    println!("cargo:rerun-if-changed=src/card_restrictions.rs");
    println!("cargo:rerun-if-env-changed=SWU_RULES_CATALOGUE");
    let catalog = env::var_os("SWU_RULES_CATALOGUE")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
                .join("../../compiler/generated")
        });
    println!("cargo:rerun-if-changed={}", catalog.display());
    if catalog.is_dir() {
        for name in ["manifest.json", "cards.json", "inputs.json", "diagnostics.json"] {
            println!("cargo:rerun-if-changed={}", catalog.join(name).display());
        }
    }
    println!("cargo:rerun-if-changed=src/ability_ir.rs");
    println!("cargo:rerun-if-changed=src/current_played_unit.rs");
    println!("cargo:rerun-if-changed=src/self_play_cost.rs");
    println!("cargo:rerun-if-changed=src/attachment_play_cost.rs");
    println!("cargo:rerun-if-changed=src/admission.rs");
    println!("cargo:rerun-if-changed=src/attached_registration.rs");
    println!("cargo:rerun-if-changed=src/piloting_attached.rs");
    println!("cargo:rerun-if-changed=src/attached_constants.rs");
    println!("cargo:rerun-if-changed=src/selector_contract.rs");
    println!("cargo:rerun-if-changed=src/static_ongoing.rs");
    println!("cargo:rerun-if-changed=src/phase_history.rs");
    println!("cargo:rerun-if-changed=src/migration_defeat_history_contract.rs");
    println!("cargo:rerun-if-changed=src/attack_history.rs");
    println!("cargo:rerun-if-changed=src/bounded_reductions.rs");
    println!("cargo:rerun-if-changed=src/live_power_maximum.rs");
    println!("cargo:rerun-if-changed=src/selected_player.rs");
    println!("cargo:rerun-if-changed=src/discard_return.rs");
    println!("cargo:rerun-if-changed=src/disclose.rs");
    println!("cargo:rerun-if-changed=src/catalogue_io.rs");
    let value = catalogue_io::load_catalogue_value(&catalog).expect("committed ability catalog missing");
    // Compile fixtures against the same authoritative shards without committing
    // a duplicate monolith or making tests depend on a process-local loader path.
    fs::write(
        PathBuf::from(env::var("OUT_DIR").unwrap()).join("catalogue.json"),
        serde_json::to_vec(&value).expect("serialize assembled catalogue"),
    ).expect("write assembled catalogue fixture");
    let c = Catalogue::load(&value).expect("invalid catalog schema");
    let mut roots = Vec::new();
    for mut program in c.programs.iter().cloned() {
        program.visit_expression_roots_mut(&mut |e| roots.push(serde_json::to_value(e).unwrap()));
    }
    roots.extend(c.cards.values().filter_map(|p|p.attach_condition.as_ref()).map(|e|serde_json::to_value(e).unwrap()));
    let expressions = expression_codegen::generate(roots);
    let mut programs = BTreeMap::new();
    for p in c.programs {
        let hash = format!("{:x}", Sha256::digest(serde_json::to_vec(&p).unwrap()));
        let bytes = serde_json::to_vec(&p).unwrap();
        programs.entry(hash).or_insert_with(|| (bytes, lower(&p)));
    }
    let mut source=String::from("pub(super) fn generated_backend(hash:&str,canonical:&[u8])->Result<usize,&'static str> {match hash {\n");
    let mut functions=String::from("impl Engine {#[allow(unused_mut,unused_variables,unused_parens)] fn generated_step(&self,s:&mut State,p:&Program,index:usize)->Result<bool,String> {match index {\n");
    let mut ready = 0;
    for (hash, (bytes, body)) in programs {
        let canonical = String::from_utf8(bytes).unwrap();
        match body {
            Ok(body) => {
                source.push_str(&format!(
                    "\"{hash}\" if canonical=={canonical:?}.as_bytes()=>Ok({ready}),\n"
                ));
                functions.push_str(&format!("{ready}=>{{{body}}},\n"));
                ready += 1;
            }
            Err(reason) => source.push_str(&format!(
                "\"{hash}\" if canonical=={canonical:?}.as_bytes()=>Err({reason:?}),\n"
            )),
        }
    }
    source.push_str("_=>Err(\"program content absent from build catalog\")}}\n");
    functions.push_str("_=>Err(\"invalid AOT executable index\".into())}}}\n");
    source.push_str(&functions);
    source.push_str(&expressions);
    fs::write(
        PathBuf::from(env::var("OUT_DIR").unwrap()).join("rules_aot.rs"),
        source,
    )
    .unwrap();
}
