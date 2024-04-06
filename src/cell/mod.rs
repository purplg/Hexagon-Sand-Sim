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
mod wind;
pub use wind::Wind;
mod behavior;

use crate::grid::BoardState;
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
    Wind,
}

#[derive(Resource, Default)]
pub struct CellRegistry {
    inner: HashMap<StateId, Box<dyn Tick + Send + Sync>>,
    color: HashMap<StateId, Color>,
}

impl CellRegistry {
    pub fn add<T>(&mut self, tickable: T)
    where
        T: Register + HexColor + Tick + Send + Sync + 'static,
    {
        self.inner.insert(T::ID, Box::new(tickable));
        self.color.insert(T::ID, T::COLOR);
    }

    pub fn get(&self, id: &StateId) -> Option<&Box<dyn Tick + Send + Sync>> {
        self.inner.get(id)
    }

    pub fn color(&self, id: &StateId) -> &Color {
        self.color
            .get(id)
            .expect(format!("StateID {:?} missing from Color registry", id).as_str())
    }
}

#[derive(Deref, DerefMut)]
pub struct BoardSlice(Vec<(Hex, StateId)>);

pub trait Tick {
    fn tick(&self, _from: Hex, _states: &BoardState, _rng: &mut SmallRng) -> Option<BoardSlice> {
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

pub trait HexColor {
    const COLOR: Color;
}
