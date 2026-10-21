//! Exact first direct hand UnitPlay Credit boundary. Staging never executes a
//! play/payment action; all board/evidence validation precedes publication.
use super::*;
use crate::direct_unit_credit_contract::{self as contract,Descriptor,Stage};
use crate::resource_payment::{PaymentPurpose,PaymentStage,ResourcePayment};
const FIELD:&str="directUnitPlayCreditState";
fn fail()->String {"unsupported direct unit Credit continuation".into()}
impl Engine {
    fn direct_credit_idle_roots(&self,s:&State,context:ResolutionContext)->bool {
        ResolutionContext::capture(s)==context
            && s.support==SupportState::default()
            && s.death_depth==0 && s.death_pending==DeathWindow::default()
            && s.death_window==DeathWindow::default()
            && s.death_frames.iter().all(|f|*f==DeathFrame::default())
            && s.event_group==EventGroupState::default() && s.payment_group==EventGroupState::default()
            && s.disclose==DiscloseState::default() && !s.regroup && s.outcome==0
            && !s.enter_smuggle && s.disabled_ongoing_source==0
            && s.resource_done==[false;2] && s.resource_selected==[0;2]
    }
    fn direct_credit_payment(&self,s:&State,d:&Descriptor)->Result<ResourcePayment,String>{
        let payer=if d.payer==0 {Seat::P1}else{Seat::P2};let id=d.source;
        if s.active!=payer || s.action_player!=Some(payer) || s.player(payer).out
            || id==0 || id>=s.next_id || !s.card(id).present || s.card(id).owner!=payer
            || !s.player(payer).hand.contains(id) || !s.credit_state_exact
            || !self.def(s,id).info.is("unit") || self.def(s,id).info.is("leader")
            || self.def(s,id).info.is("token") || !self.named_play_allowed(s,id)
            || self.def(s,id).info.has_keyword("plot") || self.def(s,id).info.has_keyword("piloting")
            || !self.named_play_allowed(s,id) {return Err(fail());}
        self.validate_credit_account(s,payer)?;
        let amount=self.cost(s,id);let credits=self.available_credit_count(s,payer);
        let ready=u16::try_from(self.ready_resources(s,payer)).map_err(|_|fail())?;
        let minimum=amount.saturating_sub(ready);let maximum=credits.min(amount);
        if amount!=d.amount || amount==0 || credits==0 || u32::from(amount)>u32::from(ready)+u32::from(credits)
            || minimum!=d.minimum || maximum!=d.maximum || maximum==0 {return Err(fail());}
        let mask=self.phase_cost_capture(s,id);
        let actual:Vec<u8>=(0..64).filter(|i|mask.0[i/32]&(1u32<<(i%32))!=0).map(|i|(i+1)as u8).collect();
        if actual!=d.mouse_ids {return Err("incomplete or foreign captured Mouse set".into());}
        s.phase_history.preflight_play(payer)?;
        s.player_phase.preflight(self.phase_direct_required(s,id,"whenPlayed"))?;
        // Check entry representation on a local copy before retaining a payment.
        let mut entry=*s;self.add_arena(&mut entry,id)?;
        Ok(ResourcePayment{phase_adjusters:mask,purpose:PaymentPurpose::UnitPlay,
            stage:if d.stage==Stage::Menu {PaymentStage::Credits}else{PaymentStage::Amount},
            payer,actor:payer,source:id,target:0,amount,minimum,maximum,can_cancel:true,
            ..ResourcePayment::default()})
    }
    pub(super) fn load_direct_unit_credit(&self,v:&Value)->Result<State,String>{
        let d=contract::parse(&v[FIELD])?;
        if v.get("actor")!=Some(&Value::Null) || v["activePlayer"]!=if d.payer==0 {"p1"}else{"p2"}
            || v["phase"]!="action" || v["winner"]!=Value::Null
            || v.get("creditStateVersion").is_none(){return Err(fail());}
        // This shared importer still rejects EVERY old pending marker by
        // presence, validates all legacy evidence and never strips a marker.
        // Phase proof is independently required by selected/physical producers
        // in the ordinary board loader; direct Credit needs no transient empty registry.
        let mut s=self.load_action_board(v)?;
        let mut idle=State::default();idle.active=s.active;
        if !self.direct_credit_idle_roots(&s,ResolutionContext::capture(&idle)){return Err(fail());}
        let request=self.direct_credit_payment(&s,&d)?;
        if !self.legal_actions(&s).iter().any(|a|a.id()==Some(d.source)){return Err(fail());}
        s.source=d.source;s.resource_payment=request;
        s.decision=if d.stage==Stage::Menu{Decision::CreditMenu}else{Decision::CreditAmount};
        Ok(s)
    }
    pub(super) fn snapshot_direct_unit_credit(&self,s:&State,v:&mut Value)->bool {
        if v.get("creditStateVersion").is_none(){return false;}
        let stage=match s.decision{Decision::CreditMenu=>Stage::Menu,Decision::CreditAmount=>Stage::Amount,_=>return false};
        let p=s.resource_payment;let mouse_ids=(0..64).filter(|i|p.phase_adjusters.0[i/32]&(1u32<<(i%32))!=0).map(|i|(i+1)as u8).collect();
        let d=Descriptor{source:s.source,payer:s.active.index()as u8,stage,amount:p.amount,mouse_ids,minimum:p.minimum,maximum:p.maximum};
        let mut expected=State::default();expected.active=s.active;expected.source=s.source;expected.decision=s.decision;expected.resource_payment=p;
        if !self.direct_credit_idle_roots(s,ResolutionContext::capture(&expected)) || self.direct_credit_payment(s,&d).as_ref()!=Ok(&p){return false;}
        let value=json!({"version":1,"source":format!("c{}",d.source-1),"payer":s.active.as_str(),"stage":if stage==Stage::Menu{"menu"}else{"amount"},"preCreditAmount":d.amount,"capturedMouseIds":d.mouse_ids,"menu":{"minimum":d.minimum,"maximum":d.maximum,"canCancel":true}});
        if contract::parse(&value).is_err(){return false;}
        v[FIELD]=value;
        // These two markers are replaced ONLY after the entire exact native
        // root and its full captured programme/quote domain passed above.
        v.as_object_mut().unwrap().remove("creditPaymentPending");
        v.as_object_mut().unwrap().remove("playerPhasePending");
        true
    }
}
