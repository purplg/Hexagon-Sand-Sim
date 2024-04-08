use std::ops::{Deref, DerefMut};

use bevy::{ecs::system::RunSystemOnce, prelude::*, window::PrimaryWindow};
use bevy_inspector_egui::{
    bevy_egui::{EguiContext, EguiPlugin},
    bevy_inspector::{self, ui_for_state},
    egui::{self, Id},
    DefaultInspectorConfigPlugin,
};

use crate::{
    cell::*,
    grid::{self, Board, BoardState, SimState, TickRate},
};

pub(super) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Palette {
            selected: Air::id(),
            brush_size: 1,
        });
        app.add_plugins(EguiPlugin);
        app.add_plugins(DefaultInspectorConfigPlugin);
        app.add_systems(Update, update_system);
    }
}

fn update_system(world: &mut World) {
    let mut egui_ctx = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .single(world)
        .clone();

    egui::SidePanel::left("sidepanel").show(egui_ctx.get_mut(), |ui| {
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
        ui.push_id(Id::from("Board"), |ui| {
            ui.heading("Board");
            bevy_inspector::ui_for_resource::<Board>(world, ui);
        });

        ui.add_space(16.);
        ui.push_id(Id::from("control"), |ui| {
            ui.strong("Board must be generated when radius changes");
            ui.horizontal_top(|ui| {
                if ui.button("Generate").clicked() {
                    world.run_system_once(grid::startup_system);
                }
                if ui.button("Clear").clicked() {
                    let mut states = world.resource_mut::<BoardState>();
                    states.current.clear();
                    states.next.clear();
                }
            });
        });
    });

    egui::TopBottomPanel::bottom("palette").show(egui_ctx.get_mut(), |ui| {
        ui.horizontal(|ui| {
            let registry = world.resource::<CellRegistry>().names().collect::<Vec<_>>();
            let mut palette = world.resource_mut::<Palette>();
            ui.add(egui::Slider::new(&mut palette.brush_size, 0..=10));
            for (id, name) in registry {
                ui.radio_value(&mut palette.selected, id, name);
            }
        });
    });
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
