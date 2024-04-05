use hexx::{EdgeDirection, Hex};

use crate::grid::States;

use super::{Behavior, Set, StateId::*, StepKind};

pub struct Water;

impl Behavior for Water {
    fn tick(from: Hex, states: &States, mut rng: impl rand::Rng) -> Option<StepKind> {
        Self::try_evaporate(from, &mut rng)
            // Try to move down
            .or_else(|| {
                Self::slide(
                    from,
                    [
                        EdgeDirection::POINTY_BOTTOM_LEFT,
                        EdgeDirection::POINTY_BOTTOM_RIGHT,
                    ],
                    [Air],
                    states,
                    &mut rng,
                )
            })
            // If it can't move down, move laterally.
            .or_else(|| {
                Self::slide(
                    from,
                    [EdgeDirection::POINTY_LEFT, EdgeDirection::POINTY_RIGHT],
                    [Air],
                    states,
                    &mut rng,
                )
            })
    }
}

impl Water {
    /// Chance to turn back into steam.
    fn try_evaporate(from: Hex, mut rng: impl rand::Rng) -> Option<StepKind> {
        let precipitate: f32 = rng.gen();
        if precipitate < 0.0001 {
            Some(StepKind::Set(Set {
                hex: from,
                id: Steam,
            }))
        } else {
            None
        }
    }
}
