use grass_example_common as common;

use bevy::prelude::*;
use grass::{GrassConfig, GrassLodBand, GrassLodConfig, GrassPatch, GrassPlugin};

#[derive(Component)]
struct OrbitCamera;

fn main() {
    App::new()
        .insert_resource(common::presets::wind::breezy(Vec2::new(0.88, 0.32)))
        .add_plugins((
            common::default_plugins("Grass Example - Stress Field"),
            bevy::diagnostic::FrameTimeDiagnosticsPlugin::default(),
            GrassPlugin::default(),
            common::GrassExampleUiPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                orbit_camera,
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
) {
    commands.spawn((
        Name::new("Orbit Camera"),
        OrbitCamera,
        Camera3d::default(),
        common::FreeFlight::default(),
        Transform::from_xyz(0.0, 14.0, 30.0).looking_at(Vec3::new(0.0, 0.0, -10.0), Vec3::Y),
    ));
    common::spawn_lighting(&mut commands);
    common::spawn_ground(&mut commands, &mut meshes, &mut materials, 200.0);
    common::spawn_overlay(&mut commands, "Grass Stress Field");

    // Aggressive LOD to keep the stress test playable.
    let stress_lod = GrassLodConfig {
        bands: vec![
            GrassLodBand {
                max_distance: 14.0,
                density_scale: 1.0,
                segments: 3,
                fade_distance: 3.0,
            },
            GrassLodBand {
                max_distance: 35.0,
                density_scale: 0.3,
                segments: 2,
                fade_distance: 6.0,
            },
            GrassLodBand {
                max_distance: 60.0,
                density_scale: 0.08,
                segments: 2,
                fade_distance: 10.0,
            },
        ],
    };

    for z in 0..3 {
        for x in 0..3 {
            let offset = Vec3::new((x as f32 - 1.0) * 22.0, 0.0, -10.0 - z as f32 * 22.0);
            common::spawn_patch(
                &mut commands,
                &format!("Stress Patch {x}-{z}"),
                Transform::from_translation(offset),
                GrassPatch {
                    half_size: Vec2::new(9.0, 9.0),
                    density_scale: 1.0,
                    seed: (x + z * 3 + 1) as u64,
                    chunking: grass::GrassChunking {
                        chunk_size: Vec2::new(9.0, 9.0),
                    },
                    ..default()
                },
                GrassConfig {
                    density_per_square_unit: 28.0,
                    max_blades_per_chunk: 1_000,
                    lod: stress_lod.clone(),
                    archetypes: vec![common::presets::archetypes::meadow()],
                    ..default()
                },
            );
        }
    }
}

fn orbit_camera(time: Res<Time>, mut camera: Single<&mut Transform, With<OrbitCamera>>) {
    let angle = time.elapsed_secs() * 0.16;
    let radius = 36.0;
    camera.translation = Vec3::new(angle.cos() * radius, 14.0, angle.sin() * radius - 16.0);
    **camera = camera.looking_at(Vec3::new(0.0, 0.0, -18.0), Vec3::Y);
}
