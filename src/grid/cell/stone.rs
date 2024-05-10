use bevy::prelude::*;

use super::*;

#[derive(UniqueTypeId)]
#[UniqueTypeIdType = "u32"]
pub struct Stone;

impl StateInfo for Stone {
    const NAME: &'static str = "Stone";
    const COLOR: HexColor = HexColor::Static(Color::DARK_GRAY);
    const HIDDEN: bool = false;
}

impl Behavior for Stone {}
