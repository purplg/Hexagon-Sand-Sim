use unique_type_id::UniqueTypeId;

use super::*;
use crate::behavior::*;

#[derive(UniqueTypeId)]
#[UniqueTypeIdType = "u32"]
pub struct Air;

impl StateInfo for Air {
    const NAME: &'static str = "Air";
    const COLOR: HexColor = HexColor::Invisible;
    const HIDDEN: bool = false;
}

impl Behavior for Air {
    fn tick(&self) -> impl Step {
        Chance {
            to: Set([Self::id()]),
            chance: 0.0001,
        }
    }
}
