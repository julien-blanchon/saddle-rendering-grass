use grass_example_common as common;

use bevy::prelude::*;
use grass::{GrassConfig, GrassPatch, GrassPlugin, GrassWind, GrassWindBridge};
use saddle_world_wind::{
    WindBlendMode, WindPlugin, WindProfile, WindZone, WindZoneFalloff, WindZoneShape,
};

#[derive(Component)]
struct CourtyardGustLane;

fn main() {
    App::new()
        .insert_resource(GrassWind {
            direction: Vec2::new(1.0, 0.2),
            sway_strength: 0.34,
            gust_strength: 0.2,
            flutter_strength: 0.09,
            ..default()
        })
        .insert_resource(GrassWindBridge {
            sample_height_offset: 0.55,
            ..default()
        })
        .add_plugins((
            common::default_plugins("Grass Example - Wind"),
            bevy::diagnostic::FrameTimeDiagnosticsPlugin::default(),
            GrassPlugin::default(),
            common::GrassExampleUiPlugin,
            WindPlugin::default().with_config(WindProfile::Gale.config()),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (animate_gust_lane, common::sync_overlay))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    common::spawn_camera(
        &mut commands,
        Vec3::new(-4.0, 5.2, 12.0),
        Vec3::new(0.0, 0.7, -5.0),
    );
    common::spawn_lighting(&mut commands);
    common::spawn_ground(&mut commands, &mut meshes, &mut materials, 70.0);
    common::spawn_overlay(&mut commands, "Grass Wind Showcase");

    common::spawn_patch(
        &mut commands,
        "Wind Courtyard Patch",
        Transform::from_xyz(-8.0, 0.0, 1.0),
        GrassPatch {
            half_size: Vec2::new(4.0, 5.0),
            density_scale: 1.1,
            seed: 100,
            ..default()
        },
        GrassConfig {
            density_per_square_unit: 26.0,
            archetypes: vec![common::meadow_archetype()],
            ..default()
        },
    );

    let mut flexible = common::meadow_archetype();
    flexible.debug_name = "Flexible".into();
    flexible.stiffness = Vec2::new(0.45, 0.7);
    flexible.lean = Vec2::new(-0.28, 0.26);
    common::spawn_patch(
        &mut commands,
        "Flexible Patch",
        Transform::from_xyz(0.0, 0.0, -4.0),
        GrassPatch {
            half_size: Vec2::new(6.0, 6.0),
            density_scale: 1.0,
            seed: 101,
            ..default()
        },
        GrassConfig {
            density_per_square_unit: 30.0,
            archetypes: vec![flexible],
            ..default()
        },
    );

    let mut stiff = common::turf_archetype();
    stiff.debug_name = "Stiff Turf".into();
    stiff.stiffness = Vec2::new(1.1, 1.5);
    common::spawn_patch(
        &mut commands,
        "Stiff Strip",
        Transform::from_xyz(8.0, 0.0, -8.0),
        GrassPatch {
            half_size: Vec2::new(4.0, 8.0),
            density_scale: 1.25,
            seed: 102,
            ..default()
        },
        GrassConfig {
            density_per_square_unit: 46.0,
            archetypes: vec![stiff],
            ..default()
        },
    );

    commands.spawn((
        Name::new("Courtyard Gust Lane"),
        CourtyardGustLane,
        WindZone {
            shape: WindZoneShape::Box {
                half_extents: Vec3::new(2.2, 1.6, 9.0),
            },
            falloff: WindZoneFalloff::SmoothStep,
            blend_mode: WindBlendMode::Additive,
            direction: Vec3::new(1.0, 0.0, 0.18),
            speed: 8.0,
            intensity: 0.9,
            turbulence_multiplier: 1.25,
            gust_multiplier: 1.35,
            priority: 6,
            ..default()
        },
        Transform::from_xyz(0.0, 1.0, -4.5),
        GlobalTransform::default(),
    ));
}

fn animate_gust_lane(
    time: Res<Time>,
    mut wind: ResMut<GrassWind>,
    mut query: Query<&mut Transform, With<CourtyardGustLane>>,
) {
    let t = time.elapsed_secs();
    wind.direction = Vec2::new(0.8 + t.sin() * 0.2, 0.15 + t.cos() * 0.25).normalize_or_zero();
    wind.sway_strength = 0.28 + 0.08 * (t * 0.5).sin().abs();
    wind.gust_strength = 0.12 + 0.1 * (t * 0.35).sin().abs();

    for mut transform in &mut query {
        transform.translation.x = (t * 0.42).sin() * 4.5;
        transform.rotation = Quat::from_rotation_y((t * 0.18).sin() * 0.22);
    }
}
