mod air;

pub use air::Air;
mod fire;
pub use fire::Fire;
mod sand;
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

impl From<StateId> for Vec<StateId> {
    fn from(value: StateId) -> Self {
        vec![value]
    }
}

pub trait Behavior {
    fn tick(_from: Hex, _states: &CellStates, mut _rng: impl rand::Rng) -> Option<StepKind> {
        None
    }

    /// Try to move in a particular direction.
    ///
    /// By default, this will only succeed if the cell in the
    /// specified direction is an Air cell;
    fn try_move(from: Hex, direction: EdgeDirection, states: &CellStates) -> Option<StepKind> {
        let to = from.neighbor(direction);
        if states.is_state(to, StateId::Air) {
            Some(StepKind::Swap(Swap { to, from }))
        } else {
            None
        }
    }
}

pub enum StepKind {
    Swap(Swap),
    Set(Set),
}

impl std::ops::Deref for StepKind {
    type Target = dyn Step;

    fn deref(&self) -> &Self::Target {
        match self {
            StepKind::Swap(inner) => inner,
            StepKind::Set(inner) => inner,
        }
    }
}

pub trait Step {
    fn apply(&self, states: &mut CellStates);
}

pub struct Swap {
    from: Hex,
    to: Hex,
}

impl Step for Swap {
    fn apply(&self, states: &mut CellStates) {
        if states.any_set([&self.from, &self.to]) {
            return;
        }

        states.set(self.from, NextState::Other(self.to));
        states.set(self.to, NextState::Other(self.from));
    }
}

/// Set the state of many cells.
///
/// The index positions in [`Self::positions`] map 1:1 to
/// [`Self::states`].
pub struct Set {
    positions: Vec<Hex>,
    states: Vec<StateId>,
}

impl Step for Set {
    fn apply(&self, states: &mut CellStates) {
        if states.any_set(&self.positions) {
            return;
        }

        for (hex, id) in self.positions.iter().zip(self.states.iter()) {
            states.set(*hex, NextState::Spawn(*id));
        }
    }
}
