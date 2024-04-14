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
use unique_type_id::UniqueTypeId as _;

use crate::{
    behavior::StateId,
    cell::*,
    grid::{self, BoardState, SimState, TickRate},
};

static EMPTY_NAME: Cow<'static, str> = Cow::Owned(String::new());

pub(super) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Palette {
            selected: Air::id(),
            brush_size: 1,
        });
        app.init_resource::<Tooltip>();
        app.add_plugins(EguiPlugin);
        app.add_plugins(DefaultInspectorConfigPlugin);
        app.add_systems(Update, update_system);
        app.add_systems(Update, tooltip_system);
    }
}

fn update_system(world: &mut World) {
    let mut egui_ctx = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .single(world)
        .clone();

    egui::SidePanel::left("sidepanel").show(egui_ctx.get_mut(), |ui| {
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

        // TODO
        // ui.add_space(16.);
        // ui.push_id(Id::from("Board"), |ui| {
        //     ui.heading("Board");
        //     bevy_inspector::ui_for_resource::<Board>(world, ui);
        // });

        ui.add_space(16.);
        ui.push_id(Id::from("control"), |ui| {
            ui.strong("Board must be generated when radius changes");
            ui.horizontal_top(|ui| {
                if ui.button("Generate").clicked() {
                    world.run_system_once(grid::startup_system);
                }
                if ui.button("Clear").clicked() {
                    let mut states = world.resource_mut::<BoardState>();
                    states.clear();
                }
            });
        });
    });

    egui::TopBottomPanel::bottom("palette").show(egui_ctx.get_mut(), |ui| {
        ui.horizontal(|ui| {
            let registry = world.resource::<CellRegistry>().names().collect::<Vec<_>>();
            let mut palette = world.resource_mut::<Palette>();
            ui.add(egui::Slider::new(&mut palette.brush_size, 0..=10));
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
    let (camera, camera_transform) = camera.single();
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
