//! Executor for the closed selected-number payment contract. No cost adjustment,
//! Credit menu, payment-result coercion, or card-ID selection is involved.
use super::*;
impl Engine {
    fn number_max(&self, s: &State) -> u16 {
        let ready=self.ready_resources(s,s.card(s.source).owner);
        if s.ability_program!=0&&crate::resource_number_experience_contract::requires(self.program(s.ability_program-1)) {ready as u16} else {ready.min(6) as u16}
    }
    pub(super) fn number_actions(&self, s: &State) -> Vec<Action> {
        (0..=self.number_max(s))
            .map(|n| Action {
                kind: "option".into(),
                card: None,
                arg: Some(Value::String(n.to_string())),
                method: None,
                label: format!("#{n}"),
            })
            .collect()
    }
    pub(super) fn apply_number_action(&self, s: &State, a: &Action) -> Result<State, String> {
        if s.ability_program == 0
            || !matches!(
                self.program(s.ability_program - 1)
                    .instructions
                    .get(s.ability_pc as usize),
                Some(Instruction::ChooseNumber)
            )
            || s.selected_number != 0
            || s.active != s.card(s.source).owner
        {
            return Err("invalid selected-number frame".into());
        }
        let text = a
            .arg
            .as_ref()
            .and_then(Value::as_str)
            .ok_or("number choice requires decimal string")?;
        let n = text.parse::<u8>().map_err(|_| "invalid number choice")?;
        if text != n.to_string() || u16::from(n) > self.number_max(s) {
            return Err("number choice outside live inclusive range".into());
        }
        if !self.number_actions(s).contains(a) {
            return Err("illegal number action".into());
        }
        let mut work = *s;
        work.selected_number = n.checked_add(1).ok_or("selected number binding capacity exceeded")?;
        work.ability_pc += 1;
        self.run_program(&mut work)?;
        Ok(work)
    }
    /// Returns true only while waiting for the mandatory number prompt.
    pub(super) fn run_number_instruction(&self, s: &mut State) -> Result<bool, String> {
        let instruction = self
            .program(s.ability_program - 1)
            .instructions
            .get(s.ability_pc as usize)
            .ok_or("number instruction missing")?
            .clone();
        if crate::resource_number_experience_contract::requires(self.program(s.ability_program-1)) {return self.run_captured_resource_number(s,instruction);}
        if matches!(instruction, Instruction::ChooseNumber) {
            if !self.commit_program(s)? {
                return Ok(true);
            }
            if s.selected_number != 0 {
                return Err("number binding already initialized".into());
            }
            s.active = s.card(s.source).owner;
            if self.number_max(s) == 0 {
                s.selected_number = 1;
                s.ability_pc += 1;
                return Ok(false);
            }
            s.decision = Decision::AbilityNumber;
            return Ok(true);
        }
        let n = s
            .selected_number
            .checked_sub(1)
            .filter(|n| *n <= 6)
            .ok_or("selected number missing or invalid")?;
        let effect = match instruction {
            Instruction::PayUnadjustedSelectedResources => {
                if usize::from(n) > self.ready_resources(s, s.card(s.source).owner) {
                    return Err("unadjusted resource cost is unavailable".into());
                }
                Effect::PayResources {
                    target: Target::Name("ownPlayer".into()),
                    amount: Expr::number(i64::from(n)),
                }
            }
            Instruction::GiveSelectedExperience => Effect::GiveExperience {
                target: Target::Name("self".into()),
                amount: Expr::number(i64::from(n)),
            },
            _ => return Err("invalid number instruction".into()),
        };
        let (amount, resolved) = self.apply_effect(s, &effect)?;
        s.ability_result = amount;
        s.ability_succeeded = resolved;
        s.ability_pc += 1;
        Ok(false)
    }
}

// The first Select captures a physical recipient. Continuation viability uses
// ANY arena unit (including a deployed leader), independently of that capture.
// Payment and XP share one nonpausing window. Build the whole result on scratch,
// capture attachment listeners before primaries, allocate all XP, exhaust the
// physical first ready resources, then perform exactly one framework sweep.
impl Engine {
 fn run_captured_resource_number(&self,s:&mut State,instruction:Instruction)->Result<bool,String>{
  if !crate::resource_number_experience_contract::program(self.program(s.ability_program-1)){return Err("invalid captured resource number program".into());}
  if matches!(instruction,Instruction::ChooseNumber){
   if !self.commit_program(s)? {return Ok(true);}
   if s.selected_number!=0{return Err("captured resource number already bound".into());}
   let payer=s.card(s.source).owner;s.active=payer;
   if self.ready_resources(s,payer)==0||self.arena_units(s).is_empty(){s.ability_pc=4;return Ok(false);}
   s.decision=Decision::AbilityNumber;return Ok(true);
  }
  if !matches!(instruction,Instruction::PayUnadjustedSelectedResources){return Err("captured simultaneous instruction entered out of order".into());}
  let n=s.selected_number.checked_sub(1).filter(|n|usize::from(*n)<=MAX_ZONE).ok_or("captured resource number missing or exceeds resource capacity")?;
  let payer=s.card(s.source).owner;if usize::from(n)>self.ready_resources(s,payer){return Err("captured unadjusted resource payment unavailable".into());}
  if s.event_group.active||s.event_group.resolving{return Err("captured resource batch cannot nest a prepared window".into());}
  let target=s.ability_target;
  let recipient=target!=0&&self.in_arena(s,target)&&self.def(s,target).info.is("unit")&&!self.def(s,target).info.is("leader");
  let mut work=*s;
  if recipient&&n>0 {
   self.experience_def.ok_or("catalogue needs Experience token")?;
   if usize::from(work.next_id)+usize::from(n)>MAX_CARDS{return Err("card capacity exceeded creating captured Experience".into());}
   let events=(0..n).map(|offset|CollectiveEvent::UpgradeAttached{parent:target,upgrade:work.next_id+u16::from(offset)}).collect::<Vec<_>>();
   self.capture_collective_window(&mut work,&events)?;
   for _ in 0..n {self.create_token(&mut work,target,true)?;}
  }
  // The existing prepared-window flag suppresses each scalar's intermediate
  // sweep; no prepared records or persistent storage are synthesized here.
  work.event_group.resolving=true;
  let payment=Effect::PayResources{target:Target::Name("ownPlayer".into()),amount:Expr::number(i64::from(n))};
  let (paid,resolved)=self.apply_scalar_effect(&mut work,&payment,vec![Eval::Player(payer)],u16::from(n),None)?;
  work.event_group.resolving=false;
  self.check_stat_capacity(&work)?;self.finish_effect_window(&mut work)?;
  work.ability_result=paid;work.ability_succeeded=resolved&&n>0;work.ability_pc+=2;*s=work;Ok(false)
 }
}
