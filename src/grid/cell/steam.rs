use bevy::prelude::*;
use hexx::EdgeDirection;

use super::*;
use crate::behavior::*;

#[derive(UniqueTypeId)]
#[UniqueTypeIdType = "u32"]
pub struct Steam;

impl StateInfo for Steam {
    const NAME: &'static str = "Steam";
    const COLOR: HexColor = HexColor::Static(Color::Rgba {
        red: 1.0,
        green: 1.0,
        blue: 1.0,
        alpha: 0.01,
    });
    const HIDDEN: bool = false;
}

impl Behavior for Steam {
    fn tick(&self) -> impl Step {
        (
            // Condense
            Chance {
                to: Set([Water::id()]),
                chance: 0.0001,
            },
            // Move up
            RandomSwap::adjacent(
                [
                    EdgeDirection::POINTY_TOP_LEFT,
                    EdgeDirection::POINTY_TOP_RIGHT,
                ],
                [Air::id(), Water::id()],
            ),
            // Move laterally.
            RandomSwap::adjacent(
                [EdgeDirection::POINTY_LEFT, EdgeDirection::POINTY_RIGHT],
                [Air::id(), Water::id(), Fire::id()],
            ),
        )
    }
}
