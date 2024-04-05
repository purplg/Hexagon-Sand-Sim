use hexx::{EdgeDirection, Hex};
use rand::rngs::SmallRng;

use crate::grid::States;

use super::{
    behavior::{self, StepKind},
    Register,
    StateId::{self, *},
    Tickable,
};

pub struct Sand;

impl Register for Sand {
    const ID: StateId = StateId::Sand;
}

impl Tickable for Sand {
    fn tick(&self, from: Hex, states: &States, rng: &mut SmallRng) -> Option<StepKind> {
        behavior::slide(
            from,
            [
                EdgeDirection::POINTY_BOTTOM_LEFT,
                EdgeDirection::POINTY_BOTTOM_RIGHT,
            ],
            [Air, Water],
            states,
            rng,
        )
    }
}
