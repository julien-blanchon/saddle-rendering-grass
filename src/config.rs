use bevy::prelude::*;

#[derive(Clone, Debug, Reflect)]
#[reflect(Default)]
pub struct GrassChunking {
    pub chunk_size: Vec2,
}

impl Default for GrassChunking {
    fn default() -> Self {
        Self {
            chunk_size: Vec2::new(8.0, 8.0),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Reflect, PartialEq, Eq)]
pub enum GrassTextureChannel {
    Red,
    Green,
    Blue,
    Alpha,
    #[default]
    Luminance,
}

#[derive(Clone, Copy, Debug, Default, Reflect, PartialEq, Eq)]
pub enum GrassDensityMapMode {
    #[default]
    PatchUv,
    SurfaceUv,
}

#[derive(Clone, Debug, Reflect)]
#[reflect(Default)]
pub struct GrassDensityMap {
    pub image: Handle<Image>,
    pub channel: GrassTextureChannel,
    pub mode: GrassDensityMapMode,
    pub invert: bool,
}

impl Default for GrassDensityMap {
    fn default() -> Self {
        Self {
            image: Default::default(),
            channel: GrassTextureChannel::Luminance,
            mode: GrassDensityMapMode::PatchUv,
            invert: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Reflect, PartialEq, Eq)]
pub enum GrassSurface {
    #[default]
    Planar,
    Mesh(Entity),
}

#[derive(Clone, Debug, Reflect)]
#[reflect(Default)]
pub struct GrassLodBand {
    pub max_distance: f32,
    pub density_scale: f32,
    pub segments: u8,
    pub fade_distance: f32,
}

impl Default for GrassLodBand {
    fn default() -> Self {
        Self {
            max_distance: 22.0,
            density_scale: 1.0,
            segments: 6,
            fade_distance: 4.0,
        }
    }
}

#[derive(Clone, Debug, Reflect)]
#[reflect(Default)]
pub struct GrassLodConfig {
    pub bands: [GrassLodBand; 3],
}

impl Default for GrassLodConfig {
    fn default() -> Self {
        Self {
            bands: [
                GrassLodBand {
                    max_distance: 18.0,
                    density_scale: 1.0,
                    segments: 6,
                    fade_distance: 4.0,
                },
                GrassLodBand {
                    max_distance: 42.0,
                    density_scale: 0.42,
                    segments: 4,
                    fade_distance: 6.0,
                },
                GrassLodBand {
                    max_distance: 78.0,
                    density_scale: 0.15,
                    segments: 2,
                    fade_distance: 10.0,
                },
            ],
        }
    }
}

#[derive(Clone, Debug, Reflect)]
#[reflect(Default)]
pub struct GrassArchetype {
    pub debug_name: String,
    pub weight: f32,
    pub blade_height: Vec2,
    pub blade_width: Vec2,
    pub forward_curve: Vec2,
    pub lean: Vec2,
    pub stiffness: Vec2,
    pub interaction_strength: Vec2,
    pub root_color: Color,
    pub tip_color: Color,
    pub color_variation: f32,
    pub roughness: f32,
    pub reflectance: f32,
    pub diffuse_transmission: f32,
}

impl Default for GrassArchetype {
    fn default() -> Self {
        Self {
            debug_name: "Base".into(),
            weight: 1.0,
            blade_height: Vec2::new(0.55, 1.0),
            blade_width: Vec2::new(0.02, 0.045),
            forward_curve: Vec2::new(0.04, 0.18),
            lean: Vec2::new(-0.12, 0.12),
            stiffness: Vec2::new(0.9, 1.15),
            interaction_strength: Vec2::new(0.85, 1.15),
            root_color: Color::srgb(0.36, 0.34, 0.30),
            tip_color: Color::srgb(0.63, 0.60, 0.54),
            color_variation: 0.08,
            roughness: 0.9,
            reflectance: 0.16,
            diffuse_transmission: 0.18,
        }
    }
}

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct GrassConfig {
    pub density_per_square_unit: f32,
    pub max_blades_per_chunk: u32,
    pub align_to_surface: f32,
    pub normal_offset: f32,
    pub density_map: Option<GrassDensityMap>,
    pub lod: GrassLodConfig,
    pub archetypes: Vec<GrassArchetype>,
    pub cast_shadows: bool,
}

impl Default for GrassConfig {
    fn default() -> Self {
        Self {
            density_per_square_unit: 38.0,
            max_blades_per_chunk: 1_600,
            align_to_surface: 0.7,
            normal_offset: 0.005,
            density_map: None,
            lod: GrassLodConfig::default(),
            archetypes: vec![GrassArchetype::default()],
            cast_shadows: false,
        }
    }
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;
