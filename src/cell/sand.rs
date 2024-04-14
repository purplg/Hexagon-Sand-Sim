use bevy::prelude::*;
use hexx::EdgeDirection;

use super::*;
use crate::behavior::*;

pub struct Sand;

impl StateInfo for Sand {
    const NAME: &'static str = "Sand";
    const COLOR: HexColor = HexColor::Noise {
        base_color: Color::YELLOW,
        offset_color: Color::Rgba {
            red: 0.2,
            green: 0.2,
            blue: 0.2,
            alpha: 0.0,
        },
        speed: Vec2::ZERO,
    };
    const HIDDEN: bool = false;
}

impl Tick for Sand {
    fn tick(&self, hex: &Hex, states: &BoardState<64>, rng: &mut SmallRng) -> Option<BoardSlice> {
        RandomSwap {
            directions: [
                EdgeDirection::POINTY_BOTTOM_LEFT,
                EdgeDirection::POINTY_BOTTOM_RIGHT,
            ],
            open: [Air::id(), Wind::id(), Steam::id()],
        }
        .apply(hex, rng, states)
    }
}
