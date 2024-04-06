use hexx::{EdgeDirection, Hex};
use rand::seq::IteratorRandom;

use crate::grid::States;

use super::{BoardSlice, StateId};

/// A mutation of the board caused by a single cell.
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

impl Step for Option<BoardSlice> {
    fn apply<R: rand::Rng>(self, _rng: R, _states: &States) -> Option<BoardSlice> {
        self
    }
}

/// Convert other nearby cells into another state on collision.
pub struct Infect<D, S>
where
    D: IntoIterator<Item = EdgeDirection>,
    S: IntoIterator<Item = StateId>,
{
    pub from: Hex,
    pub directions: D,
    pub with_state: S,
    pub into: StateId,
}

impl<D, S> Step for Infect<D, S>
where
    D: IntoIterator<Item = EdgeDirection>,
    S: IntoIterator<Item = StateId>,
{
    fn apply<R: rand::Rng>(self, mut rng: R, states: &States) -> Option<BoardSlice> {
        let to = self
            .from
            .neighbor(self.directions.into_iter().choose(&mut rng).unwrap());
        if states.is_state(to, self.with_state) {
            Some(BoardSlice(vec![(to, self.into)]))
        } else {
            None
        }
    }
}

/// Drag another cell.
pub struct Drag<D, S, P>
where
    D: IntoIterator<Item = EdgeDirection>,
    S: IntoIterator<Item = StateId>,
    P: IntoIterator<Item = StateId>,
{
    pub from: Hex,
    pub directions: D,
    pub open: S,
    pub drag: P,
}

impl<D, S, P> Step for Drag<D, S, P>
where
    D: IntoIterator<Item = EdgeDirection>,
    S: IntoIterator<Item = StateId>,
    P: IntoIterator<Item = StateId>,
{
    fn apply<R: rand::Rng>(self, rng: R, states: &States) -> Option<BoardSlice> {
        let swap = RandomSwap {
            from: self.from,
            directions: self.directions,
            with_state: self.open,
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
    fn apply<R: rand::Rng>(self, rng: R, states: &States) -> Option<BoardSlice> {
        let (from, to) = self.into_components(rng, states)?;
        Some(BoardSlice(vec![from, to]))
    }
}

impl<D, S> RandomSwap<D, S>
where
    D: IntoIterator<Item = EdgeDirection>,
    S: IntoIterator<Item = StateId>,
{
    fn into_components(
        self,
        mut rng: impl rand::Rng,
        states: &States,
    ) -> Option<((Hex, StateId), (Hex, StateId))> {
        let to = self
            .from
            .neighbor(self.directions.into_iter().choose(&mut rng).unwrap());
        states.find_state(to, self.with_state).map(|other| {
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

/// Set the state of a cell
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
