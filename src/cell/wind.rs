use bevy::prelude::*;
use hexx::EdgeDirection;

use super::*;
use crate::behavior::*;

#[derive(UniqueTypeId)]
#[UniqueTypeIdType = "u32"]
pub struct Wind;

impl StateInfo for Wind {
    const NAME: &'static str = "Wind";
    const COLOR: HexColor = HexColor::Static(Color::Rgba {
        red: 1.0,
        green: 1.0,
        blue: 1.0,
        alpha: 0.01,
    });
    const HIDDEN: bool = false;
}

impl Tick for Wind {
    fn tick(&self, hex: &Hex, states: &BoardState, rng: &mut SmallRng) -> Option<BoardSlice> {
        (
            // Dissipate
            Chance {
                to: Set([Air::id()]),
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
                open: [Air::id(), Self::id()],
                drag: [Water::id(), Fire::id(), Sand::id()],
            },
            Chance {
                to: Infect {
                    directions: [
                        EdgeDirection::POINTY_LEFT,
                        EdgeDirection::POINTY_BOTTOM_LEFT,
                        EdgeDirection::POINTY_TOP_LEFT,
                    ],
                    open: [Air::id(), Self::id()],
                    into: [Self::id()],
                },
                chance: 0.01,
            },
            RandomSwap {
                directions: [
                    EdgeDirection::POINTY_LEFT,
                    EdgeDirection::POINTY_BOTTOM_LEFT,
                    EdgeDirection::POINTY_TOP_LEFT,
                ],
                open: [Air::id(), Self::id()],
            },
        )
            .apply(hex, rng, states)
    }
}
