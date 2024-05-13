use bevy::{
    core_pipeline::prepass::{DeferredPrepass, DepthPrepass, MotionVectorPrepass},
    prelude::*,
};
use egd::camera_controller::CameraController;
use std::f32::consts::*;

fn main() {
    App::new()
        .insert_resource(Msaa::Off)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "egineder".into(),
                resolution: (960.0, 600.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(egd::EgdPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, egd::exit_app)
        .run();
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
        CameraController::first_person(),
        // CameraController::pan_orbit(),
        DepthPrepass,
        MotionVectorPrepass,
        DeferredPrepass,
    ));

    cmds.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 15_000.,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, 0.0, -FRAC_PI_4)),
        ..default()
    });

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
