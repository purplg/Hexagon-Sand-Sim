use hexx::{EdgeDirection, Hex};
use rand::seq::SliceRandom;

use crate::grid::CellStates;

use super::{Behavior, Set, StateId, StepKind, Swap};

pub struct Fire;

impl Behavior for Fire {
    fn tick(from: Hex, states: &CellStates, mut rng: impl rand::RngCore) -> Option<StepKind> {
        if let Some(step) = [
            EdgeDirection::POINTY_TOP_LEFT,
            EdgeDirection::POINTY_TOP_RIGHT,
        ]
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

        if states.is_state(to, StateId::Air) {
            Some(StepKind::Swap(Swap { to, from }))
        } else if states.is_state(to, StateId::Water) {
            Some(StepKind::Set(Set {
                positions: vec![from, to],
                states: vec![StateId::Air, StateId::Steam],
            }))
        } else {
            None
        }
    }
}
