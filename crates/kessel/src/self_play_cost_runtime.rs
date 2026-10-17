//! Runtime evaluation of validated self-play cost contracts.
use crate::model::{CardId, Seat, State};
use crate::self_play_cost::SelfPlayCost;

impl SelfPlayCost {
    pub(crate) fn reduction(
        self,
        state: &State,
        source: CardId,
        has_rebel: impl Fn(CardId) -> bool,
    ) -> u16 {
        let owner = state.card(source).owner;
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
                            Self::FriendlyRebelUnitsAndLeaders => has_rebel(id),
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
                && has_rebel(leader);
            return arena_count + u16::from(front);
        }
        let seats: &[Seat] = match self {
            Self::FriendlyDamagedUnits => std::slice::from_ref(&owner),
            Self::AllDamagedGroundUnits => &[Seat::P1, Seat::P2],
            Self::FriendlyLeaderUnits | Self::FriendlyRebelUnitsAndLeaders => unreachable!(),
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
