mod air;

pub use air::Air;
mod fire;
pub use fire::Fire;
mod sand;
use rand::seq::IteratorRandom;
pub use sand::Sand;
mod water;
pub use water::Water;
mod steam;
pub use steam::Steam;

use hexx::{EdgeDirection, Hex};

use crate::grid::{CellStates, NextState};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum StateId {
    Air,
    Fire,
    Sand,
    Water,
    Steam,
}

impl StateId {
    pub fn tick(&self, hex: Hex, states: &CellStates, mut rng: impl rand::Rng) -> Option<StepKind> {
        match self {
            StateId::Air => Air::tick(hex, states, &mut rng),
            StateId::Fire => Fire::tick(hex, states, &mut rng),
            StateId::Sand => Sand::tick(hex, states, &mut rng),
            StateId::Water => Water::tick(hex, states, &mut rng),
            StateId::Steam => Steam::tick(hex, states, &mut rng),
        }
    }
}

impl From<StateId> for Vec<StateId> {
    fn from(value: StateId) -> Self {
        vec![value]
    }
}

pub trait Behavior {
    fn tick(_from: Hex, _states: &CellStates, mut _rng: impl rand::Rng) -> Option<StepKind> {
        None
    }

    /// Try to swap with another cell `with_state` in a particular `direction`.
    fn slide(
        from: Hex,
        directions: impl IntoIterator<Item = EdgeDirection>,
        with_state: impl IntoIterator<Item = StateId>,
        states: &CellStates,
        mut rng: impl rand::Rng,
    ) -> Option<StepKind> {
        let to = from.neighbor(directions.into_iter().choose(&mut rng).unwrap());
        if states.is_state(to, with_state) {
            Some(StepKind::Swap(Swap { to, from }))
        } else {
            None
        }
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
    fn apply(&self, states: &mut CellStates);
}

#[derive(Clone, Copy)]
pub struct Swap {
    from: Hex,
    to: Hex,
}

impl Step for Swap {
    fn apply(&self, states: &mut CellStates) {
        if states.any_set([self.from, self.to]) {
            return;
        }

        states.set(self.from, NextState::Other(self.to));
        states.set(self.to, NextState::Other(self.from));
    }
}

/// Set the state of a cell.
#[derive(Clone, Copy)]
pub struct Set {
    hex: Hex,
    id: StateId,
}

impl Step for Set {
    fn apply(&self, states: &mut CellStates) {
        if states.any_set([self.hex]) {
            return;
        }

        states.set(self.hex, NextState::Spawn(self.id));
    }
}

/// Set the state of many cells.
impl Step for Vec<Set> {
    fn apply(&self, states: &mut CellStates) {
        let positions = self.into_iter().map(|set| set.hex);
        if states.any_set(positions) {
            return;
        }

        self.into_iter().for_each(|set| set.apply(states))
    }
}
