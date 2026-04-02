use bevy::prelude::*;

#[derive(Resource, Clone, Debug, Reflect)]
#[reflect(Resource, Default)]
pub struct GrassWind {
    pub direction: Vec2,
    pub sway_strength: f32,
    pub sway_frequency: f32,
    pub sway_speed: f32,
    pub gust_strength: f32,
    pub gust_frequency: f32,
    pub gust_speed: f32,
    pub flutter_strength: f32,
}

impl Default for GrassWind {
    fn default() -> Self {
        Self {
            direction: Vec2::new(0.85, 0.35),
            sway_strength: 0.18,
            sway_frequency: 0.35,
            sway_speed: 0.85,
            gust_strength: 0.08,
            gust_frequency: 0.18,
            gust_speed: 0.2,
            flutter_strength: 0.04,
        }
    }
}

#[derive(Resource, Clone, Debug, Default, Reflect)]
#[reflect(Resource, Default)]
pub struct GrassDebugSettings {
    pub draw_patch_bounds: bool,
    pub draw_chunk_bounds: bool,
    pub draw_lod_colors: bool,
    pub draw_interaction_zones: bool,
}

#[derive(Clone, Debug, Reflect)]
#[reflect(Default)]
pub struct GrassPatchDiagnostics {
    pub entity: Entity,
    pub name: String,
    pub chunk_count: u32,
    pub blade_count: u32,
    pub visible_chunk_count: u32,
    pub visible_blade_count: u32,
    pub lod_chunk_counts: [u32; 3],
    pub lod_blade_counts: [u32; 3],
    pub visible_lod_chunk_counts: [u32; 3],
    pub visible_lod_blade_counts: [u32; 3],
    pub dirty: bool,
}

impl Default for GrassPatchDiagnostics {
    fn default() -> Self {
        Self {
            entity: Entity::PLACEHOLDER,
            name: String::new(),
            chunk_count: 0,
            blade_count: 0,
            visible_chunk_count: 0,
            visible_blade_count: 0,
            lod_chunk_counts: [0; 3],
            lod_blade_counts: [0; 3],
            visible_lod_chunk_counts: [0; 3],
            visible_lod_blade_counts: [0; 3],
            dirty: false,
        }
    }
}

#[derive(Resource, Clone, Debug, Default, Reflect)]
#[reflect(Resource, Default)]
pub struct GrassDiagnostics {
    pub runtime_active: bool,
    pub active_patches: u32,
    pub active_chunks: u32,
    pub active_blades: u32,
    pub visible_chunks: u32,
    pub visible_blades: u32,
    pub interaction_zones: u32,
    pub patches: Vec<GrassPatchDiagnostics>,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct GrassInteractionSample {
    pub center: Vec3,
    pub radius: f32,
    pub bend_strength: f32,
    pub flatten_strength: f32,
    pub falloff: f32,
}

#[derive(Resource, Default)]
pub(crate) struct GrassInteractionState {
    pub zones: Vec<GrassInteractionSample>,
}

#[derive(Resource, Default)]
pub(crate) struct GrassRuntimeState {
    pub active: bool,
}
