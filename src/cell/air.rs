use super::{Register, StateId, Tickable};

pub struct Air;

impl Register for Air {
    const ID: StateId = StateId::Air;
}

impl Tickable for Air {}
