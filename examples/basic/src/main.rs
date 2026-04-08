use grass_example_common as common;

use bevy::prelude::*;
use grass::{GrassConfig, GrassPatch, GrassPlugin, GrassSurface};

fn main() {
    App::new()
        .insert_resource(common::presets::wind::breezy(Vec2::new(0.85, 0.35)))
        .add_plugins((
            common::default_plugins("Grass Example - Basic"),
            bevy::diagnostic::FrameTimeDiagnosticsPlugin::default(),
            GrassPlugin::default(),
            common::GrassExampleUiPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (common::sync_overlay, common::free_flight_system))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    common::spawn_camera(
        &mut commands,
        Vec3::new(-9.0, 7.5, 16.0),
        Vec3::new(0.0, 0.8, -6.0),
    );
    common::spawn_lighting(&mut commands);
    common::spawn_ground(&mut commands, &mut meshes, &mut materials, 54.0);
    let ramp = common::spawn_ramp_surface(&mut commands, &mut meshes, &mut materials);
    common::spawn_overlay(&mut commands, "Grass Basic");

    let density_map = common::radial_density_map(&mut images, 96);

    common::spawn_patch(
        &mut commands,
        "Courtyard Turf",
        Transform::from_xyz(-6.0, 0.0, 2.0),
        GrassPatch {
            half_size: Vec2::new(5.0, 4.0),
            density_scale: 1.0,
            seed: 11,
            ..default()
        },
        GrassConfig {
            density_per_square_unit: 42.0,
            density_map: Some(grass::GrassDensityMap {
                image: density_map,
                ..default()
            }),
            archetypes: vec![common::presets::archetypes::turf()],
            ..default()
        },
    );

    common::spawn_patch(
        &mut commands,
        "Ramp Meadow",
        Transform::default(),
        GrassPatch {
            seed: 27,
            surface: GrassSurface::Mesh(ramp),
            chunking: grass::GrassChunking {
                chunk_size: Vec2::new(3.0, 3.0),
            },
            ..default()
        },
        GrassConfig {
            density_per_square_unit: 24.0,
            archetypes: vec![
                common::presets::archetypes::meadow(),
                common::presets::archetypes::wildflower(),
            ],
            ..default()
        },
    );
}
