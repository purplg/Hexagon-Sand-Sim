use bevy::prelude::*;

use super::*;
use crate::behavior::*;

pub struct Air;

impl StateInfo for Air {
    const NAME: &'static str = "Air";
    const COLOR: Color = Color::NONE;
    const HIDDEN: bool = false;
}

impl Tick for Air {
    fn tick(&self, hex: &Hex, states: &BoardState, rng: &mut SmallRng) -> Option<BoardSlice> {
        Chance {
            step: Set(Wind::id()),
            chance: 0.0001,
        }
        .apply(hex, rng, states)
    }
}
