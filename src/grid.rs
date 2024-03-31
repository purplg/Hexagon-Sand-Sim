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
        app.init_resource::<Tracker>();
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

#[derive(Component)]
struct Cell {
    hex: Hex,
    state: CellState,
}

#[derive(Component)]
struct Next(CellState);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CellState {
    On,
    Off,
}

fn startup(mut commands: Commands, mut settings: ResMut<HexGrid>) {
    let mut input_map = InputMap::new([
        (Action::Select, MouseButton::Right),
        (Action::Info, MouseButton::Left),
    ]);
    input_map.insert(Action::PlayPause, KeyCode::Space);
    commands.spawn(InputManagerBundle::with_map(input_map));

    for hex in settings.bounds.all_coords() {
        let mut entity = commands.spawn_empty();
        entity.insert(Cell {
            hex,
            state: if rand::random() {
                CellState::On
            } else {
                CellState::Off
            },
        });
        entity.insert(TransformBundle::from_transform(
            Transform::from_translation(settings.layout.hex_to_world_pos(hex).extend(0.0)),
        ));
        settings.entities.insert(hex, entity.id());
    }
}

fn live_neighbors(pos: Hex, grid: &HexGrid, cells: &Query<(Entity, &Cell)>) -> Vec<Hex> {
    HexBounds::new(pos, 1)
        .all_coords()
        // Ignore self
        .filter(|hex| *hex != pos)
        // hex -> entity
        .filter_map(|hex| grid.entities.get(&hex))
        // entity -> cell
        .filter_map(|entity| cells.get(*entity).ok())
        // CellState::On
        .filter(|(_entity, cell)| cell.state == CellState::On)
        // cell -> hex
        .map(|(_entity, cell)| cell.hex)
        .collect()
}

fn info(
    query: Query<&ActionState<Action>>,
    grid: Res<HexGrid>,
    cells: Query<(Entity, &Cell)>,
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
            let on_count = live_neighbors(cell_hex, &grid, &cells).len();
            println!("on_count: {:?}", on_count);
        }
    }
}
fn select(
    mut commands: Commands,
    query: Query<&ActionState<Action>>,
    grid: Res<HexGrid>,
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
            if let Some(entity) = grid.entities.get(&cell_hex) {
                commands.entity(*entity).insert(Next(CellState::On));
            }
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

fn post_sim(mut commands: Commands, mut cells: Query<(Entity, &mut Cell, &Next)>) {
    for (entity, mut cell, next) in cells.iter_mut() {
        cell.state = next.0;
        commands.entity(entity).remove::<Next>();
    }
}

#[derive(Default, Resource)]
struct Tracker {
    under: usize,
    over: usize,
    born: usize,
}

impl std::fmt::Debug for Tracker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "net {}, under {}, over {}, born {}",
            self.born as isize - (self.under + self.over) as isize,
            self.under,
            self.over,
            self.born,
        )
    }
}

fn sim(
    mut commands: Commands,
    grid: Res<HexGrid>,
    cells: Query<(Entity, &Cell)>,
    mut tracker: ResMut<Tracker>,
) {
    for (entity, cell) in cells.iter() {
        let on_count = live_neighbors(cell.hex, &grid, &cells).len();

        if cell.state == CellState::On {
            if on_count < 3 {
                tracker.under += 1;
                commands.entity(entity).insert(Next(CellState::Off));
            } else if on_count > 4 {
                tracker.over += 1;
                commands.entity(entity).insert(Next(CellState::Off));
            }
        } else {
            if on_count > 1 && on_count < 4 {
                tracker.born += 1;
                commands.entity(entity).insert(Next(CellState::On));
            }
        }
    }
    println!("tracker: {:?}", *tracker);
}

fn gizmos(mut draw: Gizmos, hex: Res<HexGrid>, cells: Query<(&Transform, &Cell)>) {
    // Why 0.7? I don't know but it lines up...
    let size = hex.layout.hex_size.length() * 0.7;

    for (transform, cell) in cells.iter() {
        draw.primitive_2d(
            RegularPolygon::new(size, 6),
            transform.translation.xy(),
            0.0,
            Color::RED,
        );
        if cell.state == CellState::On {
            draw.primitive_2d(
                RegularPolygon::new(size * 0.7, 6),
                transform.translation.xy(),
                0.0,
                Color::BLUE,
            );
        }
    }
}
