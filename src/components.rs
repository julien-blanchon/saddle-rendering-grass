use bevy::prelude::*;

use crate::config::{GrassChunking, GrassConfig, GrassSurface};

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
#[require(
    GrassConfig,
    GrassPatchState,
    Transform,
    GlobalTransform,
    Visibility,
    InheritedVisibility,
    ViewVisibility
)]
pub struct GrassPatch {
    pub half_size: Vec2,
    pub density_scale: f32,
    pub seed: u64,
    pub chunking: GrassChunking,
    pub surface: GrassSurface,
}

impl Default for GrassPatch {
    fn default() -> Self {
        Self {
            half_size: Vec2::splat(6.0),
            density_scale: 1.0,
            seed: 1,
            chunking: GrassChunking::default(),
            surface: GrassSurface::Planar,
        }
    }
}

#[derive(Bundle, Clone, Debug)]
pub struct GrassPatchBundle {
    pub name: Name,
    pub patch: GrassPatch,
    pub config: GrassConfig,
}

impl Default for GrassPatchBundle {
    fn default() -> Self {
        Self {
            name: Name::new("Grass Patch"),
            patch: GrassPatch::default(),
            config: GrassConfig::default(),
        }
    }
}

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
#[require(Transform, GlobalTransform)]
pub struct GrassInteractionZone {
    pub radius: f32,
    pub bend_strength: f32,
    pub flatten_strength: f32,
    pub falloff: f32,
}

impl Default for GrassInteractionZone {
    fn default() -> Self {
        Self {
            radius: 1.4,
            bend_strength: 0.45,
            flatten_strength: 0.25,
            falloff: 1.5,
        }
    }
}

#[derive(Component, Default)]
pub(crate) struct GrassPatchState {
    pub dirty: bool,
    pub material_handles: Vec<Handle<crate::GrassMaterial>>,
    pub generated_chunks: Vec<Entity>,
}

#[derive(Component)]
pub(crate) struct GrassGenerated;

#[derive(Component, Clone, Debug)]
pub(crate) struct GrassChunkRuntime {
    pub patch: Entity,
    pub source_entity: Option<Entity>,
    pub center_local: Vec3,
    pub size_local: Vec2,
    pub lod_index: usize,
    pub blade_count: u32,
}
