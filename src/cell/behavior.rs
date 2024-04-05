use hexx::{EdgeDirection, Hex};
use rand::seq::IteratorRandom;

use crate::grid::States;

use super::StateId;

/// Try to swap with another cell `with_state` in a particular `direction`.
pub fn slide<D, S>(
    from: Hex,
    directions: D,
    with_state: S,
    states: &States,
    mut rng: impl rand::Rng,
) -> Option<StepKind>
where
    D: IntoIterator<Item = EdgeDirection>,
    S: IntoIterator<Item = StateId>,
{
    let to = from.neighbor(directions.into_iter().choose(&mut rng).unwrap());
    if states.is_state(to, with_state) {
        Some(StepKind::Swap(Swap { to, from }))
    } else {
        None
    }
}

#[derive(Clone)]
pub enum StepKind {
    Swap(Swap),
    Set(Set),
    SetMany(Vec<Set>),
}

impl std::ops::Deref for StepKind {
    type Target = dyn Step;

    fn deref(&self) -> &Self::Target {
        match self {
            StepKind::Swap(inner) => inner,
            StepKind::Set(inner) => inner,
            StepKind::SetMany(inner) => inner,
        }
    }
}

pub trait Step {
    fn apply(&self, states: &mut States);
}

#[derive(Clone, Copy)]
pub struct Swap {
    from: Hex,
    to: Hex,
}

impl Step for Swap {
    fn apply(&self, states: &mut States) {
        if states.any_set([self.from, self.to]) {
            return;
        }

        states.set(self.from, *states.get_current(self.to).unwrap());
        states.set(self.to, *states.get_current(self.from).unwrap());
    }
}

/// Set the state of a cell.
#[derive(Clone, Copy)]
pub struct Set {
    pub hex: Hex,
    pub id: StateId,
}

impl Step for Set {
    fn apply(&self, states: &mut States) {
        if states.any_set([self.hex]) {
            return;
        }

        states.set(self.hex, self.id);
    }
}

/// Set the state of many cells.
impl Step for Vec<Set> {
    fn apply(&self, states: &mut States) {
        let positions = self.iter().map(|set| set.hex);
        if states.any_set(positions) {
            return;
        }

        self.iter().for_each(|set| set.apply(states))
    }
}
