use bevy::prelude::*;
use hexx::EdgeDirection;

use super::*;
use crate::behavior::{StateQuery::*, *};

#[derive(UniqueTypeId)]
#[UniqueTypeIdType = "u8"]
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
            // Move up
            RandomSwap::adjacent(
                [
                    EdgeDirection::POINTY_TOP_LEFT,
                    EdgeDirection::POINTY_TOP_RIGHT,
                ],
                Any([Air::id(), Water::id()]),
            ),
            // Move laterally.
            RandomSwap::adjacent(
                [EdgeDirection::POINTY_LEFT, EdgeDirection::POINTY_RIGHT],
                Any([Air::id(), Water::id(), Fire::id()]),
            ),
            // Another larger chance to condense when not moving
            Near::some_adjacent(
                Any([Steam::id()]),
                5,
                Chance {
                    to: Set([Water::id()]),
                    chance: 0.01,
                },
            ),
        )
    }
}
