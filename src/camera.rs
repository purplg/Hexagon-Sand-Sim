use bevy::prelude::*;
use bevy_pancam::{PanCam, PanCamPlugin};

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::rgb(0.01, 0.01, 0.01)));
        app.add_plugins(PanCamPlugin::default());
        app.add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default()).insert(PanCam {
        grab_buttons: vec![MouseButton::Right],
        enabled: true,
        zoom_to_cursor: true,
        ..default()
    });
}
