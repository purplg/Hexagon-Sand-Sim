use hexx::{EdgeDirection, Hex};
use rand::seq::SliceRandom;

use crate::grid::CellStates;

use super::{Behavior, StateId, Step, StepKind, Swap};

pub struct Water;

impl Behavior for Water {
    fn tick(from: Hex, states: &mut CellStates) {
        if let Some(step) = [
            EdgeDirection::POINTY_BOTTOM_LEFT,
            EdgeDirection::POINTY_BOTTOM_RIGHT,
        ]
        .choose(&mut rand::thread_rng())
        .into_iter()
        .find_map(|direction| Self::try_move_down(from, *direction, states))
        {
            step.apply(states)
        } else if let Some(step) = [EdgeDirection::POINTY_LEFT, EdgeDirection::POINTY_RIGHT]
            .choose(&mut rand::thread_rng())
            .into_iter()
            .find_map(|direction| Self::try_move_horiz(from, *direction, states))
        {
            step.apply(states)
        }
    }
}

impl Water {
    fn try_move_down(from: Hex, direction: EdgeDirection, states: &CellStates) -> Option<StepKind> {
        let to = from.neighbor(direction);

        if states.is_state(to, StateId::Air) {
            return Some(StepKind::Swap(Swap { to, from }));
        }

        return None;
    }

    fn try_move_horiz(
        from: Hex,
        direction: EdgeDirection,
        states: &CellStates,
    ) -> Option<StepKind> {
        let to = from.neighbor(direction);

        if states.is_state(to, [StateId::Air, StateId::Sand]) {
            return Some(StepKind::Swap(Swap { to, from }));
        }

        return None;
    }
}
