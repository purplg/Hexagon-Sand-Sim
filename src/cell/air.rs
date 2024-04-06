use hexx::Hex;
use rand::rngs::SmallRng;

use crate::grid::States;

use super::{
    behavior::{Chance, Set, Step},
    BoardSlice, Register, StateId, Tick,
};

pub struct Air;

impl Register for Air {
    const ID: StateId = StateId::Air;
}

impl Tick for Air {
    fn tick(&self, from: Hex, states: &States, mut rng: &mut SmallRng) -> Option<BoardSlice> {
        Chance {
            step: Set {
                hex: from,
                id: StateId::Wind,
            },
            chance: 0.0001,
        }
        .apply(&mut rng, states)
    }
}
