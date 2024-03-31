use hexx::{EdgeDirection, Hex};
use rand::seq::SliceRandom;

use crate::grid::CellStates;

use super::Behavior;

pub struct Fire;

impl Behavior for Fire {
    fn tick(from: Hex, states: &mut CellStates) {
        if let Some(step) = [
            EdgeDirection::POINTY_TOP_LEFT,
            EdgeDirection::POINTY_TOP_RIGHT,
        ]
        .choose(&mut rand::thread_rng())
        .into_iter()
        .find_map(|direction| Self::try_move(from, *direction, states))
        {
            step.apply(states)
        }
    }
}
