use bevy::prelude::*;
use rand::{rngs::SmallRng, SeedableRng};

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RngSource>();
    }
}

#[derive(Deref, DerefMut, Resource)]
pub struct RngSource(pub SmallRng);

impl Default for RngSource {
    fn default() -> Self {
        Self(SmallRng::from_seed([0; 32]))
    }
}
