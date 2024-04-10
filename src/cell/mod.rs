mod air;
pub use air::Air;
mod fire;
pub use fire::{Ember, Fire};
mod sand;
pub use sand::Sand;
mod water;
pub use water::Water;
mod steam;
pub use steam::Steam;
mod wind;
pub use wind::Wind;
mod stone;
pub use stone::Stone;
mod tree;
pub use tree::*;

use std::borrow::Cow;

use crate::grid::BoardState;
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_inspector_egui::egui::util::id_type_map::TypeId;
use hexx::Hex;
use rand::rngs::SmallRng;

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        let mut registry = CellRegistry::default();
        registry.add(Air);
        registry.add(Fire);
        registry.add(Sand);
        registry.add(Water);
        registry.add(Steam);
        registry.add(Stone);
        registry.add(Wind);
        registry.add(Seed);
        registry.add(Dead);
        registry.add(Trunk);
        registry.add(BranchLeft);
        registry.add(BranchRight);
        registry.add(Leaf);
        registry.add(Sapling);
        registry.add(Twig);
        registry.add(Ember);
        app.insert_resource(registry);
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct StateId(TypeId);

impl From<TypeId> for StateId {
    fn from(value: TypeId) -> Self {
        Self(value)
    }
}

impl<'a> Into<StateId> for &'a StateId {
    fn into(self) -> StateId {
        *self
    }
}

pub struct CellEntry {
    pub behavior: Box<dyn Tick + Send + Sync>,
    pub name: Cow<'static, str>,
    pub color: Color,
    pub hidden: bool,
}

#[derive(Resource, Default, Deref)]
pub struct CellRegistry {
    inner: HashMap<StateId, CellEntry>,
}

impl CellRegistry {
    pub fn add<T>(&mut self, tickable: T)
    where
        T: StateInfo + Register + Tick + Send + Sync + 'static,
    {
        let id: StateId = TypeId::of::<T>().into();
        if self.inner.contains_key(&id) {
            panic!("StateId::{:?} already exists in Tick registry.", id);
        }
        self.inner.insert(
            id,
            CellEntry {
                behavior: Box::new(tickable),
                name: T::NAME.into(),
                color: T::COLOR,
                hidden: T::HIDDEN,
            },
        );
    }

    pub fn names(&self) -> impl Iterator<Item = (StateId, String)> + '_ {
        self.inner
            .iter()
            .filter(|(_id, entry)| !entry.hidden)
            .map(|(id, entry)| (*id, entry.name.to_string()))
    }

    pub fn color(&self, id: &StateId) -> &Color {
        self.inner
            .get(id)
            .map(|entry| &entry.color)
            .unwrap_or_else(|| panic!("StateID {:?} missing from Color registry", id))
    }
}

#[derive(Debug, Deref, DerefMut)]
pub struct BoardSlice(pub Vec<(Hex, StateId)>);

impl BoardSlice {
    pub const EMPTY: Self = Self(Vec::new());
}

pub trait Tick {
    fn tick(&self, _from: &Hex, _states: &BoardState, _rng: &mut SmallRng) -> Option<BoardSlice> {
        None
    }
}

impl From<StateId> for Vec<StateId> {
    fn from(value: StateId) -> Self {
        vec![value]
    }
}

pub trait Register
where
    Self: Sized + 'static,
{
    fn id() -> StateId {
        StateId(TypeId::of::<Self>())
    }
}

impl<T> Register for T where T: StateInfo + 'static {}

/// Meta information about a state type generally for displaying to
/// the user.
pub trait StateInfo {
    const NAME: &'static str = "Unknown";
    const COLOR: Color = Color::NONE;
    const HIDDEN: bool = true;
}
