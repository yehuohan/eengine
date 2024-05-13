//! A simple camera controller for bevy
//!
//! - Add the [`CameraControllerPlugin`] to App
//! - Attach the [`CameraController`] component to an entity with a [`Camera3dBundle`]
//!
//! Borrowed from [camera_controller](https://github.com/bevyengine/bevy/examples/helpers/camera_controller.rs) and [bevy_basic_camera](https://github.com/DGriffin91/bevy_basic_camera)

use bevy::{
    input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
    math::mat3,
    prelude::*,
    window::CursorGrabMode,
};
use std::f32::consts::FRAC_PI_2;

/// Camera controller keys
#[derive(Clone)]
pub struct CameraControllerKeys {
    pub forward: KeyCode,
    pub back: KeyCode,
    pub left: KeyCode,
    pub right: KeyCode,
    pub up: KeyCode,
    pub down: KeyCode,
    pub run: KeyCode,
}

/// Camera controller that supports first person and orbit camera
#[derive(Component, Clone)]
pub struct CameraController {
    pub ready: bool,
    pub keys: CameraControllerKeys,
    // Params
    pub sensitivity: f32,
    pub friction: f32,
    pub speed: Vec3,
    pub walk_speed: f32,
    pub run_speed: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub lock_y: bool,
    // Params for orbit camera
    pub orbit: bool,
    pub orbit_focus: Vec3,
    pub scroll_factor: f32,
    pub mouse_key_pan: MouseButton,
}

impl CameraController {
    pub fn first_person() -> Self {
        CameraController::default()
    }

    pub fn pan_orbit() -> Self {
        CameraController {
            orbit: true,
            ..default()
        }
    }
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            ready: false,
            keys: CameraControllerKeys {
                forward: KeyCode::KeyE,
                back: KeyCode::KeyD,
                left: KeyCode::KeyS,
                right: KeyCode::KeyF,
                up: KeyCode::KeyR,
                down: KeyCode::KeyW,
                run: KeyCode::ShiftLeft,
            },
            sensitivity: 0.25,
            friction: 0.5,
            speed: Vec3::ZERO,
            walk_speed: 3.0,
            run_speed: 15.0,
            pitch: 0.0,
            yaw: 0.0,
            lock_y: false,
            orbit: false,
            orbit_focus: Vec3::ZERO,
            scroll_factor: 0.1,
            mouse_key_pan: MouseButton::Left,
        }
    }
}

impl CameraController {
    /// Calculate [`CameraController::speed`] according to [`CameraControllerKeys`]
    fn calc_speed(&mut self, key_input: &Res<ButtonInput<KeyCode>>) {
        let mut axis_input = Vec3::ZERO;
        if key_input.pressed(self.keys.forward) {
            axis_input.z += 1.0;
        }
        if key_input.pressed(self.keys.back) {
            axis_input.z -= 1.0;
        }
        if key_input.pressed(self.keys.right) {
            axis_input.x += 1.0;
        }
        if key_input.pressed(self.keys.left) {
            axis_input.x -= 1.0;
        }
        if key_input.pressed(self.keys.up) {
            axis_input.y += 1.0;
        }
        if key_input.pressed(self.keys.down) {
            axis_input.y -= 1.0;
        }

        if axis_input != Vec3::ZERO {
            let speed = if key_input.pressed(self.keys.run) {
                self.run_speed
            } else {
                self.walk_speed
            };
            self.speed = axis_input.normalize() * speed;
        } else {
            self.speed *= 1.0 - self.friction;
            if self.speed.length_squared() < 1e-6 {
                self.speed = Vec3::ZERO;
            }
        }
    }

    /// Update camera location with translation
    fn update_location(&mut self, delta_time: f32, transform: &mut Transform, mouse_wheel_evr: &mut EventReader<MouseWheel>) {
        let forward = *transform.forward();
        let right = *transform.right();

        // Camera translation by CameraControllerKeys
        let mut translation_delta = mat3(right, Vec3::Y, forward) * self.speed * delta_time;
        if self.lock_y {
            translation_delta *= Vec3::new(1.0, 0.0, 1.0);
        }

        // Camera translation by mouse wheel scroll
        let mut translation_scroll = Vec3::ZERO;
        if self.orbit && self.scroll_factor > 0.0 {
            let mut scroll_distance = 0.0;
            for ev in mouse_wheel_evr.read() {
                let amount = match ev.unit {
                    MouseScrollUnit::Line => ev.y,
                    MouseScrollUnit::Pixel => ev.y / 16.0,
                };
                scroll_distance += amount;
            }
            translation_scroll = scroll_distance * transform.translation.distance(self.orbit_focus) * self.scroll_factor * forward;
        }

        transform.translation += translation_delta + translation_scroll;
        self.orbit_focus += translation_delta;
    }

    /// Update camera rotation
    fn update_rotation(
        &mut self,
        delta_time: f32,
        transform: &mut Transform,
        mouse_motion_evr: &mut EventReader<MouseMotion>,
        mouse_button_input: &Res<ButtonInput<MouseButton>>,
    ) {
        let mut mouse_delta = Vec2::ZERO;
        if !self.orbit || (self.orbit && mouse_button_input.pressed(self.mouse_key_pan)) {
            for motion in mouse_motion_evr.read() {
                mouse_delta += motion.delta;
            }
        } else {
            mouse_motion_evr.clear();
        }

        if mouse_delta != Vec2::ZERO {
            const MAX_PITCH: f32 = 0.99 * FRAC_PI_2;
            let delta = self.sensitivity * delta_time;
            let pitch = (self.pitch - mouse_delta.y * delta).clamp(-MAX_PITCH, MAX_PITCH);
            let yaw = self.yaw - mouse_delta.x * delta;
            transform.rotation = Quat::from_euler(EulerRot::ZYX, 0.0, yaw, pitch);
            self.pitch = pitch;
            self.yaw = yaw;

            if self.orbit {
                let rot = Mat3::from_quat(transform.rotation).mul_vec3(Vec3::new(
                    0.0,
                    0.0,
                    self.orbit_focus.distance(transform.translation),
                ));
                transform.translation = self.orbit_focus + rot;
            }
        }
    }
}

/// Update camera controller
///
/// - Update camera location according to [`CameraControllerKeys`] and mouse wheel events
/// - Update camera rotation according to mouse motion events
pub fn update_camera_controller(
    time: Res<Time>,
    mut window: Query<&mut Window>,
    mut mouse_motion_evr: EventReader<MouseMotion>,
    mut mouse_wheel_evr: EventReader<MouseWheel>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    key_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut CameraController), With<Camera>>,
) {
    if let Ok((mut transform, mut ctrl)) = query.get_single_mut() {
        // Orbit camera
        if !ctrl.ready && ctrl.orbit {
            let (last_yaw, last_pitch, _last_roll) = transform.rotation.to_euler(EulerRot::YXZ);
            ctrl.yaw = last_yaw;
            ctrl.pitch = last_pitch;
            ctrl.ready = true;
        }

        // First person camera
        let mut window = window.single_mut();
        if !ctrl.ready {
            if mouse_button_input.just_pressed(MouseButton::Left) {
                window.cursor.visible = false;
                window.cursor.grab_mode = CursorGrabMode::Locked;

                let (last_yaw, last_pitch, _last_roll) = transform.rotation.to_euler(EulerRot::YXZ);
                ctrl.yaw = last_yaw;
                ctrl.pitch = last_pitch;
                ctrl.ready = true;
            }
        } else {
            if key_input.just_pressed(KeyCode::KeyQ) {
                window.cursor.visible = true;
                window.cursor.grab_mode = CursorGrabMode::None;
                ctrl.ready = false;
            }
        }

        // Check ready for orbit or first person camera
        if !ctrl.ready {
            return;
        }

        let dt = time.delta_seconds();
        ctrl.calc_speed(&key_input);
        ctrl.update_location(dt, &mut transform, &mut mouse_wheel_evr);
        ctrl.update_rotation(dt, &mut transform, &mut mouse_motion_evr, &mouse_button_input);
    }
}

#[derive(Default)]
pub struct CameraControllerPlugin;

impl Plugin for CameraControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_camera_controller);
    }
}
