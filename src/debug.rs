use bevy::{
    diagnostic::{self, Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(diagnostic::FrameTimeDiagnosticsPlugin)
            .add_plugin(diagnostic::EntityCountDiagnosticsPlugin)
            .add_plugin(WorldInspectorPlugin::default())
            .add_startup_system(infotext_system)
            .add_system(change_text_system);
    }
}

#[derive(Component)]
struct TextChanges;

fn infotext_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");

    commands.spawn((
        TextBundle::from_sections([
            TextSection::from_style(TextStyle {
                font: font.clone(),
                font_size: 30.0,
                color: Color::YELLOW,
            }),
            TextSection::from_style(TextStyle {
                font: font.clone(),
                font_size: 30.0,
                color: Color::BLUE,
            }),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,

            position: UiRect {
                top: Val::Px(5.0),
                right: Val::Px(15.0),
                ..default()
            },
            ..default()
        }),
        TextChanges,
    ));
}

fn change_text_system(
    time: Res<Time>,
    diagnostics: Res<Diagnostics>,
    mut query: Query<&mut Text, With<TextChanges>>,
) {
    for mut text in &mut query {
        let mut fps = 0.0;
        if let Some(fps_diagnostic) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(fps_smoothed) = fps_diagnostic.smoothed() {
                fps = fps_smoothed;
            }
        }

        let mut frame_time = time.delta_seconds_f64();
        if let Some(frame_time_diagnostic) = diagnostics.get(FrameTimeDiagnosticsPlugin::FRAME_TIME)
        {
            if let Some(frame_time_smoothed) = frame_time_diagnostic.smoothed() {
                frame_time = frame_time_smoothed;
            }
        }

        text.sections[0].value = format!("FPS: {fps:.1} - ");

        text.sections[1].value = format!("{frame_time:.3} ms/frame");
    }
}
