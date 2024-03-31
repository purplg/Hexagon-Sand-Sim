use bevy::prelude::*;

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup);
        app.insert_resource(ClearColor(Color::BLACK));
    }
}

fn startup(mut commands: Commands) {
    let mut entity = commands.spawn_empty();
    entity.insert(Camera2dBundle::default());
}
