use bevy::prelude::*;
use hexx::EdgeDirection;
use rand::Rng;
use std::fmt::Debug;

use super::*;
use crate::behavior::{
    Assert, Chance, Choose, If, Infect, RandomSwap, Set, Step, Unless, When, WhenNearby,
};

/// A particle that falls down, and when sand and water are nearby,
/// turns into a [`Sapling`].
#[derive(Debug)]
pub struct Seed;

impl StateInfo for Seed {
    const NAME: &'static str = "Seed";
    const COLOR: Color = Color::LIME_GREEN;
    const HIDDEN: bool = false;
}

impl Tick for Seed {
    fn tick(&self, hex: &Hex, states: &BoardState, rng: &mut SmallRng) -> Option<BoardSlice> {
        (
            // Move down
            RandomSwap {
                directions: [
                    EdgeDirection::POINTY_BOTTOM_LEFT,
                    EdgeDirection::POINTY_BOTTOM_RIGHT,
                ],
                open: [Air::id(), Wind::id(), Steam::id()],
            },
            // Only attempt to grow when Sand or Water are nearby.
            WhenNearby::any_adjacent(
                [Sand::id(), Water::id()],
                Chance {
                    step: Set(Sapling::id()),
                    chance: 1.,
                },
            ),
        )
            .apply(hex, rng, states)
    }
}

/// Grows upward a random height, then turns into a [`Trunk`] when unable to grow anymore.
#[derive(Debug)]
pub struct Sapling;

impl StateInfo for Sapling {
    const NAME: &'static str = "Sapling";
    const COLOR: Color = Color::DARK_GREEN;
}

impl Tick for Sapling {
    fn tick(&self, hex: &Hex, states: &BoardState, rng: &mut SmallRng) -> Option<BoardSlice> {
        // Branch when no sand nearby, try to start branching
        let height = rng.gen_range(10..100);
        (
            WhenNearby::any_adjacent(
                [Self::id()],
                WhenNearby::any_adjacent([Sand::id(), Dead::id()], Set(Trunk::id())),
            ),
            If(
                || {
                    (states.is_state(hex.neighbor(EdgeDirection::POINTY_TOP_LEFT), Air::id())
                        || states
                            .is_state(hex.neighbor(EdgeDirection::POINTY_TOP_RIGHT), Air::id()))
                        && hex
                            .xrange(height)
                            .any(|hex| states.is_state(hex, Sand::id()))
                        && hex
                            .xrange(1)
                            .filter(|hex| states.is_state(*hex, [Self::id(), Trunk::id()]))
                            .count()
                            < 2
                },
                // Try to grow
                Infect {
                    directions: [
                        EdgeDirection::POINTY_TOP_LEFT,
                        EdgeDirection::POINTY_TOP_RIGHT,
                    ],
                    open: [Air::id(), Sand::id(), Water::id()],
                    into: Self::id(),
                },
                // Otherwise, harden
                Set(Trunk::id()),
            ),
        )
            .apply(hex, rng, states)
    }
}

/// A sapling that doesn't grow upward anymore. It can try to turn
/// into a branch when no other branches are nearby.
#[derive(Debug)]
pub struct Trunk;

impl StateInfo for Trunk {
    const NAME: &'static str = "Trunk";
    const COLOR: Color = Color::DARK_GRAY;
}

impl Tick for Trunk {
    fn tick(&self, hex: &Hex, states: &BoardState, rng: &mut SmallRng) -> Option<BoardSlice> {
        (
            WhenNearby::any(
                [Sand::id(), Dead::id()],
                5,
                (
                    Chance {
                        step: Set(Dead::id()),
                        chance: 0.01,
                    },
                    Assert(|| false),
                ),
            ),
            Choose::half(
                Unless(
                    || {
                        hex.xrange(4)
                            .any(|hex| states.is_state(hex, BranchLeft::id()))
                    },
                    Set(BranchLeft::id()),
                ),
                Unless(
                    || {
                        hex.xrange(4)
                            .any(|hex| states.is_state(hex, BranchRight::id()))
                    },
                    Set(BranchRight::id()),
                ),
            ),
        )
            .apply(hex, rng, states)
    }
}

#[derive(Debug)]
pub struct Dead;

impl StateInfo for Dead {
    const NAME: &'static str = "Dead";
    const COLOR: Color = Color::TURQUOISE;
}

impl Tick for Dead {}

#[derive(Debug)]
struct Branch {
    direction: EdgeDirection,
    grow_into: StateId,
}

impl Step for Branch {
    fn apply<R: rand::Rng>(self, hex: &Hex, rng: R, states: &BoardState) -> Option<BoardSlice> {
        (
            // When next to other tree components, just stop doing anything.
            WhenNearby::some_adjacent(
                [
                    BranchLeft::id(),
                    BranchRight::id(),
                    Dead::id(),
                    Trunk::id(),
                    Twig::id(),
                ],
                2,
                Set(Dead::id()),
            ),
            // When near other branches, also stop doing anything
            WhenNearby::any([BranchLeft::id(), BranchRight::id()], 25, Set(Dead::id())),
            // Otherwise, try and grow right.
            When(
                || states.is_state(hex.neighbor(self.direction), Air::id()),
                Infect {
                    directions: [self.direction],
                    open: [
                        Air::id(),
                        Sand::id(),
                        Water::id(),
                        Sapling::id(),
                        Seed::id(),
                    ],
                    into: self.grow_into,
                },
            ),
            // If can't grow anymore, turn into twig.
            Choose {
                a: Set(Twig::id()),
                b: Set(Dead::id()),
                chance: 0.8,
            },
        )
            .apply(hex, rng, states)
    }
}

#[derive(Debug)]
pub struct BranchLeft;

impl StateInfo for BranchLeft {
    const NAME: &'static str = "BranchLeft";
    const COLOR: Color = Color::Rgba {
        red: 1.0,
        green: 0.5,
        blue: 1.0,
        alpha: 1.0,
    };
}

impl Tick for BranchLeft {
    fn tick(&self, hex: &Hex, states: &BoardState, rng: &mut SmallRng) -> Option<BoardSlice> {
        Branch {
            direction: EdgeDirection::POINTY_TOP_LEFT,
            grow_into: Self::id(),
        }
        .apply(hex, rng, states)
    }
}

#[derive(Debug)]
pub struct BranchRight;

impl StateInfo for BranchRight {
    const NAME: &'static str = "BranchRight";
    const COLOR: Color = Color::Rgba {
        red: 1.0,
        green: 0.0,
        blue: 1.0,
        alpha: 1.0,
    };
}

impl Tick for BranchRight {
    fn tick(&self, hex: &Hex, states: &BoardState, rng: &mut SmallRng) -> Option<BoardSlice> {
        Branch {
            direction: EdgeDirection::POINTY_TOP_RIGHT,
            grow_into: Self::id(),
        }
        .apply(hex, rng, states)
    }
}

#[derive(Debug)]
pub struct Twig;

impl StateInfo for Twig {
    const NAME: &'static str = "Twig";
    const COLOR: Color = Color::PINK;
}

impl Tick for Twig {
    fn tick(&self, hex: &Hex, states: &BoardState, rng: &mut SmallRng) -> Option<BoardSlice> {
        Chance {
            step: Infect {
                directions: EdgeDirection::ALL_DIRECTIONS,
                open: Air::id(),
                into: Leaf::id(),
            },
            chance: 0.1,
        }
        .apply(hex, rng, states)
    }
}

#[derive(Debug)]
pub struct Leaf;

impl StateInfo for Leaf {
    const NAME: &'static str = "Leaf";
    const COLOR: Color = Color::GREEN;
}

impl Tick for Leaf {
    fn tick(&self, hex: &Hex, states: &BoardState, rng: &mut SmallRng) -> Option<BoardSlice> {
        (
            WhenNearby {
                nearby: Self::id(),
                range: 20,
                count: 50,
                then: (
                    Chance {
                        step: Infect {
                            directions: EdgeDirection::ALL_DIRECTIONS,
                            open: Air::id(),
                            into: Wind::id(),
                        },
                        chance: 0.01,
                    },
                    Assert(|| false),
                ),
            },
            WhenNearby {
                nearby: Twig::id(),
                range: 5,
                count: 1,
                then: Infect {
                    directions: EdgeDirection::ALL_DIRECTIONS,
                    open: Air::id(),
                    into: Self::id(),
                },
            },
        )
            .apply(hex, rng, states)
    }
}
