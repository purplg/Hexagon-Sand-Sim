use hexx::{EdgeDirection, Hex};
use rand::rngs::SmallRng;

use crate::grid::States;

use super::{
    behavior::{Chance, Infect, RandomSwap, Set, Step},
    BoardSlice, Register, StateId, Tick,
};

pub struct Fire;

impl Register for Fire {
    const ID: StateId = StateId::Fire;
}

impl Tick for Fire {
    fn tick(&self, hex: Hex, states: &States, mut rng: &mut SmallRng) -> Option<BoardSlice> {
        Chance {
            step: Set {
                hex,
                id: StateId::Air,
            },
            chance: 0.005,
        }
        .apply_or(
            &mut rng,
            states,
            Infect {
                from: hex,
                directions: [
                    EdgeDirection::POINTY_LEFT,
                    EdgeDirection::POINTY_RIGHT,
                    EdgeDirection::POINTY_TOP_LEFT,
                    EdgeDirection::POINTY_TOP_RIGHT,
                ],
                open: [StateId::Water],
                into: StateId::Steam,
            },
        )
        .apply_or(
            &mut rng,
            states,
            RandomSwap {
                from: hex,
                directions: [
                    EdgeDirection::POINTY_LEFT,
                    EdgeDirection::POINTY_RIGHT,
                    EdgeDirection::POINTY_TOP_LEFT,
                    EdgeDirection::POINTY_TOP_RIGHT,
                ],
                open: [StateId::Air, StateId::Water, StateId::Sand],
            },
        )
    }
}
