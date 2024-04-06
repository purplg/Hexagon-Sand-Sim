use bevy::prelude::*;
use hexx::{EdgeDirection, Hex};
use rand::rngs::SmallRng;

use crate::grid::BoardState;

use super::{
    behavior::{Chance, Drag, RandomSwap, Set, Step},
    BoardSlice, HexColor, Register,
    StateId::{self, *},
    Tick,
};

pub struct Water;

impl Register for Water {
    const ID: StateId = StateId::Water;
}

impl HexColor for Water {
    const COLOR: Color = Color::Rgba {
        red: 0.0,
        green: 0.0,
        blue: 1.0,
        alpha: 1.0,
    };
}
impl Tick for Water {
    fn tick(&self, from: Hex, states: &BoardState, mut rng: &mut SmallRng) -> Option<BoardSlice> {
        // Evaporate
        Chance {
            step: Set {
                hex: from,
                into: StateId::Steam,
            },
            chance: 0.0001,
        }
        // Drag sand
        .apply_or(
            &mut rng,
            states,
            Drag {
                from,
                directions: [
                    EdgeDirection::POINTY_LEFT,
                    EdgeDirection::POINTY_RIGHT,
                    EdgeDirection::POINTY_BOTTOM_LEFT,
                    EdgeDirection::POINTY_BOTTOM_RIGHT,
                ],
                open: [Air, Self::ID],
                drag: Sand,
            },
        )
        // Move down
        .apply_or(
            &mut rng,
            states,
            RandomSwap {
                from,
                directions: [
                    EdgeDirection::POINTY_BOTTOM_LEFT,
                    EdgeDirection::POINTY_BOTTOM_RIGHT,
                ],
                open: Air,
            },
        )
        // Move laterally.
        .apply_or(
            &mut rng,
            states,
            RandomSwap {
                from,
                directions: [EdgeDirection::POINTY_LEFT, EdgeDirection::POINTY_RIGHT],
                open: Air,
            },
        )
    }
}
