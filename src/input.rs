use bevy::{app::AppExit, prelude::*};
use leafwing_input_manager::prelude::*;

pub struct Plugin;

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum Action {
    Quit,
}

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<Action>::default());
        app.add_systems(Startup, startup);
        app.add_systems(Update, quit);
    }
}

fn startup(mut commands: Commands) {
    let input_map = InputMap::new([(Action::Quit, KeyCode::Escape)]);
    commands.spawn(InputManagerBundle::with_map(input_map));
}

fn quit(query: Query<&ActionState<Action>>, mut app_exit_events: ResMut<Events<AppExit>>) {
    let state = query.single();
    if state.pressed(&Action::Quit) {
        app_exit_events.send(AppExit);
    }
}
