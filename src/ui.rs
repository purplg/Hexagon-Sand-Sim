use std::{
    borrow::Cow,
    ops::{Deref, DerefMut},
};

use bevy::{ecs::system::RunSystemOnce, prelude::*, window::PrimaryWindow};
use bevy_inspector_egui::{
    bevy_egui::{EguiContext, EguiPlugin},
    bevy_inspector::{self, ui_for_state},
    egui::{self, Id},
    inspector_options::ReflectInspectorOptions,
    DefaultInspectorConfigPlugin, InspectorOptions,
};
use unique_type_id::UniqueTypeId;

use crate::{
    behavior::StateId,
    grid::{
        self,
        cell::{Air, CellRegistry},
        BoardState, FlushEvent, TickRate,
    },
    GameEvent, SimState,
};

static EMPTY_NAME: Cow<'static, str> = Cow::Owned(String::new());

pub(super) struct Plugin {
    pub initial_selected: StateId,
    pub initial_brush_size: u32,
    pub initial_save_location: String,
}

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin);
        app.add_plugins(DefaultInspectorConfigPlugin);

        app.init_resource::<Tooltip>();
        app.insert_resource(Palette {
            selected: self.initial_selected,
            brush_size: self.initial_brush_size,
        });

        app.insert_resource(SaveLocation(Cow::Owned(self.initial_save_location.clone())));
        app.add_systems(Update, update_system);
        app.add_systems(Update, tooltip_system);
    }
}

impl Default for Plugin {
    fn default() -> Self {
        Self {
            initial_selected: Air::id(),
            initial_brush_size: 1,
            initial_save_location: "/tmp/sandsim".into(),
        }
    }
}

#[derive(Reflect, Default, Resource, InspectorOptions, Deref)]
#[reflect(Resource, InspectorOptions)]
struct SaveLocation(Cow<'static, str>);

fn update_system(world: &mut World) {
    let mut egui_ctx = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .single(world)
        .clone();

    egui::SidePanel::right("sidepanel").show(egui_ctx.get_mut(), |ui| {
        ui.add_space(16.);

        bevy_inspector::ui_for_resource::<Tooltip>(world, ui);

        ui.add_space(16.);
        ui.push_id(Id::from("tickrate"), |ui| {
            ui_for_state::<SimState>(world, ui);

            let mut timer = world.resource_mut::<TickRate>();
            ui.heading("Tick Rate");
            let mut speed = timer.duration().as_millis() as u64;
            if ui
                .add(egui::Slider::new(&mut speed, 0..=100))
                .on_hover_text("Adjust the speed of the simulation.")
                .dragged()
            {
                timer.set_normal(speed);
            }
        });

        ui.add_space(16.);
        ui.push_id(Id::from("control"), |ui| {
            ui.horizontal_top(|ui| {
                if ui.button("Generate").clicked() {
                    world.run_system_once(grid::generate_system);
                }
                if ui.button("Clear").clicked() {
                    let mut states = world.resource_mut::<BoardState>();
                    states.clear();
                    world.send_event(FlushEvent);
                }
            });
            bevy_inspector::ui_for_resource::<SaveLocation>(world, ui);
            ui.horizontal_top(|ui| {
                if ui.button("Save").clicked() {
                    let filename = world.resource::<SaveLocation>().trim();
                    world.send_event(GameEvent::Save(filename.to_owned()));
                }

                if ui.button("Load").clicked() {
                    let filename = world.resource::<SaveLocation>().trim();
                    world.send_event(GameEvent::Load(filename.to_owned()));
                }
            });
        });
    });

    egui::TopBottomPanel::bottom("palette").show(egui_ctx.get_mut(), |ui| {
        ui.horizontal(|ui| {
            let registry = world.resource::<CellRegistry>().names().collect::<Vec<_>>();
            let mut palette = world.resource_mut::<Palette>();
            ui.add(egui::Slider::new(&mut palette.brush_size, 0..=100));
            let mut cells = registry.into_iter().collect::<Vec<_>>();
            cells.sort_by(|(_id_a, name_a), (_id_b, name_b)| name_a.cmp(name_b));
            for (id, name) in cells {
                ui.radio_value(&mut palette.selected, id, name);
            }
        });
    });
}

#[derive(Reflect, Default, Resource, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
struct Tooltip(Cow<'static, str>);

fn tooltip_system(
    states: Res<BoardState>,
    registry: Res<CellRegistry>,
    mut tooltip: ResMut<Tooltip>,
    camera: Query<(&Camera, &GlobalTransform)>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok((camera, camera_transform)) = camera.get_single() else {
        return;
    };

    let window = window.single();
    let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    else {
        return;
    };

    let hex = states.layout().world_pos_to_hex(world_position);
    if let Some(entry) = states.get_current(hex).and_then(|id| registry.get(id)) {
        tooltip.0.clone_from(&entry.name);
    } else {
        tooltip.0.clone_from(&EMPTY_NAME);
    }
}

#[derive(Resource)]
pub struct Palette {
    pub selected: StateId,
    pub brush_size: u32,
}

impl Deref for Palette {
    type Target = StateId;

    fn deref(&self) -> &Self::Target {
        &self.selected
    }
}

impl DerefMut for Palette {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.selected
    }
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            selected: Air::id(),
            brush_size: 1,
        }
    }
}
