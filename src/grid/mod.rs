mod state;

use std::time::Duration;

use rand::Rng;
pub use state::{Board, Cell, CellStates, EntityMap};

use crate::{cell::StateId, input::Input, rng::RngSource};
use bevy::{prelude::*, time::common_conditions::on_timer};
use hexx::*;
use leafwing_input_manager::prelude::*;

pub(super) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        // Adjust the size and layout of the board.
        app.insert_resource(Board {
            layout: HexLayout {
                orientation: HexOrientation::Pointy,
                hex_size: Vec2::ONE * 2.0,
                ..default()
            },
            bounds: HexBounds::from_radius(64),
        });
        app.init_resource::<CellStates>();
        app.init_resource::<EntityMap>();
        app.init_state::<SimState>();
        app.add_event::<TickEvent>();

        app.add_systems(Startup, startup_system);
        app.add_systems(
            Update,
            tick_system
                .run_if(on_timer(Duration::from_millis(50)))
                .run_if(in_state(SimState::Running)),
        );
        app.add_systems(Update, tick_system.run_if(in_state(SimState::Accelerated)));
        app.add_systems(Update, step_system.run_if(in_state(SimState::Paused)));
        app.add_systems(PreUpdate, sim_system.run_if(on_event::<TickEvent>()));
        app.add_systems(PostUpdate, flush_system.run_if(on_event::<TickEvent>()));
        app.add_systems(Update, render_system);
    }
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimState {
    Accelerated,
    Running,
    #[default]
    Paused,
}

#[derive(Event)]
struct TickEvent;

/// Generate a fresh board.
fn startup_system(
    mut commands: Commands,
    board: Res<Board>,
    mut entities: ResMut<EntityMap>,
    mut states: ResMut<CellStates>,
    mut rng: ResMut<RngSource>,
) {
    for hex in board.bounds.all_coords() {
        let mut entity = commands.spawn_empty();
        let chance: f32 = rng.gen();
        let state_id = if chance < 0.25 {
            StateId::Sand
        } else if chance < 0.50 {
            StateId::Fire
        } else if chance < 0.75 {
            StateId::Water
        } else {
            StateId::Air
        };
        entity.insert(Cell(hex));
        states.set(hex, state_id);
        entities.insert(hex, entity.id());
    }
    states.tick();
}

/// Sends a tick event to step the simulation forward one step.
fn tick_system(mut tick_event: EventWriter<TickEvent>) {
    tick_event.send(TickEvent);
}

/// System to run the simulation every frame.
fn sim_system(cells: Query<&Cell>, mut states: ResMut<CellStates>, mut rng: ResMut<RngSource>) {
    for cell in cells.iter() {
        let Some(state) = states.get_current(cell).copied() else {
            continue;
        };

        let Some(step) = state.tick(cell.0, &states, &mut rng.0) else {
            continue;
        };

        step.apply(&mut states);
    }
}

/// Move all the queued states into the current state.
fn flush_system(mut states: ResMut<CellStates>) {
    states.tick();
}

/// System to enable user to step one tick forward through the sim.
fn step_system(query: Query<&ActionState<Input>>, mut tick_event: EventWriter<TickEvent>) {
    let query = query.single();
    if query.just_pressed(&Input::Step) {
        tick_event.send(TickEvent);
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
        let Some(next) = states.get_current(cell) else {
            continue;
        };
        draw.primitive_2d(
            RegularPolygon::new(size, 6),
            board.layout.hex_to_world_pos(cell.into()),
            0.0,
            match next {
                StateId::Air => Color::Rgba {
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0,
                    alpha: 0.00,
                },
                StateId::Fire => Color::Rgba {
                    red: 1.0,
                    green: 0.0,
                    blue: 0.0,
                    alpha: 1.0,
                },
                StateId::Sand => Color::Rgba {
                    red: 1.0,
                    green: 1.0,
                    blue: 0.0,
                    alpha: 1.0,
                },
                StateId::Water => Color::Rgba {
                    red: 0.0,
                    green: 0.0,
                    blue: 1.0,
                    alpha: 1.0,
                },
                StateId::Steam => Color::Rgba {
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0,
                    alpha: 1.0,
                },
            },
        );
    }
}
