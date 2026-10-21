//! Exact stable registration counters; suspended limit graphs are not inferred.
use super::*;
use crate::round_limit_contract::Registration;
impl Engine {
    pub(in crate::simulator) fn needs_round_limit_state(&self)->bool {
        self.selected_definition_any(|d|d.programs.card.iter().any(|p|Registration::from_trigger(&self.program(*p).trigger).is_some()))
    }
    fn physical_round_limits(&self,s:&State)->Result<Vec<(CardId,Registration)>,String> {
        let mut out=Vec::new();
        for source in 1..s.next_id {if !s.card(source).present {continue;}
            let mut found=false;
            for (ordinal,&p) in self.card_programs(s,source).iter().enumerate() {if let Some(r)=Registration::from_trigger(&self.program(p).trigger) {
                if found||ordinal!=usize::from(r.ordinal()) {return Err("round-limit physical registration exceeds closed multiplicity".into());}
                found=true;out.push((source,r));
            }}
        }Ok(out)
    }
    pub(in crate::simulator) fn load_round_limit_state(&self,s:&mut State,v:&Value)->Result<(),String> {
        if v.get("roundLimitPending").is_some() {return Err("pending round-limit graph is not importable".into());}
        let Some(raw)=v.get("roundLimitState") else {
            if self.needs_round_limit_state()||!self.physical_round_limits(s)?.is_empty() {return Err("selected or imported pool requires exact roundLimitState version 1".into());}
            return Ok(());
        };
        let fail=||"roundLimitState requires complete exact current native registration counters".to_string();
        let o=raw.as_object().ok_or_else(fail)?;
        if o.len()!=2||raw["version"].as_u64()!=Some(1)||!o.contains_key("registrations") {return Err(fail());}
        let rows=raw["registrations"].as_array().ok_or_else(fail)?;
        let expected=self.physical_round_limits(s)?;
        if rows.len()!=expected.len(){return Err(fail());}
        let optional_presence=!self.needs_round_limit_state() && expected.is_empty();
        let mut ledger=crate::round_limit_state::Ledger::default();
        for (row,(source,r)) in rows.iter().zip(expected) {
            let o=row.as_object().ok_or_else(fail)?;
            let used=row["used"].as_array().ok_or_else(fail)?;
            if o.len()!=4||!o.contains_key("card")||!o.contains_key("side")||!o.contains_key("ordinal")||!o.contains_key("used")||row["card"]!=format!("c{}",source-1)||row["side"]!="card"||row["ordinal"].as_u64()!=Some(u64::from(r.ordinal()))||used.len()!=2||used.iter().any(|n|!matches!(n.as_u64(),Some(0|1))) {return Err(fail());}
            let h=ledger.native(source,r.ordinal())?;
            for (player,count) in used.iter().enumerate() {if count.as_u64()==Some(1){ledger.consume(h,player as u8)?;}}
        }
        ledger.normalize_quiescent(&[])?;s.round_limits=ledger;
        // Selected producers or retained physical rows already force output.
        // Preserve only otherwise-optional explicit empty evidence. Actions do
        // not clear an existing flag; closed ordinary sources retain physical
        // definitions offboard, so their rows cannot vanish through zone changes.
        s.round_limit_wire_present |= optional_presence && s.round_limits.len==0;Ok(())
    }
    pub(in crate::simulator) fn snapshot_round_limit_state(&self,s:&State,v:&mut Value) {
        if !crate::round_limit_runtime::roots(s).is_empty() {v["roundLimitPending"]=json!(true);}
        if s.round_limit_wire_present||self.needs_round_limit_state()||s.round_limits.len!=0||!self.physical_round_limits(s).expect("validated round-limit physical registrations").is_empty() {
            let rows=self.physical_round_limits(s).expect("validated round-limit physical registrations").into_iter().map(|(source,r)| {
                let used=s.round_limits.current(source,r.ordinal()).map(|h|s.round_limits.instance(h).expect("current native limit").used_by_player).unwrap_or(0);
                json!({"card":format!("c{}",source-1),"side":"card","ordinal":r.ordinal(),"used":[u8::from(used&1!=0),u8::from(used&2!=0)]})
            }).collect::<Vec<_>>();
            v["roundLimitState"]=json!({"version":1,"registrations":rows});
        }
    }
}

impl Engine {
    pub(in crate::simulator) fn validate_round_limit_handles(&self,s:&State)->Result<(),String> {
        use crate::round_limit_state::Handle;
        let check=|source:CardId,program:Option<u16>,player:Seat,h:Handle|->Result<(),String> {
            let registration=program.and_then(|p|Registration::from_trigger(&self.program(p).trigger));
            if h==Handle::NONE {if registration.is_some(){return Err("captured round-limit registration instance is missing".into());}return Ok(());}
            let instance=s.round_limits.instance(h)?;
            if registration.is_none_or(|r|r.ordinal()!=instance.registration_ordinal)||u16::from(instance.source_index)!=source {return Err(format!("round-limit handle does not belong to captured physical registration: source={source}, program={program:?}, registration={registration:?}, instance={instance:?}"));}
            if s.card(source).owner!=player {return Err("captured round-limit controller change requires an unsupported controller graph".into());}
            Ok(())
        };
        check(s.source,s.ability_program.checked_sub(1),s.active,s.round_limit)?;
        let context=|c:&ResolutionContext|check(c.source,c.ability_program.checked_sub(1),c.active,c.round_limit);
        context(&s.support.context)?;
        for f in &s.death_frames {context(&f.context)?;}
        for w in [&s.death_pending,&s.death_window].into_iter().chain(s.death_frames.iter().map(|f|&f.window)) {
            for e in &w.entries {if e.source!=0||e.round_limit!=Handle::NONE {check(e.source,if e.keyword==0{Some(e.program)}else{None},e.player,e.round_limit)?;}}
        }
        Ok(())
    }
}

#[cfg(test)]
mod wire_presence_tests {
    use super::*;
    #[test]
    fn optional_pool_retained_physical_registration_wire_reload_is_canonical() {
        for backend in [RulesBackend::Auto,RulesBackend::Interpreter] {
            let e=crate::round_limit_tests::ordinary139_wire_engine(backend,false);
            let mut fresh=e.load_state(&crate::round_limit_tests::ordinary139_wire_state()).unwrap();
            let id=fresh.next_id;fresh.next_id+=1;
            *fresh.card_mut(id)=Card{present:true,def:e.by_code["ASH_047"],owner:Seat::P1,..Card::default()};
            fresh.player_mut(Seat::P1).hand.push(id).unwrap();
            assert!(!e.needs_round_limit_state());assert!(!fresh.round_limit_wire_present);
            let snapshot=e.snapshot(&fresh);assert_eq!(snapshot["roundLimitState"]["registrations"].as_array().unwrap().len(),1);
            // The selected pool's public importer still rejects unknown physical
            // sources. Isolate the wire helper on the already trusted physical
            // State to discriminate unnecessary presence flags from ledger data.
            assert!(e.load_state(&snapshot).is_err());
            let mut loaded=fresh;e.load_round_limit_state(&mut loaded,&snapshot).unwrap();
            assert_eq!(fresh,loaded);assert_eq!(fresh.zobrist(),loaded.zobrist());assert_eq!(e.snapshot(&loaded),snapshot);
            let mut sticky=fresh;sticky.round_limit_wire_present=true;
            e.load_round_limit_state(&mut sticky,&snapshot).unwrap();assert!(sticky.round_limit_wire_present);
            assert_eq!(sticky.zobrist(),fresh.zobrist());assert_eq!(e.snapshot(&sticky),snapshot);
            let mut missing=snapshot.clone();missing["roundLimitState"]["registrations"]=json!([]);
            assert!(e.load_round_limit_state(&mut loaded,&missing).is_err());assert_eq!(loaded,fresh);
        }
    }
}
