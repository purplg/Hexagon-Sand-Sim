use bevy::prelude::*;
use bevy_inspector_egui::{
    inspector_options::ReflectInspectorOptions, quick::ResourceInspectorPlugin, InspectorOptions,
};

use crate::{cell::StateId, grid::CellStates};

pub(super) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Metrics>();
        app.add_plugins(ResourceInspectorPlugin::<Metrics>::new());
        app.add_systems(Update, update_system);
    }
}

#[derive(Reflect, Resource, Default, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
struct Metrics {
    fire: usize,
    sand: usize,
    water: usize,
    steam: usize,
    total: usize,
    movement: usize,
}

fn update_system(states: Res<CellStates>, mut metrics: ResMut<Metrics>) {
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
