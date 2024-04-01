use bevy::{prelude::*, utils::HashMap};
use hexx::*;

use crate::cell::StateId;

/// A Bevy component to just store the Hexagon position on the board
/// for an entity.
#[derive(Component, Deref)]
pub struct Cell(pub Hex);

impl From<Cell> for Hex {
    fn from(value: Cell) -> Self {
        value.0
    }
}

impl From<&Cell> for Hex {
    fn from(value: &Cell) -> Self {
        value.0
    }
}

/// Lookup Entity IDs from their position on the board.
#[derive(Resource, Default, Deref, DerefMut)]
pub struct EntityMap(HashMap<Hex, Entity>);

/// The state of the board.
#[derive(Resource, Default)]
pub struct CellStates {
    /// The visible state of the board.
    current: HashMap<Hex, StateId>,

    /// The delta for the next frame to be applied when [`Self::tick()`] is called.
    next: HashMap<Hex, StateId>,
}

impl CellStates {
    /// Get the [`StateId`] currently visible in a cell.
    pub fn get_current(&self, hex: impl Into<Hex>) -> Option<&StateId> {
        self.current.get(&hex.into())
    }

    /// Get the future [`StateId`] of a cell.
    pub fn get_next(&self, hex: impl Into<Hex>) -> Option<&StateId> {
        self.next.get(&hex.into())
    }

    /// Return `true` if a `hex` has one of `state`.
    pub fn is_state(&self, hex: Hex, state: impl IntoIterator<Item = StateId>) -> bool {
        self.get_current(hex)
            .map(|id| state.into_iter().find(|other_id| id == other_id).is_some())
            .unwrap_or(false)
    }

    /// Set the future state of a cell.
    pub fn set(&mut self, hex: Hex, id: StateId) {
        self.next.insert(hex, id);
    }

    pub fn is_set(&self, hex: Hex) -> bool {
        self.next.contains_key(&hex)
    }

    pub fn any_set(&self, hexs: impl IntoIterator<Item = Hex>) -> bool {
        hexs.into_iter().any(|hex| self.is_set(hex))
    }

    /// Apply all changes in [`Self::next`] to [`Self::current`].
    pub(super) fn tick(&mut self) {
        println!(
            "count: {:?}",
            self.current
                .values()
                .filter(|id| **id != StateId::Air)
                .count()
        );

        for (hex, id) in self.next.drain() {
            self.current.insert(hex, id);
        }
    }
}

/// The size and layout of the board.
#[derive(Resource)]
pub struct Board {
    pub layout: HexLayout,
    pub bounds: HexBounds,
}
