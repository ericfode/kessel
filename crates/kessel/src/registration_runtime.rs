//! Registration chronology is evidence, never reconstructed from stable IDs or
//! per-player/per-arena ordering. The supported ability inventory registers at
//! unit entry or ordinary-upgrade attachment. Dynamic/native late registrations
//! retain their own event boundary and do not enter this projection.
use super::*;
impl Engine {
    fn registration_definition(&self, d: &Definition) -> bool {
        self.catalogue.is_some()
            && d.programs
                .card
                .iter()
                .chain(&d.programs.unit)
                .chain(&d.programs.leader)
                .any(|id| {
                    let p = &self.catalogue.as_ref().unwrap().programs[*id as usize];
                    !p.hooks.is_empty() || crate::round_limit_contract::Registration::from_trigger(&p.trigger).is_some() || matches!(p.trigger.as_str(),"onCardsDrawn"|"onUpgradeAttached")
                        || (d.info.is("upgrade")
                            && !d.info.is("token")
                            && p.trigger == "whenDefeated")
                })
    }
    pub(super) fn needs_registration_order(&self) -> bool {
        self.selected_definition_any(|d| self.registration_definition(d))
    }
    fn state_needs_registration_order(&self, s: &State) -> bool {
        (1..s.next_id).any(|id| s.card(id).present && self.registration_definition(self.def(s, id)))
    }
    pub(super) fn load_registration_order(&self, s: &mut State, v: &Value) -> Result<(), String> {
        let Some(raw) = v.get("registrationOrder") else {
            s.registration_order = RegistrationOrder {
                exact: false,
                ..RegistrationOrder::default()
            };
            if self.needs_registration_order() || self.state_needs_registration_order(s) {
                return Err("defeat-hook/granted-death sources require explicit registrationOrder including future cards in selected pool".into());
            }
            return Ok(());
        };
        if !s.arena_order_exact {
            return Err("registrationOrder requires explicit arenaUnitOrder evidence".into());
        }
        let actual = RegistrationOrder::parse(raw, s.next_id)?;
        let expected = (1..s.next_id)
            .filter(|&id| {
                s.card(id).present
                    && (self.in_arena(s, id)
                        || (self.attached_in_play(s, id) && !self.def(s, id).info.is("token")))
            })
            .collect::<std::collections::HashSet<_>>();
        if actual
            .as_slice()
            .iter()
            .any(|id| !expected.contains(&(*id as CardId)))
        {
            return Err("registrationOrder contains foreign/nonregistered physical card".into());
        }
        if actual.len as usize != expected.len() {
            return Err("registrationOrder omits registered physical cards".into());
        }
        for index in 0..2 {
            let projection = actual
                .as_slice()
                .iter()
                .map(|id| *id as CardId)
                .filter(|id| s.arena_unit_order[index].as_slice().contains(id))
                .collect::<Vec<_>>();
            if projection != s.arena_unit_order[index].as_slice() {
                return Err("registrationOrder conflicts with arena entry order".into());
            }
        }
        for &host in s.arena_unit_order.iter().flat_map(|a| a.as_slice()) {
            let upgrades = self
                .upgrades(s, host)
                .into_iter()
                .filter(|id| !self.def(s, *id).info.is("token"))
                .collect::<Vec<_>>();
            let projection = actual
                .as_slice()
                .iter()
                .map(|id| *id as CardId)
                .filter(|id| upgrades.contains(id))
                .collect::<Vec<_>>();
            if projection != upgrades
                || upgrades
                    .iter()
                    .any(|id| actual.position(*id) <= actual.position(host))
            {
                return Err(
                    "registrationOrder conflicts with physical attachment chronology".into(),
                );
            }
        }
        s.registration_order = actual;
        Ok(())
    }
    pub(super) fn sort_captured_registrations(
        &self,
        s: &mut State,
        start: usize,
    ) -> Result<(), String> {
        if !self.needs_registration_order() && !self.state_needs_registration_order(s) {
            return Ok(());
        }
        if !s.registration_order.exact {
            return Err("defeat registration capture requires exact registrationOrder".into());
        }
        let end = s.death_pending.len as usize;
        for e in &s.death_pending.entries[start..end] {
            if s.registration_order.position(e.physical_source()).is_none() {
                return Err(
                    "captured defeat registration missing physical entry chronology".into(),
                );
            }
        }
        let order = s.registration_order;
        s.death_pending.entries[start..end].sort_by_key(|e| {
            order
                .position(e.physical_source())
                .unwrap()
        });
        Ok(())
    }
}
