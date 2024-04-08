use super::*;
use bevy::prelude::*;

pub struct Stone;

impl StateInfo for Stone {
    const NAME: &'static str = "Stone";
    const COLOR: Color = Color::DARK_GRAY;
    const HIDDEN: bool = false;
}

impl Tick for Stone {}
