use hexx::{EdgeDirection, Hex};
use rand::seq::SliceRandom;

use crate::grid::CellStates;

use super::{Behavior, Set, StateId, StepKind, Swap};

pub struct Fire;

impl Behavior for Fire {
    fn tick(from: Hex, states: &mut CellStates) {
        if let Some(step) = [
            EdgeDirection::POINTY_TOP_LEFT,
            EdgeDirection::POINTY_TOP_RIGHT,
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

        if states.is_state(to, StateId::Air) {
            return Some(StepKind::Swap(Swap { to, from }));
        } else if states.is_state(to, StateId::Water) {
            return Some(StepKind::Set(Set {
                positions: vec![to, from],
                states: vec![StateId::Steam, StateId::Steam],
            }));
        }
        return None;
    }
}
