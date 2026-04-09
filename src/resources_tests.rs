use super::*;

#[test]
fn world_wind_bridge_keeps_fallback_direction_when_sample_is_zero() {
    let fallback = GrassWind::default();
    let bridge = GrassWindBridge::default();

    let resolved = fallback.resolved_from_world_sample(&bridge, &WindSample::default());

    assert_eq!(resolved.direction, fallback.direction.normalize_or_zero());
    assert_eq!(resolved.sway_strength, fallback.sway_strength);
}

#[test]
fn world_wind_bridge_scales_runtime_response_from_sample_metrics() {
    let fallback = GrassWind {
        direction: Vec2::new(1.0, 0.0),
        sway_strength: 0.2,
        sway_frequency: 0.3,
        sway_speed: 0.8,
        gust_strength: 0.1,
        gust_frequency: 0.2,
        gust_speed: 0.3,
        flutter_strength: 0.05,
        flutter_speed: 3.0,
    };
    let bridge = GrassWindBridge::default();
    let sample = WindSample {
        direction: Vec3::new(0.0, 0.0, 1.0),
        speed: 3.0,
        sway_factor: 0.6,
        gust_factor: 0.75,
        turbulence_strength: 0.5,
        flutter_factor: 0.4,
    };

    let resolved = fallback.resolved_from_world_sample(&bridge, &sample);

    assert_eq!(resolved.direction, Vec2::Y);
    assert!(resolved.sway_strength > fallback.sway_strength);
    assert!(resolved.gust_strength > fallback.gust_strength);
    assert!(resolved.flutter_strength > fallback.flutter_strength);
}

#[test]
fn default_wind_is_neutral_and_directionless() {
    let default_wind = GrassWind::default();

    assert_eq!(default_wind.direction, Vec2::ZERO);
    assert_eq!(default_wind.sway_strength, 0.0);
    assert_eq!(default_wind.gust_strength, 0.0);
    assert_eq!(default_wind.flutter_strength, 0.0);
}

#[test]
fn default_wind_preserves_editable_time_scales() {
    let default_wind = GrassWind::default();

    assert_eq!(default_wind.sway_frequency, 0.25);
    assert_eq!(default_wind.sway_speed, 0.35);
    assert_eq!(default_wind.gust_frequency, 0.12);
    assert_eq!(default_wind.gust_speed, 0.08);
    assert_eq!(default_wind.flutter_speed, 2.5);
}
