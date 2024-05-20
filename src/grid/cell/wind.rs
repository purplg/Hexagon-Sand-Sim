use bevy::prelude::*;
use hexx::EdgeDirection;

use super::*;
use crate::behavior::*;

#[derive(UniqueTypeId)]
#[UniqueTypeIdType = "u8"]
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

impl Behavior for Wind {
    fn tick(&self) -> impl Step {
        (
            Chance {
                chance: 0.1,
                to: Choose::half(
                    // Dissipate
                    Set([Air::id()]),
                    // Create more wind
                    Infect {
                        directions: [
                            EdgeDirection::POINTY_LEFT,
                            EdgeDirection::POINTY_BOTTOM_LEFT,
                            EdgeDirection::POINTY_TOP_LEFT,
                        ],
                        open: [Air::id()],
                        into: [Self::id()],
                    },
                ),
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
            RandomSwap::adjacent(
                [
                    EdgeDirection::POINTY_LEFT,
                    EdgeDirection::POINTY_BOTTOM_LEFT,
                    EdgeDirection::POINTY_TOP_LEFT,
                ],
                [Air::id(), Self::id()],
            ),
            Set([Air::id()]),
        )
    }
}
