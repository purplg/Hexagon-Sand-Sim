use bevy::{prelude::*, utils::HashMap};
use bevy_inspector_egui::{inspector_options::ReflectInspectorOptions, InspectorOptions};
use hexx::*;

use crate::cell::{BoardSlice, StateId};

/// Lookup Entity IDs from their position on the board.
#[derive(Resource, Default, Deref, DerefMut)]
pub struct EntityMap(HashMap<Hex, Entity>);

/// The state of the board.
#[derive(Resource, Default)]
pub struct BoardState {
    /// The visible state of the board.
    pub current: HashMap<Hex, StateId>,

    /// The delta for the next frame to be applied when [`Self::tick()`] is called.
    pub next: HashMap<Hex, StateId>,
}

impl BoardState {
    /// Get the [`StateId`] currently visible in a cell.
    pub fn get_current(&self, hex: impl Into<Hex>) -> Option<&StateId> {
        self.current.get(&hex.into())
    }

    /// Get the future [`StateId`] of a cell.
    pub fn get_next(&self, hex: impl Into<Hex>) -> Option<&StateId> {
        let hex = hex.into();
        self.next.get(&hex).or_else(|| self.get_current(hex))
    }

    /// Return `true` if a `hex` has one of `state`.
    pub fn is_state(&self, hex: Hex, state: impl IntoIterator<Item = StateId>) -> bool {
        self.get_next(hex)
            .map(|id| state.into_iter().any(|other_id| id == &other_id))
            .unwrap_or(false)
    }

    pub fn find_state(
        &self,
        hex: Hex,
        state: impl IntoIterator<Item = StateId>,
    ) -> Option<StateId> {
        self.get_current(hex)
            .map(|id| state.into_iter().find(|other_id| id == other_id))
            .flatten()
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

    pub fn apply(&mut self, mut slice: BoardSlice) {
        if slice.iter().any(|(hex, _id)| self.next.contains_key(hex)) {
            return;
        }
        for (hex, id) in slice.drain(0..) {
            self.next.insert(hex, id);
        }
    }

    /// Apply all changes in [`Self::next`] to [`Self::current`].
    pub(super) fn tick(&mut self) {
        for (hex, id) in self.next.drain() {
            self.current.insert(hex, id);
        }
    }
}

/// The size and layout of the board.
#[derive(Reflect, Resource, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct Board {
    pub layout: HexLayout,
    pub bounds: HexBounds,
}

impl Default for Board {
    fn default() -> Self {
        Self {
            layout: Default::default(),
            bounds: HexBounds::new(Hex::default(), 0),
        }
    }
}
