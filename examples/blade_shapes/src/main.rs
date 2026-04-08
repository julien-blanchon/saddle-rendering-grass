use grass_example_common as common;

use bevy::prelude::*;
use grass::{
    BladeShape, GrassArchetype, GrassConfig, GrassLodBand, GrassLodConfig, GrassPatch, GrassPlugin,
};

/// Demonstrates all four blade shapes side by side with variable LOD band count.
///
/// Front row: one patch per shape (Strip, CrossBillboard, FlatCard, SingleTriangle)
/// Back row: same shapes with only 2 LOD bands (near + far) to show variable LOD
fn main() {
    App::new()
        .insert_resource(common::presets::wind::windy(Vec2::new(0.6, 0.8)))
        .add_plugins((
            common::default_plugins("Grass Example - Blade Shapes & Variable LOD"),
            bevy::diagnostic::FrameTimeDiagnosticsPlugin::default(),
            GrassPlugin::default(),
            common::GrassExampleUiPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (common::sync_overlay, common::free_flight_system))
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

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    common::spawn_camera(
        &mut commands,
        Vec3::new(0.0, 3.0, 8.0),
        Vec3::new(0.0, 0.3, 0.0),
    );
    common::spawn_lighting(&mut commands);
    common::spawn_ground(&mut commands, &mut meshes, &mut materials, 70.0);
    common::spawn_overlay(&mut commands, "Blade Shapes & Variable LOD");

    let shapes = [
        ("Strip (default)", BladeShape::Strip),
        ("Cross Billboard", BladeShape::CrossBillboard),
        ("Flat Card", BladeShape::FlatCard),
        ("Single Triangle", BladeShape::SingleTriangle),
    ];

    // Front row: single generous LOD band
    for (i, (name, shape)) in shapes.iter().enumerate() {
        common::spawn_patch(
            &mut commands,
            name,
            Transform::from_xyz(-10.0 + i as f32 * 7.0, 0.0, 0.0),
            GrassPatch {
                half_size: Vec2::new(2.5, 2.5),
                seed: 200 + i as u64,
                ..default()
            },
            GrassConfig {
                density_per_square_unit: 30.0,
                lod: demo_lod(),
                archetypes: vec![GrassArchetype {
                    debug_name: name.to_string(),
                    blade_height: Vec2::new(0.5, 1.0),
                    blade_width: Vec2::new(0.04, 0.09),
                    root_color: Color::srgb(0.14, 0.32, 0.10),
                    tip_color: Color::srgb(0.42, 0.78, 0.28),
                    color_variation: 0.12,
                    blade_shape: *shape,
                    ..default()
                }],
                ..default()
            },
        );
    }

    // Back row: 2-band LOD (demonstrates variable band count)
    let two_band_lod = GrassLodConfig {
        bands: vec![
            GrassLodBand {
                max_distance: 25.0,
                density_scale: 1.0,
                segments: 5,
                fade_distance: 5.0,
            },
            GrassLodBand {
                max_distance: 80.0,
                density_scale: 0.25,
                segments: 2,
                fade_distance: 10.0,
            },
        ],
    };

    for (i, (name, shape)) in shapes.iter().enumerate() {
        common::spawn_patch(
            &mut commands,
            &format!("{name} (2 LOD)"),
            Transform::from_xyz(-10.0 + i as f32 * 7.0, 0.0, -8.0),
            GrassPatch {
                half_size: Vec2::new(2.5, 2.5),
                seed: 300 + i as u64,
                ..default()
            },
            GrassConfig {
                density_per_square_unit: 30.0,
                lod: two_band_lod.clone(),
                archetypes: vec![GrassArchetype {
                    debug_name: format!("{name} 2LOD"),
                    blade_height: Vec2::new(0.5, 1.0),
                    blade_width: Vec2::new(0.04, 0.09),
                    root_color: Color::srgb(0.18, 0.38, 0.12),
                    tip_color: Color::srgb(0.48, 0.82, 0.32),
                    color_variation: 0.10,
                    blade_shape: *shape,
                    ..default()
                }],
                ..default()
            },
        );
    }
}
