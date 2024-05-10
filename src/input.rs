use bevy::{app::AppExit, prelude::*};
use leafwing_input_manager::prelude::*;

use crate::SimState;

pub struct Plugin;

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum Input {
    Quit,
    Select,
    Info,
    PlayPause,
    Step,
    Fast,
    Grab,
    Zoom,
    Pan,
}

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup);
        app.add_systems(Update, sim);
        app.add_systems(Update, quit);
    }
}

fn startup(mut commands: Commands) {
    commands.spawn(InputManagerBundle::with_map(
        InputMap::default()
            .insert(Input::Zoom, SingleAxis::mouse_wheel_y())
            .insert(Input::Pan, DualAxis::mouse_motion())
            .insert_multiple([
                (Input::Select, MouseButton::Left),
                (Input::Grab, MouseButton::Right),
            ])
            .insert_multiple([
                (Input::Quit, KeyCode::Escape),
                (Input::PlayPause, KeyCode::Tab),
                (Input::Step, KeyCode::Enter),
                (Input::Fast, KeyCode::Space),
            ])
            .build(),
    ));
}

fn sim(
    query: Query<&ActionState<Input>>,
    state: Res<State<SimState>>,
    mut next_state: ResMut<NextState<SimState>>,
) {
    let query = query.single();
    if query.just_pressed(&Input::PlayPause) {
        match state.get() {
            SimState::Running => next_state.set(SimState::Paused),
            SimState::Paused => next_state.set(SimState::Running),
            _ => {}
        };
    }
    if query.just_pressed(&Input::Fast) {
        match state.get() {
            SimState::Accelerated => {}
            SimState::Running => next_state.set(SimState::Accelerated),
            SimState::Paused => next_state.set(SimState::Accelerated),
        };
    } else if query.just_released(&Input::Fast) {
        if let SimState::Accelerated = state.get() {
            next_state.set(SimState::Paused)
        };
    }
}

fn quit(query: Query<&ActionState<Input>>, mut app_exit_events: ResMut<Events<AppExit>>) {
    let query = query.single();
    if query.just_pressed(&Input::Quit) {
        app_exit_events.send(AppExit);
    }
}
