use grass_example_common as common;

use bevy::prelude::*;
use grass::{
    GrassConfig, GrassExclusionZone, GrassLodBand, GrassLodConfig, GrassPatch, GrassPlugin,
    GrassScatterFilter, GrassSurface,
};

/// Demonstrates scatter filters: slope masking, altitude range, and exclusion zones.
///
/// - Left patch: flat ground with an exclusion zone sphere in the center
/// - Center: tilted ramp with slope filter (no grass above 35°)
/// - Right: altitude-filtered patch (grass only between Y=0 and Y=2)
fn main() {
    App::new()
        .insert_resource(common::presets::wind::breezy(Vec2::new(0.85, 0.35)))
        .add_plugins((
            common::default_plugins("Grass Example - Scatter Filters"),
            bevy::diagnostic::FrameTimeDiagnosticsPlugin::default(),
            GrassPlugin::default(),
            common::GrassExampleUiPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                common::sync_overlay,
                common::free_flight_system,
                animate_exclusion_zone,
            ),
        )
        .run();
}

/// Single generous LOD band for demo clarity.
fn demo_lod() -> GrassLodConfig {
    GrassLodConfig {
        bands: vec![GrassLodBand {
            max_distance: 120.0,
            density_scale: 1.0,
            segments: 5,
            fade_distance: 20.0,
        }],
    }
}

#[derive(Component)]
struct ExclusionMarker;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    common::spawn_camera(
        &mut commands,
        Vec3::new(-5.0, 4.0, 8.0),
        Vec3::new(-4.0, 0.3, -2.0),
    );
    common::spawn_lighting(&mut commands);
    common::spawn_ground(&mut commands, &mut meshes, &mut materials, 60.0);
    common::spawn_overlay(&mut commands, "Scatter Filters Demo");

    // --- Exclusion zone demo (left) ---
    commands.spawn((
        Name::new("Exclusion Sphere"),
        ExclusionMarker,
        Mesh3d(meshes.add(Sphere::new(2.5).mesh().ico(3).unwrap())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(0.9, 0.2, 0.2, 0.3),
            alpha_mode: AlphaMode::Blend,
            ..default()
        })),
        Transform::from_xyz(-12.0, 0.0, -2.0),
    ));

    common::spawn_patch(
        &mut commands,
        "Exclusion Zone Patch",
        Transform::from_xyz(-12.0, 0.0, -2.0),
        GrassPatch {
            half_size: Vec2::new(6.0, 5.0),
            seed: 42,
            ..default()
        },
        GrassConfig {
            density_per_square_unit: 36.0,
            lod: demo_lod(),
            archetypes: vec![common::presets::archetypes::meadow()],
            scatter_filter: GrassScatterFilter {
                exclusion_zones: vec![GrassExclusionZone {
                    center: Vec3::new(-12.0, 0.0, -2.0),
                    radius: 2.5,
                    falloff: 1.5,
                }],
                ..default()
            },
            ..default()
        },
    );

    // --- Slope filter demo (center) ---
    let ramp = commands
        .spawn((
            Name::new("Gentle Ramp"),
            Mesh3d(meshes.add(Cuboid::new(14.0, 0.4, 6.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.30, 0.26, 0.20),
                perceptual_roughness: 0.9,
                ..default()
            })),
            Transform {
                translation: Vec3::new(0.0, 1.2, -3.0),
                rotation: Quat::from_rotation_x(-0.35), // ~20° slope
                ..default()
            },
        ))
        .id();

    common::spawn_patch(
        &mut commands,
        "Slope Filtered Ramp",
        Transform::default(),
        GrassPatch {
            seed: 88,
            surface: GrassSurface::Mesh(ramp),
            ..default()
        },
        GrassConfig {
            density_per_square_unit: 28.0,
            lod: demo_lod(),
            archetypes: vec![common::presets::archetypes::turf()],
            scatter_filter: GrassScatterFilter {
                slope_range_degrees: Some((0.0, 35.0)),
                ..default()
            },
            ..default()
        },
    );

    // --- Altitude filter demo (right) — grass on flat ground, only below Y=1.5 ---
    common::spawn_patch(
        &mut commands,
        "Altitude Filtered",
        Transform::from_xyz(10.0, 0.0, -2.0),
        GrassPatch {
            half_size: Vec2::new(4.0, 5.0),
            seed: 55,
            ..default()
        },
        GrassConfig {
            density_per_square_unit: 32.0,
            lod: demo_lod(),
            archetypes: vec![common::presets::archetypes::meadow()],
            scatter_filter: GrassScatterFilter {
                altitude_range: Some((-1.0, 1.5)),
                ..default()
            },
            ..default()
        },
    );
}

fn animate_exclusion_zone(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<ExclusionMarker>>,
) {
    for mut transform in &mut query {
        let t = time.elapsed_secs();
        transform.translation.x = -12.0 + (t * 0.4).sin() * 3.0;
        transform.translation.z = -2.0 + (t * 0.3).cos() * 2.0;
    }
}
