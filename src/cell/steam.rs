use bevy::prelude::*;
use hexx::EdgeDirection;

use super::{behavior::*, *};

pub struct Steam;

impl HexColor for Steam {
    const COLOR: Color = Color::Rgba {
        red: 1.0,
        green: 1.0,
        blue: 1.0,
        alpha: 0.5,
    };
}

impl Tick for Steam {
    fn tick(&self, hex: &Hex, states: &BoardState, rng: &mut SmallRng) -> Option<BoardSlice> {
        Or3(
            // Condense
            Chance {
                step: Set(Water::ID),
                chance: 0.0001,
            },
            // Move up
            RandomSwap {
                directions: [
                    EdgeDirection::POINTY_TOP_LEFT,
                    EdgeDirection::POINTY_TOP_RIGHT,
                ],
                open: [Air::ID, Water::ID],
            },
            // Move laterally.
            RandomSwap {
                directions: [EdgeDirection::POINTY_LEFT, EdgeDirection::POINTY_RIGHT],
                open: [Air::ID, Water::ID, Fire::ID],
            },
        )
        .apply(hex, rng, states)
    }
}
