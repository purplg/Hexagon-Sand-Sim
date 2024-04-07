mod air;
use std::any::type_name;

pub use air::Air;
mod fire;
use bevy::utils::HashMap;
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
pub struct StateId(pub &'static str);

impl From<&'static str> for StateId {
    fn from(value: &'static str) -> Self {
        Self(value)
    }
}

#[derive(Resource, Default)]
pub struct CellRegistry {
    inner: HashMap<StateId, Box<dyn Tick + Send + Sync>>,
    color: HashMap<StateId, Color>,
}

impl CellRegistry {
    pub fn add<T>(&mut self, tickable: T)
    where
        T: HexColor + Tick + Send + Sync + 'static,
    {
        self.add_with_color(tickable, T::COLOR)
    }

    pub fn add_with_color<T>(&mut self, tickable: T, color: Color)
    where
        T: Tick + Send + Sync + 'static,
    {
        let id: StateId = type_name::<T>().into();
        if self.inner.contains_key(&id) {
            panic!("StateId::{:?} already exists in Tick registry.", id);
        }
        if self.color.contains_key(&id) {
            panic!("StateId::{:?} already exists in Color registry.", id);
        }
        self.inner.insert(id, Box::new(tickable));
        self.color.insert(id, color);
    }

    pub fn get(&self, id: &StateId) -> Option<&(dyn Tick + Send + Sync)> {
        self.inner.get(id).map(|a| &**a)
    }

    pub fn color(&self, id: &StateId) -> &Color {
        self.color
            .get(id)
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

pub trait Register {
    const ID: StateId;
}

impl<T> Register for T
where
    T: Tick,
{
    const ID: StateId = StateId(type_name::<T>());
}

pub trait HexColor {
    const COLOR: Color;
}
