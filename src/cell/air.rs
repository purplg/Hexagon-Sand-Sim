use super::*;
use crate::behavior::*;

pub struct Air;

impl StateInfo for Air {
    const NAME: &'static str = "Air";
    const COLOR: HexColor = HexColor::Invisible;
    const HIDDEN: bool = false;
}

impl Tick for Air {
    fn tick(&self, hex: &Hex, states: &BoardState<64>, rng: &mut SmallRng) -> Option<BoardSlice> {
        Chance {
            step: Set(Self::id()),
            chance: 0.0001,
        }
        .apply(hex, rng, states)
    }
}
