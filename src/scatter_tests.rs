use bevy::image::{ImageSampler, ImageSamplerDescriptor};
use bevy::mesh::Mesh;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use super::*;
use crate::GrassDensityMap;
use crate::surface::bake_mesh_surface;

fn test_image(data: [u8; 16]) -> Image {
    let mut image = Image::new(
        Extent3d {
            width: 2,
            height: 2,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data.to_vec(),
        TextureFormat::Rgba8UnormSrgb,
        bevy::asset::RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor::nearest());
    image
}

#[test]
fn density_sampling_reads_requested_channel() {
    let image = test_image([
        255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
    ]);

    let red = sample_density_image(&image, Vec2::new(0.0, 0.0), GrassTextureChannel::Red, false);
    let green = sample_density_image(
        &image,
        Vec2::new(1.0, 0.0),
        GrassTextureChannel::Green,
        false,
    );
    let blue = sample_density_image(
        &image,
        Vec2::new(0.0, 1.0),
        GrassTextureChannel::Blue,
        false,
    );

    assert_eq!(red, 1.0);
    assert_eq!(green, 1.0);
    assert_eq!(blue, 1.0);
}

#[test]
fn density_sampling_supports_inversion() {
    let image = test_image([255, 255, 255, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255]);
    let value = sample_density_image(
        &image,
        Vec2::new(0.0, 0.0),
        GrassTextureChannel::Luminance,
        true,
    );
    assert_eq!(value, 0.0);
}

fn default_global() -> GlobalTransform {
    GlobalTransform::default()
}

#[test]
fn planar_sampling_is_deterministic_for_seed() {
    let config = GrassConfig::default();
    let archetype = GrassArchetype::default();
    let lod = config.lod.bands[0].clone();
    let global = default_global();

    let first = planar_chunk_samples(
        Vec2::splat(4.0),
        Vec2::new(-2.0, -2.0),
        Vec2::new(2.0, 2.0),
        1.0,
        &config,
        &archetype,
        &lod,
        None,
        &[],
        GrassSurface::Planar,
        &global,
        42,
    );
    let second = planar_chunk_samples(
        Vec2::splat(4.0),
        Vec2::new(-2.0, -2.0),
        Vec2::new(2.0, 2.0),
        1.0,
        &config,
        &archetype,
        &lod,
        None,
        &[],
        GrassSurface::Planar,
        &global,
        42,
    );

    assert_eq!(first.len(), second.len());
    assert_eq!(first[0].root_local, second[0].root_local);
    assert_eq!(first[0].phase, second[0].phase);
}

#[test]
fn patch_density_scale_changes_sample_count() {
    let config = GrassConfig {
        density_per_square_unit: 12.0,
        max_blades_per_chunk: 1_000,
        ..default()
    };
    let archetype = GrassArchetype::default();
    let lod = config.lod.bands[0].clone();
    let global = default_global();

    let dense = planar_chunk_samples(
        Vec2::splat(4.0),
        Vec2::new(-2.0, -2.0),
        Vec2::new(2.0, 2.0),
        1.0,
        &config,
        &archetype,
        &lod,
        None,
        &[],
        GrassSurface::Planar,
        &global,
        123,
    );
    let sparse = planar_chunk_samples(
        Vec2::splat(4.0),
        Vec2::new(-2.0, -2.0),
        Vec2::new(2.0, 2.0),
        0.25,
        &config,
        &archetype,
        &lod,
        None,
        &[],
        GrassSurface::Planar,
        &global,
        123,
    );
    let none = planar_chunk_samples(
        Vec2::splat(4.0),
        Vec2::new(-2.0, -2.0),
        Vec2::new(2.0, 2.0),
        0.0,
        &config,
        &archetype,
        &lod,
        None,
        &[],
        GrassSurface::Planar,
        &global,
        123,
    );

    assert!(dense.len() > sparse.len());
    assert!(sparse.len() > none.len());
    assert!(none.is_empty());
}

#[test]
fn mesh_density_map_mode_can_use_patch_uvs() {
    let mut mesh = Mesh::from(Plane3d::default().mesh().size(4.0, 4.0));
    let uv_count = mesh.count_vertices();
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0_f32, 0.0_f32]; uv_count]);
    let surface = bake_mesh_surface(&mesh, Vec2::new(4.0, 4.0)).expect("plane should bake");

    let density_image = test_image([
        0, 0, 0, 255, 255, 255, 255, 255, 0, 0, 0, 255, 255, 255, 255, 255,
    ]);
    let archetype = GrassArchetype::default();
    let lod = GrassLodBand {
        density_scale: 1.0,
        ..default()
    };
    let global = default_global();

    let patch_uv_config = GrassConfig {
        density_per_square_unit: 16.0,
        density_map: Some(GrassDensityMap {
            image: Handle::default(),
            mode: GrassDensityMapMode::PatchUv,
            ..default()
        }),
        ..default()
    };
    let surface_uv_config = GrassConfig {
        density_per_square_unit: 16.0,
        density_map: Some(GrassDensityMap {
            image: Handle::default(),
            mode: GrassDensityMapMode::SurfaceUv,
            ..default()
        }),
        ..default()
    };

    let patch_uv_samples = mesh_chunk_samples(
        surface.triangle_indices(IVec2::ZERO),
        &surface,
        1.0,
        &patch_uv_config,
        &archetype,
        &lod,
        Some(&density_image),
        &[],
        &global,
        77,
    );
    let surface_uv_samples = mesh_chunk_samples(
        surface.triangle_indices(IVec2::ZERO),
        &surface,
        1.0,
        &surface_uv_config,
        &archetype,
        &lod,
        Some(&density_image),
        &[],
        &global,
        77,
    );

    assert!(
        patch_uv_samples.len() > surface_uv_samples.len(),
        "patch UV sampling should not collapse to the source mesh UVs"
    );
    assert!(surface_uv_samples.is_empty());
}

#[test]
fn slope_filter_rejects_steep_normals() {
    let config = GrassConfig {
        density_per_square_unit: 16.0,
        scatter_filter: GrassScatterFilter {
            slope_range_degrees: Some((0.0, 30.0)),
            ..default()
        },
        ..default()
    };
    let archetype = GrassArchetype::default();
    let lod = config.lod.bands[0].clone();

    // Flat ground (slope 0°) — should pass
    let global_flat = GlobalTransform::default();
    let flat_samples = planar_chunk_samples(
        Vec2::splat(4.0),
        Vec2::new(-2.0, -2.0),
        Vec2::new(2.0, 2.0),
        1.0,
        &config,
        &archetype,
        &lod,
        None,
        &[],
        GrassSurface::Planar,
        &global_flat,
        99,
    );
    assert!(!flat_samples.is_empty(), "flat ground should produce blades");

    // 60° tilted surface — should reject (normal is at 60° from up)
    let global_steep = GlobalTransform::from(Transform::from_rotation(
        Quat::from_rotation_x(std::f32::consts::FRAC_PI_3),
    ));
    let steep_samples = planar_chunk_samples(
        Vec2::splat(4.0),
        Vec2::new(-2.0, -2.0),
        Vec2::new(2.0, 2.0),
        1.0,
        &config,
        &archetype,
        &lod,
        None,
        &[],
        GrassSurface::Planar,
        &global_steep,
        99,
    );
    assert!(
        steep_samples.is_empty(),
        "60° slope should be rejected by 30° filter"
    );
}

#[test]
fn altitude_filter_rejects_out_of_range() {
    let config = GrassConfig {
        density_per_square_unit: 16.0,
        scatter_filter: GrassScatterFilter {
            altitude_range: Some((0.0, 10.0)),
            ..default()
        },
        ..default()
    };
    let archetype = GrassArchetype::default();
    let lod = config.lod.bands[0].clone();

    // At Y=5 — should pass
    let global_low = GlobalTransform::from(Transform::from_translation(Vec3::new(0.0, 5.0, 0.0)));
    let low_samples = planar_chunk_samples(
        Vec2::splat(4.0),
        Vec2::new(-2.0, -2.0),
        Vec2::new(2.0, 2.0),
        1.0,
        &config,
        &archetype,
        &lod,
        None,
        &[],
        GrassSurface::Planar,
        &global_low,
        42,
    );
    assert!(!low_samples.is_empty(), "Y=5 should be in range 0..10");

    // At Y=50 — should reject
    let global_high =
        GlobalTransform::from(Transform::from_translation(Vec3::new(0.0, 50.0, 0.0)));
    let high_samples = planar_chunk_samples(
        Vec2::splat(4.0),
        Vec2::new(-2.0, -2.0),
        Vec2::new(2.0, 2.0),
        1.0,
        &config,
        &archetype,
        &lod,
        None,
        &[],
        GrassSurface::Planar,
        &global_high,
        42,
    );
    assert!(high_samples.is_empty(), "Y=50 should be out of range 0..10");
}

#[test]
fn exclusion_zone_removes_blades() {
    let config = GrassConfig {
        density_per_square_unit: 16.0,
        scatter_filter: GrassScatterFilter {
            exclusion_zones: vec![crate::config::GrassExclusionZone {
                center: Vec3::ZERO,
                radius: 100.0,
                falloff: 0.0,
            }],
            ..default()
        },
        ..default()
    };
    let archetype = GrassArchetype::default();
    let lod = config.lod.bands[0].clone();
    let global = default_global();

    let samples = planar_chunk_samples(
        Vec2::splat(4.0),
        Vec2::new(-2.0, -2.0),
        Vec2::new(2.0, 2.0),
        1.0,
        &config,
        &archetype,
        &lod,
        None,
        &[],
        GrassSurface::Planar,
        &global,
        42,
    );
    assert!(
        samples.is_empty(),
        "all blades should be inside exclusion zone"
    );
}

#[test]
fn strip_blade_produces_visible_mesh() {
    use crate::mesh::build_chunk_mesh;
    let config = GrassConfig::default();
    let archetype = GrassArchetype::default();
    let lod = config.lod.bands[0].clone();
    let global = default_global();

    let samples = planar_chunk_samples(
        Vec2::splat(4.0),
        Vec2::new(-2.0, -2.0),
        Vec2::new(2.0, 2.0),
        1.0,
        &config,
        &archetype,
        &lod,
        None,
        &[],
        GrassSurface::Planar,
        &global,
        42,
    );
    assert!(!samples.is_empty(), "should have samples");

    let mesh = build_chunk_mesh(&samples, &archetype, &config, 6, Vec3::ZERO);
    assert!(mesh.is_some(), "mesh should build");
    let mesh = mesh.unwrap();

    let positions = mesh.attribute(bevy::mesh::Mesh::ATTRIBUTE_POSITION).unwrap();
    let count = match positions {
        bevy::mesh::VertexAttributeValues::Float32x3(v) => v.len(),
        _ => 0,
    };
    assert!(count > 0, "mesh should have vertices, got {count}");

    // Check that not all positions are at Y=0 (blades grow upward)
    let has_elevated = match positions {
        bevy::mesh::VertexAttributeValues::Float32x3(v) => v.iter().any(|p| p[1] > 0.1),
        _ => false,
    };
    assert!(has_elevated, "some vertices should be above ground");
}
