//! A simple camera controller for bevy
//!
//! Borrowed from [bevy_basic_camera](https://github.com/DGriffin91/bevy_basic_camera)

use bevy::{
    input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
    prelude::*,
};

/// Provides basic movement functionality to the attached camera
#[derive(Component, Clone)]
pub struct CameraController {
    pub initialized: bool,
    // Movement keys
    pub key_forward: KeyCode,
    pub key_back: KeyCode,
    pub key_left: KeyCode,
    pub key_right: KeyCode,
    pub key_up: KeyCode,
    pub key_down: KeyCode,
    pub key_run: KeyCode,
    // Params
    pub sensitivity: f32,
    pub friction: f32,
    pub speed: Vec3,
    pub walk_speed: f32,
    pub run_speed: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub lock_y: bool,
    // Params for orbit mode
    pub orbit_mode: bool,
    pub orbit_focus: Vec3,
    pub scroll_speed: f32,
    pub mouse_key_pan: MouseButton,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            initialized: false,
            key_forward: KeyCode::KeyE,
            key_back: KeyCode::KeyD,
            key_left: KeyCode::KeyS,
            key_right: KeyCode::KeyF,
            key_up: KeyCode::KeyR,
            key_down: KeyCode::KeyW,
            key_run: KeyCode::ShiftLeft,
            sensitivity: 0.25,
            friction: 0.5,
            speed: Vec3::ZERO,
            walk_speed: 3.0,
            run_speed: 15.0,
            pitch: 0.0,
            yaw: 0.0,
            lock_y: false,
            orbit_mode: false,
            orbit_focus: Vec3::ZERO,
            scroll_speed: 0.1,
            mouse_key_pan: MouseButton::Left,
        }
    }
}

pub fn update_camera_controller(
    time: Res<Time>,
    mut mouse_motion_evr: EventReader<MouseMotion>,
    mut mouse_wheel_evr: EventReader<MouseWheel>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    key_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut CameraController), With<Camera>>,
) {
    let dt = time.delta_seconds();

    if let Ok((mut transform, mut ctrl)) = query.get_single_mut() {
        if !ctrl.initialized {
            let (_roll, yaw, pitch) = transform.rotation.to_euler(EulerRot::ZYX);
            ctrl.yaw = yaw;
            ctrl.pitch = pitch;
            ctrl.initialized = true;
        }

        // Handle scroll input
        let mut scroll_distance = 0.0;
        for ev in mouse_wheel_evr.read() {
            match ev.unit {
                MouseScrollUnit::Line => {
                    scroll_distance = ev.y;
                }
                MouseScrollUnit::Pixel => (),
            }
        }

        // Handle key input
        let mut axis_input = Vec3::ZERO;
        if key_input.pressed(ctrl.key_forward) {
            axis_input.z += 1.0;
        }
        if key_input.pressed(ctrl.key_back) {
            axis_input.z -= 1.0;
        }
        if key_input.pressed(ctrl.key_right) {
            axis_input.x += 1.0;
        }
        if key_input.pressed(ctrl.key_left) {
            axis_input.x -= 1.0;
        }
        if key_input.pressed(ctrl.key_up) {
            axis_input.y += 1.0;
        }
        if key_input.pressed(ctrl.key_down) {
            axis_input.y -= 1.0;
        }

        // Compute speed
        if axis_input != Vec3::ZERO {
            let speed = if key_input.pressed(ctrl.key_run) {
                ctrl.run_speed
            } else {
                ctrl.walk_speed
            };
            ctrl.speed = axis_input.normalize() * speed;
        } else {
            let friction = ctrl.friction.clamp(0.0, 1.0);
            ctrl.speed *= 1.0 - friction;
            if ctrl.speed.length_squared() < 1e-6 {
                ctrl.speed = Vec3::ZERO;
            }
        }

        // Apply movement
        let forward = *transform.forward();
        let right = *transform.right();
        let mut translation_delta = ctrl.speed.x * dt * right + ctrl.speed.y * dt * Vec3::Y + ctrl.speed.z * dt * forward;
        let mut translation_scroll = Vec3::ZERO;
        if ctrl.orbit_mode && ctrl.scroll_speed > 0.0 {
            translation_scroll = scroll_distance * transform.translation.distance(ctrl.orbit_focus) * ctrl.scroll_speed * forward;
        }
        if ctrl.lock_y {
            translation_delta *= Vec3::new(1.0, 0.0, 1.0);
        }
        transform.translation += translation_delta + translation_scroll;
        ctrl.orbit_focus += translation_delta;

        // Handle mouse input
        let mut mouse_delta = Vec2::ZERO;
        if !ctrl.orbit_mode || (ctrl.orbit_mode && mouse_button_input.pressed(ctrl.mouse_key_pan)) {
            for motion in mouse_motion_evr.read() {
                mouse_delta += motion.delta;
            }
        } else {
            mouse_motion_evr.clear();
        }

        // Apply lookat
        if mouse_delta != Vec2::ZERO {
            let pitch = (ctrl.pitch - mouse_delta.y * 0.5 * ctrl.sensitivity * dt)
                .clamp(-0.99 * std::f32::consts::FRAC_PI_2, 0.99 * std::f32::consts::FRAC_PI_2);
            let yaw = ctrl.yaw - mouse_delta.x * ctrl.sensitivity * dt;

            transform.rotation = Quat::from_euler(EulerRot::ZYX, 0.0, yaw, pitch);
            ctrl.pitch = pitch;
            ctrl.yaw = yaw;

            if ctrl.orbit_mode {
                let rot = Mat3::from_quat(transform.rotation).mul_vec3(Vec3::new(
                    0.0,
                    0.0,
                    ctrl.orbit_focus.distance(transform.translation),
                ));
                transform.translation = ctrl.orbit_focus + rot;
            }
        }
    }
}

/// In order to function, the [`CameraController`] component should be attached to the camera entity.
#[derive(Default)]
pub struct CameraControllerPlugin;

impl Plugin for CameraControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_camera_controller);
    }
}
