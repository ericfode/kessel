//! Narrow bypass guards, not a complexity score or a whole-program dependency proof.
//! Run locally/cloud/CI: cargo test -p kessel --test architecture_boundaries
use std::{fs, path::Path};
use syn::{
    visit::{self, Visit},
    Item, Type, UseTree,
};

// These source-only envelope contracts also participate in the build-time loader.
// Keep their rules independent of runtime position/engine state. Other modules
// have different responsibilities; this is deliberately not a global import ban.
const SOURCE_CONTRACTS: &[&str] = &[
    "src/captured_hand_cost_contract.rs",
    "src/named_exhaust_contingent_contract.rs",
    "src/round_limit_contract.rs",
    "src/exhausted_live_count_contract.rs",
    "src/defeat_token_creation_contract.rs",
    "src/defeat_cause_contract.rs",
    "src/executor_upgrade_count_contract.rs",
    "src/variable_payment_contract.rs",
    "src/arena_defeat_attack_contract.rs",
    "src/unit_entry_contract.rs",
    "src/ambush_context_contract.rs",
    "src/piloting_lifecycle.rs",
    "src/cinder_contract.rs",
    "src/blade_base_damage_contract.rs",
    "src/collective_trigger_contract.rs",
    "src/distributed_damage_contract.rs",
    "src/chosen_hand_discard_contract.rs",
    "src/admission.rs",
    "src/activation_contract.rs",
    "src/selector_contract.rs",
    "src/named_play_contract.rs",
    "src/regroup_hooks.rs",
];

#[derive(Default)]
struct ContractDependencies(Vec<String>);

impl ContractDependencies {
    fn path(&mut self, parts: &[String]) {
        let first = parts
            .iter()
            .find(|p| !matches!(p.as_str(), "crate" | "self" | "super"));
        if first.is_some_and(|p| {
            matches!(
                p.as_str(),
                "simulator"
                    | "model"
                    | "search"
                    | "check"
                    | "validation"
                    | "Engine"
                    | "State"
                    | "PreparedEvent"
            )
        }) {
            self.0.push(format!(
                "source contract must not depend on execution/state: {}",
                parts.join("::")
            ));
        }
    }

    fn imports(&mut self, tree: &UseTree, prefix: &[String]) {
        let mut parts = prefix.to_vec();
        match tree {
            UseTree::Path(p) => {
                parts.push(p.ident.to_string());
                self.imports(&p.tree, &parts);
            }
            UseTree::Name(n) => {
                parts.push(n.ident.to_string());
                self.path(&parts);
            }
            UseTree::Rename(r) => {
                parts.push(r.ident.to_string());
                self.path(&parts);
            }
            UseTree::Group(g) => {
                for tree in &g.items {
                    self.imports(tree, prefix);
                }
            }
            UseTree::Glob(_) => self.path(prefix),
        }
    }
}

impl<'ast> Visit<'ast> for ContractDependencies {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        self.path(
            &path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>(),
        );
        visit::visit_path(self, path);
    }

    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        self.imports(&item.tree, &[]);
    }
}

#[derive(Default)]
struct ResolvedInputs(Vec<String>);

fn selects_or_evaluates(name: &str) -> bool {
    name == "eval"
        || name.starts_with("eval_")
        || matches!(name, "effect_amount" | "effect_targets")
}

impl<'ast> Visit<'ast> for ResolvedInputs {
    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        let name = call.method.to_string();
        if selects_or_evaluates(&name) {
            self.0.push(format!(
                "resolved mutation must receive values/recipients from its caller: {name}"
            ));
        }
        visit::visit_expr_method_call(self, call);
    }

    fn visit_path(&mut self, path: &'ast syn::Path) {
        if let Some(last) = path.segments.last() {
            let name = last.ident.to_string();
            if name == "PreparedEvent" {
                self.0
                    .push("resolved mutation must not interpret PreparedEvent storage".into());
            } else if selects_or_evaluates(&name) {
                // Also catch qualified calls and taking an evaluator method as a value.
                self.0.push(format!(
                    "resolved mutation must receive values/recipients from its caller: {name}"
                ));
            }
        }
        visit::visit_path(self, path);
    }
}

fn contract_violations(source: &str) -> Vec<String> {
    let ast = syn::parse_file(source).expect("source contract parses as Rust");
    let mut check = ContractDependencies::default();
    check.visit_file(&ast);
    check.0
}

fn mutation_violations(source: &str) -> Vec<String> {
    let ast = syn::parse_file(source).expect("runtime parses as Rust");
    let methods = ast.items.iter().filter_map(|item| match item {
        Item::Impl(i) if matches!(i.self_ty.as_ref(), Type::Path(t) if t.path.is_ident("Engine")) => Some(i),
        _ => None,
    }).flat_map(|i| &i.items).filter_map(|item| match item {
        syn::ImplItem::Fn(f) if matches!(f.sig.ident.to_string().as_str(),"apply_scalar_effect"|"apply_scalar_effect_observed") => Some(f),
        _ => None,
    }).collect::<Vec<_>>();
    assert_eq!(
        methods.len(),
        2,
        "update the boundary test deliberately if the mutation owner moves"
    );
    let mut check = ResolvedInputs::default();
    for method in methods {check.visit_impl_item_fn(method);}
    check.0
}

#[test]
fn source_contracts_do_not_depend_on_execution_state() {
    for relative in SOURCE_CONTRACTS {
        let source =
            fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)).unwrap();
        assert_eq!(
            contract_violations(&source),
            Vec::<String>::new(),
            "{relative}"
        );
    }
}

#[test]
fn resolved_scalar_mutation_does_not_select_evaluate_or_decode_records() {
    let source = include_str!("../src/generic_runtime.rs");
    assert_eq!(mutation_violations(source), Vec::<String>::new());
    // Exercise the checker against the real method, without changing repo files.
    let needle = "let mut events=StepEventResult::for_state(s);";
    assert!(source.contains(needle));
    for bypass in [
        "let _ = self.effect_targets(s, &crate::ability_ir::Target::name(\"chosen\"));",
        "let _ = Self::effect_amount(self, s, &crate::ability_ir::Expr::number(1));",
        "let evaluator = Self::eval_with_binding;",
        "let record: crate::model::PreparedEvent = Default::default();",
    ] {
        let altered = source.replace(needle, &format!("{bypass}{needle}"));
        assert!(!mutation_violations(&altered).is_empty(), "missed {bypass}");
    }
}

#[test]
fn source_dependency_examples_cover_groups_aliases_and_direct_paths() {
    for source in [
        "use crate::simulator::Engine;",
        "use crate::{simulator::{Engine as Executor}, ability_ir::Expr};",
        "use crate::model as position;",
        "fn gate(s: &crate::State) {}",
        "fn gate(s: &super::super::model::State) {}",
    ] {
        assert!(!contract_violations(source).is_empty(), "missed {source}");
    }
    let allowed = r#"
        use crate::ability_ir::{Ability, Expr};
        // use crate::simulator::Engine;
        const EXAMPLE: &str = "crate::model::State";
        fn gate(a: &Ability) -> bool { a.condition.is_none() }
    "#;
    assert!(contract_violations(allowed).is_empty());
}

#[test]
fn mutation_examples_allow_operations_and_ignore_comments_and_strings() {
    let source = r#"
        impl Engine {
            fn apply_scalar_effect_observed(&self, s: &mut State, amount: u16) {}
            fn apply_scalar_effect(&self, s: &mut State, amount: u16) {
                // self.eval(s, expr); PreparedEvent
                let example = "self.effect_targets(s, target)";
                self.heal_card(s, 1, amount);
                self.modify_phase_stats(s, 1, delta);
            }
            fn capture_elsewhere(&self) { self.eval(s, expr); }
        }
    "#;
    assert!(mutation_violations(source).is_empty());
}
