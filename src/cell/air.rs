use super::{Register, StateId, Tick};

pub struct Air;

impl Register for Air {
    const ID: StateId = StateId::Air;
}

impl Tick for Air {}
