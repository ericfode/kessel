use super::*;
fn floor_expr(value:Value)->crate::ability_ir::Expr {serde_json::from_value(value).unwrap()}
fn expr(divisor:Value)->Value {json!({"kind":"math","name":"floor","args":[{"kind":"binary","op":"/","left":{"kind":"ref","path":["player","base","damage"]},"right":{"kind":"const","value":divisor}}]})}
fn midgame(e:&Engine,seat:Seat)->State {let mut s=board(e);s.round=10;s.active=seat;s.action_player=Some(seat);s}
fn roundtrip(e:&Engine,s:&State)->State {let v=e.snapshot(s);let r=e.load_state(&v).unwrap();assert_eq!(e.snapshot(&r),v);assert_eq!(e.legal_actions(&r),e.legal_actions(s));assert_eq!(r.zobrist(),s.zobrist());r}
#[test]
fn bounded_floor_base_damage_integer_domain_and_raw_ir_rejections() {
    let e=engine();let mut s=midgame(&e,Seat::P1);let source=unit(&e,&mut s,Seat::P1,"ASH_179");let base=s.players[0].base;
    // Includes raw numeric capacity diagnostics beyond ordinary live base HP.
    for damage in [0,1,4,5,9,10,29,65535] {s.card_mut(base).damage=damage;for divisor in [1,5,65535] {let x=floor_expr(expr(json!(divisor)));x.validate_number().unwrap();assert_eq!(e.eval(&s,source,0,&x).number(),i64::from(damage)/divisor);}}
    for divisor in [json!(0),json!(-1),json!(0.5),json!(65536),json!("5")] {assert!(floor_expr(expr(divisor)).validate().is_err());}
    for mutation in ["naked","other-ref","dynamic-divisor","many-args","empty-args","nondivision","nested-floor"] {
        let mut x=expr(json!(5));match mutation {
            "naked"=>x=x["args"][0].clone(),
            "other-ref"=>x["args"][0]["left"]["path"]=json!(["source","damage"]),
            "dynamic-divisor"=>x["args"][0]["right"]=json!({"kind":"ref","path":["player","base","damage"]}),
            "many-args"=>x["args"].as_array_mut().unwrap().push(json!({"kind":"const","value":1})),
            "empty-args"=>x["args"]=json!([]),
            "nondivision"=>x["args"][0]["op"]=json!("*"),
            _=>x=json!({"kind":"math","name":"floor","args":[x]}),
        }assert!(floor_expr(x).validate().is_err(),"{mutation}");
    }
}
#[test]
fn bounded_floor_rancor_attack_amount_live_title_and_compiled_expression_match_interpreter() {
    let mut observations=vec![];
    for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {
        let mut e=engine();e.set_rules_backend(backend).unwrap();e.set_rules_instrumentation(true);let mut results=vec![];
        for seat in [Seat::P1,Seat::P2] {for damage in [0,4,5,9,10,24] {
            let mut s=midgame(&e,seat);let source=unit(&e,&mut s,seat,"ASH_179");let own=s.player(seat).base;let enemy=s.player(seat.other()).base;s.card_mut(own).damage=damage;
            let p=e.def(&s,source).programs.card[1];assert_eq!(e.program_menu_title(&s,source,p),format!("Deal {} damage to a base",damage/5));
            let amount=e.program(p).context_title[1].clone();assert!(matches!(amount,crate::ability_ir::Expr::Compiled(_)));
            *e.expression_hits.lock().unwrap()=[0,0];assert_eq!(e.eval(&s,source,0,&amount).number(),i64::from(damage/5));let hits=*e.expression_hits.lock().unwrap();assert!(hits[usize::from(backend==RulesBackend::Interpreter)]>0);
            let s=roundtrip(&e,&s);let choosing=e.apply(&s,&e.card_action(&s,source)).unwrap();let mut done=e.apply(&choosing,&e.card_action(&choosing,enemy)).unwrap();
            if damage>=5 {assert_eq!(done.decision,Decision::AbilityTarget);done=e.apply(&done,&e.card_action(&done,enemy)).unwrap();}
            assert_eq!(done.card(enemy).damage,8+damage/5);assert_eq!(done.card(own).damage,damage);let done=roundtrip(&e,&done);results.push((e.snapshot(&done),e.legal_actions(&done),done.zobrist()));
        }}observations.push(results);
    }assert_eq!(observations[0],observations[1]);
}
#[test]
fn bounded_floor_rancor_paid_entry_sequential_hits_keep_target_identity_and_shield_boundaries() {
    let e=engine();for seat in [Seat::P1,Seat::P2] {for scenario in [0,1,2] {
        let mut s=midgame(&e,seat);for _ in 0..16 {let id=add(&e,&mut s,seat,"LOF_256");s.player_mut(seat).resources.push(id).unwrap();}
        let source=add(&e,&mut s,seat,"ASH_179");s.player_mut(seat).hand.push(source).unwrap();let target=unit(&e,&mut s,seat.other(),"LOF_256");let other=unit(&e,&mut s,seat.other(),"LOF_256");let space=unit(&e,&mut s,seat.other(),"ASH_201");
        // A preexisting phase HP bonus is setup state, not altered card metadata.
        if scenario!=0 {s.card_mut(target).phase_hp=10;}
        let shield=if scenario==2 {let token=add(&e,&mut s,seat.other(),"TOKEN_SHIELD");s.card_mut(target).shield=token;e.attach(&mut s,token,target);token}else{0};
        let own=s.player(seat).base;let s=roundtrip(&e,&s);let before=e.ready_resources(&s,seat);let cost=e.cost(&s,source) as usize;let action=e.card_action(&s,source);assert!(e.legal_actions(&s).contains(&action));
        let choosing=e.apply(&s,&action).unwrap();assert_eq!(choosing.card(own).damage,5);assert_eq!(choosing.decision,Decision::AbilityTarget);assert_eq!(e.ready_resources(&choosing,seat),before-cost);assert!(e.legal_actions(&choosing).contains(&e.card_action(&choosing,target)));assert!(!e.legal_actions(&choosing).contains(&e.card_action(&choosing,space)));
        let done=e.apply(&choosing,&e.card_action(&choosing,target)).unwrap();assert!(done.player(seat).ground.contains(source));assert!(done.card(source).exhausted);assert_eq!(done.card(other).damage,0);
        if scenario==0 {assert!(done.player(seat.other()).discard.contains(target));}else{assert!(done.player(seat.other()).ground.contains(target));assert_eq!(done.card(target).damage,if scenario==1 {10}else{5});}
        if scenario==2 {assert_eq!(done.card(target).shield,0);assert_eq!(done.card(shield).parent,0);let canonical=e.load_state(&e.snapshot(&done)).unwrap();roundtrip(&e,&canonical);}else{roundtrip(&e,&done);}
    }}
}

#[test]
fn bounded_floor_rancor_terminal_entry_keeps_mandatory_target_and_latched_winner() {
    let e=engine();for seat in [Seat::P1,Seat::P2] {
        let mut s=midgame(&e,seat);for _ in 0..16 {let id=add(&e,&mut s,seat,"LOF_256");s.player_mut(seat).resources.push(id).unwrap();}
        let source=add(&e,&mut s,seat,"ASH_179");s.player_mut(seat).hand.push(source).unwrap();let target=unit(&e,&mut s,seat.other(),"LOF_256");s.card_mut(target).phase_hp=10;
        let base=s.player(seat).base;s.card_mut(base).damage=25;
        let choosing=e.apply(&s,&e.card_action(&s,source)).unwrap();assert_eq!(choosing.outcome,(seat.other().index()+1) as u8);assert_eq!(choosing.card(base).damage,30);assert_eq!(choosing.decision,Decision::AbilityTarget);assert_eq!(choosing.actor(),None);assert!(e.legal_actions(&choosing).contains(&e.card_action(&choosing,target)));
        // The paid Rancor's mandatory native continuation accepts this target
        // after its controller's base reaches lethal; the earlier winner stays.
        let done=e.apply(&choosing,&e.card_action(&choosing,target)).unwrap();assert_eq!(done.outcome,(seat.other().index()+1) as u8);assert_eq!(done.card(target).damage,10);assert_eq!(done.card(base).damage,30);
    }
}
