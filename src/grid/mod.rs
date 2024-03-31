mod state;
pub use state::{Board, Cell, CellStates, EntityMap, NextState};

use crate::{
    cell::{Air, Behavior, Fire, Sand, StateId},
    game_state::GameState,
    input::Input,
};
use bevy::{prelude::*, time::common_conditions::on_timer};
use hexx::*;
use leafwing_input_manager::prelude::*;

pub(super) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup_system);
        app.add_systems(Update, render_system);
        app.add_systems(Update, step_system.run_if(in_state(GameState::Paused)));
        app.add_systems(
            Update,
            sim_system
                .run_if(in_state(GameState::Running))
                .run_if(on_timer(std::time::Duration::from_millis(50))),
        );
        app.init_resource::<CellStates>();
        app.init_resource::<EntityMap>();

        // Adjust the size and layout of the board.
        app.insert_resource(Board {
            layout: HexLayout {
                orientation: HexOrientation::Pointy,
                hex_size: Vec2::new(8.0, 8.0),
                ..default()
            },
            bounds: HexBounds::from_radius(32),
        });
    }
}

/// Generate a fresh board.
fn startup_system(
    mut commands: Commands,
    board: Res<Board>,
    mut entities: ResMut<EntityMap>,
    mut states: ResMut<CellStates>,
) {
    for hex in board.bounds.all_coords() {
        let mut entity = commands.spawn_empty();
        let chance: f32 = rand::random();
        let state_id = if chance < 0.3 {
            StateId::Sand
        } else if chance < 0.7 {
            StateId::Fire
        } else {
            StateId::Air
        };
        entity.insert(Cell(hex));
        states.set(hex, NextState::Spawn(state_id));
        entities.insert(hex, entity.id());
    }
}

/// System to run the simulation every frame.
fn sim_system(cells: Query<&Cell>, mut states: ResMut<CellStates>) {
    for cell in cells.iter() {
        let Some(state) = states.get_current(cell.0) else {
            continue;
        };
        match state {
            StateId::Air => Air::tick(cell.0, &mut states),
            StateId::Fire => Fire::tick(cell.0, &mut states),
            StateId::Sand => Sand::tick(cell.0, &mut states),
        }
    }
    states.tick();
}

/// System to enable user to step one tick forward through the sim.
fn step_system(query: Query<&ActionState<Input>>, cells: Query<&Cell>, states: ResMut<CellStates>) {
    let query = query.single();
    if query.just_pressed(&Input::Step) {
        sim_system(cells, states);
    }
}

/// System to render the cells on the board... using Gizmos!
fn render_system(
    mut draw: Gizmos,
    board: Res<Board>,
    cells: Query<&Cell>,
    states: Res<CellStates>,
) {
    // HACK Why 0.7? I don't know but it lines up...
    let size = board.layout.hex_size.length() * 0.7;

    for cell in cells.iter() {
        if states.get_next(cell.0) == Some(&StateId::Air) {
            draw.primitive_2d(
                RegularPolygon::new(size, 6),
                board.layout.hex_to_world_pos(cell.0),
                0.0,
                Color::Rgba {
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0,
                    alpha: 0.01,
                },
            );
        }
        if states.get_next(cell.0) == Some(&StateId::Fire) {
            draw.primitive_2d(
                RegularPolygon::new(size, 6),
                board.layout.hex_to_world_pos(cell.0),
                0.0,
                Color::Rgba {
                    red: 1.0,
                    green: 0.0,
                    blue: 0.0,
                    alpha: 1.0,
                },
            );
        }
        if states.get_next(cell.0) == Some(&StateId::Sand) {
            draw.primitive_2d(
                RegularPolygon::new(size, 6),
                board.layout.hex_to_world_pos(cell.0),
                0.0,
                Color::Rgba {
                    red: 1.0,
                    green: 1.0,
                    blue: 0.0,
                    alpha: 1.0,
                },
            );
        }
    }
}
