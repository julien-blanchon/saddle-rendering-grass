use bevy::prelude::*;
use saddle_world_wind::WindSample;

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
    pub flutter_speed: f32,
}

impl Default for GrassWind {
    fn default() -> Self {
        Self::calm()
    }
}

impl GrassWind {
    /// Very gentle wind — nearly still grass with subtle motion.
    /// Suitable for sheltered meadows, indoor courtyards, or calm weather.
    pub fn calm() -> Self {
        Self {
            direction: Vec2::new(0.85, 0.35),
            sway_strength: 0.08,
            sway_frequency: 0.25,
            sway_speed: 0.35,
            gust_strength: 0.03,
            gust_frequency: 0.12,
            gust_speed: 0.08,
            flutter_strength: 0.015,
            flutter_speed: 2.5,
        }
    }

    /// Light natural breeze — grass sways gently with occasional gusts.
    /// Good default for open fields in fair weather.
    pub fn breezy() -> Self {
        Self {
            direction: Vec2::new(0.85, 0.35),
            sway_strength: 0.16,
            sway_frequency: 0.32,
            sway_speed: 0.55,
            gust_strength: 0.08,
            gust_frequency: 0.16,
            gust_speed: 0.14,
            flutter_strength: 0.035,
            flutter_speed: 3.2,
        }
    }

    /// Moderate wind — noticeable directional sway with visible gusts.
    /// Suitable for exposed hilltops or approaching weather.
    pub fn windy() -> Self {
        Self {
            direction: Vec2::new(0.85, 0.35),
            sway_strength: 0.28,
            sway_frequency: 0.40,
            sway_speed: 0.80,
            gust_strength: 0.18,
            gust_frequency: 0.22,
            gust_speed: 0.22,
            flutter_strength: 0.07,
            flutter_speed: 4.0,
        }
    }

    /// Strong turbulent wind — heavy bending and rapid gusts.
    /// Use for storms, high altitudes, or dramatic scenes.
    pub fn storm() -> Self {
        Self {
            direction: Vec2::new(0.85, 0.35),
            sway_strength: 0.45,
            sway_frequency: 0.50,
            sway_speed: 1.20,
            gust_strength: 0.35,
            gust_frequency: 0.30,
            gust_speed: 0.38,
            flutter_strength: 0.12,
            flutter_speed: 5.5,
        }
    }
}

/// Named presets for quick wind configuration.
///
/// Each variant maps to one of the `GrassWind` constructors.
#[derive(Clone, Copy, Debug, Default, Reflect, PartialEq, Eq)]
pub enum GrassWindPreset {
    /// Nearly still — sheltered courtyards, calm weather.
    #[default]
    Calm,
    /// Light natural breeze — open fields in fair weather.
    Breezy,
    /// Noticeable directional sway — hilltops, approaching weather.
    Windy,
    /// Heavy bending and rapid gusts — storms, high altitudes.
    Storm,
}

impl From<GrassWindPreset> for GrassWind {
    fn from(preset: GrassWindPreset) -> Self {
        match preset {
            GrassWindPreset::Calm => GrassWind::calm(),
            GrassWindPreset::Breezy => GrassWind::breezy(),
            GrassWindPreset::Windy => GrassWind::windy(),
            GrassWindPreset::Storm => GrassWind::storm(),
        }
    }
}

impl GrassWind {
    pub(crate) fn resolved_from_world_sample(
        &self,
        bridge: &GrassWindBridge,
        sample: &WindSample,
    ) -> Self {
        let direction = if sample.direction.xz().length_squared() <= f32::EPSILON {
            self.direction.normalize_or_zero()
        } else {
            sample.direction.xz().normalize_or_zero()
        };

        Self {
            direction,
            sway_strength: self.sway_strength.max(0.0)
                * (1.0 + sample.sway_factor.max(0.0) * bridge.sway_strength_scale.max(0.0)),
            sway_frequency: self.sway_frequency.max(0.0)
                + sample.turbulence_strength.max(0.0)
                    * bridge.sway_frequency_from_turbulence.max(0.0),
            sway_speed: self.sway_speed.max(0.0)
                + sample.speed.max(0.0) * bridge.sway_speed_from_speed.max(0.0),
            gust_strength: self.gust_strength.max(0.0)
                + sample.gust_factor.max(0.0) * bridge.gust_strength_scale.max(0.0),
            gust_frequency: self.gust_frequency.max(0.0)
                + sample.turbulence_strength.max(0.0)
                    * bridge.gust_frequency_from_turbulence.max(0.0),
            gust_speed: self.gust_speed.max(0.0)
                + sample.speed.max(0.0) * bridge.gust_speed_from_speed.max(0.0),
            flutter_strength: self.flutter_strength.max(0.0)
                + sample.flutter_factor.max(0.0) * bridge.flutter_strength_scale.max(0.0),
            flutter_speed: self.flutter_speed.max(0.0)
                + sample.speed.max(0.0) * bridge.flutter_speed_from_speed.max(0.0),
        }
    }
}

#[derive(Resource, Clone, Debug, Reflect)]
#[reflect(Resource, Default)]
pub struct GrassWindBridge {
    pub enabled: bool,
    pub sample_height_offset: f32,
    pub sway_strength_scale: f32,
    pub sway_frequency_from_turbulence: f32,
    pub sway_speed_from_speed: f32,
    pub gust_strength_scale: f32,
    pub gust_frequency_from_turbulence: f32,
    pub gust_speed_from_speed: f32,
    pub flutter_strength_scale: f32,
    pub flutter_speed_from_speed: f32,
}

impl Default for GrassWindBridge {
    fn default() -> Self {
        Self {
            enabled: true,
            sample_height_offset: 0.35,
            sway_strength_scale: 1.35,
            sway_frequency_from_turbulence: 0.9,
            sway_speed_from_speed: 0.18,
            gust_strength_scale: 0.28,
            gust_frequency_from_turbulence: 0.45,
            gust_speed_from_speed: 0.08,
            flutter_strength_scale: 0.2,
            flutter_speed_from_speed: 0.15,
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
    pub using_world_wind: bool,
    pub wind_zone_count: u32,
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

#[cfg(test)]
#[path = "resources_tests.rs"]
mod tests;
