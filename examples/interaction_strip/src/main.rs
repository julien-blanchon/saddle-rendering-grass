use grass_example_common as common;

use bevy::prelude::*;
use grass::{GrassConfig, GrassInteractionZone, GrassPatch, GrassPlugin};

#[derive(Component)]
struct StripWalker;

fn main() {
    App::new()
        .insert_resource(common::presets::wind::breezy(Vec2::new(0.8, 0.3)))
        .add_plugins((
            common::default_plugins("Grass Example - Interaction Strip"),
            bevy::diagnostic::FrameTimeDiagnosticsPlugin::default(),
            GrassPlugin::default(),
            common::GrassExampleUiPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (move_walker, common::sync_overlay, common::free_flight_system))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    common::spawn_camera(
        &mut commands,
        Vec3::new(0.0, 6.0, 12.0),
        Vec3::new(0.0, 0.5, -5.0),
    );
    common::spawn_lighting(&mut commands);
    common::spawn_ground(&mut commands, &mut meshes, &mut materials, 80.0);
    common::spawn_overlay(&mut commands, "Grass Interaction Strip");

    common::spawn_patch(
        &mut commands,
        "Interaction Strip Patch",
        Transform::from_xyz(0.0, 0.0, -4.0),
        GrassPatch {
            half_size: Vec2::new(3.0, 18.0),
            density_scale: 1.1,
            seed: 900,
            ..default()
        },
        GrassConfig {
            density_per_square_unit: 46.0,
            archetypes: vec![common::presets::archetypes::meadow()],
            ..default()
        },
    );

    commands.spawn((
        Name::new("Walker"),
        StripWalker,
        GrassInteractionZone {
            radius: 1.2,
            bend_strength: 0.7,
            flatten_strength: 0.35,
            falloff: 1.8,
        },
        Mesh3d(meshes.add(Sphere::new(0.4).mesh().uv(24, 18))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.88, 0.63, 0.28),
            metallic: 0.0,
            perceptual_roughness: 0.55,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.45, -16.0),
    ));
}

fn move_walker(time: Res<Time>, mut walker: Single<&mut Transform, With<StripWalker>>) {
    let z = -16.0 + (time.elapsed_secs() * 1.2).sin() * 12.0;
    walker.translation = Vec3::new(0.0, 0.45, z);
}
