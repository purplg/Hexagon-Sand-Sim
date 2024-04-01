use bevy::{prelude::*, window::PrimaryWindow};
use leafwing_input_manager::prelude::*;

use crate::{
    cell::StateId,
    grid::{Board, Cell, CellStates, EntityMap},
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
}

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<Input>::default());
        app.add_systems(Startup, startup);
        app.add_systems(Update, (info, select));
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
            (Input::PlayPause, KeyCode::Tab),
            (Input::Step, KeyCode::Enter),
            (Input::Fast, KeyCode::Space),
        ])
        .build(),
    ));
}

fn info(
    query: Query<&ActionState<Input>>,
    board: Res<Board>,
    cells: Query<&Cell>,
    entities: Res<EntityMap>,
    states: Res<CellStates>,
    camera: Query<(&Camera, &GlobalTransform)>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let state = query.single();
    if state.just_pressed(&Input::Info) {
        let (camera, camera_transform) = camera.single();
        let window = window.single();
        if let Some(world_position) = window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
            .map(|ray| ray.origin.truncate())
        {
            let cell_hex = board.layout.world_pos_to_hex(world_position);
            let Some(entity) = entities.get(&cell_hex) else {
                return;
            };
            let Ok(cell) = cells.get(*entity) else {
                return;
            };
            println!("cell.state_id: {:?}", states.get_current(cell));
        }
    }
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
