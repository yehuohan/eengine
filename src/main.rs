use bevy::{
    asset::LoadState,
    core_pipeline::{
        prepass::{DeferredPrepass, DepthPrepass, MotionVectorPrepass},
        Skybox,
    },
    prelude::*,
    render::render_resource::{TextureViewDescriptor, TextureViewDimension},
};
use egd::{
    camera_controller::CameraController,
    post_processing,
    // skybox::Skybox
};
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
        .add_systems(Update, asset_loaded)
        .run();
}

#[derive(Resource)]
struct Cubemap {
    loaded: bool,
    handle: Handle<Image>,
}

fn setup(
    mut cmds: Commands,
    asset_server: Res<AssetServer>,
    // mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // let skybox_handle = asset_server.load("textures/spacebox1/top.png");
    let skybox_handle = asset_server.load("../ex.swp/bevy/assets/textures/Ryfjallet_cubemap.png");
    cmds.spawn((
        Camera3dBundle {
            camera: Camera::default(),
            transform: Transform::from_xyz(0.0, 2.0, -2.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
            ..default()
        },
        CameraController::first_person(),
        // CameraController::pan_orbit(),
        Skybox {
            image: skybox_handle.clone(),
            brightness: 1000.0,
        },
        DepthPrepass,
        MotionVectorPrepass,
        DeferredPrepass,
        post_processing::PostProcessSettings { intensity: 0.02 },
    ));
    cmds.insert_resource(Cubemap {
        loaded: false,
        handle: skybox_handle,
    });

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

fn asset_loaded(
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut cubemap: ResMut<Cubemap>,
    mut skyboxes: Query<&mut Skybox>,
) {
    if !cubemap.loaded && asset_server.load_state(&cubemap.handle) == LoadState::Loaded {
        let image = images.get_mut(&cubemap.handle).unwrap();
        // NOTE: PNGs do not have any metadata that could indicate they contain a cubemap texture,
        // so they appear as one texture. The following code reconfigures the texture as necessary.
        if image.texture_descriptor.array_layer_count() == 1 {
            image.reinterpret_stacked_2d_as_array(image.height() / image.width());
            image.texture_view_descriptor = Some(TextureViewDescriptor {
                dimension: Some(TextureViewDimension::Cube),
                ..default()
            });
        }

        for mut skybox in &mut skyboxes {
            skybox.image = cubemap.handle.clone();
        }

        cubemap.loaded = true;
    }
}
