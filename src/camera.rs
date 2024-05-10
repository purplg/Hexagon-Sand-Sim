use bevy::{
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};
use leafwing_input_manager::prelude::*;

use crate::input::Input;

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::rgb(0.01, 0.01, 0.01)));
        app.add_systems(Startup, setup);
        app.add_systems(
            Update,
            (zoom, pan).run_if(|window: Query<&Window, With<PrimaryWindow>>| {
                window.single().cursor.grab_mode == CursorGrabMode::Confined
            }),
        );
        app.add_systems(Update, cursor_grab);
    }
}
fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn cursor_grab(
    mut window: Query<&mut Window, With<PrimaryWindow>>,
    input: Query<&ActionState<Input>>,
) {
    let mut window = window.single_mut();

    let Ok(state) = input.get_single() else {
        return;
    };

    if state.just_pressed(&Input::Grab) {
        window.cursor.grab_mode = CursorGrabMode::Confined;
        window.cursor.visible = false;
    } else if state.just_released(&Input::Grab) {
        window.cursor.grab_mode = CursorGrabMode::None;
        window.cursor.visible = true;
    }
}

fn zoom(mut query: Query<&mut OrthographicProjection>, input: Query<&ActionState<Input>>) {
    let mut camera = query.single_mut();
    let input = input.single();
    let zoom = input.value(&Input::Zoom);
    camera.scale += zoom;
}

fn pan(
    mut query: Query<&mut Transform, With<Camera>>,
    input: Query<&ActionState<Input>>,
    dt: Res<Time>,
) {
    let mut transform = query.single_mut();
    let input = input.single();
    let Some(pan) = input.axis_pair(&Input::Pan) else {
        return;
    };

    transform.translation.x -= pan.x() * dt.delta_seconds() * 100.;
    transform.translation.y += pan.y() * dt.delta_seconds() * 100.;
}
