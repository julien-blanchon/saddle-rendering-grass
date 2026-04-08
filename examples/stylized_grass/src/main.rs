use grass_example_common as common;

use bevy::prelude::*;
use grass::{
    BladeShape, GrassArchetype, GrassConfig, GrassLodBand, GrassLodConfig, GrassNormalSource,
    GrassPatch, GrassPlugin,
};

/// Demonstrates different art styles achievable with the grass system:
///
/// - Left: Anime / Zelda style — single-triangle blades, bright colors, ground-normal
///   projection for flat unified shading
/// - Center: Realistic meadow — multi-segment strips with tip alpha fade
/// - Right: Stylized cross-billboard — volumetric look with soft tips
fn main() {
    App::new()
        .insert_resource(common::presets::wind::breezy(Vec2::new(0.7, 0.5)))
        .add_plugins((
            common::default_plugins("Grass Example - Art Styles"),
            bevy::diagnostic::FrameTimeDiagnosticsPlugin::default(),
            GrassPlugin::default(),
            common::GrassExampleUiPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (common::sync_overlay, common::free_flight_system))
        .run();
}

/// Single LOD band with generous distance — avoids dither issues during demos.
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

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    common::spawn_camera(
        &mut commands,
        Vec3::new(0.0, 2.5, 6.0),
        Vec3::new(0.0, 0.3, -2.0),
    );
    common::spawn_lighting(&mut commands);
    common::spawn_ground(&mut commands, &mut meshes, &mut materials, 60.0);
    common::spawn_overlay(&mut commands, "Art Styles Showcase");

    // --- Anime / Zelda style (left) ---
    common::spawn_patch(
        &mut commands,
        "Anime Style",
        Transform::from_xyz(-8.0, 0.0, -2.0),
        GrassPatch {
            half_size: Vec2::new(3.0, 3.0),
            seed: 10,
            ..default()
        },
        GrassConfig {
            density_per_square_unit: 18.0,
            lod: demo_lod(),
            archetypes: vec![common::presets::archetypes::anime()],
            ..default()
        },
    );

    // --- Realistic meadow (center) ---
    common::spawn_patch(
        &mut commands,
        "Realistic Meadow",
        Transform::from_xyz(0.0, 0.0, -2.0),
        GrassPatch {
            half_size: Vec2::new(3.0, 3.0),
            seed: 20,
            ..default()
        },
        GrassConfig {
            density_per_square_unit: 38.0,
            lod: demo_lod(),
            archetypes: vec![common::presets::archetypes::meadow()],
            ..default()
        },
    );

    // --- Cross-billboard (right) ---
    common::spawn_patch(
        &mut commands,
        "Cross Billboard",
        Transform::from_xyz(8.0, 0.0, -2.0),
        GrassPatch {
            half_size: Vec2::new(3.0, 3.0),
            seed: 30,
            ..default()
        },
        GrassConfig {
            density_per_square_unit: 24.0,
            lod: demo_lod(),
            archetypes: vec![common::presets::archetypes::cross_billboard()],
            ..default()
        },
    );

    // --- All shapes row (back) ---
    let shapes = [
        ("Strip", BladeShape::Strip),
        ("CrossBillboard", BladeShape::CrossBillboard),
        ("FlatCard", BladeShape::FlatCard),
        ("SingleTriangle", BladeShape::SingleTriangle),
    ];
    for (i, (name, shape)) in shapes.iter().enumerate() {
        common::spawn_patch(
            &mut commands,
            &format!("Shape: {name}"),
            Transform::from_xyz(-10.0 + i as f32 * 7.0, 0.0, -10.0),
            GrassPatch {
                half_size: Vec2::new(2.5, 2.5),
                seed: 100 + i as u64,
                ..default()
            },
            GrassConfig {
                density_per_square_unit: 28.0,
                lod: demo_lod(),
                archetypes: vec![GrassArchetype {
                    debug_name: name.to_string(),
                    blade_height: Vec2::new(0.5, 1.0),
                    blade_width: Vec2::new(0.04, 0.10),
                    root_color: Color::srgb(0.15, 0.35, 0.10),
                    tip_color: Color::srgb(0.42, 0.80, 0.28),
                    color_variation: 0.10,
                    blade_shape: *shape,
                    normal_source: if *shape == BladeShape::SingleTriangle {
                        GrassNormalSource::GroundNormal
                    } else {
                        GrassNormalSource::BladeFacing
                    },
                    ..default()
                }],
                ..default()
            },
        );
    }
}
