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
        if let Effect::DistributeTokenUpgrade {
            target_resolver, ..
        } = effect
        {
            let amount = self.distribution_advantage_amount(s)?;
            if amount > 32 {
                return Err("prepared token distribution capacity exceeded (32)".into());
            }
            let targets = self.distribution_advantage_targets(s, s.source, target_resolver);
            return ordered_token_allocations(&targets, amount).map(|allocations| {
                allocations
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
                    .collect()
            });
        }
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

// Token creation allocates physical identities in submitted Map order. Unlike
// damage/healing, recipient permutations are observably distinct actions.
fn ordered_token_allocations(
    targets: &[CardId],
    amount: u16,
) -> Result<Vec<Vec<(CardId, u16)>>, String> {
    fn walk(
        targets: &[CardId],
        left: u16,
        picks: &mut Vec<(CardId, u16)>,
        out: &mut Vec<Vec<(CardId, u16)>>,
    ) -> Result<(), String> {
        if left == 0 {
            if out.len() == MAX_DISTRIBUTION_ACTIONS {
                return Err("ordered token distribution action capacity exceeded (4096)".into());
            }
            out.push(picks.clone());
            return Ok(());
        }
        for &id in targets {
            if picks.iter().any(|(prior, _)| *prior == id) {
                continue;
            }
            for n in 1..=left {
                picks.push((id, n));
                walk(targets, left - n, picks, out)?;
                picks.pop();
            }
        }
        Ok(())
    }
    let mut out = vec![vec![]];
    if amount > 0 {
        walk(targets, amount, &mut vec![], &mut out)?;
    }
    Ok(out)
}
impl Engine {
    pub(super) fn distribution_advantage_amount(&self, s: &State) -> Result<u16, String> {
        if s.last_known_source != s.source || s.source == 0 {
            return Err("Advantage distribution requires captured defeat power".into());
        }
        // Existing capture is u16 and hashing/copy/resume already retain it.
        Ok(s.last_known_power)
    }
    pub(super) fn distribution_advantage_targets(
        &self,
        s: &State,
        source: CardId,
        r: &TargetResolver,
    ) -> Vec<CardId> {
        let controller = if s.last_known_source == source {
            s.active
        } else {
            s.card(source).owner
        };
        let mut all = r.clone();
        all.owner = "any".into();
        self.matching_targets(s, source, &all)
            .into_iter()
            .filter(|id| s.card(*id).owner == controller && s.card(*id).damage < self.hp(s, *id))
            .collect()
    }
    /// Capture one original creation event per ordered recipient, then allocate
    /// its tokens in that event's loop order. Repeated records retain the parent
    /// event amount while each physical token owns its own attach contingent.
    /// No current admitted listener replaces token-creation events; such raw
    /// registrations remain rejected. The defeat event/LKI is never rebound.
    pub(super) fn capture_advantage_distribution(
        &self,
        s: &mut State,
        pairs: &[(CardId, u16)],
    ) -> Result<(), String> {
        let amount = self.distribution_advantage_amount(s)?;
        let total = pairs.iter().try_fold(0usize, |n, (_, v)| {
            n.checked_add(*v as usize)
                .ok_or("token allocation total overflow")
        })?;
        if !pairs.is_empty()
            && (total != amount as usize
                || pairs.iter().any(|(_, n)| *n == 0)
                || pairs
                    .iter()
                    .enumerate()
                    .any(|(i, (id, _))| pairs[..i].iter().any(|(p, _)| p == id)))
        {
            return Err("invalid ordered full token allocation".into());
        }
        if total > 32 {
            return Err("prepared token distribution capacity exceeded (32)".into());
        }
        let def = self
            .advantage_def
            .ok_or("catalogue needs Advantage token")?;
        if s.next_id as usize + total > MAX_CARDS {
            return Err("card capacity exceeded generating Advantage distribution".into());
        }
        if pairs.iter().any(|(id, _)| {
            !self.in_arena(s, *id)
                || s.card(*id).owner != s.active
                || s.card(*id).damage >= self.hp(s, *id)
        }) {
            return Err("token distribution recipient is not an eligible friendly unit".into());
        }
        let mut group = EventGroupState::default();
        group.active = true;
        group.initiator = s.active;
        for &(target, n) in pairs {
            for _ in 0..n {
                let token = s.next_id;
                s.next_id += 1;
                *s.card_mut(token) = Card {
                    present: true,
                    def,
                    owner: s.card(target).owner,
                    ..Card::default()
                };
                group.records[group.count as usize] = PreparedEvent {
                    target,
                    amount: n,
                    token,
                    ..PreparedEvent::default()
                };
                group.count += 1;
            }
        }
        s.event_group = group;
        Ok(())
    }
}

impl Engine {
    /// GiveTokenUpgrade has one empty creation handler per recipient, not one
    /// handler per token. Generated records retain that parent's requested count.
    /// The pinned negative-order comparator emits those parents in reverse Map
    /// order; its order-zero attach contingents retain generation order. The
    /// admitted pool has no token-creation listeners/replacements: their raw
    /// registrations are rejected, so these empty handlers need no extra State
    /// or synthetic invocation context. The original defeat LKI remains intact.
    pub(super) fn advantage_creation_events(
        &self,
        s: &State,
    ) -> Result<(Vec<CardId>, u16), String> {
        let records = &s.event_group.records[..s.event_group.count as usize];
        let mut parents = Vec::new();
        let mut at = 0;
        while at < records.len() {
            let first = records[at];
            let mut end = at + 1;
            while end < records.len() && records[end].target == first.target {
                end += 1;
            }
            if first.amount == 0
                || end - at != first.amount as usize
                || parents.iter().any(|(id, _)| *id == first.target)
                || records[at..end].iter().any(|r| {
                    r.op != 0
                        || r.amount != first.amount
                        || r.token == 0
                        || r.power != 0
                        || r.hp != 0
                        || Some(s.card(r.token).def) != self.advantage_def
                })
            {
                return Err("invalid original token-creation event records".into());
            }
            parents.push((first.target, first.amount));
            at = end;
        }
        let mut resolved = Vec::new();
        let mut total = 0u16;
        for (target, amount) in parents.into_iter().rev() {
            // GiveTokenUpgrade.checkEventCondition checks its original recipient;
            // a failed creation parent cancels every one of its attach contingents.
            if !self.in_arena(s, target) || s.card(target).damage >= self.hp(s, target) {
                continue;
            }
            total = total
                .checked_add(amount)
                .ok_or("token distribution result overflow")?;
            resolved.push(target);
        }
        Ok((resolved, total))
    }
}
