use hexx::{EdgeDirection, Hex};
use rand::rngs::SmallRng;

use crate::grid::States;

use super::{
    behavior::{Chance, Drag, RandomSwap, Set, Step},
    BoardSlice, Register,
    StateId::{self, *},
    Tick,
};

pub struct Water;

impl Register for Water {
    const ID: StateId = StateId::Water;
}

impl Tick for Water {
    fn tick(&self, from: Hex, states: &States, mut rng: &mut SmallRng) -> Option<BoardSlice> {
        // Evaporate
        Chance {
            step: Set {
                hex: from,
                id: StateId::Steam,
            },
            chance: 0.0001,
        }
        // Drag sand
        .apply_or(
            &mut rng,
            states,
            Drag {
                from,
                directions: [
                    EdgeDirection::POINTY_LEFT,
                    EdgeDirection::POINTY_RIGHT,
                    EdgeDirection::POINTY_BOTTOM_LEFT,
                    EdgeDirection::POINTY_BOTTOM_RIGHT,
                ],
                open: [Air, Self::ID],
                drag: [Sand],
            },
        )
        // Move down
        .apply_or(
            &mut rng,
            states,
            RandomSwap {
                from,
                directions: [
                    EdgeDirection::POINTY_BOTTOM_LEFT,
                    EdgeDirection::POINTY_BOTTOM_RIGHT,
                ],
                with_state: [Air],
            },
        )
        // Move laterally.
        .apply_or(
            &mut rng,
            states,
            RandomSwap {
                from,
                directions: [EdgeDirection::POINTY_LEFT, EdgeDirection::POINTY_RIGHT],
                with_state: [Air],
            },
        )
    }
}
