use bevy::prelude::*;

#[derive(Resource, Clone, Debug, Reflect)]
#[reflect(Resource, Default)]
pub struct WindConfig {
    pub direction: Vec3,
    pub speed: f32,
    pub sway_factor: f32,
    pub gust_factor: f32,
    pub turbulence_strength: f32,
    pub flutter_factor: f32,
}

impl Default for WindConfig {
    fn default() -> Self {
        WindProfile::Breezy.config()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Reflect)]
pub enum WindProfile {
    Calm,
    Breezy,
    Gale,
    Storm,
}

impl WindProfile {
    pub fn config(self) -> WindConfig {
        match self {
            Self::Calm => WindConfig {
                direction: Vec3::new(1.0, 0.0, 0.0),
                speed: 0.4,
                sway_factor: 0.15,
                gust_factor: 0.05,
                turbulence_strength: 0.08,
                flutter_factor: 0.03,
            },
            Self::Breezy => WindConfig {
                direction: Vec3::new(1.0, 0.0, 0.18),
                speed: 2.2,
                sway_factor: 0.55,
                gust_factor: 0.22,
                turbulence_strength: 0.28,
                flutter_factor: 0.12,
            },
            Self::Gale => WindConfig {
                direction: Vec3::new(1.0, 0.0, 0.24),
                speed: 5.0,
                sway_factor: 0.95,
                gust_factor: 0.55,
                turbulence_strength: 0.6,
                flutter_factor: 0.32,
            },
            Self::Storm => WindConfig {
                direction: Vec3::new(0.92, 0.0, 0.38),
                speed: 8.5,
                sway_factor: 1.35,
                gust_factor: 0.95,
                turbulence_strength: 0.95,
                flutter_factor: 0.5,
            },
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct WindPlugin {
    config: WindConfig,
}

impl WindPlugin {
    pub fn with_config(mut self, config: WindConfig) -> Self {
        self.config = config;
        self
    }
}

impl Plugin for WindPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.config.clone())
            .register_type::<WindConfig>()
            .register_type::<WindZone>()
            .register_type::<WindZoneShape>()
            .register_type::<WindZoneFalloff>()
            .register_type::<WindBlendMode>();
    }
}

#[derive(Clone, Debug, Default, Reflect)]
pub struct WindSample {
    pub direction: Vec3,
    pub speed: f32,
    pub sway_factor: f32,
    pub gust_factor: f32,
    pub turbulence_strength: f32,
    pub flutter_factor: f32,
}

#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component, Default)]
pub struct WindZone {
    pub shape: WindZoneShape,
    pub falloff: WindZoneFalloff,
    pub blend_mode: WindBlendMode,
    pub direction: Vec3,
    pub speed: f32,
    pub intensity: f32,
    pub turbulence_multiplier: f32,
    pub gust_multiplier: f32,
    pub priority: i32,
}

impl Default for WindZone {
    fn default() -> Self {
        Self {
            shape: WindZoneShape::Sphere { radius: 1.0 },
            falloff: WindZoneFalloff::SmoothStep,
            blend_mode: WindBlendMode::Override,
            direction: Vec3::X,
            speed: 1.0,
            intensity: 1.0,
            turbulence_multiplier: 1.0,
            gust_multiplier: 1.0,
            priority: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
pub enum WindBlendMode {
    #[default]
    Override,
    Additive,
    Max,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
pub enum WindZoneFalloff {
    Constant,
    Linear,
    #[default]
    SmoothStep,
}

#[derive(Clone, Copy, Debug, Reflect)]
pub enum WindZoneShape {
    Sphere { radius: f32 },
    Box { half_extents: Vec3 },
}

impl Default for WindZoneShape {
    fn default() -> Self {
        Self::Sphere { radius: 1.0 }
    }
}

#[derive(Clone, Debug)]
pub struct WindZoneSnapshot {
    pub zone: WindZone,
    pub translation: Vec3,
    pub rotation: Quat,
}

pub fn snapshot_zone(zone: &WindZone, transform: &GlobalTransform) -> WindZoneSnapshot {
    let (_, rotation, translation) = transform.to_scale_rotation_translation();
    WindZoneSnapshot {
        zone: zone.clone(),
        translation,
        rotation,
    }
}

pub fn sample_wind_with_zones(
    sample_point: Vec3,
    time_secs: f32,
    config: &WindConfig,
    zones: &[WindZoneSnapshot],
) -> WindSample {
    let mut sample = base_sample(config);

    for snapshot in zones {
        let influence = zone_influence(snapshot, sample_point);
        if influence <= 0.0 {
            continue;
        }

        let mut zone_sample = base_sample(&WindConfig {
            direction: snapshot.zone.direction,
            speed: snapshot.zone.speed.max(0.0),
            sway_factor: config.sway_factor * snapshot.zone.intensity.max(0.0),
            gust_factor: config.gust_factor * snapshot.zone.gust_multiplier.max(0.0),
            turbulence_strength: config.turbulence_strength
                * snapshot.zone.turbulence_multiplier.max(0.0),
            flutter_factor: config.flutter_factor * snapshot.zone.turbulence_multiplier.max(0.0),
        });

        // Add a small animated pulse so moving zones still feel alive.
        let pulse = 0.85 + 0.15 * (time_secs * 0.9 + snapshot.translation.x * 0.1).sin();
        zone_sample.speed *= pulse;
        zone_sample.sway_factor *= pulse;
        zone_sample.gust_factor *= pulse;

        sample = blend_sample(sample, zone_sample, snapshot.zone.blend_mode, influence);
    }

    sample.direction = sample.direction.normalize_or_zero();
    sample
}

fn base_sample(config: &WindConfig) -> WindSample {
    WindSample {
        direction: config.direction.normalize_or_zero(),
        speed: config.speed.max(0.0),
        sway_factor: config.sway_factor.max(0.0),
        gust_factor: config.gust_factor.max(0.0),
        turbulence_strength: config.turbulence_strength.max(0.0),
        flutter_factor: config.flutter_factor.max(0.0),
    }
}

fn blend_sample(
    base: WindSample,
    zone: WindSample,
    blend_mode: WindBlendMode,
    influence: f32,
) -> WindSample {
    let mix_factor = influence.clamp(0.0, 1.0);
    match blend_mode {
        WindBlendMode::Override => WindSample {
            direction: base
                .direction
                .lerp(zone.direction, mix_factor)
                .normalize_or_zero(),
            speed: base.speed + (zone.speed - base.speed) * mix_factor,
            sway_factor: base.sway_factor + (zone.sway_factor - base.sway_factor) * mix_factor,
            gust_factor: base.gust_factor + (zone.gust_factor - base.gust_factor) * mix_factor,
            turbulence_strength: base.turbulence_strength
                + (zone.turbulence_strength - base.turbulence_strength) * mix_factor,
            flutter_factor: base.flutter_factor
                + (zone.flutter_factor - base.flutter_factor) * mix_factor,
        },
        WindBlendMode::Additive => WindSample {
            direction: (base.direction + zone.direction * mix_factor).normalize_or_zero(),
            speed: base.speed + zone.speed * mix_factor,
            sway_factor: base.sway_factor + zone.sway_factor * mix_factor,
            gust_factor: base.gust_factor + zone.gust_factor * mix_factor,
            turbulence_strength: base.turbulence_strength + zone.turbulence_strength * mix_factor,
            flutter_factor: base.flutter_factor + zone.flutter_factor * mix_factor,
        },
        WindBlendMode::Max => WindSample {
            direction: if zone.speed > base.speed {
                zone.direction
            } else {
                base.direction
            },
            speed: base.speed.max(zone.speed * mix_factor),
            sway_factor: base.sway_factor.max(zone.sway_factor * mix_factor),
            gust_factor: base.gust_factor.max(zone.gust_factor * mix_factor),
            turbulence_strength: base
                .turbulence_strength
                .max(zone.turbulence_strength * mix_factor),
            flutter_factor: base.flutter_factor.max(zone.flutter_factor * mix_factor),
        },
    }
}

fn zone_influence(snapshot: &WindZoneSnapshot, sample_point: Vec3) -> f32 {
    let local = snapshot.rotation.inverse() * (sample_point - snapshot.translation);
    let normalized_distance = match snapshot.zone.shape {
        WindZoneShape::Sphere { radius } => {
            if radius <= f32::EPSILON {
                return 0.0;
            }
            local.length() / radius
        }
        WindZoneShape::Box { half_extents } => {
            let extents = half_extents.max(Vec3::splat(f32::EPSILON));
            let normalized = local.abs() / extents;
            normalized.max_element()
        }
    };

    if normalized_distance >= 1.0 {
        return 0.0;
    }

    let linear = 1.0 - normalized_distance.clamp(0.0, 1.0);
    match snapshot.zone.falloff {
        WindZoneFalloff::Constant => 1.0,
        WindZoneFalloff::Linear => linear,
        WindZoneFalloff::SmoothStep => linear * linear * (3.0 - 2.0 * linear),
    }
}
