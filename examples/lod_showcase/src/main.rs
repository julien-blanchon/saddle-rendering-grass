use grass_example_common as common;

use bevy::prelude::*;
use grass::{GrassConfig, GrassPatch, GrassPlugin};

#[derive(Component)]
struct DollyCamera;

fn main() {
    App::new()
        .add_plugins((
            common::default_plugins("Grass Example - LOD"),
            bevy::diagnostic::FrameTimeDiagnosticsPlugin::default(),
            GrassPlugin::default(),
            common::GrassExampleUiPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (animate_camera, common::sync_overlay))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    commands.spawn((
        Name::new("Dolly Camera"),
        DollyCamera,
        Camera3d::default(),
        Transform::from_xyz(0.0, 8.0, 32.0).looking_at(Vec3::new(0.0, 0.0, -32.0), Vec3::Y),
    ));
    common::spawn_lighting(&mut commands);
    common::spawn_ground(&mut commands, &mut meshes, &mut materials, 180.0);
    common::spawn_overlay(&mut commands, "Grass LOD Showcase");

    let density_map = common::checker_density_map(&mut images, 128);
    common::spawn_patch(
        &mut commands,
        "Open Meadow Patch",
        Transform::from_xyz(0.0, 0.0, -40.0),
        GrassPatch {
            half_size: Vec2::new(28.0, 46.0),
            density_scale: 1.0,
            seed: 7,
            ..default()
        },
        GrassConfig {
            density_per_square_unit: 34.0,
            density_map: Some(grass::GrassDensityMap {
                image: density_map,
                ..default()
            }),
            archetypes: vec![common::meadow_archetype(), common::flower_archetype()],
            ..default()
        },
    );
}

fn animate_camera(time: Res<Time>, mut camera: Single<&mut Transform, With<DollyCamera>>) {
    let z = 38.0 - 52.0 * (time.elapsed_secs() * 0.14).sin().abs();
    let x = (time.elapsed_secs() * 0.18).sin() * 7.0;
    camera.translation = Vec3::new(x, 8.0, z);
    **camera = camera.looking_at(Vec3::new(0.0, 0.0, -42.0), Vec3::Y);
}
