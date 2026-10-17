//! Runtime evaluation of validated self-play cost contracts.
use crate::model::{CardId, Seat, State};
use crate::self_play_cost::SelfPlayCost;

impl SelfPlayCost {
    pub(crate) fn reduction(self, state: &State, source: CardId) -> u16 {
        let owner = state.card(source).owner;
        let seats: &[Seat] = match self {
            Self::FriendlyDamagedUnits => std::slice::from_ref(&owner),
            Self::AllDamagedGroundUnits => &[Seat::P1, Seat::P2],
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
