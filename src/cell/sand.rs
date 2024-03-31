use hexx::{EdgeDirection, Hex};
use rand::seq::SliceRandom;

use crate::grid::CellStates;

use super::{Behavior, StateId, StepKind, Swap};

pub struct Sand;

impl Behavior for Sand {
    fn tick(from: Hex, states: &mut CellStates) {
        if let Some(step) = [
            EdgeDirection::POINTY_BOTTOM_LEFT,
            EdgeDirection::POINTY_BOTTOM_RIGHT,
        ]
        .choose(&mut rand::thread_rng())
        .into_iter()
        .find_map(|direction| Self::try_move(from, *direction, states))
        {
            step.apply(states)
        }
    }

    fn try_move(from: Hex, direction: EdgeDirection, states: &CellStates) -> Option<StepKind> {
        let to = from.neighbor(direction);

        if states.is_state(to, [StateId::Air, StateId::Water]) {
            return Some(StepKind::Swap(Swap { to, from }));
        }

        return None;
    }
}
