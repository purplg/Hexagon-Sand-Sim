use hexx::{EdgeDirection, Hex};
use pathfinding::directed::dijkstra::dijkstra;
use rand::seq::IteratorRandom;
use std::{array, fmt::Debug};

use crate::{
    cell::{Air, BoardSlice, Register as _, StateId},
    grid::BoardState,
};

pub trait States: IntoIterator<Item = StateId> + Clone + Debug {}

impl<T> States for T where T: IntoIterator<Item = StateId> + Clone + Debug {}

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

/// Try first step and if it fails, then try second.
impl<A: Step, B: Step> Step for (A, B) {
    fn apply<R: rand::Rng>(self, hex: &Hex, mut rng: R, states: &BoardState) -> Option<BoardSlice> {
        self.0
            .apply(hex, &mut rng, states)
            .or_else(|| self.1.apply(hex, &mut rng, states))
    }
}

impl<A: Step, B: Step, C: Step> Step for (A, B, C) {
    fn apply<R: rand::Rng>(self, hex: &Hex, mut rng: R, states: &BoardState) -> Option<BoardSlice> {
        self.0
            .apply(hex, &mut rng, states)
            .or_else(|| self.1.apply(hex, &mut rng, states))
            .or_else(|| self.2.apply(hex, &mut rng, states))
    }
}

impl<A: Step, B: Step, C: Step, D: Step> Step for (A, B, C, D) {
    fn apply<R: rand::Rng>(self, hex: &Hex, mut rng: R, states: &BoardState) -> Option<BoardSlice> {
        self.0
            .apply(hex, &mut rng, states)
            .or_else(|| self.1.apply(hex, &mut rng, states))
            .or_else(|| self.2.apply(hex, &mut rng, states))
            .or_else(|| self.3.apply(hex, &mut rng, states))
    }
}

impl<A: Step, B: Step, C: Step, D: Step, E: Step> Step for (A, B, C, D, E) {
    fn apply<R: rand::Rng>(self, hex: &Hex, mut rng: R, states: &BoardState) -> Option<BoardSlice> {
        self.0
            .apply(hex, &mut rng, states)
            .or_else(|| self.1.apply(hex, &mut rng, states))
            .or_else(|| self.2.apply(hex, &mut rng, states))
            .or_else(|| self.3.apply(hex, &mut rng, states))
            .or_else(|| self.4.apply(hex, &mut rng, states))
    }
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
pub struct Infect<D: Directions, O: States, I: States> {
    pub directions: D,
    pub open: O,
    pub into: I,
}

impl<D: Directions, O: States, I: States> Step for Infect<D, O, I> {
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
pub struct Drag<Dir: Directions, O: States, D: States> {
    pub directions: Dir,
    pub open: O,
    pub drag: D,
}

impl<Dir: Directions, O: States, D: States> Step for Drag<Dir, O, D> {
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

/// A chance for a step to occur.
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
    /// How likely `a` will be chosen.
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
    /// Evenly choose between `a` or `b`.
    pub fn half(a: A, b: B) -> Self {
        Self { a, b, chance: 0.5 }
    }
}

/// Assert a step is applied.
///
/// If the step fails to apply, an empty BoardSlice is returned
/// causing any other apply operations to succeed and truncating any
/// other further steps. Useful for reducing testing a section of a
/// behavior without commenting out or deleting code.
#[derive(Debug)]
pub struct Assert<S: Step>(pub S);

impl<S: Step> Step for Assert<S> {
    fn apply<R: rand::Rng>(self, hex: &Hex, rng: R, states: &BoardState) -> Option<BoardSlice> {
        self.0
            .apply(hex, rng, states)
            .or_else(|| Some(BoardSlice::EMPTY))
    }
}

/// Assert a condition is true.
///
/// If the condition returns false, an empty BoardSlice is returned
/// causing any apply operations to succeed and truncating any other
/// further steps. Useful for reducing the scope by removing
/// conditional [`Step`]'s or debugging.
///
/// # Examples
///
/// ```
/// // Do not execute anything after this statement
/// AssertFn(|| false)
/// ```
///
/// ```
/// // Assert there is an Air state to the top left of the current position.
/// AssertFn(|| states.is_state(hex.neighbor(EdgeDirection::POINTY_TOP_LEFT), &[Air::id()]))
/// ```
pub struct AssertFn<C: FnOnce() -> bool>(
    /// The condition to assert.
    pub C,
);

impl<C: FnOnce() -> bool> Step for AssertFn<C> {
    fn apply<R: rand::Rng>(self, _hex: &Hex, _rng: R, _states: &BoardState) -> Option<BoardSlice> {
        if self.0() {
            None
        } else {
            Some(BoardSlice::EMPTY)
        }
    }
}

impl<C: FnOnce() -> bool> Debug for AssertFn<C> {
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

/// Apply `then` [`Step`] only if all the `nearby` states are within
/// `range` of `count` each.
#[derive(Debug)]
pub struct Nearby<N: States, S: Step> {
    pub nearby: N,
    pub range: u32,
    pub count: usize,
    pub then: S,
}

impl<N: States, S: Step> Step for Nearby<N, S> {
    fn apply<R: rand::Rng>(self, hex: &Hex, rng: R, states: &BoardState) -> Option<BoardSlice> {
        let mut satisfied = 0;
        for state in self.nearby.clone() {
            if hex
                .xrange(self.range)
                .filter(|hex| states.is_state(*hex, state))
                .count()
                >= self.count
            {
                satisfied += 1;
            }
        }
        let count = self.nearby.into_iter().count();
        if satisfied == count {
            self.then.apply(hex, rng, states)
        } else {
            None
        }
    }
}

impl<N: States, S: Step> Nearby<N, S> {
    pub fn any_adjacent(nearby: N, then: S) -> Self {
        Self {
            nearby,
            range: 1,
            count: 1,
            then,
        }
    }

    pub fn any(nearby: N, range: u32, then: S) -> Self {
        Self {
            nearby,
            range,
            count: 1,
            then,
        }
    }

    pub fn some_adjacent(nearby: N, count: usize, then: S) -> Self {
        Self {
            nearby,
            range: 1,
            count,
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

/// Try to swap with another cell `with_state` in some random `direction`.
#[derive(Debug)]
pub struct RandomSwap<D: Directions, O: States> {
    pub directions: D,
    pub open: O,
}

impl<D: Directions, O: States> Step for RandomSwap<D, O> {
    fn apply<R: rand::Rng>(self, hex: &Hex, rng: R, states: &BoardState) -> Option<BoardSlice> {
        let (from, to) = self.into_components(hex, rng, states)?;
        Some(BoardSlice(vec![from, to]))
    }
}

impl<D: Directions, O: States> RandomSwap<D, O> {
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

/// Apply `then` while a path is `walkable` to `goal`.
#[derive(Debug)]
pub struct WhileConnected<W: States, G: States, S: Step> {
    pub walkable: W,
    pub goal: G,
    pub distance: usize,
    pub then: S,
}

impl<W: States, G: States, S: Step> Step for WhileConnected<W, G, S> {
    fn apply<R: rand::Rng>(self, start: &Hex, rng: R, states: &BoardState) -> Option<BoardSlice> {
        if let Some(_path) = dijkstra(
            start,
            |hex| {
                hex.all_neighbors()
                    // All neighbors have a weight of 1
                    .map(|hex| (hex, 1))
                    .into_iter()
                    // Only on walkable states
                    .filter(|(hex, _weight)| {
                        states.is_state(*hex, self.walkable.clone())
                            || states.is_state(*hex, self.goal.clone())
                    })
            },
            |hex| states.is_state(*hex, self.goal.clone()),
        ) {
            self.then.apply(start, rng, states)
        } else {
            None
        }
    }
}

/// Check if next to a cell in a state.
#[derive(Debug)]
pub struct NextTo<D: Directions, N: States, S: Step> {
    pub directions: D,
    pub next: N,
    pub step: S,
}

impl<D: Directions, N: States, S: Step> Step for NextTo<D, N, S> {
    fn apply<R: rand::Rng>(self, hex: &Hex, rng: R, states: &BoardState) -> Option<BoardSlice> {
        if self
            .directions
            .into_iter()
            .any(|direction| states.is_state(hex.neighbor(direction), self.next.clone()))
        {
            self.step.apply(hex, rng, states)
        } else {
            None
        }
    }
}
