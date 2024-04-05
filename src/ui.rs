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
    grid::{self, Board, States, SimState, TickRate},
};

pub(super) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Metrics>();
        app.add_plugins(EguiPlugin);
        app.add_plugins(DefaultInspectorConfigPlugin);
        app.add_systems(Update, metrics_system);
        app.add_systems(Startup, setup_system);
        app.add_systems(Update, update_system);
    }
}

fn setup_system() {}

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
            let response = ui
                .add(egui::Slider::new(&mut speed, 0..=1000))
                .on_hover_text("Adjust the speed of the simulation.");
            if response.dragged() {
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
                    let mut states = world.resource_mut::<States>();
                    states.current.clear();
                    states.next.clear();
                }
            });
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
    total: usize,
    movement: usize,
}

fn metrics_system(states: Res<States>, mut metrics: ResMut<Metrics>) {
    metrics.fire = 0;
    metrics.sand = 0;
    metrics.water = 0;
    metrics.steam = 0;
    metrics.movement = 0;
    for state in states.current.values() {
        match state {
            StateId::Fire => metrics.fire += 1,
            StateId::Sand => metrics.sand += 1,
            StateId::Water => metrics.water += 1,
            StateId::Steam => metrics.steam += 1,
            StateId::Air => {}
        }
    }
    metrics.total = metrics.fire + metrics.sand + metrics.water + metrics.steam;
    metrics.movement = states.next.keys().count();
}
