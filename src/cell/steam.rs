use hexx::{EdgeDirection, Hex};
use rand::seq::SliceRandom;

use crate::grid::CellStates;

use super::{Behavior, Set, StateId, Step, Swap, StepKind};

pub struct Steam;

impl Behavior for Steam {
    fn tick(from: Hex, states: &mut CellStates) {
        // Chance to turn back into water.
        let precipitate: f32 = rand::random();
        if precipitate < 0.01 {
            Set {
                positions: vec![from],
                states: vec![StateId::Water],
            }
            .apply(states);
            return;
        }

        if let Some(step) = [
            EdgeDirection::POINTY_TOP_LEFT,
            EdgeDirection::POINTY_TOP_RIGHT,
        ]
        .choose(&mut rand::thread_rng())
        .into_iter()
        .find_map(|direction| Self::try_move(from, *direction, states))
        {
            step.apply(states)
        } else if let Some(step) = [EdgeDirection::POINTY_LEFT, EdgeDirection::POINTY_RIGHT]
            .choose(&mut rand::thread_rng())
            .into_iter()
            .find_map(|direction| Self::try_move(from, *direction, states))
        {
            step.apply(states)
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
