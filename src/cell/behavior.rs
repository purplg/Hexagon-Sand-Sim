use std::array;

use hexx::{EdgeDirection, Hex};
use rand::seq::IteratorRandom;

use crate::grid::BoardState;

use super::{BoardSlice, StateId};

pub trait States: IntoIterator<Item = StateId> {}

impl<T> States for T where T: IntoIterator<Item = StateId> {}

pub trait Directions: IntoIterator<Item = EdgeDirection> {}

impl<T> Directions for T where T: IntoIterator<Item = EdgeDirection> {}

/// Conveniently convert a single StateId into an Iterator for more
/// ergonomic API when creating a behavior that takes in multiple
/// `StateId`'s but you only want to use one.
impl IntoIterator for StateId {
    type Item = StateId;

    type IntoIter = array::IntoIter<Self::Item, 1>;

    fn into_iter(self) -> Self::IntoIter {
        [self].into_iter()
    }
}

/// A mutation of the board caused by a single cell.
pub trait Step {
    fn apply<R: rand::Rng>(self, _rng: R, states: &BoardState) -> Option<BoardSlice>;

    fn apply_or<R: rand::Rng>(self, mut rng: R, states: &BoardState, s: impl Step) -> Option<BoardSlice>
    where
        Self: Sized,
    {
        self.apply(&mut rng, states)
            .or_else(|| s.apply(&mut rng, states))
    }
}

impl Step for Option<BoardSlice> {
    fn apply<R: rand::Rng>(self, _rng: R, _states: &BoardState) -> Option<BoardSlice> {
        self
    }
}

/// Step off screen
pub struct Offscreen<D: Directions, S: States> {
    pub from: Hex,
    pub directions: D,
    pub open: S,
}

impl<D: Directions, S: States> Step for Offscreen<D, S> {
    fn apply<R: rand::Rng>(self, mut rng: R, states: &BoardState) -> Option<BoardSlice> {
        let to = self
            .from
            .neighbor(self.directions.into_iter().choose(&mut rng).unwrap());
        if states.get_current(to).is_none() {
            Set {
                hex: self.from,
                into: StateId::Air,
            }
            .apply(rng, states)
        } else {
            None
        }
    }
}

/// Convert other nearby cells into another state on collision.
pub struct Infect<D: Directions, S: States, I: States> {
    pub from: Hex,
    pub directions: D,
    pub open: S,
    pub into: I,
}

impl<D: Directions, S: States, I: States> Step for Infect<D, S, I> {
    fn apply<R: rand::Rng>(self, mut rng: R, states: &BoardState) -> Option<BoardSlice> {
        let to = self
            .from
            .neighbor(self.directions.into_iter().choose(&mut rng).unwrap());
        if states.is_state(to, self.open) {
            Some(BoardSlice(vec![(
                to,
                self.into.into_iter().choose(&mut rng).unwrap(),
            )]))
        } else {
            None
        }
    }
}

/// Drag another cell.
pub struct Drag<D: Directions, S: States, P: States> {
    pub from: Hex,
    pub directions: D,
    pub open: S,
    pub drag: P,
}

impl<D: Directions, S: States, P: States> Step for Drag<D, S, P> {
    fn apply<R: rand::Rng>(self, rng: R, states: &BoardState) -> Option<BoardSlice> {
        let swap = RandomSwap {
            from: self.from,
            directions: self.directions,
            open: self.open,
        };
        let ((from, from_id), (to, to_id)) = swap.into_components(rng, states)?;
        let dir = to.main_direction_to(from);
        let drag = from.neighbor(dir);
        let drag_id = *states.get_current(drag)?;
        if states.is_state(drag, self.drag) {
            Some(BoardSlice(vec![
                (from, drag_id),
                (to, from_id),
                (drag, to_id),
            ]))
        } else {
            None
        }
    }
}

/// A chance for another step to occur.
pub struct Chance<S: Step> {
    pub step: S,
    pub chance: f32,
}

impl<S: Step> Step for Chance<S> {
    fn apply<R: rand::Rng>(self, mut rng: R, states: &BoardState) -> Option<BoardSlice> {
        let attempt = rng.gen::<f32>();
        if attempt < self.chance {
            self.step.apply(rng, states)
        } else {
            None
        }
    }
}

/// Try to swap with another cell `with_state` in some random `direction`.
pub struct RandomSwap<D: Directions, S: States> {
    pub from: Hex,
    pub directions: D,
    pub open: S,
}

impl<D: Directions, S: States> Step for RandomSwap<D, S> {
    fn apply<R: rand::Rng>(self, rng: R, states: &BoardState) -> Option<BoardSlice> {
        let (from, to) = self.into_components(rng, states)?;
        Some(BoardSlice(vec![from, to]))
    }
}

impl<D: Directions, S: States> RandomSwap<D, S> {
    fn into_components(
        self,
        mut rng: impl rand::Rng,
        states: &BoardState,
    ) -> Option<((Hex, StateId), (Hex, StateId))> {
        let to = self
            .from
            .neighbor(self.directions.into_iter().choose(&mut rng).unwrap());
        states.find_state(to, self.open).map(|other| {
            (
                (self.from, other),
                (to, *states.get_current(self.from).unwrap()),
            )
        })
    }
}

/// Swap places with another cell.
pub struct Swap {
    from: Hex,
    to: Hex,
}

impl Step for Swap {
    fn apply<R: rand::Rng>(self, mut _rng: R, states: &BoardState) -> Option<BoardSlice> {
        if states.any_set([self.from, self.to]) {
            None
        } else {
            Some(BoardSlice(vec![
                (self.from, *states.get_current(self.to).unwrap()),
                (self.to, *states.get_current(self.from).unwrap()),
            ]))
        }
    }
}

/// Set the state of a cell
pub struct Set<I: States> {
    pub hex: Hex,
    pub into: I,
}

impl<I: States> Step for Set<I> {
    fn apply<R: rand::Rng>(self, mut rng: R, states: &BoardState) -> Option<BoardSlice> {
        if states.any_set([self.hex]) {
            None
        } else {
            Some(BoardSlice(vec![(
                self.hex,
                self.into.into_iter().choose(&mut rng).unwrap(),
            )]))
        }
    }
}
