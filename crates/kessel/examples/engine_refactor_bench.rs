//! Small native before/after measurements; no replay fixtures or dependencies.
//! Run the identical file with `cargo run --offline --release -p kessel
//! --example engine_refactor_bench`. Setup and semantic fingerprints are untimed.
use kessel::{ability_ir::Catalogue, model::*, Engine, RulesBackend};
use serde_json::{json, Value};
use std::{hint::black_box, time::Instant};

fn measure(name: &str, mut operation: impl FnMut()) {
    // Calibrate briefly, then measure about 100 ms. Each process is one sample;
    // alternate baseline/current processes externally to limit order bias.
    let mut batch = 1usize;
    let elapsed = loop {
        let start = Instant::now();
        for _ in 0..batch { operation(); }
        let elapsed = start.elapsed().as_secs_f64();
        if elapsed >= 0.005 || batch >= 16384 { break elapsed; }
        batch *= 4;
    };
    let iterations = ((0.1 / elapsed * batch as f64) as usize).clamp(1, 50000);
    let start = Instant::now();
    for _ in 0..iterations { operation(); }
    let seconds = start.elapsed().as_secs_f64();
    println!("{}", json!({"operation":name,"iterations":iterations,
        "seconds":seconds,"nsPerOperation":seconds*1e9/iterations as f64}));
}
fn add(e: &Engine, s: &mut State, seat: Seat, code: &str) -> CardId {
    let def = e.definitions.iter().position(|d| d.code == code).expect("admitted benchmark card") as u16;
    let id = s.next_id; s.next_id += 1;
    s.cards[id as usize] = Card { present:true,def,owner:seat,..Card::default() };
    id
}
fn unit(e: &Engine, s: &mut State, seat: Seat, code: &str) -> CardId {
    let id = add(e,s,seat,code);
    s.player_mut(seat).ground.push(id).unwrap();
    s.arena_unit_order[0].push(id).unwrap();
    s.registration_order.push(id).unwrap();
    id
}
fn board(e: &Engine) -> State {
    let mut s = State { hidden_history_exact:true,force_state_exact:true,..State::default() };
    for (seat,leader) in [(Seat::P1,"SOR_014"),(Seat::P2,"IBH_053")] {
        let base=add(e,&mut s,seat,"ASH_019"); let leader=add(e,&mut s,seat,leader);
        s.player_mut(seat).base=base; s.player_mut(seat).leader=leader;
        s.player_mut(seat).force_id=s.next_id; s.next_id+=1;
        let top=add(e,&mut s,seat,"LOF_254"); s.player_mut(seat).deck.push(top).unwrap();
    }
    s
}
fn click(e: &Engine, s: &State, id: CardId) -> Action {
    e.legal_actions(s).into_iter().find(|a|a.id()==Some(id)).expect("legal benchmark click")
}
fn event(e: &Engine, code: &str) -> (State,CardId,CardId) {
    let mut s=board(e);
    let target=unit(e,&mut s,Seat::P1,"LOF_254");
    unit(e,&mut s,Seat::P2,"LOF_256");
    let event=add(e,&mut s,Seat::P1,code); s.players[0].hand.push(event).unwrap();
    for _ in 0..8 {
        let resource=add(e,&mut s,Seat::P1,"LOF_254");s.players[0].resources.push(resource).unwrap();
    }
    (s,event,target)
}
fn fingerprint(name:&str,e:&Engine,s:&State) {
    println!("{}",json!({"fingerprint":name,"stateHash":s.zobrist(),
        "legalActions":e.legal_actions(s).iter().map(Action::key).collect::<Vec<_>>() }));
}
fn main() -> Result<(),String> {
    let catalogue:Value=serde_json::from_slice(include_bytes!("../../../compiler/generated/catalog.json")).unwrap();
    let mut e=Engine::from_catalogue_value(&catalogue)?;
    println!("{}",json!({"metadata":{"profile":"release","stateBytes":std::mem::size_of::<State>(),
        "definitions":e.definitions.len(),"rejected":e.rejected_cards().unwrap().len(),
        "executionUnit":"one public Engine::apply, including legality/state copy/cleanup",
        "queryUnit":"power and HP for twelve units","admissionUnit":"one parsed catalogue load"}}));
    // A whole-catalogue forced-AOT mode rejects unrelated fallback programs.
    // Auto dispatch uses AOT for these three verified generated programs.
    for (backend,label) in [(RulesBackend::Auto,"aot"),(RulesBackend::Interpreter,"interpreter")] {
        e.set_rules_backend(backend)?;
        if backend==RulesBackend::Auto {
            let report=e.rules_backend_report();
            for code in ["TWI_175","TS26_068","TWI_124"] {
                for index in report["cards"][code]["card"].as_array().unwrap() {
                    assert_eq!(report["programs"][index.as_u64().unwrap() as usize]["backend"],"aot");
                }
            }
            println!("{}",json!({"fingerprint":"generated_dispatch","cards":["TWI_175","TS26_068","TWI_124"],"backend":"aot"}));
        }
        for (code,name) in [("TWI_175","draw_partial_fatigue"),("TS26_068","prepared_two_player_draw")] {
            let (s,event,_)=event(&e,code);let action=click(&e,&s,event);
            let result=e.apply(&s,&action)?;
            assert_eq!(result.players[0].hand.len,1);
            assert_eq!(result.card(result.players[0].base).damage,if code=="TWI_175" {6}else{3});
            if code=="TS26_068" {assert_eq!(result.players[1].hand.len,1);assert_eq!(result.card(result.players[1].base).damage,3);}
            let name=format!("{name}_{label}");fingerprint(&name,&e,&result);
            measure(&name,||{black_box(e.apply(black_box(&s),black_box(&action)).unwrap());});
        }
        let (s,event,target)=event(&e,"TWI_124");
        let prompt=e.apply(&s,&click(&e,&s,event))?;assert_eq!(prompt.decision,Decision::AbilityTarget);
        let action=click(&e,&prompt,target);let result=e.apply(&prompt,&action)?;
        assert_eq!(result.card(target).phase_power,2);assert_eq!(result.card(target).phase_hp,2);
        let name=format!("phase_stats_{label}");fingerprint(&name,&e,&result);
        measure(&name,||{black_box(e.apply(black_box(&prompt),black_box(&action)).unwrap());});
    }
    let mut s=board(&e);let base=s.players[0].base;let leader=s.players[0].leader;
    s.cards[base as usize].def=e.definitions.iter().position(|d|d.code=="TWI_019").unwrap() as u16;
    s.cards[leader as usize].def=e.definitions.iter().position(|d|d.code=="HMW_007").unwrap() as u16;
    let mut targets=vec![];
    for seat in [Seat::P1,Seat::P2] {for _ in 0..6 {targets.push(unit(&e,&mut s,seat,"ASH_261"));}}
    for deployed in [false,true] {
        let mut s=s;s.card_mut(leader).deployed=deployed;
        if deployed {s.players[0].ground.push(leader)?;s.arena_unit_order[0].push(leader)?;s.registration_order.push(leader)?;}
        let name=if deployed {"live_stats_deployed"}else{"live_stats_front"};
        let values=targets.iter().map(|id|(e.power(&s,*id,true),e.hp(&s,*id))).collect::<Vec<_>>();
        println!("{}",json!({"fingerprint":name,"values":values}));
        measure(name,||{for id in &targets {black_box((e.power(black_box(&s),*id,true),e.hp(black_box(&s),*id)));}});
    }
    let codes=["TWI_175","TS26_068","TWI_124","LOF_217","TWI_019","HMW_007",
        "LOF_052","ASH_183","LAW_150","SOR_095","ASH_036","ASH_187","ASH_251","SHD_189","TS26_005","SEC_188"];
    let originals=catalogue["cards"].as_array().unwrap().iter().filter(|v|codes.contains(&v["setCode"].as_str().unwrap())).cloned().collect::<Vec<_>>();
    assert_eq!(originals.len(),codes.len());
    let mut entries=originals.clone();
    for mut entry in originals {
        entry["setCode"]=json!(format!("{}_BENCH_REJECT",entry["setCode"].as_str().unwrap()));
        entry["historyHelpers"]=json!(["unsupported benchmark helper"]);entries.push(entry);
    }
    let slice=json!({"schema":catalogue["schema"],"cards":entries});
    let admitted=Catalogue::load(&slice)?;
    println!("{}",json!({"fingerprint":"admission_contract_mix","admitted":admitted.cards.keys().collect::<std::collections::BTreeSet<_>>(),"rejected":admitted.rejected}));
    measure("admission_contract_mix",||{black_box(Catalogue::load(black_box(&slice)).unwrap());});
    measure("full_runtime_admission",||{black_box(Engine::from_catalogue_value(black_box(&catalogue)).unwrap());});
    Ok(())
}
