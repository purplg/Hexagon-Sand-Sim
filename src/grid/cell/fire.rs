use bevy::prelude::*;
use hexx::EdgeDirection;

use super::*;
use crate::behavior::{StateQuery::*, *};

#[derive(UniqueTypeId)]
#[UniqueTypeIdType = "u8"]
pub struct Fire;

impl StateInfo for Fire {
    const NAME: &'static str = "Fire";
    const COLOR: HexColor = HexColor::Flickering {
        base_color: Color::RED,
        offset_color: Color::ORANGE,
    };
    const HIDDEN: bool = false;
}

impl Behavior for Fire {
    fn tick(&self) -> impl Step {
        (
            QueryTest(Any([Air::id()])),
            Chance {
                to: Set([Air::id()]),
                chance: 0.1,
            },
            Chance {
                to: Infect {
                    directions: EdgeDirection::ALL_DIRECTIONS,
                    open: Any([
                        Seed::id(),
                        Sapling::id(),
                        Trunk::id(),
                        DeadTrunk::id(),
                        BranchLeft::id(),
                        BranchRight::id(),
                        Twig::id(),
                        Leaf::id(),
                    ]),
                    into: [Ember::id()],
                },
                chance: 0.5,
            },
            Annihilate {
                directions: [
                    EdgeDirection::POINTY_LEFT,
                    EdgeDirection::POINTY_RIGHT,
                    EdgeDirection::POINTY_TOP_LEFT,
                    EdgeDirection::POINTY_TOP_RIGHT,
                ],
                open: Any([Water::id()]),
                into: [Steam::id()],
            },
            RandomSwap::adjacent(
                [
                    EdgeDirection::POINTY_LEFT,
                    EdgeDirection::POINTY_RIGHT,
                    EdgeDirection::POINTY_TOP_LEFT,
                    EdgeDirection::POINTY_TOP_RIGHT,
                ],
                Any([Air::id(), Water::id(), Steam::id(), Sand::id()]),
            ),
        )
    }
}

#[derive(UniqueTypeId)]
#[UniqueTypeIdType = "u8"]
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

impl Behavior for Ember {
    fn tick(&self) -> impl Step {
        (
            Chance {
                to: Set([Air::id()]),
                chance: 0.05,
            },
            Chance {
                to: Infect {
                    directions: EdgeDirection::ALL_DIRECTIONS,
                    open: Any([
                        Seed::id(),
                        Sapling::id(),
                        Trunk::id(),
                        DeadTrunk::id(),
                        BranchLeft::id(),
                        BranchRight::id(),
                        Twig::id(),
                        Leaf::id(),
                    ]),
                    into: [Self::id()],
                },
                chance: 0.5,
            },
            Infect {
                directions: EdgeDirection::ALL_DIRECTIONS,
                open: Any([Air::id()]),
                into: [Fire::id()],
            },
            Annihilate {
                directions: EdgeDirection::ALL_DIRECTIONS,
                open: Any([Water::id()]),
                into: [Steam::id()],
            },
        )
    }
}
