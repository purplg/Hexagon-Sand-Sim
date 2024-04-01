use hexx::{EdgeDirection, Hex};

use crate::grid::CellStates;

use super::{Behavior, StateId::*, StepKind};

pub struct Sand;

impl Behavior for Sand {
    fn tick(from: Hex, states: &CellStates, rng: impl rand::Rng) -> Option<StepKind> {
        Self::slide(
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
