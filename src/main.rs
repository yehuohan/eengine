use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "eEngine".into(),
                resolution: (960.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        .run();
}
