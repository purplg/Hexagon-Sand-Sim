use bevy::prelude::*;
use bevy_turborand::prelude::*;

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RngPlugin::default());
    }
}
