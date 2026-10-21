//! Exact independent watcher wire. No inference from current damage or zones.
use super::*;
use serde::Deserialize;
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Wire {
    version: u8,
    card_ids: Vec<CardId>,
}
pub(crate) fn parse(v: &Value, next: CardId) -> Result<[u64; MAX_CARDS / 64], String> {
    let w: Wire = serde_json::from_value(v.clone())
        .map_err(|e| format!("invalid damagedTargetHistory: {e}"))?;
    if w.version != 1 {
        return Err("unsupported damagedTargetHistory version".into());
    }
    if w.card_ids.len() >= MAX_CARDS {
        return Err("damagedTargetHistory exceeds physical capacity".into());
    }
    let mut bits = [0; MAX_CARDS / 64];
    let mut previous = 0;
    for id in w.card_ids {
        if id == 0 || id >= next || id as usize >= MAX_CARDS || id <= previous {
            return Err(
                "damagedTargetHistory requires sorted unique allocated physical card IDs".into(),
            );
        }
        bits[id as usize / 64] |= 1u64 << (id as usize % 64);
        previous = id;
    }
    Ok(bits)
}
impl Engine {
    pub(super) fn load_damaged_target_history(
        &self,
        s: &mut State,
        v: &Value,
    ) -> Result<(), String> {
        if let Some(v) = v.get("damagedTargetHistory") {
            s.phase_history.damaged = parse(v, s.next_id)?;
            s.phase_history.damage_exact = true;
        } else {
            s.phase_history.damage_exact = false;
            s.phase_history.damaged.fill(0);
            if self.state_needs_damaged_target_history(s) {
                return Err(
                    "allocated source requires exact damagedTargetHistory version 1".into(),
                );
            }
        }
        Ok(())
    }
    pub(super) fn damaged_target_history_snapshot(&self, s: &State) -> Value {
        json!({"version":1,"cardIds":(1..s.next_id).filter(|&id|s.phase_history.damaged[id as usize/64]&(1u64<<(id as usize%64))!=0).collect::<Vec<_>>()})
    }
}
