//! Exact inactive physical identities and their bounded lasting keyword state.
//! Existing live and native player-token references own their identities. The
//! complement accounts for every remaining allocated slot, including empty or
//! expired effects. It never infers a definition from a numeric allocation gap.
use super::*;
impl Engine {
    // These intrinsic programs can only be installed by their complete closed
    // source adapters. Inspect physical definitions, not copied Support grants,
    // pool membership alone, card names, or a remembered token count. Inventory
    // eligibility does not authorize any lasting keyword on an inactive card.
    fn has_physical_inventory_source(&self, s: &State) -> bool {
        (1..s.next_id).any(|id| {
            (s.card(id).present || s.card(id).def != 0)
                && self.card_programs(s, id).iter().any(|p| {
                    let program = self.program(*p);
                    crate::attack_base_history_contract::inventory_program(program)
                        || (crate::generated_token_continuation_contract::requires(program)&&crate::generated_token_continuation_contract::program(program))
                        || (crate::created_token_phase_keyword_contract::requires(program)
                        && crate::created_token_phase_keyword_contract::program(program))
                        || (program.trigger == "whenDefeated"
                            && program.instructions.iter().any(|i| {
                                matches!(
                                    i,
                                    crate::ability_ir::Instruction::Apply(
                                        crate::ability_ir::Effect::DistributeTokenUpgrade { .. }
                                    )
                                )
                            }))
                        || (program.trigger == "event"
                            && crate::upgrade_return_shield_contract::requires(program)
                            && crate::upgrade_return_shield_contract::program(program))
                        || (program.trigger == "onAttack"
                            && program.instructions.iter().any(|i| {
                                matches!(i, crate::ability_ir::Instruction::Apply(effect)
                                    if crate::attack_lifetime_contract::modifier(effect))
                            })
                            && crate::attack_lifetime_contract::program(program))
                })
        })
    }
    pub(super) fn snapshot_inactive_physical_state(&self, s: &State, v: &mut Value) {
        if !self.has_physical_inventory_source(s) {
            return;
        }
        let live = crate::validation::allocated_live_ids(v)
            .expect("native snapshot has valid live identities");
        let mut rows = vec![];
        for id in 1..s.next_id {
            if live.contains(&id) {
                continue;
            }
            let c = s.card(id);
            assert!(
                c.present || c.def != 0,
                "native snapshot lacks exact allocated identity c{}",
                id - 1
            );
            let mut row = self.card_ref(s, id);
            row["owner"] = json!(c.owner.as_str());
            row["controller"] = json!(c.owner.as_str());
            let mut kw = serde_json::Map::new();
            for (index, name) in crate::ability_ir::CORE_KEYWORDS.iter().enumerate() {
                if c.phase_keyword_mask() & (1 << index) != 0 {
                    kw.insert((*name).into(), json!(c.phase_keyword_amount(index)));
                }
            }
            row["phaseKeywords"] = Value::Object(kw);
            rows.push(row);
        }
        v["inactivePhaseKeywordStateVersion"] = json!(1);
        v["inactiveAllocatedCards"] = json!(rows);
    }
    // First establish exact selected-pool identity. The existing entry-stat
    // importer then sees the same physical card and retains all of its guards.
    pub(super) fn load_inactive_allocated_identities(
        &self,
        s: &mut State,
        v: &Value,
    ) -> Result<(), String> {
        // Strict and legacy snapshots without a physical producer retain their
        // existing wire; only an inventory claim or required producer needs the
        // compiled catalogue and exact complement validation below.
        let required = self.has_physical_inventory_source(s);
        if !required && v.get("inactivePhaseKeywordStateVersion").is_none()
            && v.get("inactiveAllocatedCards").is_none() {
            return Ok(());
        }
        let metadata = &self
            .catalogue
            .as_ref()
            .ok_or("inactive identities require compiled catalogue")?
            .metadata;
        let rows = crate::validation::inactive_allocated_rows(v, metadata)?;
        let Some(rows) = rows else {
            if required {
                return Err(
                    "physical token producer requires exact inactive allocated identity state"
                        .into(),
                );
            }
            return Ok(());
        };
        if !required {
            return Err(
                "inactive allocated identity state requires its physical closed token producer"
                    .into(),
            );
        }
        for row in rows {
            let id = cid(row["id"].as_str().unwrap())?;
            let owner = Seat::parse(row["owner"].as_str().unwrap())?;
            if s.card(id).present {
                return Err("inactive allocated identity already materialized outside its declared complement".into());
            }
            let next = s.next_id;
            let descriptor = json!({"id":row["id"],"name":row["name"],"setCode":row["setCode"]});
            self.load_card(s, &descriptor, owner)?;
            s.next_id = next;
            if !self.def(s, id).info.is("token")
                || !(self.def(s, id).info.is("unit") || self.def(s, id).info.is("upgrade"))
            {
                return Err(
                    "inactive allocated type must be a reviewed selected native token".into(),
                );
            }
        }
        Ok(())
    }
    // Apply keywords only after identity and old entry-stat validation succeed.
    // Recorded source-side origin proof remains evidence, not invented state.
    pub(super) fn load_inactive_phase_keywords(
        &self,
        s: &mut State,
        v: &Value,
    ) -> Result<(), String> {
        let Some(rows) = v.get("inactiveAllocatedCards").and_then(Value::as_array) else {
            return Ok(());
        };
        for row in rows {
            if row["phaseKeywords"] == json!({}) {
                continue;
            }
            let id = cid(row["id"].as_str().unwrap())?;
            let owner = s.card(id).owner;
            if !s.phase_history.exact
                || !crate::model::PhaseHistory::contains(&s.phase_history.entered, owner, id)
            {
                return Err("inactive Sentinel requires exact physical entry history".into());
            }
            let played_source = (1..s.next_id).any(|source| {
                s.card(source).owner == owner
                    && crate::model::PhaseHistory::contains(&s.phase_history.played, owner, source)
                    && self.card_programs(s, source).iter().any(|p| {
                        crate::created_token_phase_keyword_contract::requires(self.program(*p))
                    })
            });
            if !played_source {
                return Err(
                    "inactive Sentinel requires a recorded played physical creation source".into(),
                );
            }
            let index = crate::ability_ir::CORE_KEYWORDS
                .iter()
                .position(|k| *k == "sentinel")
                .unwrap();
            s.card_mut(id).add_phase_keyword(index);
        }
        Ok(())
    }
}
#[cfg(test)]
#[path = "inactive_physical_state_tests.rs"]
mod tests;
