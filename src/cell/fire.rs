use hexx::{EdgeDirection, Hex};
use rand::seq::IteratorRandom;

use crate::grid::States;

use super::{Behavior, Set, StateId, StateId::*, StepKind, Swap};

pub struct Fire;

impl Behavior for Fire {
    fn tick(from: Hex, states: &States, rng: impl rand::RngCore) -> Option<StepKind> {
        Self::slide(
            from,
            [
                EdgeDirection::POINTY_TOP_LEFT,
                EdgeDirection::POINTY_TOP_RIGHT,
            ],
            [],
            states,
            rng,
        )
    }

    fn slide<D, S>(
        from: Hex,
        directions: D,
        _with_state: S,
        states: &States,
        mut rng: impl rand::Rng,
    ) -> Option<StepKind>
    where
        D: IntoIterator<Item = EdgeDirection>,
        S: IntoIterator<Item = StateId>,
    {
        let to = from.neighbor(directions.into_iter().choose(&mut rng).unwrap());

        if states.is_state(to, [Air]) {
            Some(StepKind::Swap(Swap { to, from }))
        } else if states.is_state(to, [Water]) {
            Some(StepKind::SetMany(vec![
                Set {
                    hex: from,
                    id: StateId::Steam,
                },
                Set {
                    hex: to,
                    id: StateId::Steam,
                },
            ]))
        } else {
            None
        }
    }
}
