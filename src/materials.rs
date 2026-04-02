use bevy::asset::Asset;
use bevy::mesh::{Mesh, MeshVertexAttribute, MeshVertexBufferLayoutRef};
use bevy::pbr::{ExtendedMaterial, MaterialExtension, MaterialExtensionPipeline, StandardMaterial};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError, VertexFormat,
};
use bevy::shader::ShaderRef;

use crate::GRASS_VERTEX_SHADER_HANDLE;
use crate::resources::{GrassInteractionSample, GrassWind};

pub const ATTRIBUTE_GRASS_ROOT_PHASE: MeshVertexAttribute =
    MeshVertexAttribute::new("GrassRootPhase", 918_230_411, VertexFormat::Float32x4);
pub const ATTRIBUTE_GRASS_VARIATION: MeshVertexAttribute =
    MeshVertexAttribute::new("GrassVariation", 918_230_412, VertexFormat::Float32x4);

pub const MAX_INTERACTION_ZONES: usize = 4;

#[derive(Clone, ShaderType, Debug)]
pub struct GrassMaterialUniform {
    pub wind_direction: Vec2,
    pub sway_strength: f32,
    pub sway_frequency: f32,
    pub sway_speed: f32,
    pub gust_strength: f32,
    pub gust_frequency: f32,
    pub gust_speed: f32,
    pub flutter_strength: f32,
    pub interaction_count: u32,
    pub _padding: Vec3,
    pub zone_centers_radius: [Vec4; MAX_INTERACTION_ZONES],
    pub zone_behavior: [Vec4; MAX_INTERACTION_ZONES],
}

impl Default for GrassMaterialUniform {
    fn default() -> Self {
        Self::from_wind_and_zones(&GrassWind::default(), &[])
    }
}

impl GrassMaterialUniform {
    pub(crate) fn from_wind_and_zones(wind: &GrassWind, zones: &[GrassInteractionSample]) -> Self {
        let mut zone_centers_radius = [Vec4::ZERO; MAX_INTERACTION_ZONES];
        let mut zone_behavior = [Vec4::ZERO; MAX_INTERACTION_ZONES];

        for (index, zone) in zones.iter().take(MAX_INTERACTION_ZONES).enumerate() {
            zone_centers_radius[index] =
                Vec4::new(zone.center.x, zone.center.y, zone.center.z, zone.radius);
            zone_behavior[index] =
                Vec4::new(zone.bend_strength, zone.flatten_strength, zone.falloff, 0.0);
        }

        Self {
            wind_direction: wind.direction.normalize_or_zero(),
            sway_strength: wind.sway_strength,
            sway_frequency: wind.sway_frequency,
            sway_speed: wind.sway_speed,
            gust_strength: wind.gust_strength,
            gust_frequency: wind.gust_frequency,
            gust_speed: wind.gust_speed,
            flutter_strength: wind.flutter_strength,
            interaction_count: zones.len().min(MAX_INTERACTION_ZONES) as u32,
            _padding: Vec3::ZERO,
            zone_centers_radius,
            zone_behavior,
        }
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
pub struct GrassMaterialExtension {
    #[uniform(100)]
    pub uniform: GrassMaterialUniform,
}

impl MaterialExtension for GrassMaterialExtension {
    fn vertex_shader() -> ShaderRef {
        GRASS_VERTEX_SHADER_HANDLE.into()
    }

    fn specialize(
        _pipeline: &MaterialExtensionPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: bevy::pbr::MaterialExtensionKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.vertex.buffers = vec![layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
            Mesh::ATTRIBUTE_COLOR.at_shader_location(5),
            ATTRIBUTE_GRASS_ROOT_PHASE.at_shader_location(8),
            ATTRIBUTE_GRASS_VARIATION.at_shader_location(9),
        ])?];
        Ok(())
    }
}

pub(crate) fn build_material(
    archetype: &crate::config::GrassArchetype,
    wind: &GrassWind,
    zones: &[GrassInteractionSample],
) -> ExtendedMaterial<StandardMaterial, GrassMaterialExtension> {
    ExtendedMaterial {
        base: StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: archetype.roughness.clamp(0.089, 1.0),
            reflectance: archetype.reflectance.clamp(0.0, 1.0),
            diffuse_transmission: archetype.diffuse_transmission.clamp(0.0, 1.0),
            cull_mode: None,
            double_sided: true,
            ..default()
        },
        extension: GrassMaterialExtension {
            uniform: GrassMaterialUniform::from_wind_and_zones(wind, zones),
        },
    }
}
