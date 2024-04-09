use std::{array, fmt::Debug};

use hexx::{EdgeDirection, Hex};
use rand::seq::IteratorRandom;

use crate::{
    cell::{Air, BoardSlice, Register as _, StateId},
    grid::BoardState,
};

pub trait States: IntoIterator<Item = StateId> + Debug {}

impl<T> States for T where T: IntoIterator<Item = StateId> + Debug {}

pub trait Directions: IntoIterator<Item = EdgeDirection> + Debug {}

impl<T> Directions for T where T: IntoIterator<Item = EdgeDirection> + Debug {}

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
pub trait Step: Debug {
    /// Try to generate a [`BoardSlice`] or return `None` if not
    /// applicable.
    fn apply<R: rand::Rng>(self, _hex: &Hex, _rng: R, _states: &BoardState) -> Option<BoardSlice>;
}

impl Step for Option<BoardSlice> {
    fn apply<R: rand::Rng>(self, _hex: &Hex, _rng: R, _states: &BoardState) -> Option<BoardSlice> {
        self
    }
}

#[derive(Debug)]
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

impl<D: Directions> Debug for Offscreen<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Offscreen({:?})", self.0)
    }
}

/// Convert other nearby cells into another state on collision.
#[derive(Debug)]
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
#[derive(Debug)]
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
#[derive(Debug)]
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
#[derive(Debug)]
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

/// Assert the condition is true, then an empty BoardSlice is return
/// causing any apply operations to succeed and truncating any other
/// further steps. Useful for reducing the scope by removing
/// conditional [`Step`]'s.
pub struct Assert<C: FnOnce() -> bool>(pub C);

impl<C: FnOnce() -> bool> Step for Assert<C> {
    fn apply<R: rand::Rng>(self, _hex: &Hex, _rng: R, _states: &BoardState) -> Option<BoardSlice> {
        if self.0() {
            None
        } else {
            Some(BoardSlice::EMPTY)
        }
    }
}

impl<C: FnOnce() -> bool> Debug for Assert<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Assert")
    }
}

/// Print out the type with a prefix message and apply some step while
/// in a behavior.
#[derive(Debug)]
pub struct Output<'a, T>(pub &'a str, pub T);

impl<'a, T: Step> Step for Output<'a, T> {
    fn apply<R: rand::Rng>(self, hex: &Hex, rng: R, states: &BoardState) -> Option<BoardSlice> {
        println!("{}: {:?}", self.0, self.1);
        self.1.apply(hex, rng, states)
    }
}

/// Print out a message without doing anything.
#[derive(Debug)]
pub struct Message<'a>(pub &'a str);

impl<'a> Step for Message<'a> {
    fn apply<R: rand::Rng>(self, _hex: &Hex, _rng: R, _states: &BoardState) -> Option<BoardSlice> {
        println!("{}", self.0);
        None
    }
}

/// Apply `then` [`Step`] only if there is a cell of any provided
/// states nearby.
#[derive(Debug)]
pub struct WhenNearby<N: States, S: Step> {
    pub nearby: N,
    pub range: u32,
    pub count: usize,
    pub then: S,
}

impl<'a, N: States, S: Step> Step for WhenNearby<N, S> {
    fn apply<R: rand::Rng>(self, hex: &Hex, rng: R, states: &BoardState) -> Option<BoardSlice> {
        for state in self.nearby {
            if hex
                .xrange(self.range)
                .filter(|hex| states.is_state(*hex, state))
                .count()
                >= self.count
            {
                return self.then.apply(hex, rng, states);
            }
        }
        None
    }
}

impl<N: States, S: Step> WhenNearby<N, S> {
    pub fn any_adjacent(nearby: N, then: S) -> Self {
        Self {
            nearby,
            range: 1,
            count: 1,
            then,
        }
    }

    #[allow(unused)]
    pub fn any(nearby: N, then: S, range: u32) -> Self {
        Self {
            nearby,
            range,
            count: 1,
            then,
        }
    }
}

/// Conditionally apply `on_true` when condition returns `true`,
/// otherwise apply `on_false`.
pub struct If<C, T, F>(pub C, pub T, pub F)
where
    C: FnOnce() -> bool,
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

impl<C, T, F> Debug for If<C, T, F>
where
    C: FnOnce() -> bool,
    T: Step,
    F: Step,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "If({:?} else {:?})", self.1, self.2)
    }
}

/// Conditionally apply `on_true` when condition returns `true`.
pub struct When<C, T>(pub C, pub T)
where
    C: FnOnce() -> bool,
    T: Step;

impl<C, T> Step for When<C, T>
where
    C: FnOnce() -> bool,
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

impl<C, T> Debug for When<C, T>
where
    C: FnOnce() -> bool,
    T: Step,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "When({:?})", self.1)
    }
}

/// Conditionally apply `on_false` when predicate returns `false`.
pub struct Unless<C, F>(pub C, pub F)
where
    C: FnOnce() -> bool,
    F: Step;

impl<C, F> Step for Unless<C, F>
where
    C: FnOnce() -> bool,
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

impl<C, F> Debug for Unless<C, F>
where
    C: FnOnce() -> bool,
    F: Step,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unless({:?})", self.1)
    }
}

/// Try first step and if it fails, then try second.
#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
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
#[derive(Debug)]
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
#[derive(Debug)]
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
#[derive(Debug)]
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
