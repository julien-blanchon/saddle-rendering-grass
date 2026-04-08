use grass_example_common as common;

use bevy::prelude::*;
use grass::{GrassConfig, GrassLodBand, GrassLodConfig, GrassPatch, GrassPlugin};

#[derive(Component)]
struct DollyCamera;

fn main() {
    App::new()
        .insert_resource(common::presets::wind::breezy(Vec2::new(0.9, 0.25)))
        .add_plugins((
            common::default_plugins("Grass Example - LOD"),
            bevy::diagnostic::FrameTimeDiagnosticsPlugin::default(),
            GrassPlugin::default(),
            common::GrassExampleUiPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                animate_camera,
                common::sync_overlay,
                common::free_flight_system,
            ),
        )
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
        common::FreeFlight::default(),
        Transform::from_xyz(0.0, 6.0, 20.0).looking_at(Vec3::new(0.0, 0.0, -20.0), Vec3::Y),
    ));
    common::spawn_lighting(&mut commands);
    common::spawn_ground(&mut commands, &mut meshes, &mut materials, 160.0);
    common::spawn_overlay(&mut commands, "Grass LOD Showcase");

    let density_map = common::checker_density_map(&mut images, 128);

    // Tuned for performance: reduced area and density, aggressive LOD falloff.
    // The showcase is about demonstrating LOD transitions, not raw blade count.
    common::spawn_patch(
        &mut commands,
        "Open Meadow Patch",
        Transform::from_xyz(0.0, 0.0, -24.0),
        GrassPatch {
            half_size: Vec2::new(20.0, 30.0),
            density_scale: 1.0,
            seed: 7,
            chunking: grass::GrassChunking {
                chunk_size: Vec2::new(10.0, 10.0),
            },
            ..default()
        },
        GrassConfig {
            density_per_square_unit: 22.0,
            max_blades_per_chunk: 1_200,
            density_map: Some(grass::GrassDensityMap {
                image: density_map,
                ..default()
            }),
            lod: GrassLodConfig {
                bands: vec![
                    GrassLodBand {
                        max_distance: 16.0,
                        density_scale: 1.0,
                        segments: 4,
                        fade_distance: 3.0,
                    },
                    GrassLodBand {
                        max_distance: 40.0,
                        density_scale: 0.35,
                        segments: 2,
                        fade_distance: 6.0,
                    },
                    GrassLodBand {
                        max_distance: 70.0,
                        density_scale: 0.10,
                        segments: 2,
                        fade_distance: 10.0,
                    },
                ],
            },
            archetypes: vec![common::presets::archetypes::meadow()],
            ..default()
        },
    );
}

fn animate_camera(time: Res<Time>, mut camera: Single<&mut Transform, With<DollyCamera>>) {
    let z = 24.0 - 38.0 * (time.elapsed_secs() * 0.14).sin().abs();
    let x = (time.elapsed_secs() * 0.18).sin() * 5.0;
    camera.translation = Vec3::new(x, 6.0, z);
    **camera = camera.looking_at(Vec3::new(0.0, 0.0, -28.0), Vec3::Y);
}
