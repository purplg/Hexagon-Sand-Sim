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
impl Behavior for Water {
    fn tick(&self) -> impl Step {
        (
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
            RandomSwap::adjacent(
                [
                    EdgeDirection::POINTY_BOTTOM_LEFT,
                    EdgeDirection::POINTY_BOTTOM_RIGHT,
                ],
                [Air::id()],
            ),
            // Move through thick materials
            Chance {
                chance: 0.01,
                to: RandomSwap::adjacent(
                    [
                        EdgeDirection::POINTY_TOP_LEFT,
                        EdgeDirection::POINTY_TOP_RIGHT,
                    ],
                    [Sand::id()],
                ),
            },
            // Move laterally
            Choose {
                a: RandomSwap {
                    directions: [EdgeDirection::POINTY_LEFT, EdgeDirection::POINTY_RIGHT],
                    open: [Air::id()],
                    distance: 5,
                    collision: true,
                },
                b: RandomSwap {
                    directions: [EdgeDirection::POINTY_LEFT, EdgeDirection::POINTY_RIGHT],
                    open: [Air::id(), Water::id()],
                    distance: 5,
                    collision: true,
                },
                chance: 0.99,
            },
        )
    }

    fn random_tick(&self) -> impl Step {
        // Evaporate
        Chance {
            to: Set([Steam::id()]),
            chance: 0.001,
        }
    }
}