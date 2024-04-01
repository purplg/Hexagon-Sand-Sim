use hexx::{EdgeDirection, Hex};
use rand::seq::SliceRandom;

use crate::grid::CellStates;

use super::{Behavior, Set, StateId, StepKind, Swap};

pub struct Steam;

impl Behavior for Steam {
    fn tick(from: Hex, states: &CellStates, mut rng: impl rand::Rng) -> Option<StepKind> {
        // Chance to turn back into water.
        let precipitate: f32 = rand::random();
        if precipitate < 0.01 {
            return Some(StepKind::Set(Set{
                positions: vec![from],
                states: vec![StateId::Water],
            }));
        }

        if let Some(step) = [
            EdgeDirection::POINTY_TOP_LEFT,
            EdgeDirection::POINTY_TOP_RIGHT,
        ]
        .choose(&mut rng)
        .into_iter()
        .find_map(|direction| Self::try_move(from, *direction, states))
        {
            Some(step)
        } else if let Some(step) = [EdgeDirection::POINTY_LEFT, EdgeDirection::POINTY_RIGHT]
            .choose(&mut rng)
            .into_iter()
            .find_map(|direction| Self::try_move(from, *direction, states))
        {
            Some(step)
        } else {
            None
        }
    }

    fn try_move(from: Hex, direction: EdgeDirection, states: &CellStates) -> Option<StepKind> {
        let to = from.neighbor(direction);

        if states.is_state(to, [StateId::Air, StateId::Fire, StateId::Water]) {
            return Some(StepKind::Swap(Swap { to, from }));
        }

        return None;
    }
}
