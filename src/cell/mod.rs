mod air;

pub use air::Air;
mod fire;
use bevy::utils::HashMap;
use bevy_inspector_egui::egui::util::id_type_map::TypeId;
pub use fire::Fire;
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
mod behavior;

use crate::grid::BoardState;
use bevy::prelude::*;
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
        app.insert_resource(registry);
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct StateId(pub u128);

impl From<u128> for StateId {
    fn from(value: u128) -> Self {
        Self(value)
    }
}

struct CellEntry {
    tick: Box<dyn Tick + Send + Sync>,
    name: &'static str,
    color: Color,
    hidden: bool,
}

#[derive(Resource, Default)]
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
                tick: Box::new(tickable),
                name: T::NAME,
                color: T::COLOR,
                hidden: T::HIDDEN,
            },
        );
    }

    pub fn get(&self, id: &StateId) -> Option<&Box<dyn Tick + Send + Sync>> {
        self.inner.get(id).map(|entry| &entry.tick)
    }

    pub fn names(&self) -> impl Iterator<Item = (StateId, String)> + '_ {
        self.inner
            .iter()
            .filter(|(_id, entry)| !entry.hidden)
            .map(|(id, entry)| (id.clone(), entry.name.to_string()))
    }

    pub fn color(&self, id: &StateId) -> &Color {
        self.inner
            .get(id)
            .map(|entry| &entry.color)
            .unwrap_or_else(|| panic!("StateID {:?} missing from Color registry", id))
    }
}

#[derive(Deref, DerefMut)]
pub struct BoardSlice(Vec<(Hex, StateId)>);

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
