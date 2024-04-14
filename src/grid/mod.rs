mod state;

use std::{
    ops::{Deref, DerefMut},
    time::Duration,
};

use bevy_inspector_egui::{inspector_options::ReflectInspectorOptions, InspectorOptions};
use noisy_bevy::simplex_noise_2d;
use rand::Rng;
pub use state::BoardState;

use crate::{cell::*, input::Input, rng::RngSource, ui::Palette};
use bevy::{math::vec2, prelude::*, window::PrimaryWindow};
use hexx::*;
use leafwing_input_manager::prelude::*;

pub(super) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        // Adjust the size and layout of the board.
        app.init_resource::<BoardState<64>>();
        app.insert_resource(TickRate::new(Duration::from_millis(15)));
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
pub fn startup_system(mut states: ResMut<BoardState<64>>, mut rng: ResMut<RngSource>) {
    states.clear();
    for hex in states.bounds().all_coords() {
        let chance: f32 = rng.gen();
        let state_id = if chance < 0.25 {
            Sand::id()
        } else if chance < 0.0 {
            Fire::id()
        } else if chance < 0.0 {
            Water::id()
        } else {
            Air::id()
        };
        states.set_next(hex, state_id);
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
fn sim_system(
    mut states: ResMut<BoardState<64>>,
    registry: Res<CellRegistry>,
    mut rng: ResMut<RngSource>,
) {
    let slices = states
        .iter()
        .filter_map(|(hex, id)| registry.get(id).map(|tickable| (hex, tickable)))
        .filter_map(|(hex, cell)| cell.behavior.tick(&hex, &states, &mut rng))
        .collect::<Vec<_>>();
    for slice in slices {
        states.apply(slice);
    }
}

/// Move all the queued states into the current state.
fn flush_system(mut states: ResMut<BoardState<64>>) {
    states.tick();
}

/// System to enable user control over the simulation.
#[allow(clippy::too_many_arguments)]
fn control_system(
    query: Query<&ActionState<Input>>,
    mut tick_event: EventWriter<TickEvent>,
    mut rate: ResMut<TickRate>,
    mut states: ResMut<BoardState<64>>,
    palette: Res<Palette>,
    camera: Query<(&Camera, &GlobalTransform)>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let input = query.single();
    if input.just_pressed(&Input::Step) {
        tick_event.send(TickEvent);
    }

    if input.just_pressed(&Input::Fast) {
        rate.fast();
    }

    if input.just_released(&Input::Fast) {
        rate.normal();
    }

    if input.pressed(&Input::Select) {
        let (camera, camera_transform) = camera.single();
        let window = window.single();
        if let Some(world_position) = window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
            .map(|ray| ray.origin.truncate())
        {
            let hex = states.layout().world_pos_to_hex(world_position);
            for hex in hex.rings(0..palette.brush_size).flatten() {
                if states.bounds().is_in_bounds(hex) {
                    states.set_next(hex, palette.selected);
                }
            }
        }
    }
}

/// System to render the cells on the board... using Gizmos!
fn render_system(
    mut draw: Gizmos,
    states: Res<BoardState<64>>,
    registry: Res<CellRegistry>,
    mut rng: ResMut<RngSource>,
    time: Res<Time>,
) where
    [(); Hex::range_count(64) as usize]: Sized,
{
    // HACK Why 0.7? I don't know but it lines up...
    let size = states.layout().hex_size.length() * 0.7;

    for (hex, id) in states.iter() {
        draw.primitive_2d(
            RegularPolygon::new(size, 6),
            states.layout().hex_to_world_pos(hex),
            0.0,
            match *registry.color(id) {
                HexColor::Invisible => Color::NONE,
                HexColor::Static(color) => color,
                HexColor::Flickering {
                    base_color,
                    offset_color,
                } => Color::Rgba {
                    red: base_color.r() + rng.gen::<f32>() * offset_color.r(),
                    green: base_color.g() + rng.gen::<f32>() * offset_color.g(),
                    blue: base_color.b() + rng.gen::<f32>() * offset_color.b(),
                    alpha: base_color.a() + rng.gen::<f32>() * offset_color.a(),
                },
                HexColor::Noise {
                    base_color,
                    offset_color,
                    speed,
                } => {
                    let world_pos = states.layout().hex_to_world_pos(hex);
                    let pos = vec2(
                        world_pos.x + time.elapsed_seconds() * speed.x,
                        world_pos.y + time.elapsed_seconds() * speed.y,
                    );
                    Color::Rgba {
                        red: base_color.r() + simplex_noise_2d(pos) * offset_color.r(),
                        green: base_color.g() + simplex_noise_2d(pos) * offset_color.g(),
                        blue: base_color.b() + simplex_noise_2d(pos) * offset_color.b(),
                        alpha: base_color.a() + simplex_noise_2d(pos) * offset_color.a(),
                    }
                }
            },
        );
    }
}
