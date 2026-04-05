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
        ..default()
    };

    let resolved = fallback.resolved_from_world_sample(&bridge, &sample);

    assert_eq!(resolved.direction, Vec2::Y);
    assert!(resolved.sway_strength > fallback.sway_strength);
    assert!(resolved.gust_strength > fallback.gust_strength);
    assert!(resolved.flutter_strength > fallback.flutter_strength);
}

#[test]
fn wind_presets_increase_in_intensity() {
    let calm = GrassWind::calm();
    let breezy = GrassWind::breezy();
    let windy = GrassWind::windy();
    let storm = GrassWind::storm();

    assert!(calm.sway_strength < breezy.sway_strength);
    assert!(breezy.sway_strength < windy.sway_strength);
    assert!(windy.sway_strength < storm.sway_strength);

    assert!(calm.gust_strength < breezy.gust_strength);
    assert!(breezy.gust_strength < windy.gust_strength);
    assert!(windy.gust_strength < storm.gust_strength);

    assert!(calm.sway_speed < breezy.sway_speed);
    assert!(breezy.sway_speed < windy.sway_speed);
    assert!(windy.sway_speed < storm.sway_speed);

    assert!(calm.flutter_speed < breezy.flutter_speed);
    assert!(breezy.flutter_speed < windy.flutter_speed);
    assert!(windy.flutter_speed < storm.flutter_speed);
}

#[test]
fn default_wind_is_calm() {
    let default_wind = GrassWind::default();
    let calm = GrassWind::calm();
    assert_eq!(default_wind.sway_strength, calm.sway_strength);
    assert_eq!(default_wind.sway_speed, calm.sway_speed);
    assert_eq!(default_wind.flutter_speed, calm.flutter_speed);
}
