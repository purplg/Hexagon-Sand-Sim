use super::*;
use bevy::prelude::*;

pub struct Stone;

impl HexColor for Stone {
    const COLOR: Color = Color::DARK_GRAY;
}

impl Tick for Stone {}
