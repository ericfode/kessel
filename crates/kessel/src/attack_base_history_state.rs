//! Attack-local facts are copied with their owning resolution context.
//! The original six lifecycle booleans retain independent bits and hash order.
#[derive(Debug,Clone,Copy,Default,PartialEq,Eq)]
pub struct AttackFlags(u16);
impl AttackFlags {
    pub fn end_captured(self)->bool {self.0 & (1<<0) != 0}
    pub fn set_end_captured(&mut self,value:bool) {if value {self.0|=1<<0;}else{self.0&=!(1<<0);}}
    pub fn attacker_defeated(self)->bool {self.0 & (1<<1) != 0}
    pub fn set_attacker_defeated(&mut self,value:bool) {if value {self.0|=1<<1;}else{self.0&=!(1<<1);}}
    pub fn attacker_departed(self)->bool {self.0 & (1<<2) != 0}
    pub fn set_attacker_departed(&mut self,value:bool) {if value {self.0|=1<<2;}else{self.0&=!(1<<2);}}
    pub fn defender_defeated(self)->bool {self.0 & (1<<3) != 0}
    pub fn set_defender_defeated(&mut self,value:bool) {if value {self.0|=1<<3;}else{self.0&=!(1<<3);}}
    pub fn defender_departed(self)->bool {self.0 & (1<<4) != 0}
    pub fn set_defender_departed(&mut self,value:bool) {if value {self.0|=1<<4;}else{self.0&=!(1<<4);}}
    pub fn shield_opponent_first(self)->bool {self.0 & (1<<5) != 0}
    pub fn set_shield_opponent_first(&mut self,value:bool) {if value {self.0|=1<<5;}else{self.0&=!(1<<5);}}
    pub fn record_exists(self)->bool {self.0 & (1<<6)!=0}
    pub fn set_record_exists(&mut self) {self.0|=1<<6;}
    pub fn attacking_player(self)->crate::model::Seat {if self.0 & (1<<7)!=0 {crate::model::Seat::P2}else{crate::model::Seat::P1}}
    pub fn set_attacking_player(&mut self,seat:crate::model::Seat) {if seat==crate::model::Seat::P2 {self.0|=1<<7;}else{self.0&=!(1<<7);}}
    pub fn reentered(self)->bool {self.0 & (1<<8)!=0}
    pub fn mark_reentered(&mut self) {self.0|=1<<8;}
    pub fn post_captured(self)->bool {self.0 & (1<<9)!=0}
    pub fn set_post_captured(&mut self) {self.0|=1<<9;}
    pub fn enabled(self)->bool {self.0 & (1<<10)!=0}
    pub fn enable(&mut self) {self.0|=1<<10;}
    pub fn reset_history(&mut self) {self.0&=63;}
    pub fn history_bits(self)->u16 {self.0 & !63}
    pub fn mark_departure(&mut self,source:bool,defeat:bool) {
        if source {if !self.attacker_departed() {if defeat {self.set_attacker_defeated(true);}else{self.set_attacker_departed(true);}}}
        else if !self.defender_departed() {if defeat {self.set_defender_defeated(true);}else{self.set_defender_departed(true);}}
    }
}

/// Phase declaration projection; bit2 records optional explicit wire presence.
#[derive(Debug,Clone,Copy,Default,PartialEq,Eq)]
pub struct History(u8);
impl History {
 pub fn opponent_attacked(self,seat:crate::model::Seat)->bool {self.0 & (1<<seat.index())!=0}
 pub fn record(&mut self,seat:crate::model::Seat) {self.0|=1<<seat.index();}
 pub fn explicit(self)->bool {self.0 & 4!=0}
 pub fn set_explicit(&mut self) {self.0|=4;}
 pub fn reset(&mut self) {self.0&=4;}
 pub fn bits(self)->u8 {self.0 & 3}
}
#[cfg(test)]
mod layout {
 #[test] fn attack_fact_layout() {
  use std::mem::{size_of,align_of};use crate::model::*;
  println!("State {} align{} Context {} Frame {} Trigger {} Flags {}",size_of::<State>(),align_of::<State>(),size_of::<ResolutionContext>(),size_of::<DeathFrame>(),size_of::<DeathTrigger>(),size_of::<super::AttackFlags>());
  assert!(size_of::<State>()<=38312);
 }
}
