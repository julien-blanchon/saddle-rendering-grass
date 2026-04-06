use bevy::prelude::*;

use grass::{GrassArchetype, GrassWind};

pub mod archetypes {
    use super::*;

    pub fn meadow() -> GrassArchetype {
        GrassArchetype {
            debug_name: "Meadow".into(),
            blade_height: Vec2::new(0.75, 1.35),
            blade_width: Vec2::new(0.025, 0.06),
            forward_curve: Vec2::new(0.12, 0.34),
            lean: Vec2::new(-0.22, 0.18),
            root_color: Color::srgb(0.15, 0.29, 0.10),
            tip_color: Color::srgb(0.52, 0.84, 0.28),
            color_variation: 0.18,
            diffuse_transmission: 0.24,
            ..default()
        }
    }

    pub fn turf() -> GrassArchetype {
        GrassArchetype {
            debug_name: "Turf".into(),
            weight: 1.0,
            blade_height: Vec2::new(0.2, 0.42),
            blade_width: Vec2::new(0.018, 0.035),
            forward_curve: Vec2::new(0.02, 0.08),
            lean: Vec2::new(-0.06, 0.06),
            root_color: Color::srgb(0.16, 0.36, 0.12),
            tip_color: Color::srgb(0.35, 0.72, 0.25),
            color_variation: 0.08,
            stiffness: Vec2::new(0.6, 0.95),
            interaction_strength: Vec2::new(0.5, 0.8),
            ..default()
        }
    }

    pub fn wildflower() -> GrassArchetype {
        GrassArchetype {
            debug_name: "Wildflower".into(),
            weight: 0.22,
            blade_height: Vec2::new(0.35, 0.72),
            blade_width: Vec2::new(0.03, 0.07),
            forward_curve: Vec2::new(0.02, 0.16),
            lean: Vec2::new(-0.1, 0.1),
            root_color: Color::srgb(0.24, 0.33, 0.12),
            tip_color: Color::srgb(0.93, 0.72, 0.46),
            color_variation: 0.28,
            stiffness: Vec2::new(0.75, 1.05),
            ..default()
        }
    }
}

pub mod wind {
    use super::*;

    pub fn calm(direction: Vec2) -> GrassWind {
        profile(direction, 0.08, 0.25, 0.35, 0.03, 0.12, 0.08, 0.015, 2.5)
    }

    pub fn breezy(direction: Vec2) -> GrassWind {
        profile(direction, 0.16, 0.32, 0.55, 0.08, 0.16, 0.14, 0.035, 3.2)
    }

    pub fn windy(direction: Vec2) -> GrassWind {
        profile(direction, 0.28, 0.40, 0.80, 0.18, 0.22, 0.22, 0.07, 4.0)
    }

    pub fn storm(direction: Vec2) -> GrassWind {
        profile(direction, 0.45, 0.50, 1.20, 0.35, 0.30, 0.38, 0.12, 5.5)
    }

    fn profile(
        direction: Vec2,
        sway_strength: f32,
        sway_frequency: f32,
        sway_speed: f32,
        gust_strength: f32,
        gust_frequency: f32,
        gust_speed: f32,
        flutter_strength: f32,
        flutter_speed: f32,
    ) -> GrassWind {
        GrassWind {
            direction: direction.normalize_or_zero(),
            sway_strength,
            sway_frequency,
            sway_speed,
            gust_strength,
            gust_frequency,
            gust_speed,
            flutter_strength,
            flutter_speed,
        }
    }
}
