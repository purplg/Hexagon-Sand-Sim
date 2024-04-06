use bevy::prelude::*;
use hexx::{EdgeDirection, Hex};
use rand::rngs::SmallRng;

use crate::grid::BoardState;

use super::{
    behavior::{RandomSwap, Step},
    BoardSlice, HexColor, Register,
    StateId::{self, *},
    Tick,
};

pub struct Sand;

impl Register for Sand {
    const ID: StateId = StateId::Sand;
}

impl HexColor for Sand {
    const COLOR: Color = Color::Rgba {
        red: 1.0,
        green: 1.0,
        blue: 0.0,
        alpha: 1.0,
    };
}

impl Tick for Sand {
    fn tick(&self, from: Hex, states: &BoardState, rng: &mut SmallRng) -> Option<BoardSlice> {
        RandomSwap {
            from,
            directions: [
                EdgeDirection::POINTY_BOTTOM_LEFT,
                EdgeDirection::POINTY_BOTTOM_RIGHT,
            ],
            open: [Air, Wind, Water, Steam],
        }
        .apply(rng, &states)
    }
}
