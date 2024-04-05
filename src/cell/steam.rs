use hexx::{EdgeDirection, Hex};

use crate::grid::States;

use super::{Behavior, Set, StateId::*, StepKind};

pub struct Steam;

impl Behavior for Steam {
    fn tick(from: Hex, states: &States, mut rng: impl rand::Rng) -> Option<StepKind> {
        Self::try_condense(from, &mut rng)
            // Try to move up
            .or_else(|| {
                Self::slide(
                    from,
                    [
                        EdgeDirection::POINTY_TOP_LEFT,
                        EdgeDirection::POINTY_TOP_RIGHT,
                    ],
                    [Air, Fire, Water],
                    states,
                    &mut rng,
                )
            })
            // If it can't move up, move laterally.
            .or_else(|| {
                Self::slide(
                    from,
                    [EdgeDirection::POINTY_LEFT, EdgeDirection::POINTY_RIGHT],
                    [Air, Fire, Water],
                    states,
                    &mut rng,
                )
            })
    }
}

impl Steam {
    /// Chance to turn back into water.
    fn try_condense(from: Hex, mut rng: impl rand::Rng) -> Option<StepKind> {
        let precipitate: f32 = rng.gen();
        if precipitate < 0.01 {
            Some(StepKind::Set(Set {
                hex: from,
                id: Water,
            }))
        } else {
            None
        }
    }
}
