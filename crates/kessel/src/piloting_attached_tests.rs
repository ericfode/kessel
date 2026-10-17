//! Reviewed Pilot units that grant a host keyword while attached.
use super::*;
use serde_json::{json, Value};

fn attach(e: &Engine, s: &mut State, owner: Seat, code: &str, host: CardId) -> CardId {
    let id = add(e, s, owner, code);
    e.attach(s, id, host);
    id
}

#[test]
fn piloting_attached_keyword_admits_reviewed_pilot_sources() {
    let e = engine();
    for code in ["JTL_034", "JTL_045", "JTL_058", "JTL_211"] {
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
        ("/card/upgradePower", json!(9)),
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
