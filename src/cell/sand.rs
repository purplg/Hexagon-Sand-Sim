use hexx::{EdgeDirection, Hex};
use rand::rngs::SmallRng;

use crate::grid::States;

use super::{
    behavior::{RandomSwap, Step},
    BoardSlice, Register,
    StateId::{self, *},
    Tick,
};

pub struct Sand;

impl Register for Sand {
    const ID: StateId = StateId::Sand;
}

impl Tick for Sand {
    fn tick(&self, from: Hex, states: &States, rng: &mut SmallRng) -> Option<BoardSlice> {
        RandomSwap {
            from,
            directions: [
                EdgeDirection::POINTY_BOTTOM_LEFT,
                EdgeDirection::POINTY_BOTTOM_RIGHT,
            ],
            open: [Air, Wind, Water, Steam],
        }
        .apply(rng, &states)
    }
}
