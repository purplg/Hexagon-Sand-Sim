pub mod behavior;
mod camera;
mod cell;
mod grid;
mod input;
mod rng;
mod ui;

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
    app.add_plugins(rng::Plugin);
    app.add_plugins(camera::Plugin);
    app.add_plugins(input::Plugin);
    app.add_plugins(grid::Plugin);
    app.add_plugins(cell::Plugin);
    app.add_plugins(ui::Plugin);

    #[cfg(feature = "fps")]
    app.add_plugins(bevy_fps_counter::FpsCounterPlugin);

    app.run();
}

