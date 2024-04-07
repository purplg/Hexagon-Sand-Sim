use bevy::prelude::*;
use hexx::EdgeDirection;

use super::{behavior::*, *};

pub struct Sand;

impl HexColor for Sand {
    const COLOR: Color = Color::Rgba {
        red: 1.0,
        green: 1.0,
        blue: 0.0,
        alpha: 1.0,
    };
}

impl Tick for Sand {
    fn tick(&self, hex: &Hex, states: &BoardState, rng: &mut SmallRng) -> Option<BoardSlice> {
        RandomSwap {
            directions: [
                EdgeDirection::POINTY_BOTTOM_LEFT,
                EdgeDirection::POINTY_BOTTOM_RIGHT,
            ],
            open: [Air::ID, Wind::ID, Water::ID, Steam::ID],
        }
        .apply(hex, rng, states)
    }
}
