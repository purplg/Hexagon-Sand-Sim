use bevy::{prelude::*, time::common_conditions::on_timer, utils::HashMap, window::PrimaryWindow};
use hexx::*;
use leafwing_input_manager::prelude::*;
use rand::seq::SliceRandom;

use crate::{game_state::GameState, input::Input};

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup);
        app.add_systems(Update, gizmos);
        app.add_systems(Update, step.run_if(in_state(GameState::Paused)));
        app.add_systems(Update, (info, select));
        app.add_systems(
            FixedUpdate,
            sim.run_if(in_state(GameState::Running))
                .run_if(on_timer(std::time::Duration::from_millis(50))),
        );
        app.init_resource::<CellStates>();
        app.init_resource::<EntityMap>();

        app.insert_resource(HexGrid {
            layout: HexLayout {
                orientation: HexOrientation::Pointy,
                hex_size: Vec2::new(8.0, 8.0),
                ..default()
            },
            bounds: HexBounds::from_radius(32),
        });
    }
}

#[derive(Clone, Copy)]
enum NextState {
    /// A new cell was created.
    Spawn(StateId),

    /// The state from another cell will be used.
    Other(Hex),
}

#[derive(Resource, Default, Deref, DerefMut)]
struct EntityMap(HashMap<Hex, Entity>);

#[derive(Resource, Default)]
struct CellStates {
    /// The active state of the board.
    current: HashMap<Hex, StateId>,

    /// Used as a buffer to stage the next frame.
    stage: HashMap<Hex, StateId>,

    /// The delta for the next frame to be applied.
    next: HashMap<Hex, NextState>,
}

impl CellStates {
    fn get_current(&self, hex: Hex) -> Option<&StateId> {
        self.current.get(&hex)
    }

    fn get_next(&self, hex: Hex) -> Option<&StateId> {
        match self.next.get(&hex) {
            Some(NextState::Spawn(id)) => Some(id),
            Some(NextState::Other(other)) => self.get_current(*other),
            None => self.get_current(hex),
        }
    }

    fn is_state(&self, hex: Hex, other_id: StateId) -> bool {
        self.get_next(hex)
            .map(|id| *id == other_id)
            .unwrap_or_default()
    }

    fn set(&mut self, hex: Hex, next_state: NextState) -> bool {
        self.next.try_insert(hex, next_state).is_ok()
    }

    fn tick(&mut self) {
        println!(
            "count: {:?}",
            self.current
                .values()
                .filter(|id| **id != StateId::Air)
                .count()
        );
        for (hex, next_state) in self.next.drain() {
            match next_state {
                NextState::Spawn(id) => {
                    self.stage.insert(hex, id);
                }
                NextState::Other(other) => {
                    if let Some(other_id) = self.current.get(&other) {
                        self.stage.insert(hex, *other_id);
                    }
                }
            };
        }

        for (hex, id) in self.stage.drain() {
            self.current.insert(hex, id);
        }
    }
}

#[derive(Resource)]
struct HexGrid {
    layout: HexLayout,
    bounds: HexBounds,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum StateId {
    Air,
    Fire,
    Sand,
}

trait Step {
    fn apply(&self, states: &mut CellStates);
}

struct Swap {
    from: Hex,
    to: Hex,
}

impl Step for Swap {
    fn apply(&self, states: &mut CellStates) {
        let _ = states.set(self.from, NextState::Other(self.to))
            && states.set(self.to, NextState::Other(self.from));
    }
}

trait Behavior {
    fn state_id() -> StateId;

    fn tick(_from: Hex, _states: &mut CellStates) {}

    fn try_step(from: Hex, direction: EdgeDirection, states: &CellStates) -> Option<Box<dyn Step>> {
        let to = from.neighbor(direction);

        if !states.is_state(to, StateId::Air) {
            return None;
        }
        Some(Box::new(Swap { to, from }))
    }
}

struct Fire;

impl Behavior for Fire {
    fn state_id() -> StateId {
        StateId::Fire
    }

    fn tick(from: Hex, states: &mut CellStates) {
        if let Some(step) = [
            EdgeDirection::POINTY_TOP_LEFT,
            EdgeDirection::POINTY_TOP_RIGHT,
        ]
        .choose(&mut rand::thread_rng())
        .into_iter()
        .find_map(|direction| Self::try_step(from, *direction, states))
        {
            step.apply(states)
        }
    }
}

struct Sand;
impl Behavior for Sand {
    fn state_id() -> StateId {
        StateId::Sand
    }

    fn tick(from: Hex, states: &mut CellStates) {
        if let Some(step) = [
            EdgeDirection::POINTY_BOTTOM_LEFT,
            EdgeDirection::POINTY_BOTTOM_RIGHT,
        ]
        .choose(&mut rand::thread_rng())
        .into_iter()
        .find_map(|direction| Self::try_step(from, *direction, states))
        {
            step.apply(states)
        }
    }
}

struct Air;
impl Behavior for Air {
    fn state_id() -> StateId {
        StateId::Air
    }
}

#[derive(Component, Deref)]
struct Cell(Hex);

#[derive(Bundle)]
struct CellBundle {
    cell: Cell,
    transform: TransformBundle,
}

fn startup(
    mut commands: Commands,
    settings: Res<HexGrid>,
    mut entities: ResMut<EntityMap>,
    mut states: ResMut<CellStates>,
) {
    for hex in settings.bounds.all_coords() {
        let mut entity = commands.spawn_empty();
        let chance: f32 = rand::random();
        let state_id = if chance < 0.3 {
            StateId::Sand
        } else if chance < 0.7 {
            StateId::Fire
        } else {
            StateId::Air
        };

        entity.insert(CellBundle {
            cell: Cell(hex),
            transform: TransformBundle::from_transform(Transform::from_translation(
                settings.layout.hex_to_world_pos(hex).extend(0.0),
            )),
        });

        states.set(hex, NextState::Spawn(state_id));
        entities.insert(hex, entity.id());
    }
}

fn info(
    query: Query<&ActionState<Input>>,
    grid: Res<HexGrid>,
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
            let cell_hex = grid.layout.world_pos_to_hex(world_position);
            let Some(entity) = entities.get(&cell_hex) else {
                return;
            };
            let Ok(cell) = cells.get(*entity) else {
                return;
            };
            println!("cell.state_id: {:?}", states.get_current(cell.0));
        }
    }
}

fn select(
    query: Query<&ActionState<Input>>,
    grid: Res<HexGrid>,
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
            let hex = grid.layout.world_pos_to_hex(world_position);
            states.set(hex, NextState::Spawn(StateId::Air));
        }
    }
}

fn sim(cells: Query<&Cell>, mut states: ResMut<CellStates>) {
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

fn gizmos(
    mut draw: Gizmos,
    hex: Res<HexGrid>,
    cells: Query<(&Transform, &Cell)>,
    states: Res<CellStates>,
) {
    // Why 0.7? I don't know but it lines up...
    let size = hex.layout.hex_size.length() * 0.7;

    for (transform, cell) in cells.iter() {
        if states.get_next(cell.0) == Some(&StateId::Air) {
            draw.primitive_2d(
                RegularPolygon::new(size, 6),
                transform.translation.xy(),
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
                transform.translation.xy(),
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
                transform.translation.xy(),
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

fn step(query: Query<&ActionState<Input>>, cells: Query<&Cell>, states: ResMut<CellStates>) {
    let query = query.single();
    if query.just_pressed(&Input::Step) {
        sim(cells, states);
    }
}
