mod air;
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
mod behavior;

use crate::grid::States;
use bevy::prelude::*;
use hexx::Hex;
use rand::rngs::SmallRng;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum StateId {
    Air,
    Fire,
    Sand,
    Water,
    Steam,
}

#[derive(Resource, Default)]
pub struct StateRegistry {
    inner: HashMap<StateId, Box<dyn Tick + Send + Sync>>,
}

impl StateRegistry {
    pub fn get(&self, id: &StateId) -> Option<&Box<dyn Tick + Send + Sync>> {
        self.inner.get(id)
    }

    pub fn add<T>(&mut self, tickable: T)
    where
        T: Register + Tick + Send + Sync + 'static,
    {
        self.inner.insert(T::ID, Box::new(tickable));
    }
}

#[derive(Deref, DerefMut)]
pub struct BoardSlice(Vec<(Hex, StateId)>);

pub trait Tick {
    fn tick(&self, _from: Hex, _states: &States, _rng: &mut SmallRng) -> Option<BoardSlice> {
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
