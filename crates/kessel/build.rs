//! Compile validated catalog programs into executable Rust, keyed by content.
#[path = "src/admission.rs"]
mod admission;
#[allow(dead_code)]
#[path = "src/native_attack.rs"]
mod native_attack;

#[path = "src/death_hooks.rs"]
mod death_hooks;
#[allow(dead_code)]
#[path = "src/bounty.rs"]
mod bounty;
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
#[path = "src/phase_history.rs"]
mod phase_history;
#[allow(dead_code)]
#[path = "src/attack_history.rs"]
mod attack_history;
#[allow(dead_code)]
#[path = "src/attached_registration.rs"]
mod attached_registration;

#[path="src/advantage.rs"]
mod advantage;

#[allow(dead_code)]
#[path = "src/attached_constants.rs"]
mod attached_constants;

#[path = "src/deck_search.rs"]
mod deck_search;
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
        // "Own"/"enemy" is relative to the ability's context player, which can differ
        // from the source card's controller (Bounty is collected by the opponent of
        // the defeated unit's controller). `context_owner` is the interpreter's single
        // owner of that decision, so the generated backend cannot drift from it.
        "ownBase" => "vec![Eval::Card(s.player(self.context_owner(s)).base)]",
        "enemyBase" => "vec![Eval::Card(s.player(self.context_owner(s).other()).base)]",
        "bothBases" => "vec![Eval::Card(s.players[0].base), Eval::Card(s.players[1].base)]",
        "ownPlayer" => "vec![Eval::Player(self.context_owner(s))]",
        "enemyPlayer" => "vec![Eval::Player(self.context_owner(s).other())]",
        "bothPlayers" => "vec![Eval::Player(Seat::P1), Eval::Player(Seat::P2)]",
        _ => return Err(format!("target {n} requires interpreter")),
    }
    .into())
}
fn primitive(e: &Effect) -> Result<String, String> {
    if matches!(e,Effect::Damage{..}|Effect::Defeat{..}|Effect::Ready{..}|Effect::Exhaust{..}|Effect::PayAdjustedResources{..}) {return Ok("let Instruction::Apply(effect)=&p.instructions[s.ability_pc as usize] else {return Err(\"generated shared effect identity mismatch\".into())};let (amount,resolved)=self.apply_effect(s,effect)?;s.ability_result=amount;s.ability_succeeded=resolved;".into());}
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
    if p.trigger == "constant" {
        return Err("constant registration uses live modifier backend".into());
    }
    let mut body = String::from("match s.ability_pc {\n");
    for (pc, i) in p.instructions.iter().enumerate() {
        let step=match i {
            Instruction::EndStep=>"s.ability_pc+=1;".into(),
            Instruction::Commit=>"if !self.commit_program(s,true)? {return Ok(true);}s.ability_pc+=1;".into(),
            Instruction::Select(_)=>format!("let Instruction::Select(r)=&p.instructions[{pc}] else {{return Err(\"AOT selection identity mismatch\".into())}};if self.program_targets(s,s.source,s.ability_program-1,{pc},r,true).is_empty() {{s.ability_target=0;s.ability_selection=Selection::default();s.ability_pc+=1;}} else {{s.decision=Decision::AbilityTarget;return Ok(true);}}"),
            Instruction::Conditional {skip,..}=>format!("let Instruction::Conditional{{condition,..}}=&p.instructions[{pc}] else {{return Err(\"AOT condition identity mismatch\".into())}};s.ability_pc=if self.eval(s,s.source,s.ability_target,condition).truth() {{{}}}else{{{skip}}};",pc+1),
            Instruction::Apply(Effect::DistributeHealing{..})=>"s.decision=Decision::AbilityDistribute;return Ok(true);".into(),
            Instruction::IfResult {skip}=>format!("s.ability_pc=if s.ability_succeeded {{{}}}else{{{skip}}};",pc+1),
            Instruction::Jump {to}=>format!("s.ability_pc={to};"),
            Instruction::Apply(e)=>format!("if !self.commit_program(s,true)? {{return Ok(true);}}{}s.ability_pc+=1;",primitive(e)?),
            _=>return Err("instruction control/expression requires interpreter".into()),
        };
        body.push_str(&format!("{pc}=>{{{step}}},\n"));
    }
    body.push_str("_=>{self.end_program(s)?;return Ok(true);}} Ok(false)");
    Ok(body)
}
fn main() {
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
    println!("cargo:rerun-if-changed=src/bounty.rs");
    println!("cargo:rerun-if-changed=src/admission.rs");
    println!("cargo:rerun-if-changed=src/attached_registration.rs");
    println!("cargo:rerun-if-changed=src/attached_constants.rs");
    println!("cargo:rerun-if-changed=src/selector_contract.rs");
    println!("cargo:rerun-if-changed=src/static_ongoing.rs");
    println!("cargo:rerun-if-changed=src/phase_history.rs");
    println!("cargo:rerun-if-changed=src/attack_history.rs");
    println!("cargo:rerun-if-changed=src/bounded_reductions.rs");
    println!("cargo:rerun-if-changed=src/live_power_maximum.rs");
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
