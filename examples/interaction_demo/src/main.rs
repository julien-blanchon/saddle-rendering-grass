use grass_example_common as common;

use bevy::prelude::*;
use grass::{
    GrassConfig, GrassInteractionActor, GrassInteractionMap, GrassInteractionPolicy,
    GrassInteractionZone, GrassLodBand, GrassLodConfig, GrassPatch, GrassPlugin,
};

/// Demonstrates the grass interaction system:
///
/// - Rolling ball: bend + flatten with persistent trails that recover
/// - Orbiting sphere: bend-only (softer, faster recovery)
/// - Static hide zone: permanently hidden grass patch
/// - Legacy interaction zone: old-style radial zone for comparison
///
/// The interaction map (CPU texture) is stamped each frame by actors,
/// then sampled by the vertex shader. No zone limit — unlimited actors.
fn main() {
    App::new()
        .insert_resource(common::presets::wind::calm(Vec2::new(0.8, 0.3)))
        .insert_resource(GrassInteractionMap {
            half_extent: 25.0,
            resolution: 256,
            recovery_speed: 1.5,
            follow_camera: true,
            ..default()
        })
        .add_plugins((
            common::default_plugins("Grass Example - Interaction Demo"),
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
                move_rolling_ball,
                orbit_sphere,
            ),
        )
        .run();
}

fn demo_lod() -> GrassLodConfig {
    GrassLodConfig {
        bands: vec![GrassLodBand {
            max_distance: 100.0,
            density_scale: 1.0,
            segments: 5,
            fade_distance: 15.0,
        }],
    }
}

#[derive(Component)]
struct RollingBall;

#[derive(Component)]
struct OrbitSphere;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    common::spawn_camera(
        &mut commands,
        Vec3::new(0.0, 6.0, 14.0),
        Vec3::new(0.0, 0.3, -2.0),
    );
    common::spawn_lighting(&mut commands);
    common::spawn_ground(&mut commands, &mut meshes, &mut materials, 60.0);
    common::spawn_overlay(&mut commands, "Interaction Demo");

    // Large grass field
    common::spawn_patch(
        &mut commands,
        "Interaction Field",
        Transform::from_xyz(0.0, 0.0, -2.0),
        GrassPatch {
            half_size: Vec2::new(15.0, 12.0),
            seed: 42,
            ..default()
        },
        GrassConfig {
            density_per_square_unit: 36.0,
            lod: demo_lod(),
            archetypes: vec![common::presets::archetypes::meadow()],
            ..default()
        },
    );

    // --- Rolling ball (bend + flatten, leaves trail) ---
    commands.spawn((
        Name::new("Rolling Ball"),
        RollingBall,
        GrassInteractionActor {
            radius: 1.8,
            policy: GrassInteractionPolicy::BendAndFlatten {
                bend_strength: 0.8,
                flatten_strength: 0.5,
            },
            falloff: 1.5,
        },
        Mesh3d(meshes.add(Sphere::new(0.6).mesh().uv(24, 18))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.85, 0.55, 0.2),
            metallic: 0.3,
            perceptual_roughness: 0.4,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.6, -2.0),
    ));

    // --- Orbiting sphere (bend only, faster recovery) ---
    commands.spawn((
        Name::new("Orbit Sphere"),
        OrbitSphere,
        GrassInteractionActor {
            radius: 1.2,
            policy: GrassInteractionPolicy::Bend { strength: 0.9 },
            falloff: 2.0,
        },
        Mesh3d(meshes.add(Sphere::new(0.35).mesh().uv(16, 12))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.6, 0.9),
            metallic: 0.6,
            perceptual_roughness: 0.3,
            ..default()
        })),
        Transform::from_xyz(6.0, 0.35, -2.0),
    ));

    // --- Static hide zone (permanent cut) ---
    commands.spawn((
        Name::new("Hide Zone"),
        GrassInteractionActor {
            radius: 2.0,
            policy: GrassInteractionPolicy::Hide { permanent: true },
            falloff: 3.0,
        },
        Mesh3d(meshes.add(Cylinder::new(2.0, 0.05))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(0.8, 0.2, 0.15, 0.25),
            alpha_mode: AlphaMode::Blend,
            ..default()
        })),
        Transform::from_xyz(-6.0, 0.02, -4.0),
    ));

    // --- Legacy interaction zone (for comparison / backward compat) ---
    commands.spawn((
        Name::new("Legacy Zone"),
        GrassInteractionZone {
            radius: 1.5,
            bend_strength: 0.6,
            flatten_strength: 0.3,
            falloff: 1.8,
        },
        Mesh3d(meshes.add(Sphere::new(0.3).mesh().uv(12, 8))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.9, 0.85, 0.3),
            ..default()
        })),
        Transform::from_xyz(8.0, 0.3, -8.0),
    ));
}

fn move_rolling_ball(time: Res<Time>, mut query: Query<&mut Transform, With<RollingBall>>) {
    let t = time.elapsed_secs();
    for mut transform in &mut query {
        // Figure-8 path across the grass field
        let x = (t * 0.5).sin() * 8.0;
        let z = -2.0 + (t * 1.0).sin() * 6.0;
        transform.translation = Vec3::new(x, 0.6, z);
        // Rotate ball for visual rolling effect
        transform.rotation = Quat::from_rotation_z(-t * 2.0) * Quat::from_rotation_x(t * 1.5);
    }
}

fn orbit_sphere(time: Res<Time>, mut query: Query<&mut Transform, With<OrbitSphere>>) {
    let t = time.elapsed_secs();
    for mut transform in &mut query {
        let x = (t * 0.8).cos() * 5.0;
        let z = -2.0 + (t * 0.8).sin() * 5.0;
        transform.translation = Vec3::new(x, 0.35, z);
    }
}
