pub mod cell;
mod state;

use std::{
    fs,
    ops::{Deref, DerefMut},
    time::Duration,
};

use bevy_inspector_egui::{inspector_options::ReflectInspectorOptions, InspectorOptions};
use bevy_turborand::{DelegatedRng, GlobalRng, RngComponent};
use bytebuffer::ByteBuffer;
use noisy_bevy::simplex_noise_2d;
use rayon::iter::{ParallelBridge, ParallelIterator};
pub use state::BoardState;
use unique_type_id::UniqueTypeId as _;

use crate::{input::Input, ui::Palette, GameEvent, SimState};
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

        app.add_systems(Update, save_load_system);

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
pub struct FlushEvent;

#[derive(Component)]
struct HexCell;

#[derive(Resource, Default, Deref, DerefMut)]
struct HexEntities(HashMap<Hex, Entity>);

/// Generate a fresh board.
pub fn startup_system(mut commands: Commands, asset_loader: Res<AssetServer>) {
    let states = BoardState::new(100);
    let mut entities = HexEntities::default();
    let texture = asset_loader.load::<Image>("hex.png");
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
    commands.insert_resource(entities);
}

/// Generate a fresh board.
pub fn generate_system(mut states: ResMut<BoardState>, mut rng: ResMut<GlobalRng>) {
    states.clear();
    for hex in states.bounds().all_coords() {
        let chance = rng.f32();
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
    println!("COUNT: {:?}", states.bounds().all_coords().len());
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
fn sim_system(states: Res<BoardState>, registry: Res<CellRegistry>, mut rng: ResMut<GlobalRng>) {
    let positions = rng.sample_multiple(&states.positions, states.bounds().hex_count());

    positions
        .iter()
        .map(|hex| (**hex, rng.f32()))
        .par_bridge()
        .filter_map(|(hex, rng)| {
            let state = states.get_current(hex).unwrap();
            let cell = registry.get(state).unwrap();
            cell.behavior.tick(hex, &states, rng)
        })
        .for_each(|mut slice| {
            if let Ok(next) = states.next.read() {
                if slice.iter().any(|(hex, _id)| next.contains_key(hex)) {
                    return;
                }
            }

            if let Ok(mut next) = states.next.write() {
                for (hex, id) in slice.drain(0..) {
                    next.insert(hex, id);
                }
            }
        });
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
    states: Res<BoardState>,
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
            let center = states.layout().world_pos_to_hex(world_position);
            for hex in center
                .range(palette.brush_size)
                .filter(|hex| states.bounds().is_in_bounds(*hex))
                .filter(|hex| {
                    states
                        .get_next(*hex)
                        .map(|id| id != palette.selected)
                        .unwrap_or_default()
                })
            {
                states.set_next(hex, palette.selected);
            }
            flush_event.send(FlushEvent);
        }
    }
}

/// System to render the cells on the board... using Sprites!
fn sprite_render_system(
    commands: ParallelCommands,
    mut rng: ResMut<GlobalRng>,
    entities: Res<HexEntities>,
    states: Res<BoardState>,
    registry: Res<CellRegistry>,
    mut cells: Query<&mut RngComponent, With<HexCell>>,
    time: Res<Time>,
) {
    if let Ok(next) = states.next.read() {
        next.iter().par_bridge().for_each(|(hex, id)| {
            commands.command_scope(|mut commands| {
                let entity_id = *entities.get(hex).unwrap();
                let mut entity = commands.entity(entity_id);
                let mut rng = cells.get_mut(entity_id).unwrap();
                match *registry.color(id) {
                    HexColor::Invisible => entity.remove::<Sprite>(),
                    HexColor::Static(color) => entity.insert(Sprite { color, ..default() }),
                    HexColor::Flickering {
                        base_color,
                        offset_color,
                    } => entity.insert(Sprite {
                        color: Color::Rgba {
                            red: base_color.r() + rng.f32() * offset_color.r(),
                            green: base_color.g() + rng.f32() * offset_color.g(),
                            blue: base_color.b() + rng.f32() * offset_color.b(),
                            alpha: base_color.a() + rng.f32() * offset_color.a(),
                        },
                        ..default()
                    }),
                    HexColor::Noise {
                        base_color,
                        offset_color,
                        speed,
                        scale,
                    } => entity.insert(Sprite {
                        color: {
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
                        },
                        ..default()
                    }),
                };
            });
        });
    }
}

fn save_load_system(
    mut game_events: EventReader<GameEvent>,
    mut states: ResMut<BoardState>,
    mut flush_event: EventWriter<FlushEvent>,
) {
    for event in game_events.read() {
        match event {
            GameEvent::Save(path) => {
                let mut buffer = ByteBuffer::new();
                states.serialize(&mut buffer);
                let _ = fs::write(path, buffer.as_bytes());
            }
            GameEvent::Load(path) => {
                let Ok(contents) = fs::read(path) else {
                    return;
                };
                let mut buffer = ByteBuffer::from_vec(contents);
                if states.deserialize(&mut buffer).is_ok() {
                    flush_event.send(FlushEvent);
                }
            }
        }
    }
}
