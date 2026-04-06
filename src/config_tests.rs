use super::*;

#[test]
fn default_archetype_is_a_neutral_base_profile() {
    let archetype = GrassArchetype::default();

    assert_eq!(archetype.debug_name, "Base");
    assert_eq!(archetype.root_color, Color::srgb(0.36, 0.34, 0.30));
    assert_eq!(archetype.tip_color, Color::srgb(0.63, 0.60, 0.54));
    assert!(archetype.lean.x < 0.0);
    assert!(archetype.lean.y > 0.0);
}

#[test]
fn default_config_uses_the_base_archetype() {
    let config = GrassConfig::default();

    assert_eq!(config.archetypes.len(), 1);
    assert_eq!(config.archetypes[0].debug_name, "Base");
}
