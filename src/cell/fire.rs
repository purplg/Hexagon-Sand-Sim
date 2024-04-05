use hexx::{EdgeDirection, Hex};
use rand::rngs::SmallRng;

use crate::grid::States;

use super::{behavior::{self, StepKind}, Register, StateId, Tickable};

pub struct Fire;

impl Register for Fire {
    const ID: StateId = StateId::Fire;
}

impl Tickable for Fire {
    fn tick(&self, from: Hex, states: &States, rng: &mut SmallRng) -> Option<StepKind> {
        behavior::swap(
            from,
            [
                EdgeDirection::POINTY_LEFT,
                EdgeDirection::POINTY_RIGHT,
                EdgeDirection::POINTY_TOP_LEFT,
                EdgeDirection::POINTY_TOP_RIGHT,
            ],
            [StateId::Air, StateId::Water, StateId::Sand],
            states,
            rng,
        )
    }
}
