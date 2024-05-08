mod camera_controller;

use bevy::prelude::*;
use camera_controller::{CameraController, CameraControllerPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "egineder".into(),
                resolution: (960.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(CameraControllerPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, grab_mouse)
        .add_systems(Update, exit_app)
        .run();
}

fn exit_app(keys: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<bevy::app::AppExit>) {
    if keys.just_released(KeyCode::Escape) {
        exit.send(bevy::app::AppExit);
    }
}

fn grab_mouse(mut window: Query<&mut Window>, mouse: Res<ButtonInput<MouseButton>>, keys: Res<ButtonInput<KeyCode>>) {
    let mut window = window.single_mut();

    if mouse.just_released(MouseButton::Left) {
        window.cursor.visible = false;
        window.cursor.grab_mode = bevy::window::CursorGrabMode::Locked;
    }

    if keys.just_released(KeyCode::KeyQ) {
        window.cursor.visible = true;
        window.cursor.grab_mode = bevy::window::CursorGrabMode::None;
    }
}

fn setup(
    mut cmds: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    cmds.spawn((
        Camera3dBundle {
            camera: Camera::default(),
            transform: Transform::from_xyz(0.0, 2.0, -2.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
            ..default()
        },
        CameraController::default(),
    ));
    cmds.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        material: materials.add(Color::rgb(2.0, 0.6, 0.9)),
        ..default()
    });
    cmds.spawn(SceneBundle {
        scene: asset_server.load("scenes/Sponza/glTF/Sponza.gltf#Scene0"),
        ..default()
    });
}
