use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

pub struct Plugin;

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum Input {
    Quit,
    Select,
    Info,
    PlayPause,
    Step,
}

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<Input>::default());
        app.add_systems(Startup, startup);
    }
}

fn startup(mut commands: Commands) {
    commands.spawn(InputManagerBundle::with_map(
        InputMap::new([
            (Input::Select, MouseButton::Right),
            (Input::Info, MouseButton::Left),
        ])
        .insert_multiple([
            (Input::Quit, KeyCode::Escape),
            (Input::PlayPause, KeyCode::Space),
            (Input::Step, KeyCode::Enter),
        ])
        .build(),
    ));
}
