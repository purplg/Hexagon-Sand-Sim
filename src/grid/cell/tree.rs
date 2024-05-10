use bevy::prelude::*;
use hexx::EdgeDirection;
use std::fmt::Debug;
use unique_type_id::UniqueTypeId;

use super::*;
use crate::behavior::{
    AssertFn, Chance, Choose, Infect, Nearby, NextTo, NotNear, RandomSwap, Set, Step, When,
    WhileConnected,
};

const BROWN: Color = Color::Rgba {
    red: 0.47,
    green: 0.333,
    blue: 0.14,
    alpha: 1.0,
};

/// A particle that falls down, and when sand and water are nearby,
/// turns into a [`Sapling`].
#[derive(Debug, UniqueTypeId)]
#[UniqueTypeIdType = "u32"]
pub struct Seed;

impl StateInfo for Seed {
    const NAME: &'static str = "Seed";
    const COLOR: HexColor = HexColor::Static(Color::LIME_GREEN);
    const HIDDEN: bool = false;
}

impl Behavior for Seed {
    fn tick(&self) -> impl Step {
        (
            // Move down
            RandomSwap::adjacent(
                [
                    EdgeDirection::POINTY_BOTTOM_LEFT,
                    EdgeDirection::POINTY_BOTTOM_RIGHT,
                ],
                [Air::id(), Wind::id(), Steam::id(), Water::id()],
            ),
            // Only attempt to grow when Sand or Water are nearby.
            Nearby::any_adjacent(
                [Sand::id(), Water::id()],
                Chance {
                    to: Set([Sapling::id()]),
                    chance: 1.,
                },
            ),
        )
    }
}

/// Grows upward a random height, then turns into a [`Trunk`] when unable to grow anymore.
#[derive(Debug, UniqueTypeId)]
#[UniqueTypeIdType = "u32"]
pub struct Sapling;

impl StateInfo for Sapling {
    const NAME: &'static str = "Sapling";
    const COLOR: HexColor = HexColor::Static(Color::DARK_GREEN);
}

impl Behavior for Sapling {
    fn tick(&self) -> impl Step {
        // Branch when no sand nearby, try to start branching
        (
            WhileConnected {
                walkable: [Self::id(), Trunk::id(), DeadTrunk::id()],
                goal: [Sand::id()],
                then: (
                    // If next to Sand or Dead, change to Trunk
                    Nearby::any_adjacent(
                        [Self::id()],
                        Nearby::any_adjacent([Sand::id(), DeadTrunk::id()], Set([Trunk::id()])),
                    ),
                    // If next some trunks, turn into a trunk
                    Nearby::any_adjacent([Self::id(), Trunk::id()], Set([Trunk::id()])),
                    // Otherwise, try to grow
                    NextTo {
                        directions: [
                            EdgeDirection::POINTY_TOP_LEFT,
                            EdgeDirection::POINTY_TOP_RIGHT,
                        ],
                        next: [Air::id(), Water::id()],
                        // Try to grow
                        step: Infect {
                            directions: [
                                EdgeDirection::POINTY_TOP_LEFT,
                                EdgeDirection::POINTY_TOP_RIGHT,
                            ],
                            open: [Air::id(), Sand::id(), Water::id()],
                            into: [Self::id()],
                        },
                    },
                ),
            },
            Set([Trunk::id()]),
        )
    }
}

/// A sapling that doesn't grow upward anymore. It can try to turn
/// into a branch when no other branches are nearby.
#[derive(Debug, UniqueTypeId)]
#[UniqueTypeIdType = "u32"]
pub struct Trunk;

impl StateInfo for Trunk {
    const NAME: &'static str = "Trunk";
    const COLOR: HexColor = HexColor::Static(BROWN);
}

impl Behavior for Trunk {
    fn tick(&self) -> impl Step {
        (
            Nearby::any(
                [Sand::id(), DeadTrunk::id()],
                5,
                (
                    Chance {
                        to: Set([DeadTrunk::id()]),
                        chance: 0.01,
                    },
                    AssertFn(|| false),
                ),
            ),
            Nearby::any([Sand::id()], 5, AssertFn(|| false)),
            Choose::half(
                NotNear::any([BranchLeft::id()], 4, Set([BranchLeft::id()])),
                NotNear::any([BranchRight::id()], 4, Set([BranchRight::id()])),
            ),
        )
    }
}

#[derive(Debug, UniqueTypeId)]
#[UniqueTypeIdType = "u32"]
pub struct DeadTrunk;

impl StateInfo for DeadTrunk {
    const NAME: &'static str = "Dead Trunk";
    const COLOR: HexColor = HexColor::Static(BROWN);
}

impl Behavior for DeadTrunk {}

#[derive(Debug, UniqueTypeId)]
#[UniqueTypeIdType = "u32"]
struct Branch {
    direction: EdgeDirection,
    grow_into: StateId,
}

impl Step for Branch {
    fn apply<R: rand::Rng>(
        self,
        hex: &Hex,
        states: &BoardState,
        rng: &mut R,
    ) -> Option<BoardSlice> {
        (
            // When next to other tree components, just stop doing anything.
            Nearby::some_adjacent(
                [
                    BranchLeft::id(),
                    BranchRight::id(),
                    DeadTrunk::id(),
                    Trunk::id(),
                    Twig::id(),
                ],
                2,
                Set([DeadTrunk::id()]),
            ),
            // When near other branches, also stop doing anything
            Nearby::any(
                [BranchLeft::id(), BranchRight::id()],
                25,
                Set([DeadTrunk::id()]),
            ),
            // Otherwise, try and grow right.
            Choose {
                // Grow
                a: When(
                    || states.is_state(hex.neighbor(self.direction), [Air::id()]),
                    Infect {
                        directions: [self.direction],
                        open: [
                            Air::id(),
                            Sand::id(),
                            Water::id(),
                            Sapling::id(),
                            Seed::id(),
                        ],
                        into: [self.grow_into],
                    },
                ),
                // Chance to stop growing
                b: Choose::half(Set([Twig::id()]), Set([DeadTrunk::id()])),
                chance: 0.8,
            },
        )
            .apply(hex, states, rng)
    }
}

#[derive(Debug, UniqueTypeId)]
#[UniqueTypeIdType = "u32"]
pub struct BranchLeft;

impl StateInfo for BranchLeft {
    const NAME: &'static str = "BranchLeft";
    const COLOR: HexColor = HexColor::Static(BROWN);
}

impl Behavior for BranchLeft {
    fn tick(&self) -> impl Step {
        Branch {
            direction: EdgeDirection::POINTY_TOP_LEFT,
            grow_into: Self::id(),
        }
    }
}

#[derive(Debug, UniqueTypeId)]
#[UniqueTypeIdType = "u32"]
pub struct BranchRight;

impl StateInfo for BranchRight {
    const NAME: &'static str = "BranchRight";
    const COLOR: HexColor = HexColor::Static(BROWN);
}

impl Behavior for BranchRight {
    fn tick(&self) -> impl Step {
        Branch {
            direction: EdgeDirection::POINTY_TOP_RIGHT,
            grow_into: Self::id(),
        }
    }
}

#[derive(Debug, UniqueTypeId)]
#[UniqueTypeIdType = "u32"]
pub struct Twig;

impl StateInfo for Twig {
    const NAME: &'static str = "Twig";
    const COLOR: HexColor = HexColor::Static(BROWN);
}

impl Behavior for Twig {
    fn tick(&self) -> impl Step {
        Chance {
            to: Infect {
                directions: EdgeDirection::ALL_DIRECTIONS,
                open: [Air::id()],
                into: [Leaf::id()],
            },
            chance: 0.1,
        }
    }
}

#[derive(Debug, UniqueTypeId)]
#[UniqueTypeIdType = "u32"]
pub struct Leaf;

impl StateInfo for Leaf {
    const NAME: &'static str = "Leaf";
    const COLOR: HexColor = HexColor::Static(Color::GREEN);
}

impl Behavior for Leaf {
    fn tick(&self) -> impl Step {
        WhileConnected {
            walkable: [Self::id(), Trunk::id(), DeadTrunk::id()],
            goal: [Sand::id()],
            then: (
                Nearby {
                    nearby: [Self::id()],
                    range: 20,
                    count: 50,
                    then: (
                        Chance {
                            to: Infect {
                                directions: EdgeDirection::ALL_DIRECTIONS,
                                open: [Air::id()],
                                into: [Wind::id()],
                            },
                            chance: 0.01,
                        },
                        AssertFn(|| false),
                    ),
                },
                Nearby {
                    nearby: [Twig::id()],
                    range: 5,
                    count: 1,
                    then: Infect {
                        directions: EdgeDirection::ALL_DIRECTIONS,
                        open: [Air::id()],
                        into: [Self::id()],
                    },
                },
            ),
        }
    }
}
