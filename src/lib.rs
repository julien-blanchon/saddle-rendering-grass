use bevy::app::PostStartup;
use bevy::asset::{load_internal_asset, uuid_handle};
use bevy::camera::visibility::VisibilitySystems;
use bevy::ecs::{intern::Interned, schedule::ScheduleLabel};
use bevy::pbr::{ExtendedMaterial, MaterialPlugin, StandardMaterial};
use bevy::prelude::*;
use bevy::shader::Shader;

mod components;
mod config;
mod interaction;
mod lod;
mod materials;
mod mesh;
mod messages;
mod resources;
mod scatter;
mod surface;
mod systems;

pub use components::{GrassInteractionZone, GrassPatch, GrassPatchBundle};
pub use config::{
    BladeShape, GrassArchetype, GrassChunking, GrassConfig, GrassDensityBlendMode,
    GrassDensityLayer, GrassDensityMap, GrassDensityMapMode, GrassExclusionZone, GrassLodBand,
    GrassLodConfig, GrassNormalSource, GrassScatterFilter, GrassSurface, GrassTextureChannel,
};
pub use interaction::{GrassInteractionActor, GrassInteractionMap, GrassInteractionPolicy};
pub use messages::GrassRebuildRequest;
pub use resources::{
    GrassDebugSettings, GrassDiagnostics, GrassPatchDiagnostics, GrassWind, GrassWindBridge,
};

pub use materials::{ATTRIBUTE_GRASS_ROOT_PHASE, ATTRIBUTE_GRASS_VARIATION, MAX_INTERACTION_ZONES};

pub type GrassMaterial = ExtendedMaterial<StandardMaterial, materials::GrassMaterialExtension>;

const GRASS_VERTEX_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("f56b0e88-73b2-4bc1-b493-f0b8390351f7");

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum GrassSystems {
    Prepare,
    Scatter,
    Upload,
    Animate,
    Debug,
}

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct NeverDeactivateSchedule;

pub struct GrassPlugin {
    pub activate_schedule: Interned<dyn ScheduleLabel>,
    pub deactivate_schedule: Interned<dyn ScheduleLabel>,
    pub update_schedule: Interned<dyn ScheduleLabel>,
}

impl GrassPlugin {
    pub fn new(
        activate_schedule: impl ScheduleLabel,
        deactivate_schedule: impl ScheduleLabel,
        update_schedule: impl ScheduleLabel,
    ) -> Self {
        Self {
            activate_schedule: activate_schedule.intern(),
            deactivate_schedule: deactivate_schedule.intern(),
            update_schedule: update_schedule.intern(),
        }
    }

    pub fn always_on(update_schedule: impl ScheduleLabel) -> Self {
        Self::new(PostStartup, NeverDeactivateSchedule, update_schedule)
    }
}

impl Default for GrassPlugin {
    fn default() -> Self {
        Self::always_on(Update)
    }
}

impl Plugin for GrassPlugin {
    fn build(&self, app: &mut App) {
        if self.deactivate_schedule == NeverDeactivateSchedule.intern() {
            app.init_schedule(NeverDeactivateSchedule);
        }

        load_internal_asset!(
            app,
            GRASS_VERTEX_SHADER_HANDLE,
            "shaders/grass_vertex.wgsl",
            Shader::from_wgsl
        );

        app.add_plugins(MaterialPlugin::<GrassMaterial>::default())
            .init_resource::<resources::GrassRuntimeState>()
            .init_resource::<GrassWind>()
            .init_resource::<GrassWindBridge>()
            .init_resource::<GrassDebugSettings>()
            .init_resource::<GrassDiagnostics>()
            .init_resource::<resources::GrassInteractionState>()
            .add_message::<GrassRebuildRequest>()
            .register_type::<BladeShape>()
            .register_type::<GrassArchetype>()
            .register_type::<GrassConfig>()
            .register_type::<GrassDebugSettings>()
            .register_type::<GrassDensityBlendMode>()
            .register_type::<GrassDensityLayer>()
            .register_type::<GrassDensityMap>()
            .register_type::<GrassDensityMapMode>()
            .register_type::<GrassDiagnostics>()
            .register_type::<GrassExclusionZone>()
            .register_type::<GrassInteractionZone>()
            .register_type::<GrassLodBand>()
            .register_type::<GrassLodConfig>()
            .register_type::<GrassNormalSource>()
            .register_type::<GrassPatch>()
            .register_type::<GrassPatchDiagnostics>()
            .register_type::<GrassScatterFilter>()
            .register_type::<GrassSurface>()
            .register_type::<GrassTextureChannel>()
            .register_type::<GrassWind>()
            .register_type::<GrassWindBridge>()
            .register_type::<GrassInteractionActor>()
            .register_type::<GrassInteractionMap>()
            .register_type::<GrassInteractionPolicy>()
            .add_systems(self.activate_schedule, systems::activate_runtime)
            .add_systems(self.deactivate_schedule, systems::deactivate_runtime)
            .configure_sets(
                self.update_schedule,
                (
                    GrassSystems::Prepare,
                    GrassSystems::Scatter,
                    GrassSystems::Upload,
                    GrassSystems::Animate,
                    GrassSystems::Debug,
                )
                    .chain(),
            )
            .add_systems(
                self.update_schedule,
                (
                    systems::mark_dirty_from_requests,
                    systems::mark_dirty_from_component_changes,
                    systems::mark_dirty_from_surface_changes,
                    systems::mark_dirty_from_asset_changes,
                    systems::collect_interaction_zones,
                    interaction::ensure_interaction_map_state,
                    interaction::update_interaction_map_center,
                    interaction::stamp_interaction_map,
                )
                    .chain()
                    .in_set(GrassSystems::Prepare)
                    .run_if(systems::runtime_is_active),
            )
            .add_systems(
                self.update_schedule,
                systems::rebuild_dirty_patches
                    .in_set(GrassSystems::Scatter)
                    .run_if(systems::runtime_is_active),
            )
            .add_systems(
                self.update_schedule,
                (
                    systems::sync_chunk_transforms,
                    systems::sync_material_uniforms,
                )
                    .chain()
                    .in_set(GrassSystems::Animate)
                    .run_if(systems::runtime_is_active),
            )
            .add_systems(
                self.update_schedule,
                systems::draw_debug_gizmos
                    .in_set(GrassSystems::Debug)
                    .run_if(systems::runtime_is_active),
            )
            .add_systems(
                PostUpdate,
                systems::publish_diagnostics
                    .after(VisibilitySystems::CheckVisibility)
                    .run_if(systems::runtime_is_active),
            );
    }
}
