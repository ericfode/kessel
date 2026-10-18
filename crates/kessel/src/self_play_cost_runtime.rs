//! Runtime evaluation of validated self-play cost contracts.
use crate::model::{CardId, Seat, State};
use crate::self_play_cost::SelfPlayCost;

impl SelfPlayCost {
    pub(crate) fn reduction(
        self,
        state: &State,
        source: CardId,
        has_trait: impl Fn(CardId, &str) -> bool,
    ) -> u16 {
        let owner = state.card(source).owner;
        // These live predicates use the same physical arena inventories as
        // ordinary queries; cards in hand, base-zone leaders and attached Pilots
        // cannot contribute. The cost rule remains local to its physical source.
        let arena_count = |seat, space_only: bool, predicate: &dyn Fn(CardId) -> bool| {
            let p = state.player(seat);
            p.space
                .as_slice()
                .iter()
                .chain(
                    if space_only {
                        &[][..]
                    } else {
                        p.ground.as_slice()
                    }
                    .iter(),
                )
                .filter(|&&id| {
                    let card = state.card(id);
                    card.present && card.owner == seat && card.parent == 0 && predicate(id)
                })
                .count() as u16
        };
        match self {
            Self::Initiative => return u16::from(state.initiative == owner),
            Self::OtherFriendlyTrooper => {
                return u16::from(
                    arena_count(owner, false, &|id| id != source && has_trait(id, "trooper")) > 0,
                )
            }
            Self::ThreeOtherFriendlyUnits => {
                return u16::from(arena_count(owner, false, &|id| id != source) >= 3)
            }
            Self::ThreeEnemySpaceUnits => {
                return if arena_count(owner.other(), true, &|_| true) >= 3 {
                    3
                } else {
                    0
                }
            }
            Self::EnemyUnits => return arena_count(owner.other(), false, &|_| true),
            _ => {}
        }
        if matches!(
            self,
            Self::FriendlyLeaderUnits | Self::FriendlyRebelUnitsAndLeaders
        ) {
            let player = state.player(owner);
            // Actual arena inventories preserve physical unit mode. Printed
            // unit metadata on an attached ordinary Pilot is not a unit here.
            let arena_count = player
                .ground
                .as_slice()
                .iter()
                .chain(player.space.as_slice())
                .filter(|&&id| {
                    let card = state.card(id);
                    card.present
                        && card.owner == owner
                        && card.parent == 0
                        && match self {
                            Self::FriendlyLeaderUnits => id == player.leader && card.deployed,
                            Self::FriendlyRebelUnitsAndLeaders => has_trait(id, "rebel"),
                            _ => unreachable!(),
                        }
                })
                .count() as u16;
            let leader = player.leader;
            // Validation forbids LeaderUpgrade attachments, and the native
            // Pilot contract admits ordinary units only. No leader-upgrade or
            // gained-IsLeader interpretation is approximated by this branch.
            let front = self == Self::FriendlyRebelUnitsAndLeaders
                && leader != 0
                && state.card(leader).present
                && state.card(leader).owner == owner
                && !state.card(leader).deployed
                && state.card(leader).parent == 0
                && !player.ground.contains(leader)
                && !player.space.contains(leader)
                && has_trait(leader, "rebel");
            return arena_count + u16::from(front);
        }
        let seats: &[Seat] = match self {
            Self::FriendlyDamagedUnits => std::slice::from_ref(&owner),
            Self::AllDamagedGroundUnits => &[Seat::P1, Seat::P2],
            _ => unreachable!("other cost contracts handled above"),
        };
        seats
            .iter()
            .map(|seat| {
                let player = state.player(*seat);
                player
                    .ground
                    .as_slice()
                    .iter()
                    .chain(
                        if self == Self::FriendlyDamagedUnits {
                            player.space.as_slice()
                        } else {
                            &[]
                        }
                        .iter(),
                    )
                    .filter(|&&id| state.card(id).present && state.card(id).damage > 0)
                    .count() as u16
            })
            .sum()
    }
}
