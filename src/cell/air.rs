use super::{behavior::*, *};
use bevy::prelude::*;

pub struct Air;

impl HexColor for Air {
    const COLOR: Color = Color::NONE;
}

impl Tick for Air {
    fn tick(&self, hex: &Hex, states: &BoardState, mut rng: &mut SmallRng) -> Option<BoardSlice> {
        Chance {
            step: Set(Wind::ID),
            chance: 0.0001,
        }
        .apply(&hex, &mut rng, states)
    }
}
