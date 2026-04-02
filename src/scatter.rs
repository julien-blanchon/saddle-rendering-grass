use bevy::math::Vec3Swizzles;
use bevy::prelude::*;

use crate::config::{
    GrassArchetype, GrassConfig, GrassDensityMapMode, GrassLodBand, GrassSurface,
    GrassTextureChannel,
};
use crate::surface::{SurfaceBake, SurfaceTriangle};

#[derive(Clone, Copy, Debug)]
pub(crate) struct BladePoint {
    pub position_local: Vec3,
    pub normal_local: Vec3,
    pub uv: Vec2,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct BladeSample {
    pub root_local: Vec3,
    pub normal_local: Vec3,
    pub yaw: f32,
    pub height: f32,
    pub width: f32,
    pub forward_curve: f32,
    pub lean: f32,
    pub stiffness: f32,
    pub interaction_strength: f32,
    pub phase: f32,
    pub color_variation: f32,
}

pub(crate) struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed.max(1) }
    }

    fn next_u64(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }

    pub fn next_f32(&mut self) -> f32 {
        (self.next_u64() as f64 / u64::MAX as f64) as f32
    }

    pub fn range_f32(&mut self, min: f32, max: f32) -> f32 {
        min + (max - min) * self.next_f32()
    }
}

pub(crate) fn planar_chunk_samples(
    patch_half_size: Vec2,
    chunk_min: Vec2,
    chunk_max: Vec2,
    patch_density_scale: f32,
    config: &GrassConfig,
    archetype: &GrassArchetype,
    lod: &GrassLodBand,
    density_image: Option<&Image>,
    surface: GrassSurface,
    seed: u64,
) -> Vec<BladeSample> {
    let area = (chunk_max - chunk_min).max(Vec2::ZERO);
    let area_sq = area.x * area.y;
    if area_sq <= f32::EPSILON {
        return Vec::new();
    }

    let density = config.density_per_square_unit
        * patch_density_scale.max(0.0)
        * lod.density_scale
        * archetype.weight.max(0.0);
    let target = (area_sq * density)
        .round()
        .clamp(0.0, config.max_blades_per_chunk as f32) as usize;
    if target == 0 {
        return Vec::new();
    }

    let aspect = if area.y.abs() <= 0.001 {
        1.0
    } else {
        area.x / area.y
    };
    let cols = ((target as f32 * aspect).sqrt().ceil() as usize).max(1);
    let rows = target.div_ceil(cols).max(1);
    let cell = Vec2::new(area.x / cols as f32, area.y / rows as f32);

    let mut rng = DeterministicRng::new(seed);
    let mut samples = Vec::with_capacity(target);

    for row in 0..rows {
        for col in 0..cols {
            if samples.len() >= target {
                break;
            }

            let local = Vec2::new(
                chunk_min.x + (col as f32 + rng.next_f32()) * cell.x,
                chunk_min.y + (row as f32 + rng.next_f32()) * cell.y,
            );
            let density_uv = match surface {
                GrassSurface::Planar => Some(Vec2::new(
                    (local.x + patch_half_size.x) / (patch_half_size.x * 2.0).max(0.001),
                    (local.y + patch_half_size.y) / (patch_half_size.y * 2.0).max(0.001),
                )),
                GrassSurface::Mesh(_) => None,
            };
            if !passes_density_map(config, density_image, density_uv, rng.next_f32()) {
                continue;
            }

            samples.push(blade_sample_from_point(
                BladePoint {
                    position_local: Vec3::new(local.x, 0.0, local.y),
                    normal_local: Vec3::Y,
                    uv: density_uv.unwrap_or(Vec2::ZERO),
                },
                archetype,
                &mut rng,
            ));
        }
    }

    samples
}

pub(crate) fn mesh_chunk_samples(
    chunk_triangle_indices: &[usize],
    surface: &SurfaceBake,
    patch_density_scale: f32,
    config: &GrassConfig,
    archetype: &GrassArchetype,
    lod: &GrassLodBand,
    density_image: Option<&Image>,
    seed: u64,
) -> Vec<BladeSample> {
    let triangles: Vec<&SurfaceTriangle> = chunk_triangle_indices
        .iter()
        .filter_map(|index| surface.triangles.get(*index))
        .collect();
    let total_area = triangles.iter().map(|triangle| triangle.area).sum::<f32>();
    if total_area <= f32::EPSILON {
        return Vec::new();
    }

    let density = config.density_per_square_unit
        * patch_density_scale.max(0.0)
        * lod.density_scale
        * archetype.weight.max(0.0);
    let target = (total_area * density)
        .round()
        .clamp(0.0, config.max_blades_per_chunk as f32) as usize;
    if target == 0 {
        return Vec::new();
    }

    let mut rng = DeterministicRng::new(seed);
    let mut samples = Vec::with_capacity(target);
    let attempts = (target * 4).max(16);

    for _ in 0..attempts {
        if samples.len() >= target {
            break;
        }
        let Some(triangle) = pick_triangle(&triangles, total_area, &mut rng) else {
            break;
        };
        let barycentric = random_barycentric(&mut rng);
        let point = BladePoint {
            position_local: triangle.sample_point(barycentric),
            normal_local: triangle.sample_normal(barycentric),
            uv: triangle.sample_uv(barycentric),
        };
        let density_uv = match config.density_map.as_ref().map(|density| density.mode) {
            Some(GrassDensityMapMode::SurfaceUv) => Some(point.uv),
            Some(GrassDensityMapMode::PatchUv) => {
                Some(surface.layout.uv_of_local_point(point.position_local.xz()))
            }
            None => None,
        };
        if !passes_density_map(config, density_image, density_uv, rng.next_f32()) {
            continue;
        }
        samples.push(blade_sample_from_point(point, archetype, &mut rng));
    }

    samples
}

fn pick_triangle<'a>(
    triangles: &[&'a SurfaceTriangle],
    total_area: f32,
    rng: &mut DeterministicRng,
) -> Option<&'a SurfaceTriangle> {
    let mut sample = rng.range_f32(0.0, total_area);
    for triangle in triangles {
        sample -= triangle.area;
        if sample <= 0.0 {
            return Some(*triangle);
        }
    }
    triangles.last().copied()
}

fn random_barycentric(rng: &mut DeterministicRng) -> Vec3 {
    let a = rng.next_f32();
    let b = rng.next_f32();
    let sqrt_a = a.sqrt();
    Vec3::new(1.0 - sqrt_a, sqrt_a * (1.0 - b), sqrt_a * b)
}

fn blade_sample_from_point(
    point: BladePoint,
    archetype: &GrassArchetype,
    rng: &mut DeterministicRng,
) -> BladeSample {
    BladeSample {
        root_local: point.position_local,
        normal_local: point.normal_local.normalize_or_zero(),
        yaw: rng.range_f32(0.0, std::f32::consts::TAU),
        height: rng.range_f32(archetype.blade_height.x, archetype.blade_height.y),
        width: rng.range_f32(archetype.blade_width.x, archetype.blade_width.y),
        forward_curve: rng.range_f32(archetype.forward_curve.x, archetype.forward_curve.y),
        lean: rng.range_f32(archetype.lean.x, archetype.lean.y),
        stiffness: rng.range_f32(archetype.stiffness.x, archetype.stiffness.y),
        interaction_strength: rng.range_f32(
            archetype.interaction_strength.x,
            archetype.interaction_strength.y,
        ),
        phase: rng.next_f32() * std::f32::consts::TAU,
        color_variation: rng.range_f32(-archetype.color_variation, archetype.color_variation),
    }
}

fn passes_density_map(
    config: &GrassConfig,
    density_image: Option<&Image>,
    sample_uv: Option<Vec2>,
    threshold: f32,
) -> bool {
    let Some(density_map) = &config.density_map else {
        return true;
    };
    let Some(image) = density_image else {
        return true;
    };
    let Some(uv) = sample_uv else {
        return true;
    };

    let density = sample_density_image(image, uv, density_map.channel, density_map.invert);
    threshold <= density
}

pub(crate) fn sample_density_image(
    image: &Image,
    uv: Vec2,
    channel: GrassTextureChannel,
    invert: bool,
) -> f32 {
    let extent = image.texture_descriptor.size;
    if extent.width == 0 || extent.height == 0 || image.data.is_none() {
        return 1.0;
    }

    let uv = uv.clamp(Vec2::ZERO, Vec2::ONE);
    let x = (uv.x * (extent.width.saturating_sub(1) as f32)).round() as usize;
    let y = (uv.y * (extent.height.saturating_sub(1) as f32)).round() as usize;
    let Some(data) = &image.data else {
        return 1.0;
    };
    let pixel = (y * extent.width as usize + x) * 4;
    if pixel + 3 >= data.len() {
        return 1.0;
    }
    let rgba = [
        data[pixel] as f32 / 255.0,
        data[pixel + 1] as f32 / 255.0,
        data[pixel + 2] as f32 / 255.0,
        data[pixel + 3] as f32 / 255.0,
    ];
    let value = match channel {
        GrassTextureChannel::Red => rgba[0],
        GrassTextureChannel::Green => rgba[1],
        GrassTextureChannel::Blue => rgba[2],
        GrassTextureChannel::Alpha => rgba[3],
        GrassTextureChannel::Luminance => rgba[0] * 0.2126 + rgba[1] * 0.7152 + rgba[2] * 0.0722,
    };

    if invert { 1.0 - value } else { value }
}

#[cfg(test)]
#[path = "scatter_tests.rs"]
mod tests;
