use grass_example_common as common;

use bevy::prelude::*;
use grass::{GrassConfig, GrassPatch, GrassPlugin};

#[derive(Component)]
struct OrbitCamera;

fn main() {
    App::new()
        .add_plugins((
            common::default_plugins("Grass Example - Stress Field"),
            bevy::diagnostic::FrameTimeDiagnosticsPlugin::default(),
            GrassPlugin::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (orbit_camera, common::sync_overlay))
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
        Transform::from_xyz(0.0, 18.0, 36.0).looking_at(Vec3::new(0.0, 0.0, -10.0), Vec3::Y),
    ));
    common::spawn_lighting(&mut commands);
    common::spawn_ground(&mut commands, &mut meshes, &mut materials, 240.0);
    common::spawn_overlay(&mut commands, "Grass Stress Field");

    for z in 0..3 {
        for x in 0..3 {
            let offset = Vec3::new((x as f32 - 1.0) * 24.0, 0.0, -12.0 - z as f32 * 26.0);
            common::spawn_patch(
                &mut commands,
                &format!("Stress Patch {}-{}", x, z),
                Transform::from_translation(offset),
                GrassPatch {
                    half_size: Vec2::new(10.0, 10.0),
                    density_scale: 1.0,
                    seed: (x + z * 3 + 1) as u64,
                    ..default()
                },
                GrassConfig {
                    density_per_square_unit: 48.0,
                    archetypes: vec![common::meadow_archetype(), common::flower_archetype()],
                    ..default()
                },
            );
        }
    }
}

fn orbit_camera(time: Res<Time>, mut camera: Single<&mut Transform, With<OrbitCamera>>) {
    let angle = time.elapsed_secs() * 0.16;
    let radius = 42.0;
    camera.translation = Vec3::new(angle.cos() * radius, 18.0, angle.sin() * radius - 18.0);
    **camera = camera.looking_at(Vec3::new(0.0, 0.0, -20.0), Vec3::Y);
}
