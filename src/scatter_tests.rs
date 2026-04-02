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

#[test]
fn planar_sampling_is_deterministic_for_seed() {
    let config = GrassConfig::default();
    let archetype = GrassArchetype::default();
    let lod = config.lod.bands[0].clone();

    let first = planar_chunk_samples(
        Vec2::splat(4.0),
        Vec2::new(-2.0, -2.0),
        Vec2::new(2.0, 2.0),
        1.0,
        &config,
        &archetype,
        &lod,
        None,
        GrassSurface::Planar,
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
        GrassSurface::Planar,
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

    let dense = planar_chunk_samples(
        Vec2::splat(4.0),
        Vec2::new(-2.0, -2.0),
        Vec2::new(2.0, 2.0),
        1.0,
        &config,
        &archetype,
        &lod,
        None,
        GrassSurface::Planar,
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
        GrassSurface::Planar,
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
        GrassSurface::Planar,
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
        77,
    );

    assert!(
        patch_uv_samples.len() > surface_uv_samples.len(),
        "patch UV sampling should not collapse to the source mesh UVs"
    );
    assert!(surface_uv_samples.is_empty());
}
