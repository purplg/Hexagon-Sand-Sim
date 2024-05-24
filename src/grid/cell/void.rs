use bevy::prelude::*;
use hexx::EdgeDirection;
use unique_type_id::UniqueTypeId;

use super::*;
use crate::behavior::*;

#[derive(UniqueTypeId)]
#[UniqueTypeIdType = "u8"]
pub struct Void;

impl StateInfo for Void {
    const NAME: &'static str = "Void";
    const COLOR: HexColor = HexColor::Static(Color::rgb(0.2, 0.0, 0.2));
    const HIDDEN: bool = false;
}
impl Behavior for Void {
    fn tick(&self) -> impl Step {
        Infect {
            directions: EdgeDirection::ALL_DIRECTIONS,
            open: [Water::id(), Sand::id(), Steam::id()],
            into: [Air::id()],
        }
    }
}
