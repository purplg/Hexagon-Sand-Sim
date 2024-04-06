use std::ops::{Deref, DerefMut};

use bevy::{ecs::system::RunSystemOnce, prelude::*, window::PrimaryWindow};
use bevy_inspector_egui::{
    bevy_egui::{EguiContext, EguiPlugin},
    bevy_inspector::{self, ui_for_state},
    egui::{self, Id},
    inspector_options::ReflectInspectorOptions,
    DefaultInspectorConfigPlugin, InspectorOptions,
};

use crate::{
    cell::StateId,
    grid::{self, Board, SimState, BoardState, TickRate},
};

pub(super) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Metrics>();
        app.init_resource::<Palette>();
        app.add_plugins(EguiPlugin);
        app.add_plugins(DefaultInspectorConfigPlugin);
        app.add_systems(Update, metrics_system);
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
        ui.push_id(Id::from("metrics"), |ui| {
            ui.heading("Metrics");
            bevy_inspector::ui_for_resource::<Metrics>(world, ui);
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

    egui::TopBottomPanel::bottom("pallete").show(egui_ctx.get_mut(), |ui| {
        ui.horizontal(|ui| {
            let mut palette = world.resource_mut::<Palette>();
            ui.add(egui::Slider::new(&mut palette.brush_size, 0..=10));
            ui.radio_value(&mut palette.selected, StateId::Air, "Air");
            ui.radio_value(&mut palette.selected, StateId::Fire, "Fire");
            ui.radio_value(&mut palette.selected, StateId::Sand, "Sand");
            ui.radio_value(&mut palette.selected, StateId::Water, "Water");
            ui.radio_value(&mut palette.selected, StateId::Steam, "Steam");
            ui.radio_value(&mut palette.selected, StateId::Wind, "Wind");
        });
    });
}

#[derive(Reflect, Resource, Default, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct Metrics {
    fire: usize,
    sand: usize,
    water: usize,
    steam: usize,
    wind: usize,
    total: usize,
    movement: usize,
}

fn metrics_system(states: Res<BoardState>, mut metrics: ResMut<Metrics>) {
    metrics.fire = 0;
    metrics.sand = 0;
    metrics.water = 0;
    metrics.steam = 0;
    metrics.wind = 0;
    metrics.movement = 0;
    for state in states.current.values() {
        match state {
            StateId::Fire => metrics.fire += 1,
            StateId::Sand => metrics.sand += 1,
            StateId::Water => metrics.water += 1,
            StateId::Steam => metrics.steam += 1,
            StateId::Wind => metrics.wind += 1,
            StateId::Air => {}
        }
    }
    metrics.total = metrics.fire + metrics.sand + metrics.water + metrics.steam;
    metrics.movement = states.next.keys().count();
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
            selected: StateId::Air,
            brush_size: 2,
        }
    }
}
