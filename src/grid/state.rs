use bevy::{prelude::*, utils::HashMap};
use hexx::*;
use unique_type_id::UniqueTypeId as _;

use crate::{behavior::StateId, cell::BoardSlice};

use super::Air;

/// Lookup Entity IDs from their position on the board.
#[derive(Resource, Default, Deref, DerefMut)]
pub struct EntityMap(HashMap<Hex, Entity>);

const HEX_RANGE: u32 = 128;
const HEX_COUNT: u32 = Hex::range_count(HEX_RANGE);

/// The state of the board.
#[derive(Resource)]
pub struct BoardState {
    bounds: HexBounds,
    layout: HexLayout,

    /// The visible state of the board.
    current: [StateId; HEX_COUNT as usize],

    /// The delta for the next frame to be applied when [`Self::tick()`] is called.
    pub next: HashMap<Hex, StateId>,
}

impl BoardState {
    pub fn bounds(&self) -> &HexBounds {
        &self.bounds
    }

    pub fn layout(&self) -> &HexLayout {
        &self.layout
    }

    pub fn iter(&self) -> impl Iterator<Item = (Hex, &StateId)> {
        self.current
            .iter()
            .enumerate()
            .map(|(i, id)| (Self::index_to_hex(i), id))
    }

    pub fn count(&self) -> usize {
        self.current.len()
    }

    /// Get the [`StateId`] currently visible in a cell.
    pub fn get_current(&self, hex: impl Into<Hex>) -> Option<&StateId> {
        let hex = hex.into();
        if self.bounds.is_in_bounds(hex) {
            self.current.get(Self::hex_to_index(&hex))
        } else {
            None
        }
    }

    fn index_to_hex(i: usize) -> Hex {
        Hex::from_hexmod_coordinates(i as u32, HEX_RANGE)
    }

    fn hex_to_index(hex: &Hex) -> usize {
        hex.to_hexmod_coordinates(HEX_RANGE) as usize
    }

    /// Get the future [`StateId`] of a cell.
    pub fn get_next(&self, hex: impl Into<Hex>) -> Option<&StateId> {
        let hex = hex.into();
        self.next.get(&hex).or_else(|| self.get_current(hex))
    }

    /// Return `true` if a `hex` has one of `state`.
    pub fn is_state(&self, hex: Hex, state: impl IntoIterator<Item = StateId>) -> bool {
        self.get_next(hex)
            .map(|id| state.into_iter().any(|other_id| *id == other_id))
            .unwrap_or(false)
    }

    pub fn find_state(
        &self,
        hex: Hex,
        state: impl IntoIterator<Item = impl Into<StateId>>,
    ) -> Option<StateId> {
        self.get_next(hex).and_then(|id| {
            state
                .into_iter()
                .map(Into::into)
                .find(|other_id| id == other_id)
        })
    }

    /// Set the future state of a cell.
    pub fn set_next(&mut self, hex: Hex, id: StateId) {
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
    pub(super) fn commit(&mut self) {
        for (hex, id) in self.next.drain() {
            let i = Self::hex_to_index(&hex);
            self.current[i] = id;
        }
    }

    pub fn clear(&mut self) {
        self.current = [Air::id(); HEX_COUNT as usize];
        self.next.clear();
    }
}

impl Default for BoardState {
    fn default() -> Self {
        Self {
            bounds: HexBounds::new(Hex::default(), HEX_RANGE),
            layout: HexLayout {
                orientation: HexOrientation::Pointy,
                hex_size: Vec2::ONE * 2.0,
                ..default()
            },
            current: [Air::id(); HEX_COUNT as usize],
            next: Default::default(),
        }
    }
}
