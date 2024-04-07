use bevy::prelude::*;
use hexx::EdgeDirection;

use super::{behavior::*, *};

pub struct Wind;

impl HexColor for Wind {
    const COLOR: Color = Color::Rgba {
        red: 1.0,
        green: 1.0,
        blue: 1.0,
        alpha: 0.01,
    };
}

impl Tick for Wind {
    fn tick(&self, hex: &Hex, states: &BoardState, rng: &mut SmallRng) -> Option<BoardSlice> {
        Or5(
            // Dissipate
            Chance {
                step: Set(Air::ID),
                chance: 0.01,
            },
            Offscreen([
                EdgeDirection::POINTY_LEFT,
                EdgeDirection::POINTY_BOTTOM_LEFT,
                EdgeDirection::POINTY_TOP_LEFT,
            ]),
            Drag {
                directions: [
                    EdgeDirection::POINTY_LEFT,
                    EdgeDirection::POINTY_BOTTOM_LEFT,
                    EdgeDirection::POINTY_TOP_LEFT,
                ],
                open: [Air::ID, Self::ID],
                drag: [Water::ID, Fire::ID, Sand::ID],
            },
            Chance {
                step: Infect {
                    directions: [
                        EdgeDirection::POINTY_LEFT,
                        EdgeDirection::POINTY_BOTTOM_LEFT,
                        EdgeDirection::POINTY_TOP_LEFT,
                    ],
                    open: [Air::ID, Self::ID],
                    into: Self::ID,
                },
                chance: 0.01,
            },
            RandomSwap {
                directions: [
                    EdgeDirection::POINTY_LEFT,
                    EdgeDirection::POINTY_BOTTOM_LEFT,
                    EdgeDirection::POINTY_TOP_LEFT,
                ],
                open: [Air::ID, Self::ID],
            },
        )
        .apply(hex, rng, states)
    }
}
