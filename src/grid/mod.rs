pub mod cell;
mod state;

use std::{
    ops::{Deref, DerefMut},
    time::Duration,
};

use bevy_inspector_egui::{inspector_options::ReflectInspectorOptions, InspectorOptions};
use noisy_bevy::simplex_noise_2d;
use rand::{rngs::SmallRng, seq::SliceRandom as _, Rng};
pub use state::BoardState;
use unique_type_id::UniqueTypeId as _;

use crate::{input::Input, rng::RngSource, ui::Palette, SimState};
use bevy::{
    app::MainScheduleOrder, ecs::schedule::ScheduleLabel, math::vec2, prelude::*, utils::HashMap,
    window::PrimaryWindow,
};
use hexx::*;
use leafwing_input_manager::prelude::*;

use self::cell::*;

pub(super) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(cell::Plugin);

        // Adjust the size and layout of the board.
        app.insert_resource(TickRate::new(Duration::from_millis(15)));
        app.add_event::<TickEvent>();
        app.add_event::<FlushEvent>();

        app.add_systems(
            Startup,
            (
                startup_system,
                generate_system,
                sprite_render_system,
                flush_system,
            )
                .chain(),
        );

        app.add_systems(
            Update,
            tick_system.run_if(|state: Res<State<SimState>>| state.is_running()),
        );

        let mut schedule = app.world.resource_mut::<MainScheduleOrder>();
        schedule.insert_after(Update, CellPreUpdate);
        schedule.insert_after(CellPreUpdate, CellUpdate);
        schedule.insert_after(CellUpdate, CellRender);
        schedule.insert_after(CellRender, CellPostUpdate);

        app.add_systems(CellPreUpdate, control_system);
        app.add_systems(CellUpdate, sim_system.run_if(on_event::<TickEvent>()));
        app.add_systems(
            CellRender,
            sprite_render_system.run_if(on_event::<TickEvent>().or_else(on_event::<FlushEvent>())),
        );
        app.add_systems(
            CellPostUpdate,
            flush_system.run_if(on_event::<TickEvent>().or_else(on_event::<FlushEvent>())),
        );
    }
}

/// Before any cells have been ticked.
#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct CellPreUpdate;

/// When cells are actively being mutated.
#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct CellUpdate;

/// Cells sprites have been updated to reflect their state.
#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct CellRender;

/// Cells have been committed to the [`BoardState`].
#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct CellPostUpdate;

#[derive(Event)]
struct TickEvent;

#[derive(Event)]
struct FlushEvent;

#[derive(Component)]
struct HexCell;

#[derive(Resource, Default, Deref, DerefMut)]
struct HexEntities(HashMap<Hex, Entity>);

#[derive(Resource, Deref)]
pub struct HexTexture(Handle<Image>);

/// Generate a fresh board.
pub fn startup_system(
    mut commands: Commands,
    asset_loader: Res<AssetServer>,
    rng: ResMut<RngSource>,
) {
    let states = BoardState::default();
    let cell_iter = RandomPositionIter::new(states.bounds().all_coords().collect(), rng.clone());
    let mut entities = HexEntities::default();
    let texture = HexTexture(asset_loader.load("hex.png"));
    for hex in states.bounds().all_coords() {
        let mut entity = commands.spawn_empty();
        entities.insert(hex, entity.id());
        entity.insert(HexCell);
        entity.insert(SpriteBundle {
            transform: Transform::from_translation(
                states.layout().hex_to_world_pos(hex).extend(0.0),
            )
            .with_scale(Vec3::new(0.063, 0.063, 1.0)),
            ..default()
        });
        entity.insert(texture.clone());
    }
    commands.insert_resource(states);
    commands.insert_resource(cell_iter);
    commands.insert_resource(texture);
    commands.insert_resource(entities);
}

/// Generate a fresh board.
pub fn generate_system(mut states: ResMut<BoardState>, mut rng: ResMut<RngSource>) {
    states.clear();
    for hex in states.bounds().all_coords() {
        let chance: f32 = rng.gen();
        let state_id = if chance < 0.25 {
            Sand::id()
        } else if chance < 0.50 {
            Fire::id()
        } else if chance < 0.75 {
            Water::id()
        } else {
            Air::id()
        };
        states.set_next(hex, state_id);
    }
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

#[derive(Resource)]
pub struct RandomPositionIter<Item, Rng: rand::Rng> {
    /// The positions to iterate over.
    positions: Vec<Item>,

    /// The current index into [`positions`].
    index: usize,

    rng: Rng,
}

impl<T, Rng: rand::Rng> RandomPositionIter<T, Rng> {
    pub fn new(positions: Vec<T>, rng: Rng) -> Self {
        let mut iter = Self {
            positions,
            index: 0,
            rng,
        };
        iter.shuffle();
        iter
    }

    pub fn shuffle(&mut self) {
        self.positions.as_mut_slice().shuffle(&mut self.rng);
    }
}

impl<T: Copy, Rng: rand::Rng> Iterator for &mut RandomPositionIter<T, Rng> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.positions.len() {
            self.index = 0;
            self.shuffle();
        }
        let item = self.positions.get(self.index).copied();
        self.index += 1;
        item
    }
}

/// System to run the simulation every frame.
fn sim_system(
    mut states: ResMut<BoardState>,
    mut positions: ResMut<RandomPositionIter<Hex, SmallRng>>,
    registry: Res<CellRegistry>,
    mut rng: ResMut<RngSource>,
) {
    let rng = &mut **rng;
    for hex in positions.take(1000) {
        let state = states.get_current(hex).unwrap();
        let cell = registry.get(state).unwrap();

        cell.behavior
            .random_tick(hex, &states, rng)
            .or_else(|| cell.behavior.tick(hex, &states, rng))
            .map(|slice| {
                states.apply(slice);
            });
    }

    for hex in &mut positions.take(20_000) {
        let state = states.get_current(hex).unwrap();
        let cell = registry.get(state).unwrap();

        cell.behavior.tick(hex, &states, rng).map(|slice| {
            states.apply(slice);
        });
    }
}

/// Move all the queued states into the current state.
fn flush_system(mut states: ResMut<BoardState>) {
    states.commit();
}

/// System to enable user control over the simulation.
#[allow(clippy::too_many_arguments)]
fn control_system(
    input: Query<&ActionState<Input>>,
    mut tick_event: EventWriter<TickEvent>,
    mut flush_event: EventWriter<FlushEvent>,
    mut rate: ResMut<TickRate>,
    mut states: ResMut<BoardState>,
    palette: Res<Palette>,
    camera: Query<(&Camera, &GlobalTransform)>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(input) = input.get_single() else {
        return;
    };

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
            flush_event.send(FlushEvent);
        }
    }
}

/// System to render the cells on the board... using Sprites!
fn sprite_render_system(
    mut commands: Commands,
    mut rng: ResMut<RngSource>,
    entities: Res<HexEntities>,
    states: Res<BoardState>,
    registry: Res<CellRegistry>,
    time: Res<Time>,
) where
    [(); Hex::range_count(64) as usize]: Sized,
{
    for (hex, id) in &states.next {
        commands.entity(*entities.get(hex).unwrap()).insert(Sprite {
            color: match *registry.color(id) {
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
                    scale,
                } => {
                    let world_pos = states.layout().hex_to_world_pos(*hex);
                    let pos = vec2(
                        world_pos.x * scale.x + time.elapsed_seconds() * speed.x,
                        world_pos.y * scale.y + time.elapsed_seconds() * speed.y,
                    );
                    Color::Rgba {
                        red: base_color.r() + simplex_noise_2d(pos) * offset_color.r(),
                        green: base_color.g() + simplex_noise_2d(pos) * offset_color.g(),
                        blue: base_color.b() + simplex_noise_2d(pos) * offset_color.b(),
                        alpha: base_color.a() + simplex_noise_2d(pos) * offset_color.a(),
                    }
                }
            },
            ..default()
        });
    }
}
