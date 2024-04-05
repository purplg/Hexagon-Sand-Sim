use hexx::{EdgeDirection, Hex};
use rand::rngs::SmallRng;

use crate::grid::States;

use super::{
    behavior::{self, Set, StepKind},
    Register,
    StateId::{self, *},
    Tickable,
};

pub struct Steam;

impl Register for Steam {
    const ID: StateId = StateId::Steam;
}

impl Tickable for Steam {
    fn tick(&self, from: Hex, states: &States, mut rng: &mut SmallRng) -> Option<StepKind> {
        Self::try_condense(from, &mut rng)
            // Try to move up
            .or_else(|| {
                behavior::swap(
                    from,
                    [
                        EdgeDirection::POINTY_LEFT,
                        EdgeDirection::POINTY_RIGHT,
                        EdgeDirection::POINTY_TOP_LEFT,
                        EdgeDirection::POINTY_TOP_RIGHT,
                    ],
                    [Air, Water],
                    states,
                    &mut rng,
                )
            })
            // If it can't move up, move laterally.
            .or_else(|| {
                behavior::swap(
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
