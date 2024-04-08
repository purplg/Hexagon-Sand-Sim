use std::array;

use hexx::{EdgeDirection, Hex};
use rand::seq::IteratorRandom;

use crate::grid::BoardState;

use super::{Air, BoardSlice, Register as _, StateId};

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
    /// Try to generate a [`BoardSlice`] or return `None` if not
    /// applicable.
    fn apply<R: rand::Rng>(self, _hex: &Hex, _rng: R, _states: &BoardState) -> Option<BoardSlice>;
}

impl Step for Option<BoardSlice> {
    fn apply<R: rand::Rng>(self, _hex: &Hex, _rng: R, _states: &BoardState) -> Option<BoardSlice> {
        self
    }
}

pub struct Noop;

impl Step for Noop {
    fn apply<R: rand::Rng>(self, _hex: &Hex, _rng: R, _states: &BoardState) -> Option<BoardSlice> {
        None
    }
}

/// Fall off the screen.
pub struct Offscreen<D: Directions>(pub D);

impl<D: Directions> Step for Offscreen<D> {
    fn apply<R: rand::Rng>(self, hex: &Hex, mut rng: R, states: &BoardState) -> Option<BoardSlice> {
        let to = hex.neighbor(self.0.into_iter().choose(&mut rng).unwrap());
        if states.get_current(to).is_none() {
            Set(Air::id()).apply(hex, rng, states)
        } else {
            None
        }
    }
}

/// Convert other nearby cells into another state on collision.
pub struct Infect<D: Directions, S: States, I: States> {
    pub directions: D,
    pub open: S,
    pub into: I,
}

impl<D: Directions, S: States, I: States> Step for Infect<D, S, I> {
    fn apply<R: rand::Rng>(self, hex: &Hex, mut rng: R, states: &BoardState) -> Option<BoardSlice> {
        let to = hex.neighbor(self.directions.into_iter().choose(&mut rng).unwrap());
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
    pub directions: D,
    pub open: S,
    pub drag: P,
}

impl<D: Directions, S: States, P: States> Step for Drag<D, S, P> {
    fn apply<R: rand::Rng>(self, hex: &Hex, rng: R, states: &BoardState) -> Option<BoardSlice> {
        let swap = RandomSwap {
            directions: self.directions,
            open: self.open,
        };
        let ((from, from_id), (to, to_id)) = swap.into_components(hex, rng, states)?;
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
    fn apply<R: rand::Rng>(self, hex: &Hex, mut rng: R, states: &BoardState) -> Option<BoardSlice> {
        let attempt = rng.gen::<f32>();
        if attempt < self.chance {
            self.step.apply(hex, rng, states)
        } else {
            None
        }
    }
}

/// Randomly choose between two [`Step`]'s.
pub struct Choose<A: Step, B: Step> {
    pub a: A,
    pub b: B,
    /// How likely option `A` will be chosen.
    pub chance: f32,
}

impl<A: Step, B: Step> Step for Choose<A, B> {
    fn apply<R: rand::Rng>(self, hex: &Hex, mut rng: R, states: &BoardState) -> Option<BoardSlice> {
        if rng.gen::<f32>() < self.chance {
            self.a.apply(hex, rng, states)
        } else {
            self.b.apply(hex, rng, states)
        }
    }
}

impl<A: Step, B: Step> Choose<A, B> {
    pub fn half(a: A, b: B) -> Self {
        Self { a, b, chance: 0.5 }
    }
}

/// Conditionally apply `on_true` when predicate returns `true`,
/// otherwise apply `on_false`.
pub struct If<P, T, F>(pub P, pub T, pub F)
where
    P: FnOnce() -> bool,
    T: Step,
    F: Step;

impl<P, T, F> Step for If<P, T, F>
where
    P: FnOnce() -> bool,
    T: Step,
    F: Step,
{
    fn apply<R: rand::Rng>(self, hex: &Hex, _rng: R, states: &BoardState) -> Option<BoardSlice> {
        if (self.0)() {
            self.1.apply(hex, _rng, states)
        } else {
            self.2.apply(hex, _rng, states)
        }
    }
}

/// Conditionally apply `on_true` when predicate returns `true`.
pub struct When<P, T>(pub P, pub T)
where
    P: FnOnce() -> bool,
    T: Step;

impl<P, T> Step for When<P, T>
where
    P: FnOnce() -> bool,
    T: Step,
{
    fn apply<R: rand::Rng>(self, hex: &Hex, _rng: R, states: &BoardState) -> Option<BoardSlice> {
        if (self.0)() {
            self.1.apply(hex, _rng, states)
        } else {
            None
        }
    }
}

/// Conditionally apply `on_false` when predicate returns `false`.
pub struct Unless<P, F>(pub P, pub F)
where
    P: FnOnce() -> bool,
    F: Step;

impl<P, F> Step for Unless<P, F>
where
    P: FnOnce() -> bool,
    F: Step,
{
    fn apply<R: rand::Rng>(self, hex: &Hex, _rng: R, states: &BoardState) -> Option<BoardSlice> {
        if (self.0)() {
            None
        } else {
            self.1.apply(hex, _rng, states)
        }
    }
}

/// Try first step and if it fails, then try second.
pub struct Or<A, B>(pub A, pub B)
where
    A: Step,
    B: Step;

impl<A, B> Step for Or<A, B>
where
    A: Step,
    B: Step,
{
    fn apply<R: rand::Rng>(self, hex: &Hex, mut rng: R, states: &BoardState) -> Option<BoardSlice> {
        self.0
            .apply(hex, &mut rng, states)
            .or_else(|| self.1.apply(hex, &mut rng, states))
    }
}

pub struct Or3<A, B, C>(pub A, pub B, pub C)
where
    A: Step,
    B: Step,
    C: Step;

impl<A, B, C> Step for Or3<A, B, C>
where
    A: Step,
    B: Step,
    C: Step,
{
    fn apply<R: rand::Rng>(self, hex: &Hex, mut rng: R, states: &BoardState) -> Option<BoardSlice> {
        self.0
            .apply(hex, &mut rng, states)
            .or_else(|| self.1.apply(hex, &mut rng, states))
            .or_else(|| self.2.apply(hex, &mut rng, states))
    }
}

pub struct Or4<A, B, C, D>(pub A, pub B, pub C, pub D)
where
    A: Step,
    B: Step,
    C: Step,
    D: Step;

impl<A, B, C, D> Step for Or4<A, B, C, D>
where
    A: Step,
    B: Step,
    C: Step,
    D: Step,
{
    fn apply<R: rand::Rng>(self, hex: &Hex, mut rng: R, states: &BoardState) -> Option<BoardSlice> {
        self.0
            .apply(hex, &mut rng, states)
            .or_else(|| self.1.apply(hex, &mut rng, states))
            .or_else(|| self.2.apply(hex, &mut rng, states))
            .or_else(|| self.3.apply(hex, &mut rng, states))
    }
}

pub struct Or5<A, B, C, D, E>(pub A, pub B, pub C, pub D, pub E)
where
    A: Step,
    B: Step,
    C: Step,
    D: Step,
    E: Step;

impl<A, B, C, D, E> Step for Or5<A, B, C, D, E>
where
    A: Step,
    B: Step,
    C: Step,
    D: Step,
    E: Step,
{
    fn apply<R: rand::Rng>(self, hex: &Hex, mut rng: R, states: &BoardState) -> Option<BoardSlice> {
        self.0
            .apply(hex, &mut rng, states)
            .or_else(|| self.1.apply(hex, &mut rng, states))
            .or_else(|| self.2.apply(hex, &mut rng, states))
            .or_else(|| self.3.apply(hex, &mut rng, states))
            .or_else(|| self.4.apply(hex, &mut rng, states))
    }
}

/// Try to swap with another cell `with_state` in some random `direction`.
pub struct RandomSwap<D: Directions, S: States> {
    pub directions: D,
    pub open: S,
}

impl<D: Directions, S: States> Step for RandomSwap<D, S> {
    fn apply<R: rand::Rng>(self, hex: &Hex, rng: R, states: &BoardState) -> Option<BoardSlice> {
        let (from, to) = self.into_components(hex, rng, states)?;
        Some(BoardSlice(vec![from, to]))
    }
}

impl<D: Directions, S: States> RandomSwap<D, S> {
    fn into_components(
        self,
        hex: &Hex,
        mut rng: impl rand::Rng,
        states: &BoardState,
    ) -> Option<((Hex, StateId), (Hex, StateId))> {
        let to = hex.neighbor(self.directions.into_iter().choose(&mut rng).unwrap());
        states
            .find_state(to, self.open)
            .map(|other| ((*hex, other), (to, *states.get_current(*hex).unwrap())))
    }
}

/// Swap places with another cell.
pub struct Swap {
    other: Hex,
}

impl Step for Swap {
    fn apply<R: rand::Rng>(
        self,
        hex: &Hex,
        mut _rng: R,
        states: &BoardState,
    ) -> Option<BoardSlice> {
        if states.any_set([*hex, self.other]) {
            None
        } else {
            Some(BoardSlice(vec![
                (*hex, *states.get_current(self.other).unwrap()),
                (self.other, *states.get_current(*hex).unwrap()),
            ]))
        }
    }
}

/// Set the state of a cell
pub struct Set<I: States>(pub I);

impl<I: States> Step for Set<I> {
    fn apply<R: rand::Rng>(self, hex: &Hex, mut rng: R, states: &BoardState) -> Option<BoardSlice> {
        if states.any_set([*hex]) {
            None
        } else {
            Some(BoardSlice(vec![(
                *hex,
                self.0.into_iter().choose(&mut rng).unwrap(),
            )]))
        }
    }
}
