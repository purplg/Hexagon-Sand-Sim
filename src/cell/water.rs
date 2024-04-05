use hexx::{EdgeDirection, Hex};
use rand::rngs::SmallRng;

use crate::grid::States;

use super::{
    behavior::{self, Set, StepKind},
    Register,
    StateId::{self, *},
    Tickable,
};

pub struct Water;

impl Register for Water {
    const ID: StateId = StateId::Water;
}

impl Tickable for Water {
    fn tick(&self, from: Hex, states: &States, mut rng: &mut SmallRng) -> Option<StepKind> {
        Self::try_evaporate(from, rng)
            // Try to move down
            .or_else(|| {
                behavior::swap(
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
                behavior::swap(
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
    fn try_evaporate(from: Hex, rng: &mut impl rand::Rng) -> Option<StepKind> {
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
