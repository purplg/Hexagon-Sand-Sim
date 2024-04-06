use bevy::prelude::*;
use hexx::Hex;
use rand::rngs::SmallRng;

use crate::grid::BoardState;

use super::{
    behavior::{Chance, Set, Step}, BoardSlice, HexColor, Register, StateId, Tick
};

pub struct Air;

impl Register for Air {
    const ID: StateId = StateId::Air;
}

impl HexColor for Air {
    const COLOR: Color = Color::NONE;
}

impl Tick for Air {
    fn tick(&self, from: Hex, states: &BoardState, mut rng: &mut SmallRng) -> Option<BoardSlice> {
        Chance {
            step: Set {
                hex: from,
                into: StateId::Wind,
            },
            chance: 0.0001,
        }
        .apply(&mut rng, states)
    }
}
