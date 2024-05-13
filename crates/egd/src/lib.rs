//! bevy egineder libraries

use bevy::prelude::*;

pub mod camera_controller;

pub struct EgdPlugins;

impl PluginGroup for EgdPlugins {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        let mut group = bevy::app::PluginGroupBuilder::start::<Self>();
        group = group.add(camera_controller::CameraControllerPlugin);
        return group;
    }
}

pub fn exit_app(keys: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<bevy::app::AppExit>) {
    if keys.just_released(KeyCode::Escape) {
        exit.send(bevy::app::AppExit);
    }
}
