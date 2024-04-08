use std::fmt::Debug;

use bevy::prelude::*;
use hexx::EdgeDirection;
use rand::Rng;

use self::behavior::{Assert, If, Noop, WhenNearby};

use super::{
    behavior::{Chance, Choose, Infect, Or, Or3, RandomSwap, Set, Step, Unless, When},
    *,
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
        Or(
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
                    chance: 0.001,
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
        If(
            || {
                (states.is_state(hex.neighbor(EdgeDirection::POINTY_TOP_LEFT), Air::id())
                    || states.is_state(hex.neighbor(EdgeDirection::POINTY_TOP_RIGHT), Air::id()))
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
                        .any(|hex| states.is_state(hex, BranchLeft::id()))
                },
                Set(BranchLeft::id()),
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
        let height = rng.gen_range(10..100);
        Or3(
            WhenNearby::any_adjacent(
                [Dead::id(), BranchLeft::id(), BranchRight::id(), Trunk::id()],
                Set(Dead::id()),
            ),
            When(
                || {
                    (states.is_state(hex.neighbor(EdgeDirection::POINTY_TOP_LEFT), Air::id())
                        || states
                            .is_state(hex.neighbor(EdgeDirection::POINTY_BOTTOM_LEFT), Air::id()))
                        && !hex
                            .xrange(height)
                            .any(|hex| states.is_state(hex, BranchLeft::id()))
                },
                Chance {
                    step: Infect {
                        directions: [
                            EdgeDirection::POINTY_TOP_LEFT,
                            EdgeDirection::POINTY_BOTTOM_LEFT,
                        ],
                        open: [
                            Air::id(),
                            Sand::id(),
                            Water::id(),
                            Sapling::id(),
                            Seed::id(),
                        ],
                        into: Self::id(),
                    },
                    chance: 0.1,
                },
            ),
            Noop,
            // Set(Twig::id()),
        )
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
        let height = rng.gen_range(10..100);
        Or(
            If(
                || {
                    (states.is_state(hex.neighbor(EdgeDirection::POINTY_TOP_LEFT), Air::id())
                        || states
                            .is_state(hex.neighbor(EdgeDirection::POINTY_TOP_RIGHT), Air::id()))
                        && hex
                            .xrange(height)
                            .any(|hex| states.is_state(hex, [BranchLeft::id(), BranchRight::id()]))
                        && hex
                            .xrange(1)
                            .filter(|hex| {
                                states.is_state(*hex, [Self::id(), BranchLeft::id(), Trunk::id()])
                            })
                            .count()
                            < 2
                },
                Chance {
                    step: Infect {
                        directions: [EdgeDirection::POINTY_TOP_RIGHT],
                        open: [
                            Air::id(),
                            Sand::id(),
                            Water::id(),
                            Sapling::id(),
                            Seed::id(),
                        ],
                        into: Self::id(),
                    },
                    chance: 0.1,
                },
                Chance {
                    step: Set(BranchLeft::id()),
                    chance: 0.001,
                },
            ),
            Set(Twig::id()),
        )
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
        return None;
        Or(
            When(
                || hex.xrange(1).any(|hex| !states.is_state(hex, Air::id())),
                Set(Trunk::id()),
            ),
            Chance {
                step: Infect {
                    directions: EdgeDirection::ALL_DIRECTIONS,
                    open: Air::id(),
                    into: Leaf::id(),
                },
                chance: 0.1,
            },
        )
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
        Chance {
            step: Set(Wind::id()),
            chance: 0.0001,
        }
        .apply(hex, rng, states)
    }
}
