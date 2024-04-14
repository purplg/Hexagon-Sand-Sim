use bevy::prelude::*;
use hexx::EdgeDirection;

use super::*;
use crate::behavior::*;

pub struct Steam;

impl StateInfo for Steam {
    const NAME: &'static str = "Steam";
    const COLOR: HexColor = HexColor::Static(Color::Rgba {
        red: 1.0,
        green: 1.0,
        blue: 1.0,
        alpha: 0.5,
    });
    const HIDDEN: bool = false;
}

impl Tick for Steam {
    fn tick(&self, hex: &Hex, states: &BoardState<64>, rng: &mut SmallRng) -> Option<BoardSlice> {
        (
            // Condense
            Chance {
                step: Set(Water::id()),
                chance: 0.0001,
            },
            // Move up
            RandomSwap {
                directions: [
                    EdgeDirection::POINTY_TOP_LEFT,
                    EdgeDirection::POINTY_TOP_RIGHT,
                ],
                open: [Air::id(), Water::id()],
            },
            // Move laterally.
            RandomSwap {
                directions: [EdgeDirection::POINTY_LEFT, EdgeDirection::POINTY_RIGHT],
                open: [Air::id(), Water::id(), Fire::id()],
            },
        )
            .apply(hex, rng, states)
    }
}
