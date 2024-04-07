use bevy::prelude::*;
use hexx::EdgeDirection;

use super::{behavior::*, *};

pub struct Water;

impl HexColor for Water {
    const COLOR: Color = Color::Rgba {
        red: 0.0,
        green: 0.0,
        blue: 1.0,
        alpha: 1.0,
    };
}
impl Tick for Water {
    fn tick(&self, hex: &Hex, states: &BoardState, mut rng: &mut SmallRng) -> Option<BoardSlice> {
        Or4(
            // Evaporate
            Chance {
                step: Set(Steam::ID),
                chance: 0.0001,
            },
            // Drag sand
            Drag {
                directions: [
                    EdgeDirection::POINTY_LEFT,
                    EdgeDirection::POINTY_RIGHT,
                    EdgeDirection::POINTY_BOTTOM_LEFT,
                    EdgeDirection::POINTY_BOTTOM_RIGHT,
                ],
                open: [Air::ID, Self::ID],
                drag: Sand::ID,
            },
            // Move down
            RandomSwap {
                directions: [
                    EdgeDirection::POINTY_BOTTOM_LEFT,
                    EdgeDirection::POINTY_BOTTOM_RIGHT,
                ],
                open: Air::ID,
            },
            // Move laterally.
            RandomSwap {
                directions: [EdgeDirection::POINTY_LEFT, EdgeDirection::POINTY_RIGHT],
                open: Air::ID,
            },
        )
        .apply(hex, &mut rng, states)
    }
}
