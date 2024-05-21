use hexx::{EdgeDirection, Hex};
use pathfinding::directed::dijkstra::dijkstra;
use std::fmt::Debug;
use unique_type_id::{TypeId, UniqueTypeId as _};

use crate::grid::{
    cell::{Air, BoardSlice},
    BoardState,
};

pub type StateId = TypeId<u8>;

type States<const C: usize> = [StateId; C];

type Directions<const C: usize> = [EdgeDirection; C];

/// A mutation of the board caused by a single cell.
pub trait Step {
    /// Try to generate a [`BoardSlice`] or return `None` if not
    /// applicable.
    fn apply(self, _hex: Hex, _states: &BoardState, _rng: f32) -> Option<BoardSlice>;
}

/// Try first [`Step`] in tuple and if it fails, try second, and so
/// on, until one succeeds.
macro_rules! impl_step_or_tuple {
    ($first: tt, $($rest: tt),+) => {
        impl<$first: Step, $($rest: Step),*> Step for ($first, $($rest),*) {
            fn apply(self, hex: Hex, states: &BoardState, rng: f32) -> Option<BoardSlice> {
                #[allow(non_snake_case)]
                let ($first, $($rest,)*) = self;
                $first.apply(hex, states, rng)
                    $(
                        .or_else(|| $rest.apply(hex, states, rng))
                    )*
            }
        }
    };
}

impl_step_or_tuple!(A, B);
impl_step_or_tuple!(A, B, C);
impl_step_or_tuple!(A, B, C, D);
impl_step_or_tuple!(A, B, C, D, E);
impl_step_or_tuple!(A, B, C, D, E, F);
impl_step_or_tuple!(A, B, C, D, E, F, G);
impl_step_or_tuple!(A, B, C, D, E, F, G, H);
impl_step_or_tuple!(A, B, C, D, E, F, G, H, I);
impl_step_or_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_step_or_tuple!(A, B, C, D, E, F, G, H, I, J, K);

/// Do nothing.
///
/// Useful as a placeholder for another [`Step`] while writing a
/// cells' behavior.
#[derive(Debug)]
pub struct Noop;

impl Step for Noop {
    fn apply(self, _hex: Hex, _states: &BoardState, _rng: f32) -> Option<BoardSlice> {
        None
    }
}

/// Fall off the screen.
///
/// If this cell on touching the edge of a screen in any of the
/// specified direction, then it turns to an [`Air`] state.
pub struct Offscreen<const D: usize>(pub Directions<D>);

impl<const D: usize> Step for Offscreen<D> {
    fn apply(self, hex: Hex, states: &BoardState, rng: f32) -> Option<BoardSlice> {
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

impl<const D: usize> Debug for Offscreen<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Offscreen({:?})", self.0)
    }
}

/// Convert other nearby cells into another state on collision.
#[derive(Debug)]
pub struct Infect<const D: usize, const O: usize, const I: usize> {
    pub directions: Directions<D>,
    pub open: States<O>,
    pub into: States<I>,
}

impl<const D: usize, const O: usize, const I: usize> Step for Infect<D, O, I> {
    fn apply(self, hex: Hex, states: &BoardState, rng: f32) -> Option<BoardSlice> {
        let i = (rng * self.directions.len() as f32) as usize;
        let to = hex.neighbor(self.directions[i]);
        if states.is_state(to, self.open) {
            let i = (rng * self.into.len() as f32) as usize;
            let id = self.into[i];
            Some(BoardSlice(vec![(to, id)]))
        } else {
            None
        }
    }
}

/// Like [`Infect`], except both cells turn into the same state.
#[derive(Debug)]
pub struct Annihilate<const D: usize, const O: usize, const I: usize> {
    pub directions: Directions<D>,
    pub open: States<O>,
    pub into: States<I>,
}

impl<const D: usize, const O: usize, const I: usize> Step for Annihilate<D, O, I> {
    fn apply(self, hex: Hex, states: &BoardState, rng: f32) -> Option<BoardSlice> {
        let i = (rng * self.directions.len() as f32) as usize;
        let to = hex.neighbor(self.directions[i]);
        if states.is_state(to, self.open) {
            let i = (rng * self.into.len() as f32) as usize;
            let id = self.into[i];
            Some(BoardSlice(vec![(hex, id), (to, id)]))
        } else {
            None
        }
    }
}

/// Drag another cell.
#[derive(Debug)]
pub struct Drag<const DIR: usize, const O: usize, const D: usize> {
    pub directions: Directions<DIR>,
    pub open: States<O>,
    pub drag: States<D>,
}

impl<const DIR: usize, const O: usize, const D: usize> Step for Drag<DIR, O, D> {
    fn apply(self, hex: Hex, states: &BoardState, rng: f32) -> Option<BoardSlice> {
        let i = (rng * self.directions.len() as f32) as usize;
        let direction = self.directions[i];
        let swap = RandomSwap::adjacent(self.directions, self.open);
        let ((from, from_id), (to, to_id)) = swap.in_direction(hex, direction, 1, states)?;
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
    fn apply(self, hex: Hex, states: &BoardState, rng: f32) -> Option<BoardSlice> {
        if rng < self.chance {
            self.to.apply(
                hex,
                states,
                // Since we only tick the containing Step when less
                // than chance, we remap the `rng` value back over the
                // chance value to be back between 0.0 - 1.0
                rng / self.chance,
            )
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
    fn apply(self, hex: Hex, states: &BoardState, rng: f32) -> Option<BoardSlice> {
        if rng < self.chance {
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
    fn apply(self, hex: Hex, states: &BoardState, rng: f32) -> Option<BoardSlice> {
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
    fn apply(self, _hex: Hex, _states: &BoardState, _rng: f32) -> Option<BoardSlice> {
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

/// Do not process anymore steps.
///
/// Returns an empty [`BoardSlice`] to trick the tick into thinking a
/// successful [`Step`] was applied.
#[derive(Debug)]
pub struct Stop;

impl Step for Stop {
    fn apply(self, _hex: Hex, _states: &BoardState, _rng: f32) -> Option<BoardSlice> {
        Some(BoardSlice::EMPTY)
    }
}

/// Print out the type with a prefix message and apply some step while
/// in a behavior.
#[derive(Debug)]
pub struct Output<'a, T>(pub &'a str, pub T);

impl<'a, T: Step + Debug> Step for Output<'a, T> {
    fn apply(self, hex: Hex, states: &BoardState, rng: f32) -> Option<BoardSlice> {
        println!("{}: {:?}", self.0, self.1);
        self.1.apply(hex, states, rng)
    }
}

/// Print out a message without doing anything.
#[derive(Debug)]
pub struct Message<'a>(pub &'a str);

impl<'a> Step for Message<'a> {
    fn apply(self, _hex: Hex, _states: &BoardState, _rng: f32) -> Option<BoardSlice> {
        println!("{}", self.0);
        None
    }
}

#[derive(Debug)]
pub struct MaybeNear<const S: usize, O: Step, X: Step> {
    states: States<S>,
    range: u32,
    count: usize,
    then: O,
    otherwise: X,
}

impl<const S: usize, O: Step, X: Step> Step for MaybeNear<S, O, X> {
    fn apply(self, hex: Hex, states: &BoardState, rng: f32) -> Option<BoardSlice> {
        let mut satisfied = 0;
        for state in self.states {
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

impl<const S: usize, O: Step, X: Step> MaybeNear<S, O, X> {
    pub fn new(states: States<S>, range: u32, count: usize, then: O, otherwise: X) -> Self {
        Self {
            states,
            range,
            count,
            then,
            otherwise,
        }
    }

    pub fn any_adjacent(states: States<S>, then: O, otherwise: X) -> Self {
        Self {
            states,
            range: 1,
            count: 1,
            then,
            otherwise,
        }
    }

    pub fn any(states: States<S>, range: u32, then: O, otherwise: X) -> Self {
        Self {
            states,
            range,
            count: 1,
            then,
            otherwise,
        }
    }

    pub fn some_adjacent(states: States<S>, count: usize, then: O, otherwise: X) -> Self {
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
    #[allow(clippy::new_ret_no_self)]
    pub fn new<const S: usize, O: Step>(
        states: States<S>,
        range: u32,
        count: usize,
        then: O,
    ) -> MaybeNear<S, O, Noop> {
        MaybeNear::new(states, range, count, then, Noop)
    }

    pub fn any_adjacent<const S: usize, O: Step>(
        states: States<S>,
        then: O,
    ) -> MaybeNear<S, O, Noop> {
        MaybeNear::any_adjacent(states, then, Noop)
    }

    pub fn any<const S: usize, O: Step>(
        states: States<S>,
        range: u32,
        then: O,
    ) -> MaybeNear<S, O, Noop> {
        MaybeNear::any(states, range, then, Noop)
    }

    pub fn some_adjacent<const S: usize, O: Step>(
        states: States<S>,
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
    #[allow(clippy::new_ret_no_self)]
    pub fn new<const S: usize, X: Step>(
        states: States<S>,
        range: u32,
        count: usize,
        then: X,
    ) -> MaybeNear<S, Noop, X> {
        MaybeNear::new(states, range, count, Noop, then)
    }

    pub fn any_adjacent<const S: usize, X: Step>(
        states: States<S>,
        then: X,
    ) -> MaybeNear<S, Noop, X> {
        MaybeNear::any_adjacent(states, Noop, then)
    }

    pub fn any<const S: usize, X: Step>(
        states: States<S>,
        range: u32,
        then: X,
    ) -> MaybeNear<S, Noop, X> {
        MaybeNear::any(states, range, Noop, then)
    }

    pub fn some_adjacent<const S: usize, X: Step>(
        states: States<S>,
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
    C: FnOnce(Hex, &BoardState, f32) -> bool,
    T: Step,
    F: Step;

impl<C, T, F> Step for If<C, T, F>
where
    C: FnOnce(Hex, &BoardState, f32) -> bool,
    T: Step,
    F: Step,
{
    fn apply(self, hex: Hex, states: &BoardState, rng: f32) -> Option<BoardSlice> {
        if (self.0)(hex, states, rng) {
            self.1.apply(hex, states, rng)
        } else {
            self.2.apply(hex, states, rng)
        }
    }
}

impl<C, T, F> Debug for If<C, T, F>
where
    C: FnOnce(Hex, &BoardState, f32) -> bool,
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
    C: FnOnce(Hex, &BoardState, f32) -> bool,
    T: Step;

impl<C, T> Step for When<C, T>
where
    C: FnOnce(Hex, &BoardState, f32) -> bool,
    T: Step,
{
    fn apply(self, hex: Hex, states: &BoardState, rng: f32) -> Option<BoardSlice> {
        if (self.0)(hex, states, rng) {
            self.1.apply(hex, states, rng)
        } else {
            None
        }
    }
}

impl<C, T> Debug for When<C, T>
where
    C: FnOnce(Hex, &BoardState, f32) -> bool,
    T: Step + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "When({:?})", self.1)
    }
}

/// Conditionally apply `on_false` when predicate returns `false`.
pub struct Unless<C, F>(pub C, pub F)
where
    C: FnOnce(Hex, &BoardState, f32) -> bool,
    F: Step;

impl<C, F> Step for Unless<C, F>
where
    C: FnOnce(Hex, &BoardState, f32) -> bool,
    F: Step,
{
    fn apply(self, hex: Hex, states: &BoardState, rng: f32) -> Option<BoardSlice> {
        if (self.0)(hex, states, rng) {
            None
        } else {
            self.1.apply(hex, states, rng)
        }
    }
}

impl<C, F> Debug for Unless<C, F>
where
    C: FnOnce(Hex, &BoardState, f32) -> bool,
    F: Step + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unless({:?})", self.1)
    }
}

/// Try to swap with another cell `with_state` in some random `direction`.
#[derive(Debug)]
pub struct RandomSwap<const D: usize, const S: usize> {
    /// The directions that are available to move in.
    pub directions: Directions<D>,

    /// States that are available to swap with.
    pub open: States<S>,

    /// Max distance to move away from the start position.
    pub distance: i32,

    /// When true, try closer positions in the direction of the swap
    /// until one is available. Collision check is not continuous. It
    /// will check the furthest distance it can travel first and jump
    /// to furthest open position.
    pub collide: bool,
}

impl<const D: usize, const S: usize> RandomSwap<D, S> {
    pub fn adjacent(directions: Directions<D>, open: States<S>) -> Self {
        Self {
            directions,
            open,
            distance: 1,
            collide: false,
        }
    }
}

impl<const D: usize, const S: usize> Step for RandomSwap<D, S> {
    fn apply(mut self, hex: Hex, states: &BoardState, rng: f32) -> Option<BoardSlice> {
        let i = (rng * self.directions.len() as f32) as usize;
        let direction = self.directions[i];
        if self.collide {
            while self.distance > 0 {
                if let Some(slice) = self
                    .in_direction(hex, direction, self.distance, states)
                    .map(|(from, to)| BoardSlice(vec![from, to]))
                {
                    return Some(slice);
                }
                self.distance -= 1;
            }
            None
        } else {
            if let Some(slice) = self
                .in_direction(hex, direction, self.distance, states)
                .map(|(from, to)| BoardSlice(vec![from, to]))
            {
                return Some(slice);
            }
            None
        }
    }
}

impl<const D: usize, const S: usize> RandomSwap<D, S> {
    fn in_direction(
        &self,
        from: Hex,
        direction: EdgeDirection,
        distance: i32,
        states: &BoardState,
    ) -> Option<((Hex, StateId), (Hex, StateId))> {
        let from_id = *states.get_current(from).unwrap();
        let to = from + direction * distance;
        if let Some(components) = states
            .find_state(to, self.open)
            .map(|to_id| ((from, to_id), (to, from_id)))
        {
            return Some(components);
        }
        None
    }
}

/// Swap places with another cell.
#[derive(Debug)]
pub struct Swap {
    other: Hex,
}

impl Step for Swap {
    fn apply(self, hex: Hex, states: &BoardState, _rng: f32) -> Option<BoardSlice> {
        if states.any_set([hex, self.other]) {
            None
        } else {
            Some(BoardSlice(vec![
                (hex, *states.get_current(self.other).unwrap()),
                (self.other, *states.get_current(hex).unwrap()),
            ]))
        }
    }
}

/// Set the state of a cell
#[derive(Debug)]
pub struct Set<const I: usize>(pub States<I>);

impl<const I: usize> Step for Set<I> {
    fn apply(self, hex: Hex, states: &BoardState, rng: f32) -> Option<BoardSlice> {
        if states.any_set([hex]) {
            None
        } else {
            let i = (rng * self.0.len() as f32) as usize;
            let id = self.0[i];
            Some(BoardSlice(vec![(hex, id)]))
        }
    }
}

/// Apply `then` while a path is `walkable` to `goal`.
#[derive(Debug)]
pub struct WhileConnected<const W: usize, const G: usize, S: Step> {
    pub walkable: States<W>,
    pub goal: States<G>,
    pub then: S,
}

impl<const W: usize, const G: usize, S: Step> Step for WhileConnected<W, G, S> {
    fn apply(self, start: Hex, states: &BoardState, rng: f32) -> Option<BoardSlice> {
        if let Some(_path) = dijkstra(
            &start,
            |hex| {
                hex.all_neighbors()
                    // All neighbors have a weight of 1
                    .map(|hex| (hex, 1))
                    .into_iter()
                    // Only on walkable states
                    .filter(|(hex, _weight)| {
                        states.is_state(*hex, self.walkable) || states.is_state(*hex, self.goal)
                    })
            },
            |hex| states.is_state(*hex, self.goal),
        ) {
            self.then.apply(start, states, rng)
        } else {
            None
        }
    }
}

/// Check if next to a cell in a state.
#[derive(Debug)]
pub struct NextTo<const D: usize, const N: usize, S: Step> {
    pub directions: Directions<D>,
    pub next: States<N>,
    pub step: S,
}

impl<const D: usize, const N: usize, S: Step> Step for NextTo<D, N, S> {
    fn apply(self, hex: Hex, states: &BoardState, rng: f32) -> Option<BoardSlice> {
        if self
            .directions
            .into_iter()
            .any(|direction| states.is_state(hex.neighbor(direction), self.next))
        {
            self.step.apply(hex, states, rng)
        } else {
            None
        }
    }
}
