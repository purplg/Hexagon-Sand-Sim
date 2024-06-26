pub mod behavior;
mod camera;
mod grid;
mod input;
mod rng;
mod ui;

use bevy::prelude::*;
use bevy_inspector_egui::{inspector_options::ReflectInspectorOptions, InspectorOptions};
use grid::cell::Air;
use input::Input;
use leafwing_input_manager::plugin::InputManagerPlugin;
use unique_type_id::UniqueTypeId as _;

#[derive(
    States, Default, Debug, Clone, PartialEq, Eq, Hash, Reflect, Resource, InspectorOptions,
)]
#[reflect(Resource, InspectorOptions)]
pub enum SimState {
    Accelerated,
    #[default]
    Running,
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

#[derive(Event)]
pub enum GameEvent {
    Save(String),
    Load(String),
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Hexagonal Sand Sim".to_string(),
            canvas: Some("#sandsim-canvas".into()),
            ..default()
        }),
        ..default()
    }));
    app.add_plugins(InputManagerPlugin::<Input>::default());

    app.init_state::<SimState>();
    app.add_event::<GameEvent>();
    app.add_plugins(rng::Plugin);
    app.add_plugins(camera::Plugin);
    app.add_plugins(input::Plugin);
    app.add_plugins(grid::Plugin::new(100));
    app.add_plugins(ui::Plugin {
        initial_selected: Air::id(),
        initial_brush_size: 1,
        ..default()
    });

    #[cfg(feature = "fps")]
    app.add_plugins(bevy_fps_counter::FpsCounterPlugin);

    app.run();
}
