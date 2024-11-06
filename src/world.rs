use crate::{DEFAULT_RENDER_LAYER, VIEW_MODEL_RENDER_LAYER};
use bevy::{color::palettes::tailwind, prelude::*, render::view::RenderLayers};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // let floor = meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(10.0)));
    let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let material = materials.add(Color::WHITE);

    // The world model camera will render the floor and the cubes spawned in this system.
    // Assigning no `RenderLayers` component defaults to layer 0.

    for x in 0..16 {
        for y in 0..16 {
            for z in 0..16 {
                commands.spawn(MaterialMeshBundle {
                    mesh: cube.clone(),
                    material: material.clone(),
                    transform: Transform::from_xyz(
                        (x as f32) * 1.0,
                        (y as f32) * 1.0,
                        (z as f32) * 1.0,
                    ),
                    ..default()
                });
            }
        }
    }

    // commands.spawn(MaterialMeshBundle {
    //     mesh: floor,
    //     material: material.clone(),
    //     ..default()
    // });
    //
    // commands.spawn(MaterialMeshBundle {
    //     mesh: cube.clone(),
    //     material: material.clone(),
    //     transform: Transform::from_xyz(0.0, 0.25, -3.0),
    //     ..default()
    // });
    //
    // commands.spawn(MaterialMeshBundle {
    //     mesh: cube,
    //     material,
    //     transform: Transform::from_xyz(0.75, 1.75, 0.0),
    //     ..default()
    // });

    commands.spawn((
        PointLightBundle {
            point_light: PointLight {
                color: Color::from(tailwind::ROSE_300),
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(-2.0, 4.0, -0.75),
            ..default()
        },
        // The light source illuminates both the world model and the view model.
        RenderLayers::from_layers(&[DEFAULT_RENDER_LAYER, VIEW_MODEL_RENDER_LAYER]),
    ));
}
