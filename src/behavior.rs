use hexx::{EdgeDirection, Hex};
use pathfinding::directed::dijkstra::dijkstra;
use rand::seq::IteratorRandom;
use std::fmt::Debug;
use unique_type_id::{TypeId, UniqueTypeId as _};

use crate::grid::{
    cell::{Air, BoardSlice},
    BoardState,
};

pub type StateId = TypeId<u32>;

pub trait States: IntoIterator<Item = StateId> + Clone + Debug {}

impl<T> States for T where T: IntoIterator<Item = StateId> + Clone + Debug {}

pub trait Directions: IntoIterator<Item = EdgeDirection> + Debug {}

impl<T> Directions for T where T: IntoIterator<Item = EdgeDirection> + Debug {}

/// A mutation of the board caused by a single cell.
pub trait Step {
    /// Try to generate a [`BoardSlice`] or return `None` if not
    /// applicable.
    fn apply<R: rand::Rng>(
        self,
        _hex: &Hex,
        _states: &BoardState,
        _rng: &mut R,
    ) -> Option<BoardSlice>;
}

/// Try first step and if it fails, then try second.
impl<A: Step, B: Step> Step for (A, B) {
    fn apply<R: rand::Rng>(
        self,
        hex: &Hex,
        states: &BoardState,
        rng: &mut R,
    ) -> Option<BoardSlice> {
        self.0
            .apply(hex, states, rng)
            .or_else(|| self.1.apply(hex, states, rng))
    }
}

/// Try first step and if it fails, then try second, and so on...
impl<A: Step, B: Step, C: Step> Step for (A, B, C) {
    fn apply<R: rand::Rng>(
        self,
        hex: &Hex,
        states: &BoardState,
        rng: &mut R,
    ) -> Option<BoardSlice> {
        self.0
            .apply(hex, states, rng)
            .or_else(|| self.1.apply(hex, states, rng))
            .or_else(|| self.2.apply(hex, states, rng))
    }
}

/// Try first step and if it fails, then try second, and so on...
impl<A: Step, B: Step, C: Step, D: Step> Step for (A, B, C, D) {
    fn apply<R: rand::Rng>(
        self,
        hex: &Hex,
        states: &BoardState,
        rng: &mut R,
    ) -> Option<BoardSlice> {
        self.0
            .apply(hex, states, rng)
            .or_else(|| self.1.apply(hex, states, rng))
            .or_else(|| self.2.apply(hex, states, rng))
            .or_else(|| self.3.apply(hex, states, rng))
    }
}

/// Try first step and if it fails, then try second, and so on...
impl<A: Step, B: Step, C: Step, D: Step, E: Step> Step for (A, B, C, D, E) {
    fn apply<R: rand::Rng>(
        self,
        hex: &Hex,
        states: &BoardState,
        rng: &mut R,
    ) -> Option<BoardSlice> {
        self.0
            .apply(hex, states, rng)
            .or_else(|| self.1.apply(hex, states, rng))
            .or_else(|| self.2.apply(hex, states, rng))
            .or_else(|| self.3.apply(hex, states, rng))
            .or_else(|| self.4.apply(hex, states, rng))
    }
}

/// Do nothing.
///
/// Useful as a placeholder for another [`Step`] while writing a
/// cells' behavior.
#[derive(Debug)]
pub struct Noop;

impl Step for Noop {
    fn apply<R: rand::Rng>(
        self,
        _hex: &Hex,
        _states: &BoardState,
        _rng: &mut R,
    ) -> Option<BoardSlice> {
        None
    }
}

/// Fall off the screen.
///
/// If this cell on touching the edge of a screen in any of the
/// specified direction, then it turns to an [`Air`] state.
pub struct Offscreen<D: Directions>(pub D);

impl<D: Directions> Step for Offscreen<D> {
    fn apply<R: rand::Rng>(
        self,
        hex: &Hex,
        states: &BoardState,
        rng: &mut R,
    ) -> Option<BoardSlice> {
        if self
            .0
            .into_iter()
            .map(|direction| hex.neighbor(direction))
            .any(|hex| states.get_current(hex).is_none())
        {
            Set([Air::id()]).apply(hex, states, rng)
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
    fn apply<R: rand::Rng>(
        self,
        hex: &Hex,
        states: &BoardState,
        rng: &mut R,
    ) -> Option<BoardSlice> {
        let to = hex.neighbor(self.directions.into_iter().choose(rng).unwrap());
        if states.is_state(to, self.open) {
            Some(BoardSlice(vec![(
                to,
                self.into.into_iter().choose(rng).unwrap(),
            )]))
        } else {
            None
        }
    }
}

/// Like [`Infect`], except both cells turn into the same state.
#[derive(Debug)]
pub struct Annihilate<D: Directions, O: States, I: States> {
    pub directions: D,
    pub open: O,
    pub into: I,
}

impl<D: Directions, O: States, I: States> Step for Annihilate<D, O, I> {
    fn apply<R: rand::Rng>(
        self,
        hex: &Hex,
        states: &BoardState,
        rng: &mut R,
    ) -> Option<BoardSlice> {
        let to = hex.neighbor(self.directions.into_iter().choose(rng).unwrap());
        if states.is_state(to, self.open) {
            let id = self.into.into_iter().choose(rng).unwrap();
            Some(BoardSlice(vec![(*hex, id), (to, id)]))
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
    fn apply<R: rand::Rng>(
        self,
        hex: &Hex,
        states: &BoardState,
        rng: &mut R,
    ) -> Option<BoardSlice> {
        let swap = RandomSwap::adjacent(self.directions, self.open);
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
    pub to: S,
    pub chance: f32,
}

impl<S: Step> Step for Chance<S> {
    fn apply<R: rand::Rng>(
        self,
        hex: &Hex,
        states: &BoardState,
        rng: &mut R,
    ) -> Option<BoardSlice> {
        let attempt = rng.gen::<f32>();
        if attempt < self.chance {
            self.to.apply(hex, states, rng)
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
    fn apply<R: rand::Rng>(
        self,
        hex: &Hex,
        states: &BoardState,
        rng: &mut R,
    ) -> Option<BoardSlice> {
        if rng.gen::<f32>() < self.chance {
            self.a.apply(hex, states, rng)
        } else {
            self.b.apply(hex, states, rng)
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
    fn apply<R: rand::Rng>(
        self,
        hex: &Hex,
        states: &BoardState,
        rng: &mut R,
    ) -> Option<BoardSlice> {
        self.0.apply(hex, states, rng).or(Some(BoardSlice::EMPTY))
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
    fn apply<R: rand::Rng>(
        self,
        _hex: &Hex,
        _states: &BoardState,
        _rng: &mut R,
    ) -> Option<BoardSlice> {
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

impl<'a, T: Step + Debug> Step for Output<'a, T> {
    fn apply<R: rand::Rng>(
        self,
        hex: &Hex,
        states: &BoardState,
        rng: &mut R,
    ) -> Option<BoardSlice> {
        println!("{}: {:?}", self.0, self.1);
        self.1.apply(hex, states, rng)
    }
}

/// Print out a message without doing anything.
#[derive(Debug)]
pub struct Message<'a>(pub &'a str);

impl<'a> Step for Message<'a> {
    fn apply<R: rand::Rng>(
        self,
        _hex: &Hex,
        _states: &BoardState,
        _rng: &mut R,
    ) -> Option<BoardSlice> {
        println!("{}", self.0);
        None
    }
}

#[derive(Debug)]
pub struct MaybeNear<S: States, O: Step, X: Step> {
    states: S,
    range: u32,
    count: usize,
    then: O,
    otherwise: X,
}

impl<S: States, O: Step, X: Step> Step for MaybeNear<S, O, X> {
    fn apply<R: rand::Rng>(
        self,
        hex: &Hex,
        states: &BoardState,
        rng: &mut R,
    ) -> Option<BoardSlice> {
        let mut satisfied = 0;
        for state in self.states.clone() {
            if hex
                .xrange(self.range)
                .filter(|hex| states.is_state(*hex, [state]))
                .count()
                >= self.count
            {
                satisfied += 1;
            }
        }
        let count = self.states.into_iter().count();
        if satisfied == count {
            self.then.apply(hex, states, rng)
        } else {
            self.otherwise.apply(hex, states, rng)
        }
    }
}

impl<S: States, O: Step, X: Step> MaybeNear<S, O, X> {
    pub fn new(states: S, range: u32, count: usize, then: O, otherwise: X) -> Self {
        Self {
            states,
            range,
            count,
            then,
            otherwise,
        }
    }

    pub fn any_adjacent(states: S, then: O, otherwise: X) -> Self {
        Self {
            states,
            range: 1,
            count: 1,
            then,
            otherwise,
        }
    }

    pub fn any(states: S, range: u32, then: O, otherwise: X) -> Self {
        Self {
            states,
            range,
            count: 1,
            then,
            otherwise,
        }
    }

    pub fn some_adjacent(states: S, count: usize, then: O, otherwise: X) -> Self {
        Self {
            states,
            range: 1,
            count,
            then,
            otherwise,
        }
    }
}

/// Apply `then` [`Step`] only if all the `nearby` states are within
/// `range` of `count` each.
pub struct Near;

impl Near {
    pub fn new<S: States, O: Step>(
        states: S,
        range: u32,
        count: usize,
        then: O,
    ) -> MaybeNear<S, O, Noop> {
        MaybeNear::new(states, range, count, then, Noop)
    }

    pub fn any_adjacent<S: States, O: Step>(states: S, then: O) -> MaybeNear<S, O, Noop> {
        MaybeNear::any_adjacent(states, then, Noop)
    }

    pub fn any<S: States, O: Step>(states: S, range: u32, then: O) -> MaybeNear<S, O, Noop> {
        MaybeNear::any(states, range, then, Noop)
    }

    pub fn some_adjacent<S: States, O: Step>(
        states: S,
        count: usize,
        then: O,
    ) -> MaybeNear<S, O, Noop> {
        MaybeNear::some_adjacent(states, count, then, Noop)
    }
}

/// Apply `then` [`Step`] only if none of the `nearby` states are
/// within `range` of `count` each.
pub struct NotNear;

impl NotNear {
    pub fn new<S: States, X: Step>(
        states: S,
        range: u32,
        count: usize,
        then: X,
    ) -> MaybeNear<S, Noop, X> {
        MaybeNear::new(states, range, count, Noop, then)
    }

    pub fn any_adjacent<S: States, X: Step>(states: S, then: X) -> MaybeNear<S, Noop, X> {
        MaybeNear::any_adjacent(states, Noop, then)
    }

    pub fn any<S: States, X: Step>(states: S, range: u32, then: X) -> MaybeNear<S, Noop, X> {
        MaybeNear::any(states, range, Noop, then)
    }

    pub fn some_adjacent<S: States, X: Step>(
        states: S,
        count: usize,
        then: X,
    ) -> MaybeNear<S, Noop, X> {
        MaybeNear::some_adjacent(states, count, Noop, then)
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
    fn apply<R: rand::Rng>(
        self,
        hex: &Hex,
        states: &BoardState,
        _rng: &mut R,
    ) -> Option<BoardSlice> {
        if (self.0)() {
            self.1.apply(hex, states, _rng)
        } else {
            self.2.apply(hex, states, _rng)
        }
    }
}

impl<C, T, F> Debug for If<C, T, F>
where
    C: FnOnce() -> bool,
    T: Step + Debug,
    F: Step + Debug,
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
    fn apply<R: rand::Rng>(
        self,
        hex: &Hex,
        states: &BoardState,
        _rng: &mut R,
    ) -> Option<BoardSlice> {
        if (self.0)() {
            self.1.apply(hex, states, _rng)
        } else {
            None
        }
    }
}

impl<C, T> Debug for When<C, T>
where
    C: FnOnce() -> bool,
    T: Step + Debug,
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
    fn apply<R: rand::Rng>(
        self,
        hex: &Hex,
        states: &BoardState,
        _rng: &mut R,
    ) -> Option<BoardSlice> {
        if (self.0)() {
            None
        } else {
            self.1.apply(hex, states, _rng)
        }
    }
}

impl<C, F> Debug for Unless<C, F>
where
    C: FnOnce() -> bool,
    F: Step + Debug,
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
    pub distance: i32,
    pub try_farthest: bool,
}

impl<D: Directions, S: States> RandomSwap<D, S> {
    pub fn adjacent(directions: D, open: S) -> Self {
        Self {
            directions,
            open,
            distance: 1,
            try_farthest: false,
        }
    }
}

impl<D: Directions, O: States> Step for RandomSwap<D, O> {
    fn apply<R: rand::Rng>(
        self,
        hex: &Hex,
        states: &BoardState,
        rng: &mut R,
    ) -> Option<BoardSlice> {
        if self.try_farthest {
            while let Some((from, to)) = self.into_components(hex, rng, states) {
                return Some(BoardSlice(vec![from, to]));
            }
            None
        } else {
            let (from, to) = self.into_components(hex, rng, states)?;
            Some(BoardSlice(vec![from, to]))
        }
    }
}

impl<D: Directions, O: States> RandomSwap<D, O> {
    fn into_components<R: rand::Rng>(
        self,
        hex: &Hex,
        rng: &mut R,
        states: &BoardState,
    ) -> Option<((Hex, StateId), (Hex, StateId))> {
        let to: Hex = (self.directions.into_iter().choose(rng).unwrap() * self.distance) + *hex;
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
        states: &BoardState,
        _rng: &mut R,
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
    fn apply<R: rand::Rng>(
        self,
        hex: &Hex,
        states: &BoardState,
        rng: &mut R,
    ) -> Option<BoardSlice> {
        if states.any_set([*hex]) {
            None
        } else {
            Some(BoardSlice(vec![(
                *hex,
                self.0.into_iter().choose(rng).unwrap(),
            )]))
        }
    }
}

/// Apply `then` while a path is `walkable` to `goal`.
#[derive(Debug)]
pub struct WhileConnected<W: States, G: States, S: Step> {
    pub walkable: W,
    pub goal: G,
    pub then: S,
}

impl<W: States, G: States, S: Step> Step for WhileConnected<W, G, S> {
    fn apply<R: rand::Rng>(
        self,
        start: &Hex,
        states: &BoardState,
        rng: &mut R,
    ) -> Option<BoardSlice> {
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
            self.then.apply(start, states, rng)
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
    fn apply<R: rand::Rng>(
        self,
        hex: &Hex,
        states: &BoardState,
        rng: &mut R,
    ) -> Option<BoardSlice> {
        if self
            .directions
            .into_iter()
            .any(|direction| states.is_state(hex.neighbor(direction), self.next.clone()))
        {
            self.step.apply(hex, states, rng)
        } else {
            None
        }
    }
}
