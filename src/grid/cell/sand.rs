use bevy::prelude::*;
use hexx::EdgeDirection;

use super::*;
use crate::behavior::{StateQuery::*, *};

#[derive(UniqueTypeId)]
#[UniqueTypeIdType = "u8"]
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
        scale: Vec2::ONE,
    };
    const HIDDEN: bool = false;
}

impl Behavior for Sand {
    fn tick(&self) -> impl Step {
        RandomSwap::adjacent(
            [
                EdgeDirection::POINTY_BOTTOM_LEFT,
                EdgeDirection::POINTY_BOTTOM_RIGHT,
            ],
            Any([Air::id(), Sand::id(), Steam::id()]),
        )
    }
}
