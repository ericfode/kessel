//! Distribution captures choices once; prepared windows own replacement and
//! mutation, and their post-primary damage callback sweep owns defeat timing.
use super::*;
const MAX_DISTRIBUTION_ACTIONS: usize = 4096;
fn allocations(
    targets: &[CardId],
    amount: u16,
    max_targets: usize,
    less: bool,
    empty: bool,
) -> Result<Vec<Vec<(CardId, u16)>>, String> {
    fn walk(
        targets: &[CardId],
        at: usize,
        left: u16,
        max: usize,
        picks: &mut Vec<(CardId, u16)>,
        out: &mut Vec<Vec<(CardId, u16)>>,
    ) -> Result<(), String> {
        if left == 0 {
            if out.len() == MAX_DISTRIBUTION_ACTIONS {
                return Err("distribution action capacity exceeded (4096)".into());
            }
            out.push(picks.clone());
            return Ok(());
        }
        if picks.len() == max {
            return Ok(());
        }
        for i in at..targets.len() {
            for n in 1..=left {
                picks.push((targets[i], n));
                walk(targets, i + 1, left - n, max, picks, out)?;
                picks.pop();
            }
        }
        Ok(())
    }
    let mut targets = targets.to_vec();
    targets.sort_by_key(|id| format!("c{}", id - 1));
    let mut out = vec![];
    if empty {
        out.push(vec![]);
    }
    for total in 1..=amount {
        if total == amount || less {
            walk(&targets, 0, total, max_targets, &mut vec![], &mut out)?;
        }
    }
    Ok(out)
}
impl Engine {
    pub(super) fn distribution_damage_targets(
        &self,
        s: &State,
        source: CardId,
        resolver: &TargetResolver,
    ) -> Vec<CardId> {
        // A defeated stolen source has reverted to its owner. The trigger's
        // captured player owns its ability even while that source is in discard.
        let controller = if s.last_known_source == source {
            s.active
        } else {
            s.card(source).owner
        };
        let mut all = resolver.clone();
        all.owner = "any".into();
        self.matching_targets(s, source, &all)
            .into_iter()
            .filter(|id| s.card(*id).owner != controller)
            .collect()
    }

    pub(super) fn distribution_actions(&self, s: &State) -> Result<Vec<Action>, String> {
        let Instruction::Apply(effect) =
            &self.program(s.ability_program - 1).instructions[s.ability_pc as usize]
        else {
            return Err("missing distribution effect".into());
        };
        let (amount, max, less, empty, resolver) = match effect {
            Effect::DistributeDamage {
                target_resolver, ..
            } => (6, 6, false, false, target_resolver),
            Effect::DistributeHealing {
                amount,
                max_targets,
                can_distribute_less,
                can_choose_no_targets,
                target_resolver,
            } => (
                u16::try_from(self.eval(s, s.source, 0, amount).number())
                    .map_err(|_| "invalid distribution amount")?,
                *max_targets as usize,
                *can_distribute_less,
                *can_choose_no_targets,
                target_resolver,
            ),
            _ => return Err("invalid distribution effect".into()),
        };
        let targets = if matches!(effect, Effect::DistributeDamage { .. }) {
            self.distribution_damage_targets(s, s.source, resolver)
        } else {
            self.matching_targets(s, s.source, resolver)
        };
        if targets.len() > 64 {
            return Err("distribution target capacity exceeded (64)".into());
        }
        Ok(allocations(&targets, amount, max, less, empty)?
            .into_iter()
            .map(|pairs| Action {
                kind: "distribute".into(),
                card: None,
                arg: Some(json!(pairs
                    .into_iter()
                    .map(|(id, n)| json!([format!("c{}", id - 1), n]))
                    .collect::<Vec<_>>())),
                method: None,
                label: format!("distribute {amount}"),
            })
            .collect())
    }
    pub(super) fn validate_distribution(
        &self,
        s: &State,
        a: &Action,
    ) -> Result<Vec<(CardId, u16)>, String> {
        // Membership validates exact total, positivity, distinct IDs and eligible
        // target membership together, before publishing any prepared records.
        if a.kind != "distribute" || a.card.is_some() || a.method.is_some() {
            return Err("illegal distribution action".into());
        }
        if !self
            .distribution_actions(s)?
            .iter()
            .any(|legal| legal.arg == a.arg)
        {
            return Err("illegal distribution allocation".into());
        }
        a.arg
            .as_ref()
            .and_then(Value::as_array)
            .ok_or("distribution pairs missing")?
            .iter()
            .map(|p| {
                Ok((
                    cid(p[0].as_str().ok_or("distribution target missing")?)?,
                    u16::try_from(p[1].as_u64().ok_or("distribution amount missing")?)
                        .map_err(|_| "distribution amount overflow")?,
                ))
            })
            .collect()
    }
    pub(super) fn capture_distribution(
        &self,
        s: &mut State,
        pairs: &[(CardId, u16)],
    ) -> Result<(), String> {
        if pairs.is_empty()
            || pairs.len() > 6
            || pairs.iter().any(|(_, n)| !(1..=6).contains(n))
            || pairs.iter().map(|(_, n)| n).sum::<u16>() != 6
            || pairs
                .iter()
                .enumerate()
                .any(|(i, (id, _))| pairs[..i].iter().any(|(previous, _)| previous == id))
        {
            return Err("invalid damage distribution support".into());
        }
        let mut group = EventGroupState::default();
        group.active = true;
        group.initiator = s.active;
        group.count = pairs.len() as u8;
        // The trace key sorts canonical IDs, while the concrete adapter submits
        // allocations in the selector's ground-then-space order. Capture that
        // order for replacement contexts; negative child event ordering is
        // applied later, at the common prepared primary-handler boundary.
        let physical = [Seat::P1, Seat::P2]
            .into_iter()
            .flat_map(|seat| {
                s.player(seat)
                    .ground
                    .as_slice()
                    .iter()
                    .chain(s.player(seat).space.as_slice())
                    .copied()
            })
            .collect::<Vec<_>>();
        let mut ordered = pairs
            .iter()
            .map(|(id, amount)| {
                Ok((
                    physical
                        .iter()
                        .position(|candidate| candidate == id)
                        .ok_or("distribution recipient left play before capture")?,
                    *id,
                    *amount,
                ))
            })
            .collect::<Result<Vec<_>, String>>()?;
        ordered.sort_by_key(|(position, _, _)| *position);
        for (i, (_, target, amount)) in ordered.iter().enumerate() {
            group.records[i] = PreparedEvent {
                target: *target,
                amount: *amount,
                ..PreparedEvent::default()
            };
        }
        s.event_group = group;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_positive_compositions_and_lexical_ids() {
        let a = allocations(&[3, 11, 2], 6, 6, false, false).unwrap();
        assert_eq!(a.len(), 28);
        assert!(a.contains(&vec![(2, 1), (11, 2), (3, 3)]));
        assert!(a.contains(&vec![(3, 6)]));
        assert!(a.iter().all(|p| p.iter().map(|(_, n)| n).sum::<u16>() == 6));
    }
    #[test]
    fn exact_cap_rejects_instead_of_pruning() {
        assert_eq!(
            allocations(&(1..=6).collect::<Vec<_>>(), 6, 6, false, false)
                .unwrap()
                .len(),
            462
        );
        assert!(
            allocations(&(1..=10).collect::<Vec<_>>(), 6, 6, false, false)
                .unwrap_err()
                .contains("capacity")
        );
    }
}
#[cfg(test)]
#[path = "distributed_damage_tests.rs"]
mod distributed_damage_tests;
