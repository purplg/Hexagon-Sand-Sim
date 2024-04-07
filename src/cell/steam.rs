use bevy::prelude::*;
use hexx::{EdgeDirection, Hex};
use rand::rngs::SmallRng;

use crate::grid::BoardState;

use super::{
    behavior::{Chance, RandomSwap, Set, Step},
    BoardSlice, HexColor, Register,
    StateId::{self, *},
    Tick,
};

pub struct Steam;

impl Register for Steam {
    const ID: StateId = StateId::Steam;
}

impl HexColor for Steam {
    const COLOR: Color = Color::Rgba {
        red: 0.0,
        green: 0.0,
        blue: 1.0,
        alpha: 0.5,
    };
}

impl Tick for Steam {
    fn tick(&self, hex: &Hex, states: &BoardState, mut rng: &mut SmallRng) -> Option<BoardSlice> {
        // Condense
        Chance {
            step: Set::new(StateId::Water),
            chance: 0.0001,
        }
        // Move up
        .apply_or(
            hex,
            &mut rng,
            states,
            RandomSwap {
                directions: [
                    EdgeDirection::POINTY_TOP_LEFT,
                    EdgeDirection::POINTY_TOP_RIGHT,
                ],
                open: [Air, Water],
            },
        )
        // Move laterally.
        .apply_or(
            hex,
            &mut rng,
            states,
            RandomSwap {
                directions: [EdgeDirection::POINTY_LEFT, EdgeDirection::POINTY_RIGHT],
                open: [Air, Water, Fire],
            },
        )
    }
}
