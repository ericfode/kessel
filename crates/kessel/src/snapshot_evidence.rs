//! Snapshot evidence is explicit and capability-specific. Missing legacy
//! histories remain unknown: current boards or newly observed suffixes cannot
//! reconstruct exact past chronology. Schema checks run before these loaders;
//! registration, arena and Support evidence retain their own validators.
use super::*;
impl Engine {
    pub(super) fn load_hidden_evidence(&self, s: &mut State, v: &Value) -> Result<(), String> {
        s.hidden_history_exact = v.get("hiddenHistoryVersion").is_some();
        if !s.hidden_history_exact && self.needs_hidden_history() {
            return Err("selected pool requires explicit Hidden phase history".into());
        }
        Ok(())
    }
    pub(super) fn load_force_evidence(&self, s: &mut State, v: &Value) -> Result<(), String> {
        if let Some(version) = v.get("forceStateVersion") {
            if version.as_u64() != Some(1) {
                return Err("unsupported forceStateVersion".into());
            }
            s.force_state_exact = true;
            for seat in [Seat::P1, Seat::P2] {
                let token = &v["players"][seat.as_str()]["forceToken"];
                s.player_mut(seat).force_id =
                    cid(token["id"].as_str().ok_or("Force identity missing")?)?;
                s.player_mut(seat).force_held = token["held"]
                    .as_bool()
                    .ok_or("Force held must be boolean")?;
            }
            if self.force_def.is_none() && s.players.iter().any(|p| p.force_held) {
                return Err("held Force requires reviewed native definition".into());
            }
        } else if self.needs_force_state() {
            return Err("selected pool requires explicit Force state version".into());
        }
        Ok(())
    }
    pub(super) fn load_phase_history_evidence(
        &self,
        s: &mut State,
        v: &Value,
    ) -> Result<(), String> {
        if let Some(history) = v.get("phaseHistory") {
            s.phase_history = crate::model::PhaseHistory::parse(history, s.next_id)?;
            for seat in [Seat::P1, Seat::P2] {
                for id in 1..s.next_id {
                    if crate::model::PhaseHistory::contains(&s.phase_history.played, seat, id)
                        && (!s.card(id).present
                            || self.def(s, id).info.is("base")
                            || self.def(s, id).info.is("leader")
                            || self.def(s, id).info.is("token"))
                    {
                        return Err("played phase history requires retained playable physical card metadata".into());
                    }
                    if crate::model::PhaseHistory::contains(&s.phase_history.entered, seat, id)
                        && (s.players.iter().any(|p| p.base == id || p.force_id == id)
                            || (s.card(id).present
                                && !self.def(s, id).info.is("unit")
                                && !self.def(s, id).info.is("leader")))
                    {
                        return Err("entry phase history requires a physical unit identity".into());
                    }
                }
            }
        } else {
            s.phase_history.exact = false;
            s.phase_history.defeats_exact = false;
            if self.needs_phase_history() {
                return Err("selected pool requires explicit phaseHistory version".into());
            }
        }
        if (!s.phase_history.exact || !s.phase_history.defeats_exact)
            && (self.needs_defeat_history() || self.state_needs_defeat_history(s))
        {
            return Err("selected or imported pool requires exact defeat phase history version 2".into());
        }
        Ok(())
    }
    pub(super) fn write_snapshot_evidence(&self, s: &State, value: &mut Value) {
        // The wire has no proven current-attack grant provenance adapter. Keep
        // retained native diagnostics explicit and unimportable; normal attack
        // cleanup removes this marker together with its lasting-effect grants.
        if s.support.grants[..s.support.len as usize].iter().any(|g|!g.native_attack&&g.conditional_program!=0&&g.conditional_mask==0){value["onAttackModifiersPending"]=json!(true);}

        if s.force_state_exact {
            value["forceStateVersion"] = json!(1);
            for seat in [Seat::P1, Seat::P2] {
                let p = s.player(seat);
                value["players"][seat.as_str()]["forceToken"] =
                    json!({"id":format!("c{}",p.force_id-1),"held":p.force_held});
            }
        }
        if s.hidden_history_exact {
            value["hiddenHistoryVersion"] = json!(1);
        }
        if s.phase_history.damage_exact {value["damagedTargetHistory"]=self.damaged_target_history_snapshot(s);}
        if s.phase_history.exact {
            value["phaseHistory"] = s.phase_history.snapshot();
        }
        if s.registration_order.exact {
            value["registrationOrder"] = json!({"version":1,"cards":s.registration_order.as_slice().iter().map(|id|format!("c{}",*id as u16-1)).collect::<Vec<_>>()});
        }
        if s.arena_order_exact {
            value["arenaUnitOrder"] = json!({"ground":s.arena_unit_order[0].as_slice().iter().map(|id|format!("c{}",id-1)).collect::<Vec<_>>(),"space":s.arena_unit_order[1].as_slice().iter().map(|id|format!("c{}",id-1)).collect::<Vec<_>>()});
        }
    }
}
