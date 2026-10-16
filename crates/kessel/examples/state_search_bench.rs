//! Bounded native measurements; no engine replay or external dependencies.
use kessel::{
    check,
    model::*,
    search::{Limits, Search},
    Engine, State,
};
use serde_json::{json, Value};
use std::{
    hint::black_box,
    io::BufRead,
    mem::{align_of, size_of},
    path::Path,
    time::Instant,
};

fn emit(value: Value) {
    println!("{value}");
}
#[inline(never)]
fn copy_position(output: &mut State, input: &State) {
    *black_box(output) = *black_box(input);
}
// Size-only control, not an older semantic State implementation.
#[inline(never)]
fn copy_size_control(output: &mut [u8; 20472], input: &[u8; 20472]) {
    *black_box(output) = *black_box(input);
}
fn copies(s: &State, iterations: usize, round: usize) {
    let mut positions = vec![*s; 64];
    let start = Instant::now();
    for i in 0..iterations {
        let (left, right) = positions.split_at_mut(32);
        copy_position(&mut left[i % 32], &right[i % 32]);
    }
    let elapsed = start.elapsed().as_secs_f64();
    black_box(&positions);
    emit(
        json!({"operation":"state_copy","round":round,"iterations":iterations,"bytesPerCopy":size_of::<State>(),"seconds":elapsed,"nsPerCopy":elapsed*1e9/iterations as f64,"gibPerSecond":iterations as f64*size_of::<State>() as f64/elapsed/1073741824.0,"checksum":positions[0].zobrist()}),
    );
    let mut controls = vec![[0x5au8; 20472]; 64];
    let start = Instant::now();
    for i in 0..iterations {
        let (left, right) = controls.split_at_mut(32);
        copy_size_control(&mut left[i % 32], &right[i % 32]);
    }
    let elapsed = start.elapsed().as_secs_f64();
    black_box(&controls);
    emit(
        json!({"operation":"size_only_copy_control","round":round,"iterations":iterations,"bytesPerCopy":20472,"seconds":elapsed,"nsPerCopy":elapsed*1e9/iterations as f64,"checksum":controls[0][0]}),
    );
}
fn recorded(
    e: &Engine,
    path: &Path,
    index: usize,
    empty_support_compat: bool,
) -> Result<State, String> {
    let mut count = 0;
    for row in check::open_jsonl(path)?.lines() {
        let row = row.map_err(|e| e.to_string())?;
        let mut v: Value = serde_json::from_str(&row).map_err(|e| e.to_string())?;
        if v["type"] == "state" {
            if count == index {
                if empty_support_compat && v["state"].get("supportStateVersion").is_none() {
                    v["state"]["supportStateVersion"] = json!(1);
                    v["state"]["supportGrants"] = json!([]);
                }
                return e.load_state(&v["state"]);
            }
            count += 1;
        }
    }
    Err(format!("missing state {index} in {}", path.display()))
}
fn add(e: &Engine, s: &mut State, seat: Seat, code: &str) -> CardId {
    let def = e
        .definitions
        .iter()
        .position(|d| d.code == code)
        .expect("benchmark card admitted") as u16;
    let id = s.next_id;
    s.next_id += 1;
    s.cards[id as usize] = Card {
        present: true,
        def,
        owner: seat,
        ..Card::default()
    };
    id
}
fn unit(e: &Engine, s: &mut State, seat: Seat, code: &str) -> CardId {
    let id = add(e, s, seat, code);
    s.player_mut(seat).ground.push(id).unwrap();
    s.arena_unit_order[0].push(id).unwrap();
    // These units are constructed in known physical entry order.
    s.registration_order.push(id).unwrap();
    id
}
fn click(e: &Engine, s: &State, id: CardId) -> State {
    let a = e
        .legal_actions(s)
        .into_iter()
        .find(|a| a.id() == Some(id))
        .expect("legal benchmark click");
    e.apply(s, &a).unwrap()
}
fn support(e: &Engine) -> Vec<(&'static str, State)> {
    let mut s = State {
        round: 6,
        hidden_history_exact: true,
        force_state_exact: true,
        ..State::default()
    };
    for (seat, leader) in [(Seat::P1, "SOR_014"), (Seat::P2, "IBH_053")] {
        let base = add(e, &mut s, seat, "ASH_019");
        let lead = add(e, &mut s, seat, leader);
        s.player_mut(seat).base = base;
        s.player_mut(seat).leader = lead;
        s.player_mut(seat).force_id = s.next_id;
        s.next_id += 1;
        for _ in 0..8 {
            let id = add(e, &mut s, seat, "LOF_254");
            s.player_mut(seat).deck.push(id).unwrap();
        }
    }
    let attacker = unit(e, &mut s, Seat::P1, "LOF_256");
    unit(e, &mut s, Seat::P1, "LOF_254");
    unit(e, &mut s, Seat::P2, "LOF_254");
    unit(e, &mut s, Seat::P2, "LOF_256");
    let donor = add(e, &mut s, Seat::P1, "ASH_154");
    s.players[0].hand.push(donor).unwrap();
    for _ in 0..10 {
        let id = add(e, &mut s, Seat::P1, "LOF_254");
        s.players[0].resources.push(id).unwrap();
    }
    let s = e.load_state(&e.snapshot(&s)).unwrap();
    let s = click(e, &s, donor);
    assert_eq!(s.decision, Decision::Support);
    let pending = click(e, &s, attacker);
    assert!(pending.support.pending);
    assert_eq!(pending.support.len, 1);
    let pass = e
        .legal_actions(&pending)
        .into_iter()
        .find(|a| a.label == "Pass attack")
        .unwrap();
    let retained = e.apply(&pending, &pass).unwrap();
    assert!(!retained.support.pending);
    assert_eq!(retained.support.len, 1);
    let roundtrip = e.load_state(&e.snapshot(&retained)).unwrap();
    assert_eq!(retained.zobrist(), roundtrip.zobrist());
    assert_eq!(e.legal_actions(&retained), e.legal_actions(&roundtrip));
    vec![
        ("support_pending_attack", pending),
        ("support_retained_grant", retained),
    ]
}
fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    let number = |name: &str, default: usize| -> Result<usize, String> {
        if let Some(i) = args.iter().position(|s| s == name) {
            args.get(i + 1)
                .ok_or_else(|| format!("missing {name}"))?
                .parse()
                .map_err(|_| format!("invalid {name}"))
        } else {
            Ok(default)
        }
    };
    let rounds = number("--rounds", 5)?;
    let nodes = number("--nodes", 2000)?;
    let iterations = number("--copy-iterations", 200000)?;
    if rounds > 32 || rounds == 0 || nodes > 20000 || nodes == 0 || iterations > 2000000 {
        return Err("bounded benchmark limits exceeded".into());
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    emit(
        json!({"operation":"layout","stateBytes":size_of::<State>(),"stateAlignment":align_of::<State>(),"supportStateBytes":size_of::<SupportState>(),"supportGrantBytes":size_of::<SupportGrant>(),"resolutionContextBytes":size_of::<ResolutionContext>(),"cardBytes":size_of::<Card>(),"supportGrantCapacity":MAX_SUPPORT_GRANTS,"supportProgramCapacity":MAX_SUPPORT_PROGRAMS,"sizeOnlyControlBytes":20472,"profile":"run with --release for timings","recordedCompiledCompatibility":"explicitEmptySupportStateV1"}),
    );
    let strict = Engine::from_pool(&root.join("data/pool.json"))?;
    let control = recorded(
        &strict,
        &root.join("tests/fixtures/heuristic-vs-heuristic-1.jsonl.gz"),
        30,
        false,
    )?;
    let catalogue = kessel::catalogue_io::load_catalogue_bytes(&root.join("../../compiler/generated"))
        .map_err(|e| e.to_string())?;
    let compiled = Engine::from_compiled_pool_bytes(
        &std::fs::read(root.join("tests/compiled-fixtures/pool.json"))
            .map_err(|e| e.to_string())?,
        &catalogue,
    )?;
    let ordinary = recorded(
        &compiled,
        &root.join("tests/compiled-fixtures/effect-interactions-4001.jsonl.gz"),
        30,
        true,
    )?;
    let full = Engine::from_catalogue_bytes(&catalogue)?;
    let supports = support(&full);
    let mut workloads = vec![
        ("strict_midgame", &strict, control),
        ("compiled_midgame", &compiled, ordinary),
    ];
    workloads.extend(supports.into_iter().map(|(name, s)| (name, &full, s)));
    for round in 0..=rounds {
        if iterations > 0 {
            copies(&workloads[3].2, iterations, round);
        }
        for i in 0..workloads.len() {
            let (name, e, s) = &workloads[(i + round) % workloads.len()];
            let mut search = Search::new(*e);
            let r = search.solve(
                *s,
                Limits {
                    depth: 6,
                    nodes: Some(nodes as u64),
                    time: None,
                    table_entries: 512,
                },
            )?;
            emit(
                json!({"operation":"search","workload":name,"round":round,"warmup":round==0,"stateHash":s.zobrist(),"supportGrants":s.support.len,"supportPending":s.support.pending,"legalActions":e.legal_actions(s).len(),"requestedNodes":nodes,"tableEntries":512,"result":r}),
            );
        }
    }
    Ok(())
}
