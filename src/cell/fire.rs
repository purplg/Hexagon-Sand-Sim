use bevy::prelude::*;
use hexx::EdgeDirection;

use super::*;
use crate::behavior::*;

#[derive(UniqueTypeId)]
#[UniqueTypeIdType = "u32"]
pub struct Fire;

impl StateInfo for Fire {
    const NAME: &'static str = "Fire";
    const COLOR: HexColor = HexColor::Static(Color::Rgba {
        red: 1.0,
        green: 0.0,
        blue: 0.0,
        alpha: 1.0,
    });
    const HIDDEN: bool = false;
}

impl Tick for Fire {
    fn tick(&self, hex: &Hex, states: &BoardState, rng: &mut SmallRng) -> Option<BoardSlice> {
        (
            Chance {
                to: Set([Air::id()]),
                chance: 0.05,
            },
            Chance {
                to: Infect {
                    directions: EdgeDirection::ALL_DIRECTIONS,
                    open: [
                        Seed::id(),
                        Sapling::id(),
                        Trunk::id(),
                        Dead::id(),
                        BranchLeft::id(),
                        BranchRight::id(),
                        Twig::id(),
                        Leaf::id(),
                    ],
                    into: [Ember::id()],
                },
                chance: 0.05,
            },
            Infect {
                directions: [
                    EdgeDirection::POINTY_LEFT,
                    EdgeDirection::POINTY_RIGHT,
                    EdgeDirection::POINTY_TOP_LEFT,
                    EdgeDirection::POINTY_TOP_RIGHT,
                ],
                open: [Water::id()],
                into: [Steam::id()],
            },
            RandomSwap {
                directions: [
                    EdgeDirection::POINTY_LEFT,
                    EdgeDirection::POINTY_RIGHT,
                    EdgeDirection::POINTY_TOP_LEFT,
                    EdgeDirection::POINTY_TOP_RIGHT,
                ],
                open: [Air::id(), Water::id(), Steam::id(), Sand::id()],
            },
        )
            .apply(hex, rng, states)
    }
}

#[derive(UniqueTypeId)]
#[UniqueTypeIdType = "u32"]
pub struct Ember;

impl StateInfo for Ember {
    const NAME: &'static str = "Ember";
    const COLOR: HexColor = HexColor::Flickering {
        base_color: Color::ORANGE,
        offset_color: Color::Rgba {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
            alpha: -1.0,
        },
    };
    const HIDDEN: bool = true;
}

impl Tick for Ember {
    fn tick(&self, hex: &Hex, states: &BoardState, rng: &mut SmallRng) -> Option<BoardSlice> {
        (
            Chance {
                to: Set([Air::id()]),
                chance: 0.005,
            },
            Annihilate {
                directions: EdgeDirection::ALL_DIRECTIONS,
                open: [Water::id()],
                into: [Steam::id()],
            },
            Chance {
                to: Infect {
                    directions: EdgeDirection::ALL_DIRECTIONS,
                    open: [
                        Seed::id(),
                        Sapling::id(),
                        Trunk::id(),
                        Dead::id(),
                        BranchLeft::id(),
                        BranchRight::id(),
                        Twig::id(),
                        Leaf::id(),
                    ],
                    into: [Self::id()],
                },
                chance: 0.05,
            },
        )
            .apply(hex, rng, states)
    }
}
