//! Exact application registry projection. Spent rows are retained until expiry.
use super::*;
use crate::ability_ir::{Effect, Instruction};
use crate::player_phase_contract::Kind;
use crate::player_phase_state::{Ledger, Row, CAPACITY};
impl Engine {
    pub(super) fn needs_player_phase_state(&self) -> bool {
        self.selected_definition_any(|d| {
            d.programs
                .card
                .iter()
                .any(|&p| self.phase_program_kind(p).is_some())
        })
    }
    fn phase_program_kind(&self, p: u16) -> Option<Kind> {
        self.program(p).instructions.iter().find_map(|i| {
            if let Instruction::Apply(Effect::InstallPlayerPhase { phase_kind }) = i {
                Some(*phase_kind)
            } else {
                None
            }
        })
    }
    fn physical_phase_source(&self, s: &State) -> bool {
        (1..s.next_id).any(|id| {
            s.card(id).present
                && self
                    .def(s, id)
                    .programs
                    .card
                    .iter()
                    .any(|&p| self.phase_program_kind(p).is_some())
        })
    }
    pub(super) fn load_player_phase_state(&self, s: &mut State, v: &Value) -> Result<(), String> {
        if v.get("playerPhasePending").is_some() {
            return Err("pending player phase captures are not importable".into());
        }
        let Some(raw) = v.get("playerPhaseState") else {
            if self.needs_player_phase_state() || self.physical_phase_source(s) {
                return Err("selected or retained physical phase producer requires playerPhaseState version 1".into());
            }
            return Ok(());
        };
        let fail = || "invalid exact playerPhaseState application registry".to_string();
        if !raw
            .as_object()
            .is_some_and(|o| o.len() == 2 && o.contains_key("version") && o.contains_key("effects"))
            || raw["version"].as_u64() != Some(1)
        {
            return Err(fail());
        }
        let rows = raw["effects"].as_array().ok_or_else(fail)?;
        if rows.len() > CAPACITY {
            return Err("unsupported player phase application capacity (64)".into());
        }
        let mut ledger = Ledger::default();
        // Mandatory emission is derived from the selected/retained producer, so
        // only optional producer-free presence needs an independent provenance bit.
        ledger.wire_present = !self.needs_player_phase_state() && !self.physical_phase_source(s);
        for (index, raw) in rows.iter().enumerate() {
            if !raw.as_object().is_some_and(|o| {
                o.len() == 8
                    && [
                        "id",
                        "source",
                        "side",
                        "ordinal",
                        "kind",
                        "target",
                        "limitPlayer",
                        "used",
                    ]
                    .iter()
                    .all(|k| o.contains_key(*k))
            }) || raw["id"].as_u64() != Some((index + 1) as u64)
                || raw["side"] != "card"
                || raw["ordinal"].as_u64() != Some(0)
            {
                return Err(fail());
            }
            let source = raw["source"]
                .as_str()
                .and_then(|x| x.strip_prefix('c'))
                .and_then(|x| x.parse::<u16>().ok())
                .and_then(|x| x.checked_add(1))
                .filter(|&id| id > 0 && id < s.next_id && s.card(id).present)
                .ok_or_else(fail)?;
            if raw["source"] != format!("c{}", source - 1) {
                return Err(fail());
            }
            let program = *self.def(s, source).programs.card.first().ok_or_else(fail)?;
            let kind = self.phase_program_kind(program).ok_or_else(fail)?;
            let declared: Kind = serde_json::from_value(raw["kind"].clone()).map_err(|_| fail())?;
            let owner = s.card(source).owner;
            if kind != declared
                || raw["target"] != owner.as_str()
                || raw["limitPlayer"] != owner.as_str()
            {
                return Err(fail());
            }
            let used = raw["used"].as_array().ok_or_else(fail)?;
            if used.len() != 2
                || used.iter().any(|x| !matches!(x.as_u64(), Some(0 | 1)))
                || used[owner.other().index()].as_u64() != Some(0)
            {
                return Err(fail());
            }
            ledger.append(Row::new(
                source,
                program,
                kind,
                owner.index() as u8,
                owner.index() as u8,
                (used[0].as_u64().unwrap() | (used[1].as_u64().unwrap() << 1)) as u8,
            )?)?;
        }
        s.player_phase = ledger;
        Ok(())
    }
    pub(super) fn player_phase_pending(&self, s: &State) -> bool {
        let program = |p: u16| p != 0 && self.phase_program_kind(p - 1).is_some();
        let context = |c: &ResolutionContext| {
            program(c.ability_program)
                || (c.source != 0
                    && c.decision == Decision::Unique
                    && self.phase_direct_required(s, c.source, "whenPlayed") != 0)
                || c.resource_payment.phase_adjusters.count() != 0
                || (c.source != 0
                    && c.pending_programs != 0
                    && self
                        .def(s, c.source)
                        .programs
                        .card
                        .iter()
                        .any(|&p| self.phase_program_kind(p).is_some()))
        };
        context(&ResolutionContext::capture(s))
            || (s.support.pending && context(&s.support.context))
            || s.death_frames[..s.death_depth as usize]
                .iter()
                .any(|f| context(&f.context))
            || [&s.death_pending, &s.death_window]
                .into_iter()
                .chain(
                    s.death_frames[..s.death_depth as usize]
                        .iter()
                        .map(|f| &f.window),
                )
                .any(|w| {
                    w.entries[..w.len as usize].iter().any(|e| {
                        e.source != 0
                            && e.keyword == 0
                            && self.phase_program_kind(e.program).is_some()
                    })
                })
    }
    pub(super) fn snapshot_player_phase_state(&self, s: &State, v: &mut Value) {
        if self.player_phase_pending(s) {
            v["playerPhasePending"] = json!(true);
        }
        if s.player_phase.wire_present
            || s.player_phase.len != 0
            || self.needs_player_phase_state()
            || self.physical_phase_source(s)
        {
            let effects=s.player_phase.rows[..s.player_phase.len as usize].iter().enumerate().map(|(n,r)|json!({"id":n+1,"source":format!("c{}",r.source-1),"side":"card","ordinal":0,"kind":r.kind(),"target":if r.target()==0{"p1"}else{"p2"},"limitPlayer":if r.limit_player()==0{"p1"}else{"p2"},"used":[r.used()&1,(r.used()>>1)&1]})).collect::<Vec<_>>();
            v["playerPhaseState"] = json!({"version":1,"effects":effects});
        }
    }
}
