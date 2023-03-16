use bevy::prelude::*;

use self::{
    components::Player,
    resources::{MouseState, MovementSettings},
    systems::{
        camera::{cursor_grab, initial_grab_cursor, player_look},
        movement::player_move,
    },
};

pub mod components;
mod resources;
mod systems;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MouseState>()
            .init_resource::<MovementSettings>()
            .add_startup_system(setup_player)
            .add_startup_system(initial_grab_cursor)
            .add_system(player_move)
            .add_system(player_look)
            .add_system(cursor_grab);
    }
}

fn setup_player(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 25.0, 0.0),
            ..Default::default()
        },
        Player,
    ));
}
