use bevy::prelude::*;
use hexx::{EdgeDirection, Hex};
use rand::rngs::SmallRng;

use crate::grid::BoardState;

use super::{
    behavior::{Chance, Drag, Infect, Offscreen, RandomSwap, Set, Step}, BoardSlice, HexColor, Register, StateId::{self, *}, Tick
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
    fn tick(&self, from: Hex, states: &BoardState, mut rng: &mut SmallRng) -> Option<BoardSlice> {
        // Dissipate
        Chance {
            step: Set {
                hex: from,
                into: StateId::Air,
            },
            chance: 0.01,
        }
        .apply_or(
            &mut rng,
            states,
            Offscreen {
                from,
                directions: [
                    EdgeDirection::POINTY_LEFT,
                    EdgeDirection::POINTY_BOTTOM_LEFT,
                    EdgeDirection::POINTY_TOP_LEFT,
                ],
                open: Air,
            },
        )
        .apply_or(
            &mut rng,
            states,
            Drag {
                from,
                directions: [
                    EdgeDirection::POINTY_LEFT,
                    EdgeDirection::POINTY_BOTTOM_LEFT,
                    EdgeDirection::POINTY_TOP_LEFT,
                ],
                open: [Air, Self::ID],
                drag: [Water, Fire, Sand],
            },
        )
        .apply_or(
            &mut rng,
            states,
            Chance {
                step: Infect {
                    from,
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
        )
        .apply_or(
            &mut rng,
            states,
            RandomSwap {
                from,
                directions: [
                    EdgeDirection::POINTY_LEFT,
                    EdgeDirection::POINTY_BOTTOM_LEFT,
                    EdgeDirection::POINTY_TOP_LEFT,
                ],
                open: [Air, Self::ID],
            },
        )
    }
}
