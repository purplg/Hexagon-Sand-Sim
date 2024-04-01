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

/// The future state of a cell.
///
/// [`CellStates`] uses this when trying resolve the future state of a
/// cell.
#[derive(Clone, Copy)]
pub enum NextState {
    /// A new cell was created.
    Spawn(StateId),

    /// The state from another cell will be used.
    Other(Hex),
}

/// The state of the board.
#[derive(Resource, Default)]
pub struct CellStates {
    /// The visible state of the board.
    current: HashMap<Hex, StateId>,

    /// Used as a buffer during [`Self::tick()`] to stage the next frame.
    stage: HashMap<Hex, StateId>,

    /// The delta for the next frame to be applied when [`Self::tick()`] is called.
    next: HashMap<Hex, NextState>,
}

impl CellStates {
    /// Get the [`StateId`] currently visible in a cell.
    pub fn get_current(&self, hex: impl Into<Hex>) -> Option<&StateId> {
        self.current.get(&hex.into())
    }

    /// Get the future [`StateId`] of a cell.
    pub fn get_next(&self, hex: impl Into<Hex>) -> Option<&StateId> {
        let hex = hex.into();
        match self.next.get(&hex) {
            Some(NextState::Spawn(id)) => Some(id),
            Some(NextState::Other(other)) => self.get_current(*other),
            None => self.get_current(hex),
        }
    }

    /// Return `true` if a `hex` has one of `state`.
    pub fn is_state(&self, hex: Hex, state: impl IntoIterator<Item = StateId>) -> bool {
        self.get_next(hex)
            .map(|id| state.into_iter().find(|other_id| id == other_id).is_some())
            .unwrap_or(false)
    }

    /// Set the future state of a cell.
    pub fn set(&mut self, hex: Hex, next_state: NextState) {
        self.next.insert(hex, next_state);
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
        for (hex, next_state) in self.next.drain() {
            match next_state {
                NextState::Spawn(id) => {
                    self.stage.insert(hex, id);
                }
                NextState::Other(other) => {
                    if let Some(other_id) = self.current.get(&other) {
                        self.stage.insert(hex, *other_id);
                    }
                }
            };
        }

        for (hex, id) in self.stage.drain() {
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
