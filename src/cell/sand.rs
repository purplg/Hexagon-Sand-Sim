use bevy::prelude::*;
use hexx::EdgeDirection;

use super::{behavior::*, *};

pub struct Sand;

impl StateInfo for Sand {
    const NAME: &'static str = "Sand";
    const COLOR: Color = Color::Rgba {
        red: 1.0,
        green: 1.0,
        blue: 0.0,
        alpha: 1.0,
    };
    const HIDDEN: bool = false;
}

impl Tick for Sand {
    fn tick(&self, hex: &Hex, states: &BoardState, rng: &mut SmallRng) -> Option<BoardSlice> {
        RandomSwap {
            directions: [
                EdgeDirection::POINTY_BOTTOM_LEFT,
                EdgeDirection::POINTY_BOTTOM_RIGHT,
            ],
            open: [Air::id(), Wind::id(), Steam::id()],
        }
        .apply(hex, rng, states)
    }
}
