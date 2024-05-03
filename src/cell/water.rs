use bevy::prelude::*;
use hexx::EdgeDirection;
use unique_type_id::UniqueTypeId;

use super::*;
use crate::behavior::*;

#[derive(UniqueTypeId)]
#[UniqueTypeIdType = "u32"]
pub struct Water;

impl StateInfo for Water {
    const NAME: &'static str = "Water";
    const COLOR: HexColor = HexColor::Noise {
        base_color: Color::Rgba {
            red: 0.0,
            green: 0.0,
            blue: 1.0,
            alpha: 1.0,
        },
        offset_color: Color::Rgba {
            red: 0.0,
            green: 0.0,
            blue: 0.2,
            alpha: -0.2,
        },
        speed: Vec2::X,
        scale: Vec2::splat(0.02),
    };
    const HIDDEN: bool = false;
}
impl Tick for Water {
    fn tick(&self, hex: &Hex, states: &BoardState, rng: &mut SmallRng) -> Option<BoardSlice> {
        (
            // Evaporate
            Chance {
                to: Set([Steam::id()]),
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
                drag: [Sand::id(), Seed::id()],
            },
            // Move down
            RandomSwap {
                directions: [
                    EdgeDirection::POINTY_BOTTOM_LEFT,
                    EdgeDirection::POINTY_BOTTOM_RIGHT,
                ],
                open: [Air::id()],
            },
            // Move through thick materials
            Chance {
                chance: 0.01,
                to: RandomSwap {
                    directions: [
                        EdgeDirection::POINTY_TOP_LEFT,
                        EdgeDirection::POINTY_TOP_RIGHT,
                    ],
                    open: [Sand::id()],
                },
            },
            // Move laterally.
            RandomSwap {
                directions: [EdgeDirection::POINTY_LEFT, EdgeDirection::POINTY_RIGHT],
                open: [Air::id()],
            },
        )
            .apply(hex, rng, states)
    }
}
