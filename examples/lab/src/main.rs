#[cfg(feature = "e2e")]
mod e2e;
#[cfg(feature = "e2e")]
mod scenarios;

use grass_example_common as common;

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
#[cfg(feature = "dev")]
use bevy::remote::{RemotePlugin, http::RemoteHttpPlugin};
use bevy::window::WindowResolution;
use bevy::winit::WinitSettings;
#[cfg(feature = "dev")]
use bevy_brp_extras::BrpExtrasPlugin;
use grass::{GrassConfig, GrassInteractionZone, GrassPatch, GrassPlugin, GrassSurface, GrassWind};

#[derive(Resource, Clone, Copy)]
pub struct LabState {
    pub camera: Entity,
    pub strip_walker: Entity,
    pub courtyard_patch: Entity,
    pub meadow_patch: Entity,
    pub slope_patch: Entity,
    pub strip_patch: Entity,
}

#[derive(Component)]
struct StripWalker;

fn main() {
    let mut app = App::new();
    app.insert_resource(common::presets::wind::calm(Vec2::new(0.82, 0.18)));
    app.insert_resource(WinitSettings::continuous());
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Grass Lab".into(),
                resolution: if cfg!(feature = "e2e") {
                    WindowResolution::new(960, 540)
                } else {
                    WindowResolution::new(1440, 900)
                },
                ..default()
            }),
            ..default()
        }),
        FrameTimeDiagnosticsPlugin::default(),
        GrassPlugin::default(),
        common::GrassExampleUiPlugin,
    ));
    #[cfg(feature = "dev")]
    app.add_plugins((
        RemotePlugin::default(),
        BrpExtrasPlugin::with_http_plugin(RemoteHttpPlugin::default()),
    ));
    #[cfg(feature = "e2e")]
    app.add_plugins(e2e::GrassLabE2EPlugin);

    app.add_systems(Startup, setup);
    app.add_systems(Update, (animate_wind, move_walker, common::sync_overlay));
    app.run();
}

fn lab_density_scale() -> f32 {
    if cfg!(feature = "e2e") { 0.18 } else { 1.0 }
}

fn lab_patch_scale() -> f32 {
    if cfg!(feature = "e2e") { 0.45 } else { 1.0 }
}

fn lab_chunk_scale() -> f32 {
    if cfg!(feature = "e2e") { 1.5 } else { 1.0 }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let density_scale = lab_density_scale();
    let patch_scale = lab_patch_scale();
    let chunk_scale = lab_chunk_scale();

    let camera = common::spawn_camera(
        &mut commands,
        Vec3::new(-18.0, 12.0, 30.0),
        Vec3::new(0.0, 1.0, -20.0),
    );
    if cfg!(feature = "e2e") {
        commands.entity(camera).insert(Msaa::Off);
    }
    common::spawn_lighting(&mut commands);
    common::spawn_ground(&mut commands, &mut meshes, &mut materials, 220.0);
    let ramp = common::spawn_ramp_surface(&mut commands, &mut meshes, &mut materials);
    common::spawn_overlay(&mut commands, "Grass Lab");

    let radial_density = common::radial_density_map(&mut images, 128);
    let checker_density = common::checker_density_map(&mut images, 128);

    let courtyard_patch = common::spawn_patch(
        &mut commands,
        "Courtyard Turf",
        Transform::from_xyz(-15.0, 0.0, 6.0),
        GrassPatch {
            half_size: Vec2::new(6.0, 5.0) * patch_scale,
            density_scale: 1.0,
            seed: 11,
            ..default()
        },
        GrassConfig {
            density_per_square_unit: 44.0 * density_scale,
            density_map: Some(grass::GrassDensityMap {
                image: radial_density,
                ..default()
            }),
            archetypes: vec![common::presets::archetypes::turf()],
            ..default()
        },
    );

    let meadow_patch = common::spawn_patch(
        &mut commands,
        "Open Meadow LOD",
        Transform::from_xyz(0.0, 0.0, -52.0),
        GrassPatch {
            half_size: Vec2::new(24.0, 34.0) * patch_scale,
            density_scale: 1.0,
            seed: 7,
            chunking: grass::GrassChunking {
                chunk_size: Vec2::new(6.0, 6.0) * chunk_scale,
            },
            ..default()
        },
        GrassConfig {
            density_per_square_unit: 30.0 * density_scale,
            density_map: Some(grass::GrassDensityMap {
                image: checker_density,
                ..default()
            }),
            archetypes: vec![
                common::presets::archetypes::meadow(),
                common::presets::archetypes::wildflower(),
            ],
            ..default()
        },
    );

    let slope_patch = common::spawn_patch(
        &mut commands,
        "Slope Meadow",
        Transform::default(),
        GrassPatch {
            seed: 27,
            surface: GrassSurface::Mesh(ramp),
            chunking: grass::GrassChunking {
                chunk_size: Vec2::new(3.0, 2.5) * chunk_scale,
            },
            ..default()
        },
        GrassConfig {
            density_per_square_unit: 26.0 * density_scale,
            archetypes: vec![
                common::presets::archetypes::meadow(),
                common::presets::archetypes::wildflower(),
            ],
            ..default()
        },
    );

    let strip_patch = common::spawn_patch(
        &mut commands,
        "Interaction Strip",
        Transform::from_xyz(14.0, 0.0, -8.0),
        GrassPatch {
            half_size: Vec2::new(3.5, 18.0) * patch_scale,
            density_scale: 1.1,
            seed: 900,
            ..default()
        },
        GrassConfig {
            density_per_square_unit: 48.0 * density_scale,
            archetypes: vec![common::presets::archetypes::meadow()],
            ..default()
        },
    );

    let strip_walker = commands
        .spawn((
            Name::new("Strip Walker"),
            StripWalker,
            GrassInteractionZone {
                radius: 1.25,
                bend_strength: 0.72,
                flatten_strength: 0.35,
                falloff: 1.8,
            },
            Mesh3d(meshes.add(Sphere::new(0.42).mesh().uv(24, 18))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.88, 0.63, 0.28),
                metallic: 0.0,
                perceptual_roughness: 0.55,
                ..default()
            })),
            Transform::from_xyz(14.0, 0.46, -22.0),
        ))
        .id();

    commands.insert_resource(LabState {
        camera,
        strip_walker,
        courtyard_patch,
        meadow_patch,
        slope_patch,
        strip_patch,
    });
}

fn animate_wind(time: Res<Time>, mut wind: ResMut<GrassWind>) {
    let t = time.elapsed_secs();
    // Slow, organic direction drift — full revolution takes ~2 minutes.
    wind.direction = Vec2::new(
        0.82 + (t * 0.08).sin() * 0.18,
        0.18 + (t * 0.06).cos() * 0.14,
    )
    .normalize_or_zero();
    // Gentle sway modulation around the calm baseline.
    wind.sway_strength = 0.10 + 0.04 * (t * 0.15).sin().abs();
    wind.gust_strength = 0.04 + 0.03 * (t * 0.12).sin().abs();
    wind.flutter_strength = 0.02 + 0.01 * (t * 0.25).cos().abs();
}

fn move_walker(time: Res<Time>, mut walker: Single<&mut Transform, With<StripWalker>>) {
    let z = -22.0 + (time.elapsed_secs() * 1.15).sin() * 13.0;
    walker.translation = Vec3::new(14.0, 0.46, z);
}
