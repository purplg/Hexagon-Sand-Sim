use bevy::prelude::*;
use hexx::{EdgeDirection, Hex};
use rand::rngs::SmallRng;

use crate::grid::BoardState;

use super::{
    behavior::{Chance, Drag, Infect, Offscreen, Or5, RandomSwap, Set, Step},
    BoardSlice, HexColor, Register,
    StateId::{self, *},
    Tick,
};

pub struct Wind;

impl Register for Wind {
    const ID: StateId = StateId::Wind;
}

impl HexColor for Wind {
    const COLOR: Color = Color::Rgba {
        red: 1.0,
        green: 1.0,
        blue: 1.0,
        alpha: 0.2,
    };
}

impl Tick for Wind {
    fn tick(&self, hex: &Hex, states: &BoardState, mut rng: &mut SmallRng) -> Option<BoardSlice> {
        Or5(
            // Dissipate
            Chance {
                step: Set::new(StateId::Air),
                chance: 0.01,
            },
            Offscreen {
                directions: [
                    EdgeDirection::POINTY_LEFT,
                    EdgeDirection::POINTY_BOTTOM_LEFT,
                    EdgeDirection::POINTY_TOP_LEFT,
                ],
                open: Air,
            },
            Drag {
                directions: [
                    EdgeDirection::POINTY_LEFT,
                    EdgeDirection::POINTY_BOTTOM_LEFT,
                    EdgeDirection::POINTY_TOP_LEFT,
                ],
                open: [Air, Self::ID],
                drag: [Water, Fire, Sand],
            },
            Chance {
                step: Infect {
                    directions: [
                        EdgeDirection::POINTY_LEFT,
                        EdgeDirection::POINTY_BOTTOM_LEFT,
                        EdgeDirection::POINTY_TOP_LEFT,
                    ],
                    open: [Air, Self::ID],
                    into: Self::ID,
                },
                chance: 0.01,
            },
            RandomSwap {
                directions: [
                    EdgeDirection::POINTY_LEFT,
                    EdgeDirection::POINTY_BOTTOM_LEFT,
                    EdgeDirection::POINTY_TOP_LEFT,
                ],
                open: [Air, Self::ID],
            },
        )
        .apply(hex, &mut rng, states)
    }
}
