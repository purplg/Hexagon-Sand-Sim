mod air;
mod fire;
mod sand;

pub use air::Air;
pub use fire::Fire;
pub use sand::Sand;

use hexx::{EdgeDirection, Hex};

use crate::grid::{CellStates, NextState};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum StateId {
    Air,
    Fire,
    Sand,
}

pub trait Behavior {
    fn tick(_from: Hex, _states: &mut CellStates) {}

    /// Try to move in a particular direction.
    ///
    /// By default, this will only succeed if the cell in the
    /// specified direction is an Air cell;
    fn try_move(from: Hex, direction: EdgeDirection, states: &CellStates) -> Option<Box<dyn Step>> {
        let to = from.neighbor(direction);

        if !states.is_state(to, StateId::Air) {
            return None;
        }

        Some(Box::new(Swap { to, from }))
    }
}

pub trait Step {
    fn apply(&self, states: &mut CellStates);
}

struct Swap {
    from: Hex,
    to: Hex,
}

impl Step for Swap {
    fn apply(&self, states: &mut CellStates) {
        let _ = states.set(self.from, NextState::Other(self.to))
            && states.set(self.to, NextState::Other(self.from));
    }
}
