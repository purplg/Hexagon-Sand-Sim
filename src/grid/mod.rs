mod state;

use std::{
    ops::{Deref, DerefMut},
    time::Duration,
};

use bevy_inspector_egui::{inspector_options::ReflectInspectorOptions, InspectorOptions};
use rand::Rng;
pub use state::{Board, CellStates};

use crate::{cell::StateId, input::Input, rng::RngSource};
use bevy::prelude::*;
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
        app.insert_resource(TickRate::new(Duration::from_millis(50)));
        app.init_state::<SimState>();
        app.add_event::<TickEvent>();

        app.add_systems(Startup, startup_system);
        app.add_systems(
            Update,
            tick_system.run_if(|state: Res<State<SimState>>| state.is_running()),
        );
        app.add_systems(Update, control_system);
        app.add_systems(PreUpdate, sim_system.run_if(on_event::<TickEvent>()));
        app.add_systems(PostUpdate, flush_system.run_if(on_event::<TickEvent>()));
        app.add_systems(Update, render_system);
    }
}

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

#[derive(Event)]
struct TickEvent;

/// Generate a fresh board.
pub fn startup_system(
    board: Res<Board>,
    mut states: ResMut<CellStates>,
    mut rng: ResMut<RngSource>,
) {
    states.current.clear();
    states.next.clear();
    for hex in board.bounds.all_coords() {
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
        states.set(hex, state_id);
    }
    states.tick();
}

#[derive(Reflect, Resource, Default, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct TickRate {
    pub timer: Timer,
    pub normal_speed: Duration,
}

impl Deref for TickRate {
    type Target = Timer;

    fn deref(&self) -> &Self::Target {
        &self.timer
    }
}

impl DerefMut for TickRate {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.timer
    }
}

impl TickRate {
    fn new(duration: Duration) -> Self {
        Self {
            timer: Timer::new(duration, TimerMode::Repeating),
            normal_speed: duration,
        }
    }

    pub fn fast(&mut self) {
        self.timer.set_duration(Duration::ZERO);
    }

    pub fn normal(&mut self) {
        self.timer.set_duration(self.normal_speed);
    }

    pub fn set_normal(&mut self, millis: u64) {
        self.normal_speed = Duration::from_millis(millis);
        self.normal();
    }
}

/// Sends a tick event to step the simulation forward one step.
fn tick_system(
    mut tick_event: EventWriter<TickEvent>,
    mut rate: ResMut<TickRate>,
    time: Res<Time>,
) {
    rate.tick(time.delta());
    if rate.just_finished() {
        tick_event.send(TickEvent);
    }
}

/// System to run the simulation every frame.
fn sim_system(mut states: ResMut<CellStates>, mut rng: ResMut<RngSource>) {
    for step in states
        .current
        .iter()
        .filter_map(|(hex, state)| state.tick(*hex, &states, &mut rng.0))
        .collect::<Vec<_>>()
        .into_iter()
    {
        step.apply(&mut states);
    }
}

/// Move all the queued states into the current state.
fn flush_system(mut states: ResMut<CellStates>) {
    states.tick();
}

/// System to enable enable use control over the simulation.
fn control_system(
    query: Query<&ActionState<Input>>,
    mut tick_event: EventWriter<TickEvent>,
    mut rate: ResMut<TickRate>,
) {
    let query = query.single();
    if query.just_pressed(&Input::Step) {
        tick_event.send(TickEvent);
    }

    if query.just_pressed(&Input::Fast) {
        rate.fast();
    }

    if query.just_released(&Input::Fast) {
        rate.normal();
    }
}

/// System to render the cells on the board... using Gizmos!
fn render_system(mut draw: Gizmos, board: Res<Board>, states: Res<CellStates>) {
    // HACK Why 0.7? I don't know but it lines up...
    let size = board.layout.hex_size.length() * 0.7;

    for (hex, id) in states.current.iter() {
        draw.primitive_2d(
            RegularPolygon::new(size, 6),
            board.layout.hex_to_world_pos(*hex),
            0.0,
            match id {
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
