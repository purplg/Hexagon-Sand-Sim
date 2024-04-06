use hexx::{EdgeDirection, Hex};
use rand::rngs::SmallRng;

use crate::grid::States;

use super::{
    behavior::{Chance, RandomSwap, Set, Step},
    BoardSlice, Register,
    StateId::{self, *},
    Tick,
};

pub struct Steam;

impl Register for Steam {
    const ID: StateId = StateId::Steam;
}

impl Tick for Steam {
    fn tick(&self, from: Hex, states: &States, mut rng: &mut SmallRng) -> Option<BoardSlice> {
        // Condense
        Chance {
            step: Set {
                hex: from,
                id: StateId::Water,
            },
            chance: 0.0001,
        }
        // Move up
        .apply_or(
            &mut rng,
            states,
            RandomSwap {
                from,
                directions: [
                    EdgeDirection::POINTY_TOP_LEFT,
                    EdgeDirection::POINTY_TOP_RIGHT,
                ],
                with_state: [Air, Water],
            },
        )
        // Move laterally.
        .apply_or(
            &mut rng,
            states,
            RandomSwap {
                from,
                directions: [EdgeDirection::POINTY_LEFT, EdgeDirection::POINTY_RIGHT],
                with_state: [Air, Water, Fire],
            },
        )
    }
}
