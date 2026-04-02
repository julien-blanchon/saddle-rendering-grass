use bevy::asset::RenderAssetUsages;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::window::WindowResolution;

use grass::{GrassArchetype, GrassConfig, GrassDiagnostics, GrassPatch, GrassPatchBundle};

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

pub fn meadow_archetype() -> GrassArchetype {
    GrassArchetype {
        debug_name: "Meadow".into(),
        blade_height: Vec2::new(0.75, 1.35),
        blade_width: Vec2::new(0.025, 0.06),
        forward_curve: Vec2::new(0.12, 0.34),
        lean: Vec2::new(-0.22, 0.18),
        root_color: Color::srgb(0.15, 0.29, 0.10),
        tip_color: Color::srgb(0.52, 0.84, 0.28),
        color_variation: 0.18,
        diffuse_transmission: 0.24,
        ..default()
    }
}

pub fn turf_archetype() -> GrassArchetype {
    GrassArchetype {
        debug_name: "Turf".into(),
        weight: 1.0,
        blade_height: Vec2::new(0.2, 0.42),
        blade_width: Vec2::new(0.018, 0.035),
        forward_curve: Vec2::new(0.02, 0.08),
        lean: Vec2::new(-0.06, 0.06),
        root_color: Color::srgb(0.16, 0.36, 0.12),
        tip_color: Color::srgb(0.35, 0.72, 0.25),
        color_variation: 0.08,
        stiffness: Vec2::new(0.6, 0.95),
        interaction_strength: Vec2::new(0.5, 0.8),
        ..default()
    }
}

pub fn flower_archetype() -> GrassArchetype {
    GrassArchetype {
        debug_name: "Wildflower".into(),
        weight: 0.22,
        blade_height: Vec2::new(0.35, 0.72),
        blade_width: Vec2::new(0.03, 0.07),
        forward_curve: Vec2::new(0.02, 0.16),
        lean: Vec2::new(-0.1, 0.1),
        root_color: Color::srgb(0.24, 0.33, 0.12),
        tip_color: Color::srgb(0.93, 0.72, 0.46),
        color_variation: 0.28,
        stiffness: Vec2::new(0.75, 1.05),
        ..default()
    }
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
            "Interaction zones: {}  FPS: {:.1}",
            diagnostics.interaction_zones, fps
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

    overlay.0 = lines.join("\n");
}
