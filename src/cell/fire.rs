use hexx::{EdgeDirection, Hex};
use rand::rngs::SmallRng;

use crate::grid::States;

use super::{
    behavior::{Chance, RandomSwap, Set, Step},
    BoardSlice, Register, StateId, Tick,
};

pub struct Fire;

impl Register for Fire {
    const ID: StateId = StateId::Fire;
}

impl Tick for Fire {
    fn tick(&self, hex: Hex, states: &States, rng: &mut SmallRng) -> Option<BoardSlice> {
        Chance {
            step: Set {
                hex,
                id: StateId::Air,
            },
            chance: 0.005,
        }
        .apply_or(
            rng,
            states,
            RandomSwap {
                from: hex,
                directions: [
                    EdgeDirection::POINTY_LEFT,
                    EdgeDirection::POINTY_RIGHT,
                    EdgeDirection::POINTY_TOP_LEFT,
                    EdgeDirection::POINTY_TOP_RIGHT,
                ],
                with_state: [StateId::Air, StateId::Water, StateId::Sand],
            },
        )
    }
}
