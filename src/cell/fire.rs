use bevy::prelude::*;
use hexx::EdgeDirection;

use super::*;
use crate::behavior::*;

pub struct Fire;

impl StateInfo for Fire {
    const NAME: &'static str = "Fire";
    const COLOR: Color = Color::Rgba {
        red: 1.0,
        green: 0.0,
        blue: 0.0,
        alpha: 1.0,
    };
    const HIDDEN: bool = false;
}

impl Tick for Fire {
    fn tick(&self, hex: &Hex, states: &BoardState, rng: &mut SmallRng) -> Option<BoardSlice> {
        (
            Chance {
                step: Set(Air::id()),
                chance: 0.005,
            },
            Infect {
                directions: [
                    EdgeDirection::POINTY_LEFT,
                    EdgeDirection::POINTY_RIGHT,
                    EdgeDirection::POINTY_TOP_LEFT,
                    EdgeDirection::POINTY_TOP_RIGHT,
                ],
                open: [Water::id()],
                into: [Steam::id()],
            },
            RandomSwap {
                directions: [
                    EdgeDirection::POINTY_LEFT,
                    EdgeDirection::POINTY_RIGHT,
                    EdgeDirection::POINTY_TOP_LEFT,
                    EdgeDirection::POINTY_TOP_RIGHT,
                ],
                open: [Air::id(), Water::id(), Sand::id()],
            },
        )
            .apply(hex, rng, states)
    }
}
