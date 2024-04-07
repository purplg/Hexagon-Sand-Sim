use bevy::prelude::*;
use hexx::{EdgeDirection, Hex};
use rand::rngs::SmallRng;

use crate::grid::BoardState;

use super::{
    behavior::{Chance, Infect, RandomSwap, Set, Step},
    BoardSlice, HexColor, Register, StateId, Tick,
};

pub struct Fire;

impl Register for Fire {
    const ID: StateId = StateId::Fire;
}

impl HexColor for Fire {
    const COLOR: Color = Color::Rgba {
        red: 1.0,
        green: 0.0,
        blue: 0.0,
        alpha: 1.0,
    };
}

impl Tick for Fire {
    fn tick(&self, hex: &Hex, states: &BoardState, mut rng: &mut SmallRng) -> Option<BoardSlice> {
        Chance {
            step: Set::new(StateId::Air),
            chance: 0.005,
        }
        .apply_or(
            hex,
            &mut rng,
            states,
            Infect {
                directions: [
                    EdgeDirection::POINTY_LEFT,
                    EdgeDirection::POINTY_RIGHT,
                    EdgeDirection::POINTY_TOP_LEFT,
                    EdgeDirection::POINTY_TOP_RIGHT,
                ],
                open: StateId::Water,
                into: StateId::Steam,
            },
        )
        .apply_or(
            hex,
            &mut rng,
            states,
            RandomSwap {
                directions: [
                    EdgeDirection::POINTY_LEFT,
                    EdgeDirection::POINTY_RIGHT,
                    EdgeDirection::POINTY_TOP_LEFT,
                    EdgeDirection::POINTY_TOP_RIGHT,
                ],
                open: [StateId::Air, StateId::Water, StateId::Sand],
            },
        )
    }
}
