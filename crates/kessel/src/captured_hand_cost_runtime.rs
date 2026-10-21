//! Shared executor for the two closed physical hand-card continuations.
//! As with other compiled contracts, Engine configuration must remain unchanged
//! after construction. Public Definitions mutation is not defended as gameplay.
//! Supported imports and State/Support copies use the validated definition graph.
use super::*;
impl Engine {
    /// Queen freezes a scalar AFTER reveal acknowledgement;163 rereads its
    /// first specific discard card lazily. One physical binding is equivalent
    /// only in this immutable, override-free graph. No absent-cost fallback for
    /// malformed WithCost units is invented. Copied relevant graphs recheck it.
    pub(super) fn validate_hand_cost_graph(&self,state:Option<&State>)->Result<(),String>{
        if self.hand_cost_all_units_costed {return Ok(())}
        let hand_definition=|d:&Definition|d.programs.card.iter().chain(&d.programs.unit).chain(&d.programs.leader).any(|p|self.program(*p).instructions.iter().any(|i|matches!(i,Instruction::HandCost{..})));
        let hand_program=|p:u16|p!=0&&self.catalogue.as_ref().and_then(|c|c.programs.get(p as usize-1)).is_some_and(|p|p.instructions.iter().any(|i|matches!(i,Instruction::HandCost{..})));
        let missing=|d:&Definition|(d.info.is("unit")||d.info.is("leader"))&&d.info.cost.is_none();
        let relevant=self.selected_definition_any(hand_definition)||state.is_some_and(|s|hand_program(s.ability_program)||s.death_frames[..s.death_depth as usize].iter().any(|f|hand_program(f.context.ability_program))||s.support.pending&&hand_program(s.support.context.ability_program)||(1..s.next_id).any(|id|s.card(id).present&&hand_definition(self.def(s,id))));
        if relevant&&(self.selected_definition_any(missing)||state.is_some_and(|s|(1..s.next_id).any(|id|s.card(id).present&&missing(self.def(s,id))))) {return Err("captured hand-cost graph contains a WithCost unit without printed cost".into())}
        Ok(())
    }
    pub(super) fn hand_cost_step(&self,s:&State)->Option<(bool,u8)> {
        if s.ability_program==0 {return None}
        match self.program(s.ability_program-1).instructions.get(s.ability_pc as usize) {Some(Instruction::HandCost{reveal,stage})=>Some((*reveal,*stage)),_=>None}
    }
    pub(super) fn hand_cost_prompt(&self,s:&State)->bool {matches!((self.hand_cost_step(s),s.decision),(Some((_,0|2)),Decision::AbilityTarget)|(Some((true,1)),Decision::AbilityDisplay))}
    fn hand_cost_hand(&self,s:&State)->Vec<CardId> {
        let owner=s.card(s.source).owner;
        s.player(owner).hand.as_slice().iter().copied().filter(|id|self.def(s,*id).info.is("unit")).collect()
    }
    fn hand_cost_targets(&self,s:&State,reveal:bool)->Vec<CardId> {
        let id=s.captured_hand_card.id();
        if id==0 || id>=s.next_id || !s.card(id).present {return vec![]}
        let cost=self.def(s,id).info.cost;
        self.arena_units(s).into_iter().filter(|candidate|self.def(s,*candidate).info.cost.is_some_and(|n|if reveal {Some(n)==cost}else{n>cost.unwrap_or(0)}) && self.effect_can_affect(s,s.source,*candidate,&Effect::Damage{target:Target::Name("chosen".into()),amount:Expr::number(if reveal{3}else{5})},true)).collect()
    }
    pub(super) fn hand_cost_actions(&self,s:&State)->Vec<Action> {
        let Some((reveal,stage))=self.hand_cost_step(s) else{return vec![]};
        match stage {
            0=>{let mut a=Vec::new();if reveal {a.push(Action::button(if self.hand_cost_hand(s).is_empty(){"Skip"}else{"Choose nothing"},"done"));}a.extend(self.hand_cost_hand(s).into_iter().map(|id|self.card_action(s,id)));a},
            1=>vec![Action::button("Done","done")],
            2=>self.hand_cost_targets(s,reveal).into_iter().map(|id|self.card_action(s,id)).collect(),
            _=>vec![],
        }
    }
    pub(super) fn run_hand_cost_instruction(&self,s:&mut State)->Result<bool,String> {
        let (reveal,stage)=self.hand_cost_step(s).ok_or("missing hand-cost instruction")?;
        if !self.commit_program(s)? {return Ok(true)}
        let owner=s.card(s.source).owner;
        match stage {
            0=>{if s.captured_hand_card.id()!=0{return Err("hand binding already captured".into())}if !reveal && self.hand_cost_hand(s).is_empty(){s.ability_pc=self.program(s.ability_program-1).instructions.len() as u8;return Ok(false)}s.active=owner;s.decision=Decision::AbilityTarget;},
            1=>{if !reveal||s.captured_hand_card.id()==0{return Err("invalid reveal display binding".into())}s.active=owner.other();s.decision=Decision::AbilityDisplay;},
            2=>{s.active=owner;if self.hand_cost_targets(s,reveal).is_empty(){s.ability_pc+=1;return Ok(false)}s.decision=Decision::AbilityTarget;},
            _=>return Err("invalid hand-cost stage".into()),
        }
        Ok(true)
    }
    pub(super) fn apply_hand_cost_action(&self,s:&State,a:&Action)->Result<State,String> {
        let (reveal,stage)=self.hand_cost_step(s).ok_or("missing hand-cost instruction")?;
        if !self.hand_cost_actions(s).contains(a){return Err("illegal physical hand-cost choice".into())}
        let mut t=*s;
        if stage==0 && a.arg.as_ref().and_then(Value::as_str)==Some("done") {return self.end_program(&mut t).map(|_|t)}
        match stage {
            0=>{let id=a.id().ok_or("hand selection missing card")?;t.captured_hand_card=crate::captured_hand_cost_contract::PhysicalHandCard::capture(id)?;
                if !reveal {t.ability_target=id;let (_,success)=self.apply_effect(&mut t,&Effect::DiscardSpecificCard{target:Target::Name("chosen".into())})?;if !success{return self.end_program(&mut t).map(|_|t)}}
                t.ability_pc+=1;
            },
            1=>{t.active=t.card(t.source).owner;t.ability_pc+=1;},
            2=>{let id=a.id().ok_or("damage selection missing card")?;t.ability_target=id;let (amount,success)=self.apply_effect(&mut t,&Effect::Damage{target:Target::Name("chosen".into()),amount:Expr::number(if reveal{3}else{5})})?;t.ability_result=amount;t.ability_succeeded=success;t.ability_pc+=1;},
            _=>return Err("invalid physical hand-cost stage".into()),
        }
        self.run_program(&mut t)?;Ok(t)
    }
}
