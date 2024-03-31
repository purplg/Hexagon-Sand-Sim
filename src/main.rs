mod camera;
mod game_state;
mod grid;
mod input;

use bevy::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Bevy App".to_string(),
            ..default()
        }),
        ..default()
    }));
    app.add_plugins(camera::Plugin);
    app.add_plugins(game_state::Plugin);
    app.add_plugins(input::Plugin);
    app.add_plugins(grid::Plugin);
    app.run();
}
