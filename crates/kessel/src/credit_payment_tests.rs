//! Source-derived tests for pure Credit storage, menu math, and provenance.
//! Engine cost windows and snapshot integration require separate runtime fixtures.
use crate::credits::{
    creation_source, native_entry, CreditList, MenuChoice, PaymentKind, PaymentPlan,
};
use serde_json::{json, Value};

fn catalog() -> Value {
    serde_json::from_slice(include_bytes!("../../../compiler/generated/catalog.json")).unwrap()
}
fn entry(code: &str) -> Value {
    catalog()["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == code)
        .unwrap()
        .clone()
}
fn list(ids: &[u16]) -> CreditList {
    let mut out = CreditList::default();
    for id in ids {
        out.append(*id).unwrap();
    }
    out
}

#[test]
fn credits_keep_physical_order_across_transfer_defeat_and_fresh_creation() {
    let mut donor = list(&[9, 2, 14]);
    let mut recipient = list(&[20, 6]);
    assert_eq!(donor.prefix(2).unwrap(), &[9, 2]);
    assert_eq!(donor.transfer_prefix_to(&mut recipient, 2).unwrap(), 2);
    assert_eq!(donor.as_slice(), &[14]);
    assert_eq!(recipient.as_slice(), &[20, 6, 9, 2]);
    assert_eq!(recipient.first(), Some(20));
    recipient.remove(6).unwrap();
    recipient.remove(20).unwrap();
    assert_eq!(recipient.as_slice(), &[9, 2]);
    recipient.append(21).unwrap();
    assert_eq!(recipient.as_slice(), &[9, 2, 21]);
    assert_eq!(donor.transfer_prefix_to(&mut recipient, 10).unwrap(), 1);
    assert_eq!(recipient.as_slice(), &[9, 2, 21, 14]);
    assert!(donor.is_empty());
}

#[test]
fn credits_check_identity_conversion_capacity_and_atomic_transfer() {
    let mut credits = CreditList::default();
    for id in 1..=255 {
        credits.append(id).unwrap();
    }
    assert_eq!(credits.len(), 255);
    assert!(credits.contains(255));
    let before = credits;
    for id in [0, 256, u16::MAX, 1] {
        assert!(credits.append(id).is_err());
        assert_eq!(credits, before);
    }
    assert!(credits.prefix(256).is_err());
    assert!(credits.remove(0).is_err());
    assert_eq!(credits, before);
    let mut donor = list(&[3, 4]);
    let mut recipient = list(&[9, 4]);
    let before = (donor, recipient);
    assert!(donor.transfer_prefix_to(&mut recipient, 2).is_err());
    assert_eq!((donor, recipient), before);
    assert!(!credits.contains(256));
}

#[test]
fn empty_credits_are_hash_neutral_but_identity_and_order_are_not() {
    let hash = |credits: CreditList| {
        let mut values = vec![];
        credits.hash_values(123, &mut |value| values.push(value));
        values
    };
    assert!(hash(CreditList::default()).is_empty());
    assert_ne!(hash(list(&[2, 3])), hash(list(&[3, 2])));
    assert_ne!(hash(list(&[2, 3])), hash(list(&[2, 4])));
    let mut credits = list(&[2]);
    credits.remove(2).unwrap();
    assert_eq!(credits, CreditList::default());
}

#[test]
fn credit_menu_matches_minimum_maximum_skip_and_cancellation_branches() {
    for (cost, ready, credits, cancel, expected) in [
        (3, 2, 1, true, vec![MenuChoice::Use(1), MenuChoice::Cancel]),
        (3, 3, 1, false, vec![MenuChoice::Use(1), MenuChoice::Skip]),
        (3, 1, 2, false, vec![MenuChoice::Use(2)]),
        (
            4,
            2,
            3,
            true,
            vec![MenuChoice::SelectAmount, MenuChoice::Cancel],
        ),
        (
            2,
            2,
            4,
            true,
            vec![
                MenuChoice::SelectAmount,
                MenuChoice::Skip,
                MenuChoice::Cancel,
            ],
        ),
    ] {
        let plan = PaymentPlan::new(cost, ready, credits, false, cancel)
            .unwrap()
            .unwrap();
        assert_eq!(plan.choices(), expected);
        assert_eq!(plan.minimum, cost.saturating_sub(ready));
        assert_eq!(plan.maximum, credits.min(cost));
        assert!(plan.validate_amount(0).is_err());
        assert!(plan.validate_amount(plan.maximum + 1).is_err());
        assert!(plan.validate_amount(plan.maximum).is_ok());
    }
    assert_eq!(PaymentPlan::new(0, 0, 5, false, true).unwrap(), None);
    assert_eq!(PaymentPlan::new(3, 3, 0, false, true).unwrap(), None);
    assert!(PaymentPlan::new(4, 1, 2, false, true).is_err());
}

#[test]
fn credit_number_range_and_exact_prompt_text_preserve_payment_kind() {
    assert_eq!(
        PaymentKind::PlayCard.prompt_title("Windfall"),
        "Use Credit tokens to pay for Windfall"
    );
    assert_eq!(
        PaymentKind::Ability.prompt_title("Lady Proxima"),
        "Use Credit tokens to pay for Lady Proxima's ability"
    );
    assert_eq!(
        PaymentKind::CardEffect.prompt_title("Criminal Contact"),
        "Use Credit tokens to pay for Criminal Contact's effect"
    );
    assert_eq!(
        crate::credits::NUMBER_PROMPT_TITLE,
        "Select amount of Credit tokens"
    );
    assert_eq!(MenuChoice::Use(1).label(), "Use 1 Credit");
    assert_eq!(MenuChoice::Use(2).label(), "Use 2 Credits");
    assert_eq!(MenuChoice::SelectAmount.label(), "Select amount");
    assert_eq!(MenuChoice::Skip.label(), "Pay costs without Credit tokens");
    assert_eq!(MenuChoice::Cancel.label(), "Cancel");
    let optional = PaymentPlan::new(5, 5, 4, false, true).unwrap().unwrap();
    assert_eq!(
        optional.number_range().unwrap().collect::<Vec<_>>(),
        vec![1, 2, 3, 4]
    );
    let required = PaymentPlan::new(5, 2, 4, false, false).unwrap().unwrap();
    assert_eq!(
        required.number_range().unwrap().collect::<Vec<_>>(),
        vec![3, 4]
    );
    let fixed = PaymentPlan::new(5, 2, 3, false, false).unwrap().unwrap();
    assert_eq!(fixed.number_range(), None);
}

#[test]
fn first_credit_blanking_disables_the_entire_adjuster() {
    assert_eq!(PaymentPlan::available(5, true), 0);
    assert_eq!(PaymentPlan::available(5, false), 5);
    assert_eq!(PaymentPlan::new(2, 2, 5, true, true).unwrap(), None);
    assert!(PaymentPlan::new(2, 1, 5, true, true).is_err());
    // All-Credit payment remains payable with zero resources. Runtime success
    // follows the exhaust-resource EVENT, even though its numerical amount is 0.
    let plan = PaymentPlan::new(2, 0, 2, false, false).unwrap().unwrap();
    assert_eq!(plan.choices(), vec![MenuChoice::Use(2)]);
}

#[test]
fn credit_native_contract_rejects_metadata_source_and_context_mutations() {
    let original = entry("TOKEN_CREDIT");
    assert!(native_entry(&original));
    for (field, value) in [
        ("sourceHash", json!("changed")),
        ("baseClass", json!("NonLeaderUnitCard")),
        ("className", json!("ForceToken")),
        ("stateWatchers", json!([{}])),
        ("inheritedSource", json!([{}])),
        ("piloting", json!({})),
    ] {
        let mut changed = original.clone();
        changed[field] = value;
        assert!(!native_entry(&changed), "{field}");
    }
    for (field, value) in [
        ("types", json!(["token", "upgrade"])),
        ("traits", json!(["innate"])),
        ("cost", json!(1)),
        ("unique", json!(true)),
        ("upgradePower", json!(0)),
    ] {
        let mut changed = original.clone();
        changed["card"][field] = value;
        assert!(!native_entry(&changed), "{field}");
    }
}

#[test]
fn credit_initial_creation_contracts_are_complete_and_reusable() {
    for (code, amount, event) in [
        ("LAW_232", 1, false),
        ("LAW_262", 1, false),
        ("LAW_244", 1, true),
        ("LAW_248", 3, true),
        ("LAW_235", 1, false),
        ("LAW_121", 1, false),
    ] {
        let original = entry(code);
        assert_eq!(
            creation_source(&original).unwrap(),
            Some(crate::credits::Creation { amount, event })
        );
        for (field, value) in [
            ("stateWatchers", json!([{}])),
            ("inheritedSource", json!([{}])),
            ("piloting", json!({})),
            ("attachCondition", json!({})),
        ] {
            let mut changed = original.clone();
            changed[field] = value;
            assert!(creation_source(&changed).is_err(), "{code}/{field}");
        }
        for (field, value) in [
            ("optional", json!(true)),
            ("coordinate", json!(true)),
            ("attachedContext", json!("parent")),
            ("gainedHost", json!(false)),
            (
                "then",
                json!({"title":"extra","effect":{"kind":"noAction"}}),
            ),
        ] {
            let mut changed = original.clone();
            changed["abilities"]["card"][0][field] = value;
            assert!(creation_source(&changed).is_err(), "{code}/{field}");
        }
        let mut changed = original.clone();
        changed["abilities"]["card"][0]["effect"]["target"] = json!("enemyPlayer");
        assert!(creation_source(&changed).is_err());
        let mut changed = original.clone();
        changed["card"]["text"] = json!("Create a Credit token. Then draw a card.");
        assert!(creation_source(&changed).is_err());
        let mut changed = original.clone();
        changed["abilities"]["card"]
            .as_array_mut()
            .unwrap()
            .push(original["abilities"]["card"][0].clone());
        assert!(creation_source(&changed).is_err());
    }
    for code in [
        "LAW_116", "LAW_247",
    ] {
        assert!(
            creation_source(&entry(code)).is_err(),
            "{code} needs its complete extension"
        );
    }
    assert_eq!(creation_source(&entry("TWI_212")).unwrap(), None);
}
