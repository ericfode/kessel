use kessel::model::*;

#[test]
fn numeric_views_preserve_inline_storage_and_hashes() {
    fn copy<T: Copy>() {}
    copy::<State>();
    let mut s=State::default();
    // The checkpoint adds exact Credit/payment continuations and bounded live
    // attack provenance plus eight bytes of exact aggregate defeat history.
    // Inactive legacy storage stays off the historical hash stream.
    assert_eq!(std::mem::size_of::<State>(),32312);
    // Legacy v1 hashes stay stable; v2 distinguishes exact defeat projections.
    s.phase_history.defeats_exact=false;
    assert_eq!(s.zobrist(),0x08e5596b5f608cd6);
    s.set_trigger_context(TriggerContext::ShieldOrder);
    s.ability_return=ProgramContinuation::Deaths.raw();
    s.death_frames[0].resume=DeathContinuation::NativeAttackAfterEarlyEffects.raw();
    s.event_result=StepEventResult{source:4,program:9,pc:3,damage_dealt:7,defeated_units:[2,1],will_defeat:false};
    s.maximum_capture=MaximumCapture{power:4,sole:6,count:2};
    assert_eq!(s.zobrist(),0x6c2b899b1d02ab05);
    assert!(s.trigger_context_is(TriggerContext::ShieldOrder));
    assert_eq!(s.death_frames[0].continuation(),Some(DeathContinuation::NativeAttackAfterEarlyEffects));
}

#[test]
fn numeric_views_do_not_reinterpret_unknown_or_empty_boundaries() {
    for raw in 0..=u8::MAX {
        assert_eq!(TriggerContext::from_raw(raw).map(TriggerContext::raw), (raw<=4).then_some(raw));
        assert_eq!(ProgramContinuation::from_raw(raw).map(ProgramContinuation::raw), (raw<=2).then_some(raw));
        assert_eq!(DeathContinuation::from_raw(raw).map(DeathContinuation::raw), (1..=9).contains(&raw).then_some(raw));
    }
    assert_eq!(DeathFrame::default().continuation(),None);
    let mut s=State::default();s.trigger_context=255;
    assert!(!s.trigger_context_is(TriggerContext::Action));
    assert_eq!(s.trigger_context,255);
}

#[test]
fn nested_context_restores_result_and_factory_scope_without_new_counters() {
    let mut s=State::default();s.source=3;s.ability_program=8;s.ability_pc=5;
    s.maximum_capture=MaximumCapture{power:7,sole:9,count:2};
    s.event_result=StepEventResult{damage_dealt:4,defeated_units:[1,0],..StepEventResult::for_state(&s)};
    let outer=ResolutionContext::capture(&s);
    s.source=12;s.ability_program=14;s.ability_pc=2;
    s.maximum_capture=MaximumCapture::default();s.event_result=StepEventResult::for_state(&s);
    let inner=ResolutionContext::capture(&s);
    assert!(s.event_result.belongs_to(12,14,2));
    assert!(!s.event_result.belongs_to(3,8,5));
    s.event_result.damage_dealt=99;s.maximum_capture.observe(11,13).unwrap();
    inner.restore(&mut s);
    assert_eq!(s.event_result.damage_dealt,0);
    assert_eq!(s.maximum_capture,MaximumCapture::default());
    outer.restore(&mut s);
    assert!(s.event_result.belongs_to(3,8,5));
    assert!(!s.event_result.belongs_to(3,8,6));
    assert_eq!(s.event_result.damage_dealt,4);
    assert_eq!(s.event_result.defeated_units,[1,0]);
    assert_eq!(s.maximum_capture,MaximumCapture{power:7,sole:9,count:2});
}

#[test]
fn maximum_capture_keeps_zero_floor_order_ties_and_explicit_capacity() {
    let mut capture=MaximumCapture::default();
    capture.observe(-1,9).unwrap();assert_eq!(capture,MaximumCapture::default());
    capture.observe(0,5).unwrap();capture.observe(0,3).unwrap();
    assert_eq!(capture,MaximumCapture{power:0,sole:5,count:2});
    capture.observe(7,9).unwrap();capture.observe(4,1).unwrap();capture.observe(7,2).unwrap();
    assert_eq!(capture,MaximumCapture{power:7,sole:9,count:2});
    capture.count=u16::MAX;let before=capture;
    assert_eq!(capture.observe(7,3),Err("maximum tie count exceeds capacity".into()));
    assert_eq!(capture,before);
    capture.observe(8,4).unwrap();
    assert_eq!(capture,MaximumCapture{power:8,sole:4,count:1});
}

#[test]
fn physical_registration_owner_keeps_host_execution_identity() {
    let ordinary=DeathTrigger{source:3,..DeathTrigger::default()};
    let granted=DeathTrigger{source:3,grantor:9,..DeathTrigger::default()};
    assert_eq!(ordinary.physical_source(),3);
    assert_eq!(granted.physical_source(),9);
    assert_eq!(granted.source,3);
}

#[test]
fn new_flags_are_hash_neutral_when_inactive_and_distinct_when_active() {
    let original=State::default();
    let mut state=original;
    state.enter_smuggle=true;
    assert_ne!(state.zobrist(),original.zobrist());
    state.enter_smuggle=false;
    assert_eq!(state.zobrist(),original.zobrist());
    state.event_result=StepEventResult{source:4,program:9,pc:3,damage_dealt:7,defeated_units:[2,1],will_defeat:false};
    let before=state.zobrist();
    state.event_result.will_defeat=true;
    assert_ne!(state.zobrist(),before);
    state.event_result.will_defeat=false;
    assert_eq!(state.zobrist(),before);
}
