use bevy::prelude::*;
use hexx::EdgeDirection;

use super::*;
use crate::behavior::*;

pub struct Water;

impl StateInfo for Water {
    const NAME: &'static str = "Water";
    const COLOR: Color = Color::Rgba {
        red: 0.0,
        green: 0.0,
        blue: 1.0,
        alpha: 1.0,
    };
    const HIDDEN: bool = false;
}
impl Tick for Water {
    fn tick(&self, hex: &Hex, states: &BoardState, rng: &mut SmallRng) -> Option<BoardSlice> {
        Or5(
            // Evaporate
            Chance {
                step: Set(Steam::id()),
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
                open: [Air::id(), Self::id()],
                drag: Sand::id(),
            },
            // Move down
            RandomSwap {
                directions: [
                    EdgeDirection::POINTY_BOTTOM_LEFT,
                    EdgeDirection::POINTY_BOTTOM_RIGHT,
                ],
                open: Air::id(),
            },
            // Move through thick materials
            Chance {
                chance: 0.01,
                step: RandomSwap {
                    directions: [
                        EdgeDirection::POINTY_TOP_LEFT,
                        EdgeDirection::POINTY_TOP_RIGHT,
                    ],
                    open: Sand::id(),
                },
            },
            // Move laterally.
            RandomSwap {
                directions: [EdgeDirection::POINTY_LEFT, EdgeDirection::POINTY_RIGHT],
                open: Air::id(),
            },
        )
        .apply(hex, rng, states)
    }
}
