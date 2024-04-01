use hexx::{EdgeDirection, Hex};
use rand::seq::SliceRandom;

use crate::grid::CellStates;

use super::{Behavior, Set, StateId, StepKind, Swap};

pub struct Water;

impl Behavior for Water {
    fn tick(from: Hex, states: &CellStates, mut rng: impl rand::Rng) -> Option<StepKind> {
        // Chance to turn back into steam.
        let evaporate: f32 = rng.gen();
        if evaporate < 0.0001 {
            return Some(StepKind::Set(Set {
                positions: vec![from],
                states: vec![StateId::Steam],
            }));
        }

        // Try to move down
        if let Some(step) = [
            EdgeDirection::POINTY_BOTTOM_LEFT,
            EdgeDirection::POINTY_BOTTOM_RIGHT,
        ]
        .choose(&mut rng)
        .into_iter()
        .find_map(|direction| Self::try_move_air(from, *direction, states))
        {
            return Some(step);
        }

        // If it can't move down, move laterally.
        if let Some(step) = [EdgeDirection::POINTY_LEFT, EdgeDirection::POINTY_RIGHT]
            .choose(&mut rng)
            .into_iter()
            .find_map(|direction| Self::try_move_horiz(from, *direction, states))
        {
            return Some(step);
        }

        return None;
    }
}

impl Water {
    fn try_move_air(from: Hex, direction: EdgeDirection, states: &CellStates) -> Option<StepKind> {
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

        if states.is_state(to, [StateId::Air]) {
            return Some(StepKind::Swap(Swap { to, from }));
        }

        return None;
    }
}
