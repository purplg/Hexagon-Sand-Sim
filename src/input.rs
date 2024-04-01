use bevy::{prelude::*, window::PrimaryWindow};
use leafwing_input_manager::prelude::*;

use crate::{
    cell::StateId,
    grid::{Board, CellStates},
};

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
        app.add_plugins(InputManagerPlugin::<Input>::default());
        app.add_systems(Startup, startup);
        app.add_systems(Update, select);
    }
}

fn startup(mut commands: Commands) {
    commands.spawn(InputManagerBundle::with_map(
        InputMap::new([
            (Input::Select, MouseButton::Left),
            (Input::Grab, MouseButton::Right),
        ])
        .insert(Input::Zoom, SingleAxis::mouse_wheel_y())
        .insert(Input::Pan, DualAxis::mouse_motion())
        .insert_multiple([
            (Input::Quit, KeyCode::Escape),
            (Input::PlayPause, KeyCode::Tab),
            (Input::Step, KeyCode::Enter),
            (Input::Fast, KeyCode::Space),
        ])
        .build(),
    ));
}

fn select(
    query: Query<&ActionState<Input>>,
    board: Res<Board>,
    mut states: ResMut<CellStates>,
    camera: Query<(&Camera, &GlobalTransform)>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let state = query.single();
    if state.pressed(&Input::Select) {
        let (camera, camera_transform) = camera.single();
        let window = window.single();
        if let Some(world_position) = window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
            .map(|ray| ray.origin.truncate())
        {
            let hex = board.layout.world_pos_to_hex(world_position);
            states.set(hex, StateId::Air);
        }
    }
}
