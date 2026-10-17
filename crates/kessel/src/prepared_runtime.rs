//! Prepared event windows: capture, replacement, primary handlers, contingents,
//! and post-event visibility. Inline State records remain the compatibility
//! representation; execution passes captured values to the shared Effect handlers.
use super::*;

#[derive(Clone, Copy)]
struct CapturedDraw { base: CardId, requested: u16, fatigue: u16 }
impl CapturedDraw {
    fn record(self, op: usize) -> PreparedEvent {
        PreparedEvent { op: op as u16, target: self.base, amount: self.requested,
            hp: i32::from(self.fatigue), ..PreparedEvent::default() }
    }
    fn from_record(record: PreparedEvent) -> Self {
        Self { base: record.target, requested: record.amount, fatigue: record.hp as u16 }
    }
}

// Force presence can alter ongoing HP. This exact existing group shape retains
// damage-handler lethal snapshots until post-event recomputation. It is a
// visibility contract, not admission of arbitrary damage/Force compositions.
fn damage_before_force(effect: &Effect) -> bool {
    matches!(effect,Effect::Simultaneous {effects} if effects.len()==2
        && matches!(&effects[0],Effect::Select {effect,..} if matches!(effect.as_ref(),Effect::Damage {..}))
        && matches!(&effects[1],Effect::GainForce {..}))
}

// Damage callbacks share one observation/flush algorithm in uniform and
// heterogeneous windows. The policy specifies when lethality is observed;
// mutation ordering is not itself a policy for cache visibility.
struct DamageObservations {
    capture_lethals: bool,
    callbacks: Vec<CardId>,
    handler_lethals: Vec<CardId>,
}
impl DamageObservations {
    fn new(capture_lethals: bool) -> Self {
        Self {capture_lethals,callbacks:vec![],handler_lethals:vec![]}
    }
    fn observe(&mut self, engine: &Engine, s: &State, target: CardId, amount: u16) {
        if amount==0 || !engine.in_arena(s,target) {return;}
        self.callbacks.push(target);
        if self.capture_lethals && s.card(target).damage>=engine.hp(s,target) {
            self.handler_lethals.push(target);
        }
    }
    fn finish(self, engine: &Engine, s: &mut State) -> Result<(),String> {
        if self.capture_lethals {engine.defeat_many(s,&self.handler_lethals)?;}
        engine.finish_group_damage(s,self.callbacks)
    }
}

enum GroupOp<'a> {
    Leaf(&'a Effect),
    Select(&'a TargetResolver, &'a Effect, usize),
    Pop,
    Branch(&'a Expr, usize),
    Jump(usize),
}
fn group_ops(effect: &Effect) -> Vec<GroupOp<'_>> {
    fn add<'a>(e: &'a Effect, out: &mut Vec<GroupOp<'a>>) {
        match e {
            Effect::Simultaneous { effects } | Effect::PhaseModifiers { effects } => { for e in effects { add(e,out); } }
            Effect::Select { target_resolver, effect } => {
                let index=out.len(); out.push(GroupOp::Select(target_resolver,effect,0));
                add(effect,out); out.push(GroupOp::Pop);
                let end=out.len(); out[index]=GroupOp::Select(target_resolver,effect,end);
            }
            Effect::Conditional { condition,on_true,on_false } => {
                let branch=out.len(); out.push(GroupOp::Branch(condition,0)); add(on_true,out);
                let jump=out.len(); out.push(GroupOp::Jump(0));
                let alternate=out.len(); if let Some(e)=on_false { add(e,out); }
                let end=out.len(); out[branch]=GroupOp::Branch(condition,alternate); out[jump]=GroupOp::Jump(end);
            }
            Effect::NoAction => {},
            _ => out.push(GroupOp::Leaf(e)),
        }
    }
    let mut out=vec![]; add(effect,&mut out); out
}

impl Engine {
    pub(super) fn current_group_resolver<'a>(&self, s: &State, p: &'a Program) -> Option<(&'a TargetResolver,&'a Effect)> {
        let effect=p.instructions[s.ability_pc as usize].group_effect()?;
        match group_ops(effect).get(s.event_group.pc as usize) {
            Some(GroupOp::Select(r,e,_))=>Some((r,e)), _=>None,
        }
    }
    pub(super) fn group_candidates(&self,s:&State,r:&TargetResolver,e:&Effect,require_change:bool)->Vec<CardId> {
        self.matching_targets(s,s.source,r).into_iter().filter(|id| self.effect_can_affect(s,s.source,*id,e,require_change)).collect()
    }
    pub(super) fn capture_group_leaf(&self,s:&mut State,op:usize,e:&Effect)->Result<(),String> {
        if let Effect::ResourceCard {target}=e {
            let eligible=match target {Target::Name(n) if n=="self"=>self.resource_source_eligible(s,s.source),Target::Name(n) if n=="ownTopDeck"=>true,_=>return Err("unreviewed grouped resource target".into())};
            if !eligible {return Ok(());}
            for value in self.effect_targets(s,target) {let Eval::Card(id)=value else{return Err("resource group requires card target".into());};
                let legal=if matches!(target,Target::Name(n) if n=="self") {id==s.source&&self.resource_source_eligible(s,id)} else {self.resource_top_deck_eligible(s,id)};
                if !legal {continue;}
                let n=s.event_group.count as usize;if n>=32{return Err("prepared resource event capacity exceeded".into());}
                s.event_group.records[n]=PreparedEvent{op:op as u16,target:id,amount:1,..PreparedEvent::default()};s.event_group.count+=1;
            }
            return Ok(());
        }
        if matches!(e,Effect::GainForce{..}) {
            let Effect::GainForce{target}=e else{unreachable!()};
            let targets=self.effect_targets(s,target);if s.event_group.count as usize+targets.len()>32{return Err("prepared player event capacity exceeded".into());}
            for target in targets {let Eval::Player(player)=target else{return Err("Force group target must be a player".into());};let n=s.event_group.count as usize;s.event_group.records[n]=PreparedEvent{op:op as u16,target:s.player(player).base,amount:1,..PreparedEvent::default()};s.event_group.count+=1;}
            return Ok(());
        }

        if let Effect::DiscardDeck {target,amount}=e {
            let Eval::Player(player)=self.effect_targets(s,target).into_iter().next().ok_or("deck discard target missing")? else{return Err("deck discard requires a player target".into());};
            let requested=self.effect_amount(s,amount)?;
            if requested==0 {return Ok(());}
            let cards=s.player(player).deck.as_slice().iter().take(requested as usize).copied().collect::<Vec<_>>();
            if s.event_group.count as usize+cards.len()+1>32 {return Err("prepared deck discard event capacity exceeded".into());}
            for card in cards {
                let n=s.event_group.count as usize;
                s.event_group.records[n]=PreparedEvent {op:op as u16,target:card,amount:1,..PreparedEvent::default()};s.event_group.count+=1;
            }
            // Append the successful aggregate event even for an empty deck.
            let n=s.event_group.count as usize;
            s.event_group.records[n]=PreparedEvent {op:op as u16,target:s.player(player).base,amount:requested,..PreparedEvent::default()};s.event_group.count+=1;
            return Ok(());
        }
        if let Effect::Draw {target,amount}=e {
            let Eval::Player(player)=self.effect_targets(s,target).into_iter().next().ok_or("draw target missing")? else{return Err("group draw requires player target".into());};
            let n=s.event_group.count as usize;if n==32 {return Err("prepared draw event capacity exceeded".into());}
            let amount=self.effect_amount(s,amount)?;
            // DrawSystem generates fatigue contingents from the deck at event generation.
            let fatigue=amount.saturating_sub(s.player(player).deck.len as u16)*3;
            let draw=CapturedDraw {base:s.player(player).base,requested:amount,fatigue};
            s.event_group.records[n]=draw.record(op);s.event_group.count+=1;return Ok(());
        }
        if let Effect::CreateTokenUnit {token_type,amount,target,enters_ready}=e {
            let code=token_unit_code(token_type).ok_or("unsupported token unit type")?;
            let def=*self.by_code.get(code).ok_or("created token definition missing from loaded pool")?;
            let amount=self.effect_amount(s,amount)? as usize;
            let targets=self.effect_targets(s,target);
            if s.next_id as usize+amount*targets.len()>MAX_CARDS {return Err("card capacity exceeded generating token units".into());}
            if s.event_group.count as usize+amount*targets.len()>32 {return Err("prepared token unit event capacity exceeded".into());}
            for target in targets {let Eval::Player(player)=target else{return Err("token unit target is not a player".into());};
                for _ in 0..amount {
                    let token=s.next_id;s.next_id+=1;*s.card_mut(token)=Card {present:true,owner:player,def,exhausted:!*enters_ready,..Card::default()};
                    let n=s.event_group.count as usize;s.event_group.records[n]=PreparedEvent {op:op as u16,target:s.player(player).base,amount:1,token,..PreparedEvent::default()};s.event_group.count+=1;
                }
            }
            return Ok(());
        }
        // Phase restriction marks are primitive captures with no payload: each
        // surviving recipient records one idempotent, capacity-free mark.
        if let Effect::CardRestriction { target, duration, restriction, condition } = e {
            if condition.is_some() || duration != "phase" { return Err("grouped card restriction requires a phase mark".into()); }
            for value in self.effect_targets(s,target) {
                let Eval::Card(id)=value else { continue; };
                if id == 0 || !(self.in_arena(s,id) || self.def(s,id).info.is("base")) { continue; }
                if s.card(id).phase_restrictions & restriction.bit() != 0 { continue; }
                if s.event_group.count as usize == 32 { return Err("prepared event capacity exceeded".into()); }
                let n=s.event_group.count as usize;
                s.event_group.records[n]=PreparedEvent{op:op as u16,target:id,amount:1,..PreparedEvent::default()};s.event_group.count+=1;
            }
            return Ok(());
        }
        if !matches!(e,Effect::Damage {..}|Effect::Heal {..}|Effect::GiveShield {..}|Effect::GiveExperience {..}
            |Effect::GiveWeakness {..}|Effect::GiveAdvantage {..}|Effect::GainKeyword {..}|Effect::Ready {..}
            |Effect::Exhaust {..}|Effect::Defeat {..}|Effect::ModifyStats {..}|Effect::LoseKeyword {..}) {return Err("unsupported group primitive".into());}
        let (target,amount)=e.scalar_args().expect("group primitive has scalar fields");
        let amount=amount.map(|amount|self.effect_amount(s,amount)).transpose()?.unwrap_or(1);
        let (power,hp)=if let Effect::ModifyStats{power,hp,..}=e {
            let power=i32::try_from(self.eval(s,s.source,s.ability_target,power).number()).map_err(|_|"phase power delta exceeds capacity")?;
            let hp=i32::try_from(self.eval(s,s.source,s.ability_target,hp).number()).map_err(|_|"phase HP delta exceeds capacity")?;
            if !(-65535..=65535).contains(&power)||!(-65535..=65535).contains(&hp) {return Err("phase stat delta exceeds capacity".into());}
            (power,hp)
        } else {(0,0)};
        let destination=target;
        for target in self.effect_targets(s,destination) {
            let Eval::Card(target)=target else { continue; };
            if target==0 || !(self.in_arena(s,target)||self.def(s,target).info.is("base")) { continue; }
            // A "chosen" group is one event per selected card in the engine, so a
            // selected card that left play before resolution cannot suppress the
            // remaining events. The previous single outer-target check let an
            // Exploit-defeated first target hide its surviving siblings.
            // A "chosen" group is one event per selected card in the engine, so a
            // selected card that left play before resolution cannot suppress the
            // remaining events. The outer ability target would do exactly that
            // when Exploit defeats the first selected card before the event
            // effect resolves; check each chosen recipient instead.
            let affect_target=if matches!(destination,Target::Name(n) if n=="chosen"){target}else{s.ability_target};
            if !self.effect_can_affect(s,s.source,affect_target,e,false) { continue; }
            if !self.in_arena(s,target) && !matches!(e,Effect::Damage{..}|Effect::Heal{..}) { continue; }
            let tokens=matches!(e,Effect::GiveShield{..}|Effect::GiveExperience{..}|Effect::GiveWeakness{..}|Effect::GiveAdvantage{..});
            for _ in 0..if tokens { amount } else { 1 } {
                let mut record=PreparedEvent{op:op as u16,target,amount,power,hp,token:0};
                if tokens && !self.program(s.ability_program-1).instructions[s.ability_pc as usize].is_named_group() {
                    if s.next_id as usize>=MAX_CARDS { return Err("card capacity exceeded generating token".into()); }
                    record.token=s.next_id; s.next_id+=1;
                    let def=if matches!(e,Effect::GiveAdvantage{..}) {self.advantage_def.ok_or("catalogue needs Advantage token")?}else if matches!(e,Effect::GiveWeakness{..}) {self.weakness_def.ok_or("catalogue needs Weakness token")?}else if matches!(e,Effect::GiveExperience{..}) { self.experience_def.ok_or("catalogue needs Experience token")? } else { self.shield_def };
                    *s.card_mut(record.token)=Card { present:true,def,owner:s.card(target).owner,..Card::default() };
                }
                if s.event_group.count>=32 { return Err("prepared event capacity exceeded".into()); }
                s.event_group.records[s.event_group.count as usize]=record; s.event_group.count+=1;
            }
        }
        Ok(())
    }
    pub(super) fn allocate_named_tokens(&self,s:&mut State,e:&Effect)->Result<(),String> {
        let ops=group_ops(e);
        let records=(0..s.event_group.count as usize).filter(|i|s.event_group.records[*i].token==0 && matches!(ops[s.event_group.records[*i].op as usize],GroupOp::Leaf(Effect::GiveShield{..}|Effect::GiveExperience{..}|Effect::GiveWeakness{..}|Effect::GiveAdvantage{..}))).collect::<Vec<_>>();
        if s.next_id as usize+records.len()>MAX_CARDS {return Err("card capacity exceeded generating named tokens".into());}
        if records.iter().any(|i|matches!(ops[s.event_group.records[*i].op as usize],GroupOp::Leaf(Effect::GiveExperience{..}))) && self.experience_def.is_none() {return Err("catalogue needs Experience token".into());}
        if records.iter().any(|i|matches!(ops[s.event_group.records[*i].op as usize],GroupOp::Leaf(Effect::GiveWeakness{..}))) && self.weakness_def.is_none() {return Err("catalogue needs Weakness token".into());}
        if records.iter().any(|i|matches!(ops[s.event_group.records[*i].op as usize],GroupOp::Leaf(Effect::GiveAdvantage{..}))) && self.advantage_def.is_none() {return Err("catalogue needs Advantage token".into());}
        for i in records {
            let r=s.event_group.records[i];let token=s.next_id;s.next_id+=1;
            let def=if matches!(ops[r.op as usize],GroupOp::Leaf(Effect::GiveAdvantage{..})) {self.advantage_def.unwrap()}else if matches!(ops[r.op as usize],GroupOp::Leaf(Effect::GiveWeakness{..})) {self.weakness_def.unwrap()}else if matches!(ops[r.op as usize],GroupOp::Leaf(Effect::GiveExperience{..})) {self.experience_def.unwrap()} else {self.shield_def};
            *s.card_mut(token)=Card{present:true,def,owner:s.card(r.target).owner,..Card::default()};s.event_group.records[i].token=token;
        }
        Ok(())
    }
    pub(super) fn uniform_damage_amount(&self,e:&Effect)->Option<u16> {
        match e { Effect::Damage {target:Target::ArenaList(_),amount}=>match amount.original() {Expr::Const {value:Value::Number(n)}=>n.as_u64().and_then(|n|u16::try_from(n).ok()),_=>None},_=>None }
    }
    pub(super) fn generate_uniform_damage(&self,s:&mut State,e:&Effect)->Result<bool,String> {
        if s.event_group.uniform_damage.active {return Ok(true);}
        let Effect::Damage {target,..}=e else {unreachable!()};
        if matches!(target,Target::ArenaList(l) if l.controller_order.is_some()) && !s.arena_order_exact {return Err("ordered ground damage requires recorded arena order".into());}
        let targets=self.effect_targets(s,target);
        let mut uniform=UniformDamageState::default();let mut stash=vec![];
        for value in targets {
            let Eval::Card(target)=value else {return Err("arena damage target is not a card".into());};
            if uniform.records[..uniform.count as usize].iter().any(|r|r.target()==target) {return Err("duplicate arena damage target".into());}
            let index=uniform.count as usize;if index==MAX_CARDS {return Err("arena damage capacity exceeded".into());}
            let mut shields=vec![];let mut shield=s.card(target).shield;
            while shield!=0 {shields.push(shield);shield=s.card(shield).next_token;}
            shields.reverse();
            uniform.records[index]=UniformDamageRecord::new(target,shields.first().copied().unwrap_or(0))?;
            uniform.count+=1;stash.extend(shields.into_iter().skip(1));
        }
        if stash.len()>u8::MAX as usize {return Err("Shield context capacity exceeded".into());}
        uniform.active=true;s.event_group.uniform_damage=uniform;
        s.event_group.shield_stash_len=stash.len() as u8;
        s.event_group.shield_stash[..stash.len()].copy_from_slice(&stash);
        Ok(true)
    }
    pub(super) fn generate_group(&self,s:&mut State,e:&Effect)->Result<bool,String> {
        if s.event_group.pc==0&&matches!(e,Effect::GiveShield{..})&&s.event_group.context.len>3 {
            let Effect::GiveShield{target,..}=e else{unreachable!()};let n=self.effect_targets(s,target).into_iter().filter(|v|matches!(v,Eval::Card(id) if self.in_arena(s,*id))).count();
            if s.next_id as usize+n>MAX_CARDS||s.event_group.count as usize+n>32{return Err("five-pick Shield generation capacity exceeded".into());}
        }

        // A named single Weakness selection allocates only after its chosen record
        // exists; allocate_named_tokens performs the same capacity/token checks.
        if s.event_group.pc==0&&effect_has(e,|e|matches!(e,Effect::GiveWeakness{..}))&&!crate::ability_ir::native_single_weakness_group(e) {
            self.weakness_def.ok_or("catalogue needs Weakness token")?;
            let mut tokens=0usize;let mut records=0usize;
            for op in group_ops(e) {let GroupOp::Leaf(effect)=op else{return Err("Weakness window must have scalar primary events".into());};
                let target=match effect {Effect::GiveWeakness{target,..}|Effect::Damage{target,..}|Effect::Heal{target,..}=>target,_=>return Err("unsupported Weakness window primitive".into())};
                let count=self.effect_targets(s,target).into_iter().filter(|v|matches!(v,Eval::Card(id) if *id!=0&&self.in_arena(s,*id))).count();
                if let Effect::GiveWeakness{amount,..}=effect {let n=self.effect_amount(s,amount)? as usize;if !(1..=2).contains(&n){return Err("Weakness window amount must be one or two".into());}tokens+=n*count;records+=n*count;}else{records+=count;}
            }
            if s.next_id as usize+tokens>MAX_CARDS {return Err("card capacity exceeded generating Weakness".into());}
            if s.event_group.count as usize+records>32 {return Err("prepared Weakness window capacity exceeded".into());}
        }

        if self.uniform_damage_amount(e).is_some() {return self.generate_uniform_damage(s,e);}
        let ops=group_ops(e);
        while let Some(op)=ops.get(s.event_group.pc as usize) {
            s.ability_target=s.event_group.context.ids[0];
            match op {
                GroupOp::Branch(expr,to)=> { if !self.eval(s,s.source,s.ability_target,expr).truth() { s.event_group.pc=*to as u16; continue; } }
                GroupOp::Jump(to)=> { s.event_group.pc=*to as u16; continue; }
                GroupOp::Select(r,child,end)=> {
                    if let Instruction::NamedChooserGroup{choosers,..}=&self.program(s.ability_program-1).instructions[s.ability_pc as usize] {
                        let index=ops[..s.event_group.pc as usize].iter().filter(|op|matches!(op,GroupOp::Select(..))).count();
                        s.active=s.event_group.initiator;
                        if matches!(choosers[index],crate::ability_ir::NamedChooser::Opponent) {
                            // Opponent pretarget always delays until original-player costs are paid.
                            self.commit_program(s)?;s.active=s.event_group.initiator.other();
                        }
                    }
                    if self.group_candidates(s,r,child,true).is_empty() {
                        // A missing pretarget delays the remaining named resolver chain until costs.
                        if !s.ability_committed && self.program(s.ability_program-1).instructions[s.ability_pc as usize].is_named_group() && self.group_candidates(s,r,child,false).is_empty() {self.commit_program(s)?;}
                        if matches!(self.program(s.ability_program-1).instructions[s.ability_pc as usize],Instruction::NamedChooserGroup{..}) {s.active=s.event_group.initiator;}
                        s.event_group.pc=*end as u16; continue;
                    }
                    s.decision=Decision::AbilityTarget; return Ok(false);
                }
                GroupOp::Pop=> {
                    s.event_group.depth-=1;
                    s.event_group.context=s.event_group.stack[s.event_group.depth as usize];
                    s.event_group.stack[s.event_group.depth as usize]=Selection::default();
                }
                GroupOp::Leaf(Effect::DiscardOwnHand {amount,target})=> {
                    let Eval::Player(player)=self.effect_targets(s,target).into_iter().next().ok_or("discard target missing")? else{return Err("discard target is not a player".into());};
                    let count=(self.effect_amount(s,amount)? as usize).min(s.player(player).hand.len as usize);
                    if count>0 {s.event_group.discard_owner=player;s.event_group.discard_count=count as u8;s.active=player;s.decision=Decision::AbilityDiscard;return Ok(false);}
                }
                GroupOp::Leaf(effect)=>self.capture_group_leaf(s,s.event_group.pc as usize,effect)?,
            }
            s.event_group.pc+=1;
        }
        Ok(true)
    }
    pub(super) fn group_shield_targets(&self,s:&State,e:&Effect,seat:Seat)->Vec<CardId> {
        let uniform=&s.event_group.uniform_damage;
        if uniform.active {return uniform.records[..uniform.count as usize].iter().enumerate().filter(|(i,r)|!uniform.is_prevented(*i)&&r.shield()!=0&&s.card(r.target()).owner==seat).map(|(_,r)|r.target()).collect();}
        let ops=group_ops(e); let mut targets=vec![];
        for r in &s.event_group.records[..s.event_group.count as usize] {
            if matches!(&ops[r.op as usize],GroupOp::Leaf(Effect::Damage{..})) && r.amount>0 && s.card(r.target).shield!=0 && s.card(r.target).owner==seat && !targets.contains(&r.target) { targets.push(r.target); }
        }
        targets
    }
    pub(super) fn shield_attached(&self,s:&State,target:CardId,shield:CardId)->bool {
        let mut token=s.card(target).shield;
        while token!=0 { if token==shield { return true; } token=s.card(token).next_token; }
        false
    }
    pub(super) fn consolidate_group_shields(&self,s:&mut State,e:Option<&Effect>) {
        let ops=e.map(group_ops);
        let damage=|r:&PreparedEvent|ops.as_ref().is_none_or(|ops|matches!(ops[r.op as usize],GroupOp::Leaf(Effect::Damage{..})));
        if s.event_group.uniform_damage.active {
            let mut contexts=s.event_group.shield_stash[..s.event_group.shield_stash_len as usize].to_vec();
            let pending=(0..s.event_group.uniform_damage.count as usize).filter(|i|!s.event_group.uniform_damage.is_prevented(*i)&&s.event_group.uniform_damage.records[*i].shield()!=0).collect::<Vec<_>>();
            contexts.extend(pending.iter().map(|i|s.event_group.uniform_damage.records[*i].shield()));
            let mut selected=vec![];
            for &i in &pending {
                let target=s.event_group.uniform_damage.records[i].target();
                if let Some(token)=contexts.iter().find(|id|self.shield_attached(s,target,**id)) {s.event_group.uniform_damage.records[i].shield=*token as u8;selected.push(*token);}
            }
            let retained=contexts.into_iter().filter(|id|!selected.contains(id)&&pending.iter().any(|i|self.shield_attached(s,s.event_group.uniform_damage.records[*i].target(),*id))).collect::<Vec<_>>();
            s.event_group.shield_stash=[0;MAX_CARDS];s.event_group.shield_stash_len=retained.len() as u8;
            s.event_group.shield_stash[..retained.len()].copy_from_slice(&retained);return;
        }
        // Mirror ReplacementEffectWindow: stashed alternate contexts precede
        // the currently retained contexts; retain the first valid one per unit.
        let mut contexts=s.event_group.shield_stash[..s.event_group.shield_stash_len as usize].to_vec();
        for r in &s.event_group.records[..s.event_group.count as usize] {
            if damage(r)&&r.amount>0 && r.token!=0 { contexts.push(r.token); }
        }
        let pending=(0..s.event_group.count as usize).filter(|i|damage(&s.event_group.records[*i])&&s.event_group.records[*i].amount>0 && s.event_group.records[*i].token!=0).collect::<Vec<_>>();
        let mut selected=vec![];
        for i in pending {
            let target=s.event_group.records[i].target;
            if let Some(token)=contexts.iter().find(|id|self.shield_attached(s,target,**id)) {
                s.event_group.records[i].token=*token; selected.push(*token);
            }
        }
        let retained=contexts.into_iter().filter(|id|!selected.contains(id) && s.event_group.records[..s.event_group.count as usize].iter().any(|r|damage(r)&&r.amount>0 && r.token!=0 && self.shield_attached(s,r.target,*id))).collect::<Vec<_>>();
        s.event_group.shield_stash=[0;MAX_CARDS]; s.event_group.shield_stash_len=retained.len() as u8;
        s.event_group.shield_stash[..retained.len()].copy_from_slice(&retained);
    }
    pub(super) fn replace_group_damage(&self,s:&mut State,e:&Effect,target:CardId) {
        if s.event_group.uniform_damage.active {
            if let Some(i)=(0..s.event_group.uniform_damage.count as usize).find(|i|s.event_group.uniform_damage.records[*i].target()==target&&!s.event_group.uniform_damage.is_prevented(*i)) {s.event_group.uniform_damage.prevent(i);self.consolidate_group_shields(s,Some(e));}
            return;
        }
        let ops=group_ops(e);
        if let Some(index)=(0..s.event_group.count as usize).find(|i| {
            let r=s.event_group.records[*i]; r.target==target && r.amount>0 && matches!(&ops[r.op as usize],GroupOp::Leaf(Effect::Damage{..}))
        }) {
            s.event_group.records[index].amount=0;
            self.consolidate_group_shields(s,Some(e));
        }
    }
    pub(in crate::simulator) fn combat_shield_contexts(&self,s:&State,first:CardId,second:CardId)->(CardId,CardId) {
        // Combat uses the same replacement-window consolidation as prepared
        // effect groups. Capture both units' contexts before resolving either.
        let mut contexts=*s; contexts.event_group=EventGroupState::default(); contexts.event_group.count=2;
        for (index,target) in [first,second].into_iter().enumerate() {
            let mut tokens=vec![]; let mut token=s.card(target).shield;
            while token!=0 { tokens.push(token); token=s.card(token).next_token; }
            tokens.reverse();
            contexts.event_group.records[index]=PreparedEvent {target,amount:1,token:tokens[0],..PreparedEvent::default()};
            for token in tokens.into_iter().skip(1) {
                let len=contexts.event_group.shield_stash_len as usize;
                contexts.event_group.shield_stash[len]=token; contexts.event_group.shield_stash_len+=1;
            }
        }
        let first_token=contexts.event_group.records[0].token;
        contexts.event_group.records[0].amount=0;
        self.consolidate_group_shields(&mut contexts,None);
        (first_token,contexts.event_group.records[1].token)
    }
    pub(in crate::simulator) fn remove_group_shield(&self,s:&mut State,target:CardId,selected:CardId) {
        let mut token=s.card(target).shield; let mut previous=0;
        while token!=0 && token!=selected { previous=token; token=s.card(token).next_token; }
        if token==0 { return; }
        if previous==0 { s.card_mut(target).shield=s.card(selected).next_token; }
        else { s.card_mut(previous).next_token=s.card(selected).next_token; }
        self.detach(s,selected);s.card_mut(selected).next_token=0;
    }
    // Resume the shared replacement window without consuming primary events.
    // Record capture and shield storage remain specific to each group layout.
    pub(super) fn resolve_group_shield_replacements(&self, s: &mut State, e: &Effect) -> bool {
        while s.event_group.shield_mask!=0 {
            if s.event_group.shield_mask==3 && s.event_group.shield_priority==0 {
                s.active=s.event_group.initiator; s.event_group.shield_count=0;
                s.decision=Decision::AbilityGroupShield; return false;
            }
            let seat=if s.event_group.shield_priority!=0 { if s.event_group.shield_priority==1 { Seat::P1 } else { Seat::P2 } }
                else if s.event_group.shield_mask&1!=0 { Seat::P1 } else { Seat::P2 };
            let targets=self.group_shield_targets(s,e,seat);
            if targets.len()>1 {
                s.active=seat; s.event_group.shield_count=targets.len() as u8;
                s.decision=Decision::AbilityGroupShield; return false;
            }
            for target in targets { self.replace_group_damage(s,e,target); }
            s.event_group.shield_mask&=!(1<<seat.index()); s.event_group.shield_priority=0;
        }
        s.active=s.event_group.initiator;
        true
    }
    pub(super) fn resolve_uniform_damage(&self,s:&mut State,e:&Effect)->Result<bool,String> {
        let amount=self.uniform_damage_amount(e).ok_or("invalid uniform damage effect")?;
        if !s.event_group.replacement_done {
            s.event_group.initiator=s.active;s.event_group.replacement_done=true;
            for r in &s.event_group.uniform_damage.records[..s.event_group.uniform_damage.count as usize] {if r.shield()!=0 {s.event_group.shield_mask|=1<<s.card(r.target()).owner.index();}}
        }
        if !self.resolve_group_shield_replacements(s, e) { return Ok(false); }
        s.ability_result=0;s.ability_succeeded=false;s.event_group.resolving=true;
        let mut events=StepEventResult::for_state(s);
        let mut observations=DamageObservations::new(false);
        for i in 0..s.event_group.uniform_damage.count as usize {
            let record=s.event_group.uniform_damage.records[i];let target=record.target();
            if !self.in_arena(s,target) {continue;}
            if s.event_group.uniform_damage.is_prevented(i) {self.remove_group_shield(s,target,record.shield());continue;}
            s.event_group.context=Selection::one(target);s.ability_target=target;
            let remaining=self.hp(s,target).saturating_sub(s.card(target).damage);
            let (applied,resolved)=self.apply_scalar_effect(s,e,vec![Eval::Card(target)],amount.min(remaining),None)?;
            events.damage_dealt=events.damage_dealt.checked_add(s.event_result.damage_dealt).ok_or("actual damage result exceeds capacity")?;
            observations.observe(self,s,target,applied);
            s.ability_result=s.ability_result.saturating_add(applied);s.ability_succeeded|=resolved;
        }
        s.event_result=events;
        s.event_group.resolving=false;observations.finish(self,s)?;Ok(true)
    }
    pub(super) fn resolve_resource_pair(&self,s:&mut State)->Result<bool,String> {
        // Preparation captured both physical IDs before either handler. Work on
        // one inline copy so capacity/derived-state failures publish no partial
        // movement or changed event flags, results, actor or source identity.
        if s.source==0||!s.card(s.source).present||!self.def(s,s.source).info.is("event")||!self.resource_source_eligible(s,s.source)||!s.player(s.card(s.source).owner).discard.contains(s.source) {return Err("resource pair source must be the paid event in discard".into());}
        let owner=s.card(s.source).owner;
        if !(1..=2).contains(&s.event_group.count) {return Err("resource pair record count invalid".into());}
        let records=&s.event_group.records[..s.event_group.count as usize];
        if records.iter().any(|r|r.token!=0||r.power!=0||r.hp!=0) {return Err("resource pair record payload invalid".into());}
        if records[0].op!=0||records[0].target!=s.source||records[0].amount!=1 {return Err("resource pair source record invalid".into());}
        if records.len()==2&&(records[1].op!=1||records[1].amount!=1||records[1].target==s.source||!self.resource_top_deck_eligible(s,records[1].target)) {return Err("resource pair top record invalid".into());}
        // A missing/null top is legal; a present deck-front cannot be omitted.
        if records.len()==1&&s.player(owner).deck.len>0 {return Err("resource pair top record missing".into());}
        let added=records.iter().filter(|r|!s.player(owner).resources.contains(r.target)).count();
        if s.player(owner).resources.len as usize+added>crate::model::MAX_ZONE {return Err("resource zone capacity exceeded".into());}
        let mut work=*s;
        let mut result=self.resource_source(&mut work,records[0].target)?;
        if records.len()==2 {result+=self.resource_top_deck(&mut work,records[1].target)?;}
        self.check_stat_capacity(&work)?;
        work.event_result=StepEventResult::for_state(&work);
        work.ability_result=result;work.ability_succeeded=result>0;
        // The existing window finisher observes only the completed pair.
        self.finish_group_damage(&mut work,vec![])?;
        *s=work;
        Ok(true)
    }
    pub(super) fn resolve_group(&self,s:&mut State,e:&Effect)->Result<bool,String> {
        if crate::ability_ir::resource_pair_effect(e) {return self.resolve_resource_pair(s);}
        let mut observations=DamageObservations::new(damage_before_force(e));

        if s.event_group.uniform_damage.active {return self.resolve_uniform_damage(s,e);}
        let ops=group_ops(e);
        if !s.event_group.replacement_done {
            s.event_group.initiator=s.active;
            s.event_group.replacement_done=true;
            for r in &s.event_group.records[..s.event_group.count as usize] {
                if matches!(&ops[r.op as usize],GroupOp::Leaf(Effect::Damage{..})) && r.amount>0 && s.card(r.target).shield!=0 {
                    s.event_group.shield_mask|=1<<s.card(r.target).owner.index();
                }
            }
        }
        if s.event_group.shield_mask!=0 && s.event_group.records[..s.event_group.count as usize].iter().filter(|r|matches!(&ops[r.op as usize],GroupOp::Leaf(Effect::Damage{..}))).all(|r|r.token==0) {
            for i in 0..s.event_group.count as usize {
                let r=s.event_group.records[i];
                if !matches!(&ops[r.op as usize],GroupOp::Leaf(Effect::Damage{..})) || r.amount==0 { continue; }
                let mut tokens=vec![]; let mut token=s.card(r.target).shield;
                while token!=0 { tokens.push(token); token=s.card(token).next_token; }
                tokens.reverse();
                if let Some(first)=tokens.first() { s.event_group.records[i].token=*first; }
                for token in tokens.into_iter().skip(1) {
                    let len=s.event_group.shield_stash_len as usize;
                    s.event_group.shield_stash[len]=token; s.event_group.shield_stash_len+=1;
                }
            }
        }
        if !self.resolve_group_shield_replacements(s, e) { return Ok(false); }
        // All explicit defeat registrations see the same pre-movement event state.
        let mut explicit_sources=vec![];
        for r in &s.event_group.records[..s.event_group.count as usize] {
            if matches!(&ops[r.op as usize],GroupOp::Leaf(Effect::Defeat{..})) && self.in_arena(s,r.target) && !explicit_sources.contains(&r.target) {explicit_sources.push(r.target);}
        }
        for &id in &explicit_sources {self.capture_explicit_death(s,id)?;}
        // Cleanup contingents belong to the whole prepared window. The pinned
        // unary event-order comparator reverses their combined insertion order
        // ahead of every primary handler, rather than interleaving per parent.
        let cleanup=s.event_group.records[..s.event_group.count as usize].iter().filter(|r|matches!(&ops[r.op as usize],GroupOp::Leaf(Effect::Defeat {..})) && self.in_arena(s,r.target)).flat_map(|r|self.upgrades(s,r.target)).collect::<Vec<_>>();
        for upgrade in cleanup.into_iter().rev() {if s.card(upgrade).parent!=0 {self.leave_play_primary(s,upgrade)?;}}
        // Generation can read the preceding step's numeric event context.
        // Reset the new group's result only after those values are captured.
        s.ability_result=0;s.ability_succeeded=false;
        let mut events=StepEventResult::for_state(s);
        s.event_group.resolving=true;
        // Attachment, unit-entry and draw-fatigue contingents retain record order
        // after every primary handler has finished.
        let count=s.event_group.count as usize;
        for contingent in [false,true] {
            for index in 0..count {
                let mut r=s.event_group.records[index];
                let GroupOp::Leaf(effect)=&ops[r.op as usize] else { unreachable!() };
                let attachment=matches!(effect,Effect::GiveShield {..}|Effect::GiveExperience {..}|Effect::GiveWeakness {..}|Effect::GiveAdvantage {..}|Effect::CreateTokenUnit {..});
                let applies=if contingent {attachment || matches!(effect,Effect::Draw {..})} else {!attachment};
                if !applies {continue;}
                if let Effect::CardRestriction { restriction, .. } = effect {
                    let bit=restriction.bit();
                    if self.in_arena(s,r.target) || self.def(s,r.target).info.is("base") {
                        if s.card(r.target).phase_restrictions & bit == 0 {
                            s.card_mut(r.target).phase_restrictions |= bit;
                            s.ability_result=s.ability_result.saturating_add(1);
                        }
                        s.ability_succeeded=true;
                    }
                    continue;
                }
                match effect {
                    Effect::DiscardDeck {..}=> {
                        if self.def(s,r.target).info.is("base") {s.ability_succeeded=true;}
                        else {let owner=s.card(r.target).owner;if s.player_mut(owner).deck.remove(r.target) {s.player_mut(owner).discard.push(r.target)?;s.ability_result=s.ability_result.saturating_add(1);}}
                        continue;
                    }
                    Effect::DiscardOwnHand {..}=> {
                        let owner=s.card(r.target).owner;
                        if s.player_mut(owner).hand.remove(r.target) {s.player_mut(owner).discard.push(r.target)?;s.ability_result=s.ability_result.saturating_add(1);s.ability_succeeded=true;}
                        continue;
                    }
                    Effect::Draw {target,..}=> {
                        let draw=CapturedDraw::from_record(r);
                        if contingent {self.apply_draw_fatigue(s,draw.base,draw.fatigue);continue;}
                        let (amount,resolved)=self.apply_scalar_effect(s,effect,self.effect_targets(s,target),draw.requested,None)?;
                        s.ability_result=s.ability_result.saturating_add(amount);s.ability_succeeded|=resolved;continue;
                    }
                    Effect::CreateTokenUnit {..}=> {
                        s.card_mut(r.token).set_hidden_qualified(true);self.add_arena(s,r.token)?;s.phase_history.entered(s.card(r.token).owner,r.token);
                        s.ability_result=s.ability_result.saturating_add(1);s.ability_succeeded=true;continue;
                    }
                    Effect::GainForce {..}=> {
                        let player=[Seat::P1,Seat::P2].into_iter().find(|player|s.player(*player).base==r.target).ok_or("captured Force player identity missing")?;
                        if !s.force_state_exact||s.player(player).force_id==0||self.force_def.is_none(){return Err("Force requires exact reviewed identity".into());}
                        if !s.player(player).force_held{s.player_mut(player).force_held=true;s.ability_result=s.ability_result.saturating_add(1);}s.ability_succeeded=true;continue;
                    }
                    _=>{},
                }
                if !self.in_arena(s,r.target)&&!self.def(s,r.target).info.is("base") {continue;}
                let damage=matches!(effect,Effect::Damage {..});
                if r.token!=0 {
                    if damage {self.remove_group_shield(s,r.target,r.token);continue;}
                    if !matches!(effect,Effect::GiveWeakness {..}|Effect::GiveAdvantage {..}) {
                        let card=s.card_mut(r.target);
                        let head=if matches!(effect,Effect::GiveExperience {..}) {&mut card.experience}else{&mut card.shield};
                        let previous=std::mem::replace(head,r.token);
                        s.card_mut(r.token).next_token=previous;
                    }
                    self.attach(s,r.token,r.target);continue;
                }
                s.event_group.context=Selection::one(r.target);s.ability_target=r.target;
                if damage {
                    let hp=if self.def(s,r.target).info.is("base") {self.def(s,r.target).info.hp.unwrap_or(30).max(0) as u16}else{self.hp(s,r.target)};
                    r.amount=r.amount.min(hp.saturating_sub(s.card(r.target).damage));
                }
                // These records already resolved their scalar fields and physical
                // recipient. No Effect reconstruction or expression reevaluation.
                let modifier=match effect {
                    Effect::ModifyStats {..}=>Some(PhaseStatDelta {power:i64::from(r.power),hp:i64::from(r.hp)}),
                    Effect::Damage {..}|Effect::Heal {..}|Effect::Ready {..}|Effect::Exhaust {..}|Effect::Defeat {..}
                    |Effect::GainKeyword {..}|Effect::LoseKeyword {..}=>None,
                    _=>return Err("unsupported captured primitive".into()),
                };
                let (amount,resolved)=self.apply_scalar_effect(s,effect,vec![Eval::Card(r.target)],r.amount,modifier)?;
                events.damage_dealt=events.damage_dealt.checked_add(s.event_result.damage_dealt).ok_or("actual damage result exceeds capacity")?;
                for p in 0..2 {events.defeated_units[p]=events.defeated_units[p].checked_add(s.event_result.defeated_units[p]).ok_or("defeat result exceeds capacity")?;}
                if damage {observations.observe(self,s,r.target,amount);}
                s.ability_result=s.ability_result.saturating_add(amount);s.ability_succeeded|=resolved;
            }
        }
        let created=s.event_group.records[..s.event_group.count as usize].iter().filter(|r|matches!(&ops[r.op as usize],GroupOp::Leaf(Effect::CreateTokenUnit{..}))).map(|r|r.token).collect::<Vec<_>>();
        for source in created {
            let flags=self.effective_flags(s,source);
            for keyword in [T_SHIELD,T_AMBUSH] {
                if (keyword==T_SHIELD && flags&SHIELDED==0)||(keyword==T_AMBUSH&&flags&AMBUSH==0) {continue;}
                let n=s.death_pending.len as usize;if n==MAX_DEATH_TRIGGERS {return Err("created token trigger capacity exceeded".into());}
                s.death_pending.entries[n]=DeathTrigger {source,keyword,player:s.card(source).owner,after_pc:self.program(s.ability_program-1).event_end,defer_depth:s.death_depth,..DeathTrigger::default()};s.death_pending.len+=1;
            }
        }
        s.event_result=events;
        s.event_group.resolving=false;
        observations.finish(self,s)?;
        Ok(true)
    }
    pub(super) fn finish_group_damage(&self,s:&mut State,damage_callbacks:Vec<CardId>)->Result<(),String> {
        // Damage callbacks capture their lethal set in handler/selection order,
        // before any queued source defeat can change another unit's HP aura.
        let mut damage_defeats=vec![];
        for id in damage_callbacks {
            if self.in_arena(s,id) && s.card(id).damage>=self.hp(s,id) && !damage_defeats.contains(&id) {damage_defeats.push(id);}
        }
        self.check_stat_capacity(s)?; self.check_win(s);
        // Framework HP checks queue their deaths before the defeat subwindow
        // removes any pending source; a negative aura cannot rescue them mid-batch.
        let framework_dead=self.arena_units(s).into_iter()
            .filter(|id|s.card(*id).damage>=self.hp(s,*id)).collect::<Vec<_>>();
        for id in framework_dead { if !damage_defeats.contains(&id) { damage_defeats.push(id); } }
        self.defeat_many(s,&damage_defeats)?;
        self.sweep_stat_defeats(s)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine() -> Engine {
        Engine::from_catalogue_bytes(include_bytes!(concat!(env!("OUT_DIR"), "/catalogue.json"))).unwrap()
    }
    fn add(engine: &Engine, s: &mut State, seat: Seat, code: &str) -> CardId {
        let id=s.next_id;s.next_id+=1;
        *s.card_mut(id)=Card {present:true,def:engine.by_code[code],owner:seat,..Card::default()};id
    }
    fn position(engine: &Engine) -> State {
        let mut s=State::default();s.hidden_history_exact=true;s.force_state_exact=true;
        for (seat,leader) in [(Seat::P1,"SOR_014"),(Seat::P2,"IBH_053")] {
            let base=add(engine,&mut s,seat,"ASH_019");let leader=add(engine,&mut s,seat,leader);
            s.player_mut(seat).base=base;s.player_mut(seat).leader=leader;
            s.player_mut(seat).force_id=s.next_id;s.next_id+=1;
        }
        s
    }

    #[test]
    fn prepared_values_survive_source_changes_before_execution() {
        let e=engine();
        for kind in ["damage","heal","stats"] {
            let mut s=position(&e);
            let source=add(&e,&mut s,Seat::P1,"SHD_056");e.add_arena(&mut s,source).unwrap();
            let target=if kind=="stats" {let id=add(&e,&mut s,Seat::P2,"SHD_056");e.add_arena(&mut s,id).unwrap();id}else{s.players[1].base};
            s.source=source;s.ability_target=target;s.card_mut(source).damage=1;
            s.card_mut(target).damage=if kind=="stats" {0}else{5};
            s.event_group.active=true;s.event_group.context=Selection::one(target);
            let dynamic=Expr::Ref {path:vec!["source".into(),"damage".into()]};
            let effect=match kind {
                "damage"=>Effect::Damage {target:Target::name("chosen"),amount:dynamic},
                "heal"=>Effect::Heal {target:Target::name("chosen"),amount:dynamic},
                _=>Effect::ModifyStats {target:Target::name("chosen"),power:dynamic.clone(),hp:dynamic,duration:"phase".into()},
            };
            e.capture_group_leaf(&mut s,0,&effect).unwrap();assert_eq!(s.event_group.count,1);
            let records=s.event_group.records;
            s.card_mut(source).damage=2;
            let group=Effect::Simultaneous {effects:vec![effect]};
            assert!(e.resolve_group(&mut s,&group).unwrap());
            assert_eq!(records,s.event_group.records,"capture records remain the stored representation");
            match kind {
                "damage"=>assert_eq!(s.card(target).damage,6),
                "heal"=>assert_eq!(s.card(target).damage,4),
                _=>{assert_eq!(s.card(target).phase_power,1);assert_eq!(s.card(target).phase_hp,1);},
            }
            assert_eq!(s.ability_result,1);assert!(s.ability_succeeded);
        }
    }

    #[test]
    fn scalar_and_group_domains_stay_distinct_with_shared_fields() {
        let e=engine();let s=position(&e);
        for effect in [Effect::ReadyResources {target:Target::name("ownPlayer"),amount:Expr::number(1)},
            Effect::ReturnToHand {target:Target::name("self")},Effect::UseForce {target:Target::name("ownPlayer")}] {
            assert!(effect.scalar_args().is_some());let mut work=s;
            assert_eq!(e.capture_group_leaf(&mut work,0,&effect).unwrap_err(),"unsupported group primitive");
        }
        for effect in [Effect::DiscardDeck {target:Target::name("ownPlayer"),amount:Expr::number(1)},
            Effect::DiscardOwnHand {target:Target::name("ownPlayer"),amount:Expr::number(1)},
            Effect::CreateTokenUnit {token_type:"battleDroid".into(),amount:Expr::number(1),target:Target::name("ownPlayer"),enters_ready:false}] {
            assert!(effect.scalar_args().is_some());let mut work=s;
            assert_eq!(e.apply_effect(&mut work,&effect).unwrap_err(),"nonprimitive instruction reached executor");
        }
    }

    #[test]
    fn visibility_policies_cannot_be_collapsed_across_healing_and_force_hp_changes() {
        let e=engine();
        for capture_lethals in [false,true] {
            let mut s=position(&e);let unit=add(&e,&mut s,Seat::P1,"SHD_056");e.add_arena(&mut s,unit).unwrap();
            s.card_mut(unit).damage=e.hp(&s,unit);
            let mut observations=DamageObservations::new(capture_lethals);observations.observe(&e,&s,unit,1);
            // A later healing primary rescues the unit in an ordinary window.
            s.card_mut(unit).damage=0;observations.finish(&e,&mut s).unwrap();
            assert_eq!(e.in_arena(&s,unit),!capture_lethals);

            let mut s=position(&e);let aura=add(&e,&mut s,Seat::P1,"SEC_032");e.add_arena(&mut s,aura).unwrap();
            let unit=add(&e,&mut s,Seat::P1,"LOF_196");e.add_arena(&mut s,unit).unwrap();assert_eq!(e.hp(&s,unit),4);
            s.card_mut(unit).damage=4;
            let mut observations=DamageObservations::new(capture_lethals);observations.observe(&e,&s,unit,1);
            s.players[0].force_held=true;assert_eq!(e.hp(&s,unit),6);
            observations.finish(&e,&mut s).unwrap();
            // Force's cached damage-handler lethal must survive the HP gain.
            assert_eq!(e.in_arena(&s,unit),!capture_lethals);
        }
    }

    #[test]
    fn visibility_contract_matches_every_admitted_prepared_group() {
        let e=engine();let mut groups=0;
        for program in &e.catalogue.as_ref().unwrap().programs {
            for instruction in &program.instructions {
                let Some(effect)=instruction.group_effect() else {continue;};groups+=1;
                let legacy=matches!(effect,Effect::Simultaneous {effects} if effects.len()==2
                    &&matches!(&effects[0],Effect::Select {effect,..} if matches!(effect.as_ref(),Effect::Damage {..}))
                    &&matches!(&effects[1],Effect::GainForce {..}));
                assert_eq!(damage_before_force(effect),legacy);
            }
        }
        assert!(groups>10);
    }

}
