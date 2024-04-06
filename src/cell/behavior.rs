use hexx::{EdgeDirection, Hex};
use rand::seq::IteratorRandom;

use crate::grid::States;

use super::{BoardSlice, StateId};

pub trait Step {
    fn apply<R: rand::Rng>(self, _rng: R, states: &States) -> Option<BoardSlice>;

    fn apply_or<R: rand::Rng>(self, mut rng: R, states: &States, s: impl Step) -> Option<BoardSlice>
    where
        Self: Sized,
    {
        self.apply(&mut rng, states)
            .or_else(|| s.apply(&mut rng, states))
    }
}

pub struct Chance<S: Step> {
    pub step: S,
    pub chance: f32,
}

impl<S: Step> Chance<S> {
    pub fn new(step: S, chance: f32) -> Self {
        Self { step, chance }
    }
}

impl<S> Step for Chance<S>
where
    S: Step,
{
    fn apply<R: rand::Rng>(self, mut rng: R, states: &States) -> Option<BoardSlice> {
        let attempt = rng.gen::<f32>();
        if attempt < self.chance {
            self.step.apply(rng, states)
        } else {
            None
        }
    }
}

/// Try to swap with another cell `with_state` in some random `direction`.
pub struct RandomSwap<D, S>
where
    D: IntoIterator<Item = EdgeDirection>,
    S: IntoIterator<Item = StateId>,
{
    pub from: Hex,
    pub directions: D,
    pub with_state: S,
}

impl<D, S> Step for RandomSwap<D, S>
where
    D: IntoIterator<Item = EdgeDirection>,
    S: IntoIterator<Item = StateId>,
{
    fn apply<R: rand::Rng>(self, mut rng: R, states: &States) -> Option<BoardSlice> {
        let to = self
            .from
            .neighbor(self.directions.into_iter().choose(&mut rng).unwrap());
        states.find_state(to, self.with_state).map(|other| {
            BoardSlice(vec![
                (self.from, other),
                (to, *states.get_current(self.from).unwrap()),
            ])
        })
    }
}

impl Step for Option<BoardSlice> {
    fn apply<R: rand::Rng>(self, _rng: R, _states: &States) -> Option<BoardSlice> {
        self
    }
}

#[derive(Clone, Copy)]
pub struct Swap {
    from: Hex,
    to: Hex,
}

impl Step for Swap {
    fn apply<R: rand::Rng>(self, mut _rng: R, states: &States) -> Option<BoardSlice> {
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

/// Set the state of a cell.
#[derive(Clone, Copy)]
pub struct Set {
    pub hex: Hex,
    pub id: StateId,
}

impl Step for Set {
    fn apply<R: rand::Rng>(self, mut _rng: R, states: &States) -> Option<BoardSlice> {
        if states.any_set([self.hex]) {
            None
        } else {
            Some(BoardSlice(vec![(self.hex, self.id)]))
        }
    }
}
