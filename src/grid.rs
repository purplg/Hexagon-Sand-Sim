use bevy::{prelude::*, time::common_conditions::on_timer, utils::HashMap, window::PrimaryWindow};
use hexx::*;
use leafwing_input_manager::prelude::*;

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<Action>::default());
        app.insert_state(GameState::Paused);
        app.add_systems(Startup, startup);
        app.add_systems(Update, play_pause);
        app.add_systems(Update, gizmos);
        app.add_systems(Update, (info, select));
        app.add_systems(
            FixedUpdate,
            (sim, post_sim)
                .chain()
                .run_if(in_state(GameState::Running))
                .run_if(on_timer(std::time::Duration::from_millis(200))),
        );
        let mut states = StateRegistry::default();
        states.insert("air", Air);
        states.insert("sand", Sand);
        states.insert("fire", Fire);
        app.insert_resource(states);
        app.init_resource::<CellStates>();

        app.insert_resource(HexGrid {
            layout: HexLayout {
                orientation: HexOrientation::Pointy,
                hex_size: Vec2::new(16.0, 16.0),
                ..default()
            },
            bounds: HexBounds::from_radius(16),
            entities: Default::default(),
        });
    }
}

#[derive(Resource, Default)]
struct CellStates {
    current: HashMap<Hex, StateId>,
    next: HashMap<Hex, StateId>,
}

impl CellStates {
    fn get_current(&self, hex: &Hex) -> Option<&StateId> {
        self.current.get(hex)
    }

    fn get_next(&self, hex: &Hex) -> Option<&StateId> {
        self.next.get(hex).or_else(|| self.get_current(hex))
    }

    fn is_state(&self, hex: &Hex, state_id: impl Into<StateId>) -> bool {
        self.get_next(hex)
            .is_some_and(|next| *next == state_id.into())
    }

    fn set(&mut self, hex: &Hex, state_id: impl Into<StateId>) -> Option<StateId> {
        self.next.insert(*hex, state_id.into())
    }

    fn flush(&mut self) {
        for (k, v) in self.next.drain() {
            self.current.insert(k, v);
        }
    }
}

#[derive(Resource)]
struct StateRegistry {
    inner: HashMap<StateId, Box<dyn CellState + Send + Sync>>,
    default: Box<dyn CellState + Send + Sync>,
}

impl Default for StateRegistry {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            default: Box::new(Air),
        }
    }
}

impl StateRegistry {
    fn get(&self, id: impl Into<StateId>) -> &Box<dyn CellState + Send + Sync> {
        self.inner.get(&id.into()).unwrap_or(&self.default)
    }

    fn insert(&mut self, id: &'static str, state: impl CellState + Send + Sync + 'static) {
        self.inner.insert(StateId(id), Box::new(state));
    }
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
enum GameState {
    Running,
    Paused,
}

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum Action {
    Select,
    Info,
    PlayPause,
}

#[derive(Resource)]
struct HexGrid {
    entities: HashMap<Hex, Entity>,
    layout: HexLayout,
    bounds: HexBounds,
}

#[derive(Deref, DerefMut, Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct StateId(&'static str);

impl From<&'static str> for StateId {
    fn from(value: &'static str) -> Self {
        StateId(value)
    }
}

impl From<&StateId> for StateId {
    fn from(value: &StateId) -> Self {
        *value
    }
}

trait CellState {
    fn tick(
        &self,
        _center: &Cell,
        _grid: &HexGrid,
        _states: &mut CellStates,
        _cells: &Query<&Cell>,
    ) {
    }

    fn try_swap(
        &self,
        from: Hex,
        to: Hex,
        grid: &HexGrid,
        states: &mut CellStates,
        cells: &Query<&Cell>,
    ) -> bool {
        let Some(entity) = grid.entities.get(&to) else {
            return false;
        };

        let Some(to) = cells.get(*entity).ok() else {
            return false;
        };

        if !states.is_state(to, "air") {
            return false;
        }

        let from_state = states.get_current(&from).unwrap().clone();
        states.set(&to.0, from_state);
        states.set(&from, "air");
        return true;
    }
}

struct Fire;

impl CellState for Fire {
    fn tick(&self, center: &Cell, grid: &HexGrid, states: &mut CellStates, cells: &Query<&Cell>) {
        if rand::random() {
            if !self.try_swap(
                center.0,
                center.neighbor(EdgeDirection::POINTY_TOP_LEFT),
                grid,
                states,
                &cells,
            ) {
                self.try_swap(
                    center.0,
                    center.neighbor(EdgeDirection::POINTY_TOP_LEFT),
                    grid,
                    states,
                    &cells,
                );
            }
        } else {
            if !self.try_swap(
                center.0,
                center.neighbor(EdgeDirection::POINTY_TOP_RIGHT),
                grid,
                states,
                &cells,
            ) {
                self.try_swap(
                    center.0,
                    center.neighbor(EdgeDirection::POINTY_TOP_LEFT),
                    grid,
                    states,
                    &cells,
                );
            }
        }
    }
}

struct Sand;

impl CellState for Sand {
    fn tick(&self, center: &Cell, grid: &HexGrid, states: &mut CellStates, cells: &Query<&Cell>) {
        if rand::random() {
            if !self.try_swap(
                center.0,
                center.neighbor(EdgeDirection::POINTY_BOTTOM_LEFT),
                grid,
                states,
                &cells,
            ) {
                self.try_swap(
                    center.0,
                    center.neighbor(EdgeDirection::POINTY_BOTTOM_LEFT),
                    grid,
                    states,
                    &cells,
                );
            }
        } else {
            if !self.try_swap(
                center.0,
                center.neighbor(EdgeDirection::POINTY_BOTTOM_RIGHT),
                grid,
                states,
                &cells,
            ) {
                self.try_swap(
                    center.0,
                    center.neighbor(EdgeDirection::POINTY_BOTTOM_LEFT),
                    grid,
                    states,
                    &cells,
                );
            }
        }
    }
}

struct Air;
impl CellState for Air {}

#[derive(Component, Deref)]
struct Cell(Hex);

#[derive(Bundle)]
struct CellBundle {
    cell: Cell,
    transform: TransformBundle,
}

fn startup(mut commands: Commands, mut settings: ResMut<HexGrid>, mut states: ResMut<CellStates>) {
    let mut input_map = InputMap::new([
        (Action::Select, MouseButton::Right),
        (Action::Info, MouseButton::Left),
    ]);
    input_map.insert(Action::PlayPause, KeyCode::Space);
    commands.spawn(InputManagerBundle::with_map(input_map));

    for hex in settings.bounds.all_coords() {
        let mut entity = commands.spawn_empty();
        entity.insert(CellBundle {
            cell: Cell(hex),
            transform: TransformBundle::from_transform(Transform::from_translation(
                settings.layout.hex_to_world_pos(hex).extend(0.0),
            )),
        });
        let chance: f32 = rand::random();
        states.set(
            &hex,
            if chance < 0.3 {
                "air"
            } else if chance < 0.7 {
                "sand"
            } else {
                "fire"
            },
        );
        settings.entities.insert(hex, entity.id());
    }
    states.flush();
}

fn info(
    query: Query<&ActionState<Action>>,
    grid: Res<HexGrid>,
    cells: Query<&Cell>,
    states: Res<CellStates>,
    camera: Query<(&Camera, &GlobalTransform)>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let state = query.single();
    if state.just_pressed(&Action::Info) {
        let (camera, camera_transform) = camera.single();
        let window = window.single();
        if let Some(world_position) = window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
            .map(|ray| ray.origin.truncate())
        {
            let cell_hex = grid.layout.world_pos_to_hex(world_position);
            let Some(entity) = grid.entities.get(&cell_hex) else {
                return;
            };
            let Ok(cell) = cells.get(*entity) else {
                return;
            };
            println!("cell.state_id: {:?}", states.get_current(&cell.0));
        }
    }
}

fn select(
    query: Query<&ActionState<Action>>,
    grid: Res<HexGrid>,
    mut states: ResMut<CellStates>,
    camera: Query<(&Camera, &GlobalTransform)>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let state = query.single();
    if state.just_pressed(&Action::Select) {
        let (camera, camera_transform) = camera.single();
        let window = window.single();
        if let Some(world_position) = window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
            .map(|ray| ray.origin.truncate())
        {
            let cell_hex = grid.layout.world_pos_to_hex(world_position);
            states.set(&cell_hex, "sand");
        }
    }
}

fn play_pause(
    query: Query<&ActionState<Action>>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let query = query.single();
    if query.just_pressed(&Action::PlayPause) {
        match state.get() {
            GameState::Running => next_state.set(GameState::Paused),
            GameState::Paused => next_state.set(GameState::Running),
        };
    }
}

fn sim(
    grid: Res<HexGrid>,
    state_ids: Res<StateRegistry>,
    cells: Query<&Cell>,
    mut states: ResMut<CellStates>,
) {
    for cell in cells.iter() {
        let Some(state_id) = states.get_current(&cell.0) else {
            continue;
        };
        let state = state_ids.get(state_id);
        state.tick(cell, &grid, &mut states, &cells);
    }
}

fn post_sim(mut states: ResMut<CellStates>) {
    states.flush();
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
        if states.is_state(cell, "fire") {
            draw.primitive_2d(
                RegularPolygon::new(size, 6),
                transform.translation.xy(),
                0.0,
                Color::RED,
            );
        }
        if states.is_state(cell, "sand") {
            draw.primitive_2d(
                RegularPolygon::new(size, 6),
                transform.translation.xy(),
                0.0,
                Color::YELLOW,
            );
        }
    }
}
