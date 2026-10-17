//! Reviewed Pilot units that grant a host keyword while attached.
use super::*;
use serde_json::{json, Value};

fn attach(e: &Engine, s: &mut State, owner: Seat, code: &str, host: CardId) -> CardId {
    let id = add(e, s, owner, code);
    e.attach(s, id, host);
    id
}

fn vehicle_with_trait(e: &Engine, s: &mut State, owner: Seat, trait_name: &str) -> CardId {
    let mut codes: Vec<_> = e
        .by_code
        .keys()
        .filter(|code| {
            let def = &e.definitions[e.by_code[*code] as usize];
            def.info.is("unit")
                && !def.info.is("leader")
                && !def.info.is("token")
                && def.programs.card.is_empty()
                && def.traits.iter().any(|t| t == trait_name)
                && !def.traits.iter().any(|t| {
                    t != trait_name && matches!(t.as_str(), "fighter" | "speeder" | "transport")
                })
                && def.info.upgrade_power.is_none()
                && def.flags & ((1 << 1) | (1 << 4)) == 0 // no printed overwhelm/grit
        })
        .cloned()
        .collect();
    codes.sort();
    let code = codes
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("missing ordinary {trait_name} vehicle"));
    unit(e, s, owner, &code)
}

#[test]
fn piloting_attached_keyword_admits_reviewed_pilot_sources() {
    let e = engine();
    for code in ["JTL_034", "JTL_045", "JTL_058", "JTL_211", "JTL_150"] {
        assert!(
            e.by_code.contains_key(code),
            "{code}: {:?}",
            e.rejected_cards().unwrap().get(code)
        );
    }
}

#[test]
fn piloting_attached_keyword_grants_host_keyword_and_upgrade_stats() {
    let e = engine();
    for (code, keyword) in [
        ("JTL_034", "grit"),
        ("JTL_045", "restore"),
        ("JTL_058", "sentinel"),
        ("JTL_211", "raid"),
    ] {
        for seat in [Seat::P1, Seat::P2] {
            let mut s = board(&e);
            s.active = seat;
            s.action_player = Some(seat);
            let host = unit(&e, &mut s, seat, "ASH_043");
            let printed = e.def(&s, host).info.power.unwrap_or(0) as u16;
            let printed_hp = e.def(&s, host).info.hp.unwrap_or(0) as u16;
            assert!(!e.has_keyword(&s, host, keyword), "{code} host baseline");
            let pilot = attach(&e, &mut s, seat, code, host);
            assert!(e.attached_in_play(&s, pilot));
            e.validate_attachment_links(&s).unwrap();
            assert!(e.has_keyword(&s, host, keyword), "{code} grants {keyword}");
            let up = e.def(&s, pilot).info.upgrade_power.unwrap() as u16;
            let uh = e.def(&s, pilot).info.upgrade_hp.unwrap() as u16;
            assert_eq!(e.power(&s, host, false), printed + up, "{code} upgrade power");
            assert_eq!(e.hp(&s, host), printed_hp + uh, "{code} upgrade hp");
            // Detach restores the host.
            e.detach(&mut s, pilot);
            assert!(!e.has_keyword(&s, host, keyword), "{code} clears on detach");
            assert_eq!(e.power(&s, host, false), printed);
        }
    }
}

#[test]
fn piloting_unit_play_keeps_intrinsic_keyword_without_host_grant() {
    let e = engine();
    for (code, keyword) in [("JTL_034", "grit"), ("JTL_058", "sentinel"), ("JTL_211", "raid")] {
        let mut s = board(&e);
        let pilot = unit(&e, &mut s, Seat::P1, code);
        assert!(e.has_keyword(&s, pilot, keyword), "{code} as unit");
        assert_eq!(s.card(pilot).parent, 0);
    }
}

#[test]
fn piloting_attached_conditional_grants_follow_host_traits() {
    let e = engine();
    assert!(e.by_code.contains_key("JTL_150"), "{:?}", e.rejected_cards().unwrap().get("JTL_150"));
    for seat in [Seat::P1, Seat::P2] {
        let mut s = board(&e);
        s.active = seat;
        s.action_player = Some(seat);
        let fighter = vehicle_with_trait(&e, &mut s, seat, "fighter");
        let transport = vehicle_with_trait(&e, &mut s, seat, "transport");
        let speeder = vehicle_with_trait(&e, &mut s, seat, "speeder");
        let fighter_hp = e.hp(&s, fighter);
        let transport_hp = e.hp(&s, transport);
        let speeder_hp = e.hp(&s, speeder);
        assert!(!e.has_keyword(&s, fighter, "overwhelm"));
        assert!(!e.has_keyword(&s, speeder, "grit"));
        let on_fighter = attach(&e, &mut s, seat, "JTL_150", fighter);
        let upgrade_hp = e.def(&s, on_fighter).info.upgrade_hp.unwrap() as u16;
        assert!(e.has_keyword(&s, fighter, "overwhelm"), "fighter gains Overwhelm");
        assert!(!e.has_keyword(&s, fighter, "grit"));
        assert_eq!(e.hp(&s, fighter), fighter_hp + upgrade_hp);
        e.detach(&mut s, on_fighter);
        let on_transport = attach(&e, &mut s, seat, "JTL_150", transport);
        assert!(!e.has_keyword(&s, transport, "overwhelm"));
        assert!(!e.has_keyword(&s, transport, "grit"));
        assert_eq!(e.hp(&s, transport), transport_hp + upgrade_hp + 1, "transport +0/+1");
        e.detach(&mut s, on_transport);
        let on_speeder = attach(&e, &mut s, seat, "JTL_150", speeder);
        assert!(e.has_keyword(&s, speeder, "grit"), "speeder gains Grit");
        assert!(!e.has_keyword(&s, speeder, "overwhelm"));
        assert_eq!(e.hp(&s, speeder), speeder_hp + upgrade_hp);
        e.detach(&mut s, on_speeder);
        assert!(!e.has_keyword(&s, speeder, "grit"));
    }
}

#[test]
fn piloting_attached_unit_play_skips_parent_trait_conditions() {
    let e = engine();
    let mut s = board(&e);
    let pilot = unit(&e, &mut s, Seat::P1, "JTL_150");
    assert!(!e.has_keyword(&s, pilot, "overwhelm"));
    assert!(!e.has_keyword(&s, pilot, "grit"));
    assert_eq!(s.card(pilot).parent, 0);
}

#[test]
fn piloting_attached_keyword_rejects_partial_shapes() {
    let catalog: Value =
        serde_json::from_slice(include_bytes!("../../../compiler/generated/catalog.json")).unwrap();
    let original = catalog["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["setCode"] == "JTL_034")
        .unwrap()
        .clone();
    let admitted = |e: Value| {
        crate::ability_ir::Catalogue::load(&json!({"schema": crate::ability_ir::SCHEMA, "cards": [e]}))
            .unwrap()
            .cards
            .contains_key("JTL_034")
    };
    assert!(admitted(original.clone()));
    for (path, value) in [
        ("/baseClass", json!("UpgradeCard")),
        ("/card/types", json!(["upgrade"])),
        ("/card/pilotText", json!(null)),
        ("/card/keywords", json!(["grit"])),
        ("/keywordValues", json!({"grit": 1, "raid": 1})),
        ("/abilities/card/0/effect/kind", json!("gainKeyword")),
        (
            "/abilities/card/0/effect/modifier/keyword",
            json!("ambush"),
        ),
        ("/abilities/card/0/effect/requiresGrantorInPlay", json!(false)),
        ("/card/upgradePower", json!(1001)),
        (
            "/abilities/card/0/effect/gainCondition",
            json!({"kind": "const", "value": true}),
        ),
    ] {
        let mut bad = original.clone();
        let parts: Vec<_> = path.split('/').skip(1).collect();
        let mut target = &mut bad;
        for k in &parts[..parts.len() - 1] {
            target = if target.is_array() {
                &mut target[k.parse::<usize>().unwrap()]
            } else {
                &mut target[*k]
            };
        }
        target[parts[parts.len() - 1]] = value;
        assert!(!admitted(bad), "{path}");
    }
}
