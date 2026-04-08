use bevy::prelude::*;

/// Controls which blades pass placement filters during scatter.
///
/// All filters are combined with AND logic: a candidate blade must pass
/// every enabled filter to be placed. Set a field to `None` to skip that
/// filter.
#[derive(Clone, Debug, Default, Reflect)]
#[reflect(Default)]
pub struct GrassScatterFilter {
    /// Allowed slope range in degrees (`0` = flat, `90` = vertical wall).
    /// Blades whose surface normal exceeds this range are rejected.
    /// Example: `Some((0.0, 45.0))` keeps grass off cliffs steeper than 45°.
    pub slope_range_degrees: Option<(f32, f32)>,

    /// Allowed world-space altitude (Y) range.
    /// Example: `Some((0.0, 80.0))` keeps grass below the snow line.
    pub altitude_range: Option<(f32, f32)>,

    /// Spherical exclusion zones in **world space**.
    /// Any blade whose root falls inside `(center, radius)` is rejected.
    /// Useful for clearing grass around buildings, roads, or props.
    pub exclusion_zones: Vec<GrassExclusionZone>,
}

/// A spherical exclusion zone that prevents grass placement.
#[derive(Clone, Debug, Reflect)]
#[reflect(Default)]
pub struct GrassExclusionZone {
    /// World-space center of the exclusion sphere.
    pub center: Vec3,
    /// Radius of the exclusion sphere.
    pub radius: f32,
    /// Soft falloff distance beyond the radius where density ramps from 0 to 1.
    /// `0.0` = hard cutoff at `radius`.
    pub falloff: f32,
}

impl Default for GrassExclusionZone {
    fn default() -> Self {
        Self {
            center: Vec3::ZERO,
            radius: 2.0,
            falloff: 0.0,
        }
    }
}

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

/// How a density layer is combined with the running density value.
#[derive(Clone, Copy, Debug, Default, Reflect, PartialEq, Eq)]
pub enum GrassDensityBlendMode {
    /// `result = running * layer`
    #[default]
    Multiply,
    /// `result = min(running, layer)`
    Min,
    /// `result = max(running, layer)`
    Max,
    /// `result = clamp(running + layer - 1, 0, 1)` (only keeps overlap)
    Add,
}

/// An additional density map layer applied after the primary `density_map`.
#[derive(Clone, Debug, Reflect)]
#[reflect(Default)]
pub struct GrassDensityLayer {
    pub image: Handle<Image>,
    pub channel: GrassTextureChannel,
    pub mode: GrassDensityMapMode,
    pub invert: bool,
    pub blend: GrassDensityBlendMode,
}

impl Default for GrassDensityLayer {
    fn default() -> Self {
        Self {
            image: Default::default(),
            channel: GrassTextureChannel::Luminance,
            mode: GrassDensityMapMode::PatchUv,
            invert: false,
            blend: GrassDensityBlendMode::Multiply,
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
    /// Variable-length LOD bands (1 to N). Sorted by ascending `max_distance`.
    /// Default provides 3 bands matching the original behavior.
    pub bands: Vec<GrassLodBand>,
}

impl Default for GrassLodConfig {
    fn default() -> Self {
        Self {
            bands: vec![
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

/// Blade geometry shape.
#[derive(Clone, Copy, Debug, Default, Reflect, PartialEq, Eq)]
pub enum BladeShape {
    /// Tapered flat strip (default) — multi-segment ribbon with width tapering to tip.
    #[default]
    Strip,
    /// Two perpendicular strips forming an X when viewed from above.
    /// Good for stylized grass and cheaper far-distance filler.
    CrossBillboard,
    /// Single flat quad (2 triangles). Cheapest option — good for far LOD or
    /// texture-card grass with alpha cutout.
    FlatCard,
    /// Single triangle tapering to a point. Stylized / anime look.
    SingleTriangle,
}

/// Where the blade normal comes from — affects shading style.
#[derive(Clone, Copy, Debug, Default, Reflect, PartialEq, Eq)]
pub enum GrassNormalSource {
    /// Normal follows the blade facing direction (standard).
    #[default]
    BladeFacing,
    /// Normal is projected from the ground surface — produces flat/unified
    /// shading across all blades. Essential for anime / cel-shaded styles.
    GroundNormal,
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
    /// Blade geometry shape. Default: `Strip`.
    pub blade_shape: BladeShape,
    /// Vertex alpha at the blade tip (`0.0` = fully transparent tip, `1.0` = opaque).
    /// Root alpha is always `1.0`. The alpha fades linearly from root to tip.
    /// Useful for soft blade tips and stylized fade-out.
    pub tip_alpha: f32,
    /// Where blade normals come from. `GroundNormal` gives flat cel-shaded look.
    pub normal_source: GrassNormalSource,
    /// Optional blade texture (albedo). When set, UVs map across the blade strip
    /// and `base_color_texture` is applied. Combined with vertex color multiplicatively.
    pub blade_texture: Option<Handle<Image>>,
    /// Alpha cutoff for blade textures. Fragments below this alpha are discarded.
    /// Only used when `blade_texture` is `Some`. Default `0.5`.
    pub alpha_cutoff: f32,
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
            blade_shape: BladeShape::default(),
            tip_alpha: 1.0,
            normal_source: GrassNormalSource::default(),
            blade_texture: None,
            alpha_cutoff: 0.5,
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
    /// Compositable density layers applied after the primary density map.
    /// Each layer's result is combined with the running density via its blend mode.
    pub density_layers: Vec<GrassDensityLayer>,
    pub lod: GrassLodConfig,
    pub archetypes: Vec<GrassArchetype>,
    pub cast_shadows: bool,
    /// Scatter-time placement filters (slope, altitude, exclusion zones).
    pub scatter_filter: GrassScatterFilter,
}

impl Default for GrassConfig {
    fn default() -> Self {
        Self {
            density_per_square_unit: 38.0,
            max_blades_per_chunk: 1_600,
            align_to_surface: 0.7,
            normal_offset: 0.005,
            density_map: None,
            density_layers: Vec::new(),
            lod: GrassLodConfig::default(),
            archetypes: vec![GrassArchetype::default()],
            cast_shadows: false,
            scatter_filter: GrassScatterFilter::default(),
        }
    }
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;
