use bevy::{app::AppExit, prelude::*};
use leafwing_input_manager::prelude::*;

use crate::input::Input;

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.insert_state(GameState::Paused);
        app.add_systems(Update, input);
    }
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    Running,
    Paused,
}

fn input(
    query: Query<&ActionState<Input>>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut app_exit_events: ResMut<Events<AppExit>>,
) {
    let query = query.single();
    if query.just_pressed(&Input::PlayPause) {
        match state.get() {
            GameState::Running => next_state.set(GameState::Paused),
            GameState::Paused => next_state.set(GameState::Running),
        };
    }

    if query.just_pressed(&Input::Quit) {
        app_exit_events.send(AppExit);
    }
}
