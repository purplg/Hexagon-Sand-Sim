use bevy::prelude::*;
use hexx::EdgeDirection;

use super::{behavior::*, *};

pub struct Fire;

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
            step: Set(Air::ID),
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
                open: Water::ID,
                into: Steam::ID,
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
                open: [Air::ID, Water::ID, Sand::ID],
            },
        )
    }
}
