use bevy::{
    diagnostic,
    prelude::{App, Plugin},
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(diagnostic::LogDiagnosticsPlugin::default())
            .add_plugin(diagnostic::FrameTimeDiagnosticsPlugin::default())
            .add_plugin(diagnostic::EntityCountDiagnosticsPlugin::default())
            .add_plugin(WorldInspectorPlugin::default());
    }
}
