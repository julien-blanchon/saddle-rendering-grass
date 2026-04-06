pub mod presets;

use bevy::app::PostStartup;
use bevy::asset::RenderAssetUsages;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::window::WindowResolution;
use bevy_flair::FlairPlugin;
use bevy_input_focus::{InputDispatchPlugin, tab_navigation::TabNavigationPlugin};
use bevy_ui_widgets::UiWidgetsPlugins;
use saddle_pane::prelude::*;

use grass::{
    GrassArchetype, GrassConfig, GrassDiagnostics, GrassPatch, GrassPatchBundle, GrassWind,
    GrassWindBridge,
};

pub struct GrassExampleUiPlugin;

#[derive(Resource, Debug, Clone, Pane)]
#[pane(title = "Grass Demo", position = "top-right")]
pub struct GrassExamplePane {
    #[pane(slider, min = 4.0, max = 64.0, step = 1.0)]
    pub density_per_square_unit: f32,
    #[pane(slider, min = 0.1, max = 2.5, step = 0.05)]
    pub blade_height_max: f32,
    #[pane(slider, min = 0.01, max = 0.12, step = 0.005)]
    pub blade_width_max: f32,
    #[pane]
    pub cast_shadows: bool,
    #[pane(slider, min = 8.0, max = 36.0, step = 0.5)]
    pub near_lod_distance: f32,
    #[pane(slider, min = 18.0, max = 72.0, step = 1.0)]
    pub mid_lod_distance: f32,
    #[pane(slider, min = 36.0, max = 120.0, step = 1.0)]
    pub far_lod_distance: f32,
    #[pane]
    pub use_world_wind: bool,
    #[pane(slider, min = 0.0, max = 0.6, step = 0.005)]
    pub sway_strength: f32,
    #[pane(slider, min = 0.0, max = 1.5, step = 0.01)]
    pub sway_speed: f32,
    #[pane(slider, min = 0.0, max = 1.0, step = 0.01)]
    pub sway_frequency: f32,
    #[pane(slider, min = 0.0, max = 0.5, step = 0.005)]
    pub gust_strength: f32,
    #[pane(slider, min = 0.0, max = 0.5, step = 0.01)]
    pub gust_speed: f32,
    #[pane(slider, min = 0.0, max = 0.2, step = 0.005)]
    pub flutter_strength: f32,
    #[pane(slider, min = 0.5, max = 8.0, step = 0.1)]
    pub flutter_speed: f32,
    #[pane(slider, min = -1.0, max = 1.0, step = 0.01)]
    pub wind_direction_x: f32,
    #[pane(slider, min = -1.0, max = 1.0, step = 0.01)]
    pub wind_direction_z: f32,
    #[pane(monitor)]
    pub visible_blades: u32,
    #[pane(monitor)]
    pub active_chunks: u32,
    #[pane(monitor)]
    pub wind_zone_count: u32,
}

impl Default for GrassExamplePane {
    fn default() -> Self {
        let wind = GrassWind::default();
        let archetype = GrassArchetype::default();
        Self {
            density_per_square_unit: 32.0,
            blade_height_max: archetype.blade_height.y,
            blade_width_max: archetype.blade_width.y,
            cast_shadows: false,
            near_lod_distance: 18.0,
            mid_lod_distance: 42.0,
            far_lod_distance: 78.0,
            use_world_wind: true,
            sway_strength: wind.sway_strength,
            sway_speed: wind.sway_speed,
            sway_frequency: wind.sway_frequency,
            gust_strength: wind.gust_strength,
            gust_speed: wind.gust_speed,
            flutter_strength: wind.flutter_strength,
            flutter_speed: wind.flutter_speed,
            wind_direction_x: wind.direction.x,
            wind_direction_z: wind.direction.y,
            visible_blades: 0,
            active_chunks: 0,
            wind_zone_count: 0,
        }
    }
}

impl Plugin for GrassExampleUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GrassExamplePane>()
            .add_plugins((
                FlairPlugin,
                InputDispatchPlugin,
                UiWidgetsPlugins,
                TabNavigationPlugin,
                PanePlugin,
            ))
            .register_pane::<GrassExamplePane>()
            .add_systems(PostStartup, initialize_grass_pane)
            .add_systems(Update, (sync_grass_pane, sync_grass_pane_monitors));
    }
}

pub fn default_plugins(title: &str) -> bevy::app::PluginGroupBuilder {
    DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: title.into(),
            resolution: WindowResolution::new(1440, 900),
            ..default()
        }),
        ..default()
    })
}

pub fn spawn_camera(commands: &mut Commands, translation: Vec3, target: Vec3) -> Entity {
    commands
        .spawn((
            Name::new("Example Camera"),
            Camera3d::default(),
            Transform::from_translation(translation).looking_at(target, Vec3::Y),
        ))
        .id()
}

pub fn spawn_lighting(commands: &mut Commands) {
    commands.spawn((
        Name::new("Sun"),
        DirectionalLight {
            illuminance: 35_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(20.0, 30.0, 15.0).looking_at(Vec3::new(0.0, 0.0, -8.0), Vec3::Y),
    ));
    commands.spawn((
        Name::new("Fill Light"),
        PointLight {
            intensity: 12_000.0,
            range: 90.0,
            color: Color::srgb(1.0, 0.92, 0.84),
            ..default()
        },
        Transform::from_xyz(-18.0, 10.0, 10.0),
    ));
}

pub fn spawn_ground(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    size: f32,
) -> Entity {
    commands
        .spawn((
            Name::new("Ground"),
            Mesh3d(meshes.add(Plane3d::default().mesh().size(size, size).subdivisions(4))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.2, 0.23, 0.16),
                perceptual_roughness: 0.95,
                ..default()
            })),
        ))
        .id()
}

pub fn spawn_ramp_surface(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> Entity {
    commands
        .spawn((
            Name::new("Ramp Surface"),
            Mesh3d(meshes.add(Cuboid::new(12.0, 0.6, 4.5))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.31, 0.27, 0.22),
                perceptual_roughness: 0.88,
                ..default()
            })),
            Transform {
                translation: Vec3::new(0.0, 1.5, -10.0),
                rotation: Quat::from_rotation_x(-0.28),
                ..default()
            },
        ))
        .id()
}

pub fn radial_density_map(images: &mut Assets<Image>, size: u32) -> Handle<Image> {
    let mut data = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let uv = Vec2::new(x as f32 / size as f32, y as f32 / size as f32);
            let centered = uv * 2.0 - Vec2::ONE;
            let radial = (1.0 - centered.length()).clamp(0.0, 1.0);
            let value = (radial * radial * 255.0) as u8;
            data.extend_from_slice(&[value, value, value, 255]);
        }
    }

    images.add(Image::new(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    ))
}

pub fn checker_density_map(images: &mut Assets<Image>, size: u32) -> Handle<Image> {
    let mut data = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let cell = ((x / 16) + (y / 16)) % 2;
            let value = if cell == 0 { 230 } else { 70 };
            data.extend_from_slice(&[value, value, value, 255]);
        }
    }

    images.add(Image::new(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    ))
}

pub fn spawn_patch(
    commands: &mut Commands,
    name: &str,
    transform: Transform,
    patch: GrassPatch,
    config: GrassConfig,
) -> Entity {
    commands
        .spawn((
            GrassPatchBundle {
                name: Name::new(name.to_owned()),
                patch,
                config,
            },
            transform,
        ))
        .id()
}

#[derive(Component)]
pub struct DiagnosticsOverlay;

pub fn spawn_overlay(commands: &mut Commands, title: &str) {
    commands.spawn((
        Name::new("Diagnostics Overlay"),
        DiagnosticsOverlay,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(18.0),
            top: Val::Px(18.0),
            width: Val::Px(420.0),
            padding: UiRect::all(Val::Px(14.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.03, 0.05, 0.08, 0.72)),
        Text::new(title),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

pub fn sync_overlay(
    diagnostics: Res<GrassDiagnostics>,
    frame_time: Res<DiagnosticsStore>,
    mut overlay: Single<&mut Text, With<DiagnosticsOverlay>>,
) {
    let fps = frame_time
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|entry| entry.smoothed())
        .unwrap_or_default();

    let mut lines = vec![
        format!(
            "Grass patches: {}  chunks: {} / {} visible  blades: {} / {} visible",
            diagnostics.active_patches,
            diagnostics.active_chunks,
            diagnostics.visible_chunks,
            diagnostics.active_blades,
            diagnostics.visible_blades,
        ),
        format!(
            "Interaction zones: {}  FPS: {:.1}  Wind: {}",
            diagnostics.interaction_zones,
            fps,
            if diagnostics.using_world_wind {
                "world"
            } else {
                "local"
            },
        ),
    ];
    for patch in diagnostics.patches.iter().take(4) {
        lines.push(format!(
            "{}  chunks {:?} vis {:?}  blades {:?}",
            patch.name,
            patch.lod_chunk_counts,
            patch.visible_lod_chunk_counts,
            patch.lod_blade_counts
        ));
    }
    lines.push(String::new());
    lines.push("Use the pane (top-right) to tweak wind, density, and LOD.".into());

    overlay.0 = lines.join("\n");
}

fn initialize_grass_pane(
    mut pane: ResMut<GrassExamplePane>,
    wind: Res<GrassWind>,
    bridge: Res<GrassWindBridge>,
    configs: Query<&GrassConfig>,
) {
    if let Some(config) = configs.iter().next() {
        pane.density_per_square_unit = config.density_per_square_unit;
        pane.cast_shadows = config.cast_shadows;
        pane.near_lod_distance = config.lod.bands[0].max_distance;
        pane.mid_lod_distance = config.lod.bands[1].max_distance;
        pane.far_lod_distance = config.lod.bands[2].max_distance;

        if let Some(archetype) = config.archetypes.first() {
            pane.blade_height_max = archetype.blade_height.y;
            pane.blade_width_max = archetype.blade_width.y;
        }
    }

    pane.use_world_wind = bridge.enabled;
    pane.sway_strength = wind.sway_strength;
    pane.sway_speed = wind.sway_speed;
    pane.sway_frequency = wind.sway_frequency;
    pane.gust_strength = wind.gust_strength;
    pane.gust_speed = wind.gust_speed;
    pane.flutter_strength = wind.flutter_strength;
    pane.flutter_speed = wind.flutter_speed;
    pane.wind_direction_x = wind.direction.x;
    pane.wind_direction_z = wind.direction.y;
}

fn sync_grass_pane(
    pane: Res<GrassExamplePane>,
    mut configs: Query<&mut GrassConfig>,
    mut wind: ResMut<GrassWind>,
    mut bridge: ResMut<GrassWindBridge>,
) {
    if !pane.is_changed() {
        return;
    }

    for mut config in &mut configs {
        config.density_per_square_unit = pane.density_per_square_unit;
        config.cast_shadows = pane.cast_shadows;
        config.lod.bands[0].max_distance = pane.near_lod_distance;
        config.lod.bands[1].max_distance = pane.mid_lod_distance.max(pane.near_lod_distance + 1.0);
        config.lod.bands[2].max_distance = pane.far_lod_distance.max(pane.mid_lod_distance + 1.0);
        for archetype in &mut config.archetypes {
            archetype.blade_height.y = pane.blade_height_max;
            archetype.blade_width.y = pane.blade_width_max;
        }
    }

    wind.direction = Vec2::new(pane.wind_direction_x, pane.wind_direction_z);
    wind.sway_strength = pane.sway_strength;
    wind.sway_speed = pane.sway_speed;
    wind.sway_frequency = pane.sway_frequency;
    wind.gust_strength = pane.gust_strength;
    wind.gust_speed = pane.gust_speed;
    wind.flutter_strength = pane.flutter_strength;
    wind.flutter_speed = pane.flutter_speed;
    bridge.enabled = pane.use_world_wind;
}

fn sync_grass_pane_monitors(
    diagnostics: Res<GrassDiagnostics>,
    mut pane: ResMut<GrassExamplePane>,
) {
    pane.visible_blades = diagnostics.visible_blades;
    pane.active_chunks = diagnostics.active_chunks;
    pane.wind_zone_count = diagnostics.wind_zone_count;
}
