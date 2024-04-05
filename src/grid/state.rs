use bevy::{prelude::*, utils::HashMap};
use bevy_inspector_egui::{inspector_options::ReflectInspectorOptions, InspectorOptions};
use hexx::*;

use crate::cell::StateId;

/// Lookup Entity IDs from their position on the board.
#[derive(Resource, Default, Deref, DerefMut)]
pub struct EntityMap(HashMap<Hex, Entity>);

/// The state of the board.
#[derive(Resource, Default)]
pub struct States {
    /// The visible state of the board.
    pub current: HashMap<Hex, StateId>,

    /// The delta for the next frame to be applied when [`Self::tick()`] is called.
    pub next: HashMap<Hex, StateId>,
}

impl States {
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
