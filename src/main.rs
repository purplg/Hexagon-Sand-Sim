pub mod behavior;
mod camera;
mod cell;
mod grid;
mod input;
mod rng;
mod ui;

use bevy::prelude::*;
use bevy_inspector_egui::{inspector_options::ReflectInspectorOptions, InspectorOptions};
use input::Input;
use leafwing_input_manager::plugin::InputManagerPlugin;

#[derive(
    States, Default, Debug, Clone, PartialEq, Eq, Hash, Reflect, Resource, InspectorOptions,
)]
#[reflect(Resource, InspectorOptions)]
pub enum SimState {
    Accelerated,
    Running,
    #[default]
    Paused,
}

impl SimState {
    pub fn is_running(&self) -> bool {
        match self {
            SimState::Accelerated | SimState::Running => true,
            SimState::Paused => false,
        }
    }
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Bevy App".to_string(),
            ..default()
        }),
        ..default()
    }));
    app.add_plugins(InputManagerPlugin::<Input>::default());

    app.init_state::<SimState>();
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
