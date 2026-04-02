use bevy::color::LinearRgba;
use bevy::mesh::{Indices, Mesh};
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;

use crate::config::{GrassArchetype, GrassConfig};
use crate::materials::{ATTRIBUTE_GRASS_ROOT_PHASE, ATTRIBUTE_GRASS_VARIATION};
use crate::scatter::BladeSample;

pub(crate) fn build_chunk_mesh(
    samples: &[BladeSample],
    archetype: &GrassArchetype,
    config: &GrassConfig,
    segments: u8,
    chunk_center: Vec3,
) -> Option<Mesh> {
    if samples.is_empty() || segments < 1 {
        return None;
    }

    let mut positions = Vec::<[f32; 3]>::new();
    let mut normals = Vec::<[f32; 3]>::new();
    let mut uvs = Vec::<[f32; 2]>::new();
    let mut colors = Vec::<[f32; 4]>::new();
    let mut root_phase = Vec::<[f32; 4]>::new();
    let mut variation = Vec::<[f32; 4]>::new();
    let mut indices = Vec::<u32>::new();

    for sample in samples {
        append_blade(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut colors,
            &mut root_phase,
            &mut variation,
            &mut indices,
            sample,
            archetype,
            config.align_to_surface,
            chunk_center,
            segments,
        );
    }

    if positions.is_empty() {
        return None;
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_attribute(ATTRIBUTE_GRASS_ROOT_PHASE, root_phase);
    mesh.insert_attribute(ATTRIBUTE_GRASS_VARIATION, variation);
    mesh.insert_indices(Indices::U32(indices));
    Some(mesh)
}

#[allow(clippy::too_many_arguments)]
fn append_blade(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    colors: &mut Vec<[f32; 4]>,
    root_phase: &mut Vec<[f32; 4]>,
    variation: &mut Vec<[f32; 4]>,
    indices: &mut Vec<u32>,
    sample: &BladeSample,
    archetype: &GrassArchetype,
    align_to_surface: f32,
    chunk_center: Vec3,
    segments: u8,
) {
    let base_index = positions.len() as u32;
    let up = Vec3::Y.lerp(sample.normal_local, align_to_surface.clamp(0.0, 1.0));
    let up = up.normalize_or_zero();
    let facing = Quat::from_rotation_y(sample.yaw) * Vec3::Z;
    let right = up.cross(facing).normalize_or_zero();
    let forward = right.cross(up).normalize_or_zero();

    let root_color = vary_color(archetype.root_color, sample.color_variation).to_linear();
    let tip_color = vary_color(archetype.tip_color, sample.color_variation * 0.6).to_linear();

    for step in 0..=segments {
        let t = step as f32 / segments as f32;
        let center = sample.root_local
            + up * (sample.height * t)
            + forward * (sample.forward_curve * t * t)
            + forward * (sample.lean * sample.height * t * t);
        let width = sample.width * (1.0 - t).powf(0.75);
        let left = center - right * width * 0.5 - chunk_center;
        let right_pos = center + right * width * 0.5 - chunk_center;
        let vertex_color = lerp_linear(root_color, tip_color, t);
        let normal = forward.normalize_or_zero();
        let root = sample.root_local - chunk_center;
        let variation_value = [
            sample.stiffness,
            sample.interaction_strength,
            sample.color_variation,
            sample.lean,
        ];
        let root_phase_value = [root.x, root.y, root.z, sample.phase];

        positions.push(left.to_array());
        positions.push(right_pos.to_array());
        normals.push(normal.to_array());
        normals.push(normal.to_array());
        uvs.push([0.0, t]);
        uvs.push([1.0, t]);
        colors.push(vertex_color);
        colors.push(vertex_color);
        root_phase.push(root_phase_value);
        root_phase.push(root_phase_value);
        variation.push(variation_value);
        variation.push(variation_value);
    }

    for step in 0..segments as u32 {
        let row = base_index + step * 2;
        indices.extend_from_slice(&[row, row + 1, row + 2, row + 1, row + 3, row + 2]);
    }
}

fn vary_color(color: Color, offset: f32) -> Color {
    let linear = color.to_linear();
    Color::linear_rgba(
        (linear.red + offset).clamp(0.0, 1.0),
        (linear.green + offset * 0.5).clamp(0.0, 1.0),
        (linear.blue + offset * 0.25).clamp(0.0, 1.0),
        linear.alpha,
    )
}

fn lerp_linear(a: LinearRgba, b: LinearRgba, t: f32) -> [f32; 4] {
    [
        a.red + (b.red - a.red) * t,
        a.green + (b.green - a.green) * t,
        a.blue + (b.blue - a.blue) * t,
        a.alpha + (b.alpha - a.alpha) * t,
    ]
}
