mod support;

use bevy::prelude::*;
use grass::{GrassDiagnostics, GrassDebugSettings, GrassRebuildRequest, GrassWind};
use saddle_bevy_e2e::{
    action::Action,
    actions::{assertions, inspect},
    scenario::Scenario,
};

use crate::LabState;

pub fn list_scenarios() -> Vec<&'static str> {
    vec![
        "grass_smoke",
        "grass_wind_showcase",
        "grass_lod_showcase",
        "grass_interaction_strip",
        "grass_rebuild_request",
        "grass_debug_settings",
        "grass_scatter_filters",
        "grass_blade_shapes",
        "grass_stylized",
    ]
}

pub fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "grass_smoke" => Some(build_smoke()),
        "grass_wind_showcase" => Some(build_wind_showcase()),
        "grass_lod_showcase" => Some(build_lod_showcase()),
        "grass_interaction_strip" => Some(build_interaction_strip()),
        "grass_rebuild_request" => Some(build_rebuild_request()),
        "grass_debug_settings" => Some(build_debug_settings()),
        "grass_scatter_filters" => Some(build_scatter_filters()),
        "grass_blade_shapes" => Some(build_blade_shapes()),
        "grass_stylized" => Some(build_stylized()),
        _ => None,
    }
}

#[derive(Resource, Clone, Copy)]
struct WindSnapshot {
    direction: Vec2,
    sway_strength: f32,
}

#[derive(Resource, Clone, Copy)]
struct LodSnapshot {
    visible_blades: u32,
    lod0_chunks: u32,
}

#[derive(Resource, Clone, Copy)]
struct WalkerSnapshot {
    position: Vec3,
}

fn build_smoke() -> Scenario {
    Scenario::builder("grass_smoke")
        .description(
            "Boot the crate-local grass lab, verify all patch types generate, and capture an overview frame.",
        )
        .then(Action::WaitFrames(75))
        .then(assertions::resource_exists::<LabState>("lab state exists"))
        .then(assertions::resource_exists::<GrassDiagnostics>(
            "diagnostics resource exists",
        ))
        .then(assertions::resource_satisfies::<GrassDiagnostics>(
            "runtime active",
            |diagnostics| diagnostics.runtime_active,
        ))
        .then(assertions::resource_satisfies::<GrassDiagnostics>(
            "multiple patches generate",
            |diagnostics| diagnostics.active_patches >= 4,
        ))
        .then(assertions::resource_satisfies::<GrassDiagnostics>(
            "visible grass is populated",
            |diagnostics| diagnostics.visible_blades > 2_000,
        ))
        .then(assertions::custom("mesh-aligned slope patch generated", |world| {
            support::patch_has_blades(world, "Slope Meadow")
                && support::patch(world, "Slope Meadow")
                    .is_some_and(|patch| patch.chunk_count > 0)
        }))
        .then(assertions::custom("interaction strip generated", |world| {
            support::patch_has_visible_chunks(world, "Interaction Strip")
        }))
        .then(inspect::log_resource::<GrassDiagnostics>("smoke diagnostics"))
        .then(Action::Screenshot("grass_smoke".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("grass_smoke summary"))
        .build()
}

fn build_wind_showcase() -> Scenario {
    Scenario::builder("grass_wind_showcase")
        .description(
            "Capture two wind checkpoints and verify the animated wind resource actually changes over time.",
        )
        .then(support::focus_camera_action(
            Vec3::new(-4.0, 5.5, 12.0),
            Vec3::new(0.0, 0.8, -6.0),
        ))
        .then(Action::WaitFrames(45))
        .then(Action::Custom(Box::new(|world| {
            let wind = world.resource::<GrassWind>();
            world.insert_resource(WindSnapshot {
                direction: wind.direction,
                sway_strength: wind.sway_strength,
            });
        })))
        .then(Action::Screenshot("grass_wind_a".into()))
        .then(Action::WaitFrames(1))
        .then(Action::WaitFrames(90))
        .then(assertions::custom("wind resource changes over time", |world| {
            let snapshot = world.resource::<WindSnapshot>();
            let wind = world.resource::<GrassWind>();
            wind.direction.distance(snapshot.direction) > 0.05
                || (wind.sway_strength - snapshot.sway_strength).abs() > 0.02
        }))
        .then(assertions::resource_satisfies::<GrassDiagnostics>(
            "wind showcase keeps visible grass",
            |diagnostics| diagnostics.visible_blades > 800,
        ))
        .then(inspect::log_resource::<GrassWind>("animated wind"))
        .then(Action::Screenshot("grass_wind_b".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("grass_wind_showcase summary"))
        .build()
}

fn build_lod_showcase() -> Scenario {
    Scenario::builder("grass_lod_showcase")
        .description(
            "Move between near and far views of the large meadow and verify visible blade density drops with distance.",
        )
        .then(support::focus_camera_action(
            Vec3::new(0.0, 6.5, -36.0),
            Vec3::new(0.0, 0.0, -52.0),
        ))
        .then(Action::WaitFrames(75))
        .then(Action::Custom(Box::new(|world| {
            let patch = support::patch(world, "Open Meadow LOD")
                .expect("Open Meadow LOD patch should exist");
            world.insert_resource(LodSnapshot {
                visible_blades: patch.visible_blade_count,
                lod0_chunks: patch.visible_lod_chunk_counts.first().copied().unwrap_or(0),
            });
        })))
        .then(Action::Screenshot("grass_lod_near".into()))
        .then(Action::WaitFrames(1))
        .then(support::focus_camera_action(
            Vec3::new(0.0, 10.0, 16.0),
            Vec3::new(0.0, 0.0, -52.0),
        ))
        .then(Action::WaitFrames(120))
        .then(assertions::custom("far view reduces visible blades", |world| {
            let snapshot = world.resource::<LodSnapshot>();
            let Some(patch) = support::patch(world, "Open Meadow LOD") else {
                return false;
            };
            patch.visible_blade_count < snapshot.visible_blades
        }))
        .then(assertions::custom("far view reduces visible LOD0 chunks", |world| {
            let snapshot = world.resource::<LodSnapshot>();
            let Some(patch) = support::patch(world, "Open Meadow LOD") else {
                return false;
            };
            let lod0 = patch.visible_lod_chunk_counts.first().copied().unwrap_or(0);
            let lod2 = patch.visible_lod_chunk_counts.get(2).copied().unwrap_or(0);
            lod0 < snapshot.lod0_chunks && lod2 > 0
        }))
        .then(inspect::log_resource::<GrassDiagnostics>("lod diagnostics"))
        .then(Action::Screenshot("grass_lod_far".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("grass_lod_showcase summary"))
        .build()
}

fn build_interaction_strip() -> Scenario {
    Scenario::builder("grass_interaction_strip")
        .description(
            "Focus on the interaction strip, verify the moving bend zone travels through the grass, and capture two checkpoints.",
        )
        .then(support::focus_camera_action(
            Vec3::new(14.0, 5.5, 9.0),
            Vec3::new(14.0, 0.5, -8.0),
        ))
        .then(Action::WaitFrames(45))
        .then(Action::Custom(Box::new(|world| {
            let position =
                support::walker_translation(world).expect("strip walker should exist for interaction lab");
            world.insert_resource(WalkerSnapshot { position });
        })))
        .then(Action::Screenshot("grass_interaction_a".into()))
        .then(Action::WaitFrames(1))
        .then(Action::WaitFrames(100))
        .then(assertions::resource_satisfies::<GrassDiagnostics>(
            "interaction zone stays active",
            |diagnostics| diagnostics.interaction_zones == 1,
        ))
        .then(assertions::custom("interaction walker moves along strip", |world| {
            let snapshot = world.resource::<WalkerSnapshot>();
            let Some(position) = support::walker_translation(world) else {
                return false;
            };
            position.distance(snapshot.position) > 4.0
        }))
        .then(assertions::custom("interaction strip remains visible", |world| {
            support::patch(world, "Interaction Strip")
                .is_some_and(|patch| patch.visible_blade_count > 200)
        }))
        .then(Action::Screenshot("grass_interaction_b".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("grass_interaction_strip summary"))
        .build()
}

fn build_rebuild_request() -> Scenario {
    Scenario::builder("grass_rebuild_request")
        .description(
            "Record blade count on the courtyard patch, send a GrassRebuildRequest for it, and \
             verify the patch is marked dirty then regenerates a comparable blade count.",
        )
        .then(Action::WaitFrames(90))
        .then(assertions::resource_satisfies::<GrassDiagnostics>(
            "courtyard patch is generated before rebuild",
            |diagnostics| {
                diagnostics
                    .patches
                    .iter()
                    .any(|p| p.name == "Courtyard Turf" && p.blade_count > 0)
            },
        ))
        .then(Action::Custom(Box::new(|world: &mut World| {
            let state = *world.resource::<LabState>();
            world.write_message(GrassRebuildRequest { patch: state.courtyard_patch });
        })))
        .then(Action::WaitFrames(4))
        .then(assertions::custom("courtyard patch marked dirty after rebuild request", |world| {
            support::patch_is_dirty(world, "Courtyard Turf")
        }))
        .then(Action::WaitFrames(60))
        .then(assertions::custom("courtyard patch regenerated after rebuild request", |world| {
            support::patch(world, "Courtyard Turf")
                .is_some_and(|patch| patch.blade_count > 0 && !patch.dirty)
        }))
        .then(inspect::log_resource::<GrassDiagnostics>("rebuild diagnostics"))
        .then(Action::Screenshot("grass_rebuild_request".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("grass_rebuild_request summary"))
        .build()
}

fn build_debug_settings() -> Scenario {
    Scenario::builder("grass_debug_settings")
        .description(
            "Enable patch and chunk bound gizmos via GrassDebugSettings, capture the overlay, \
             then disable all debug flags and verify the resource reflects the cleared state.",
        )
        .then(Action::WaitFrames(60))
        .then(Action::Custom(Box::new(|world: &mut World| {
            support::move_camera(
                world,
                Vec3::new(-18.0, 12.0, 30.0),
                Vec3::new(0.0, 1.0, -20.0),
            );
            let mut settings = world.resource_mut::<GrassDebugSettings>();
            settings.draw_patch_bounds = true;
            settings.draw_chunk_bounds = true;
            settings.draw_interaction_zones = true;
        })))
        .then(Action::WaitFrames(15))
        .then(assertions::resource_satisfies::<GrassDebugSettings>(
            "patch bounds flag is set",
            |settings| settings.draw_patch_bounds,
        ))
        .then(assertions::resource_satisfies::<GrassDebugSettings>(
            "chunk bounds flag is set",
            |settings| settings.draw_chunk_bounds,
        ))
        .then(assertions::resource_satisfies::<GrassDebugSettings>(
            "interaction zones flag is set",
            |settings| settings.draw_interaction_zones,
        ))
        .then(Action::Screenshot("grass_debug_settings_on".into()))
        .then(Action::WaitFrames(1))
        .then(Action::Custom(Box::new(|world: &mut World| {
            let mut settings = world.resource_mut::<GrassDebugSettings>();
            settings.draw_patch_bounds = false;
            settings.draw_chunk_bounds = false;
            settings.draw_interaction_zones = false;
        })))
        .then(Action::WaitFrames(8))
        .then(assertions::resource_satisfies::<GrassDebugSettings>(
            "all debug flags cleared",
            |settings| {
                !settings.draw_patch_bounds
                    && !settings.draw_chunk_bounds
                    && !settings.draw_interaction_zones
            },
        ))
        .then(Action::Screenshot("grass_debug_settings_off".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("grass_debug_settings summary"))
        .build()
}

fn build_scatter_filters() -> Scenario {
    Scenario::builder("grass_scatter_filters")
        .description(
            "Spawn a slope-filtered patch on a steep ramp, an altitude-filtered patch, \
             and an exclusion-zone patch. Verify filters reject blades correctly.",
        )
        .then(Action::Custom(Box::new(|world| {
            // Spawn a steep ramp entity for slope filter testing
            let steep_mesh = world
                .resource_mut::<Assets<Mesh>>()
                .add(bevy::prelude::Cuboid::new(8.0, 0.3, 5.0));
            let steep_material = world
                .resource_mut::<Assets<StandardMaterial>>()
                .add(StandardMaterial {
                    base_color: Color::srgb(0.3, 0.26, 0.2),
                    ..default()
                });
            let steep_ramp = world
                .spawn((
                    Name::new("Steep Ramp"),
                    Mesh3d(steep_mesh),
                    MeshMaterial3d(steep_material),
                    Transform {
                        translation: Vec3::new(22.0, 3.0, 6.0),
                        rotation: Quat::from_rotation_x(-1.1), // ~63° slope
                        ..default()
                    },
                    Visibility::default(),
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
                ))
                .id();

            // Spawn patch with slope filter that rejects > 30°
            let config = grass::GrassConfig {
                density_per_square_unit: 20.0,
                archetypes: vec![grass::GrassArchetype::default()],
                scatter_filter: grass::GrassScatterFilter {
                    slope_range_degrees: Some((0.0, 30.0)),
                    ..default()
                },
                ..default()
            };
            world.spawn((
                Name::new("Slope Filter Test"),
                grass::GrassPatch {
                    seed: 777,
                    surface: grass::GrassSurface::Mesh(steep_ramp),
                    ..default()
                },
                config,
                Transform::default(),
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ));

            // Altitude-filtered patch at height 50 (range 0..10 → should be empty)
            let altitude_config = grass::GrassConfig {
                density_per_square_unit: 20.0,
                archetypes: vec![grass::GrassArchetype::default()],
                scatter_filter: grass::GrassScatterFilter {
                    altitude_range: Some((0.0, 10.0)),
                    ..default()
                },
                ..default()
            };
            world.spawn((
                Name::new("Altitude Filter Test"),
                grass::GrassPatch {
                    half_size: Vec2::new(3.0, 3.0),
                    seed: 888,
                    ..default()
                },
                altitude_config,
                Transform::from_xyz(30.0, 50.0, 6.0),
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ));
        })))
        .then(Action::WaitFrames(90))
        .then(assertions::custom("slope filter rejects steep ramp blades", |world| {
            support::patch(world, "Slope Filter Test")
                .is_some_and(|patch| patch.blade_count == 0)
        }))
        .then(assertions::custom("altitude filter rejects high-altitude blades", |world| {
            support::patch(world, "Altitude Filter Test")
                .is_some_and(|patch| patch.blade_count == 0)
        }))
        .then(Action::Screenshot("grass_scatter_filters".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("grass_scatter_filters summary"))
        .build()
}

fn build_blade_shapes() -> Scenario {
    Scenario::builder("grass_blade_shapes")
        .description(
            "Spawn one patch per blade shape (Strip, CrossBillboard, FlatCard, SingleTriangle) \
             and verify each generates visible geometry.",
        )
        .then(Action::Custom(Box::new(|world| {
            let shapes = [
                ("Shape: Strip", grass::BladeShape::Strip),
                ("Shape: Cross", grass::BladeShape::CrossBillboard),
                ("Shape: Card", grass::BladeShape::FlatCard),
                ("Shape: Triangle", grass::BladeShape::SingleTriangle),
            ];

            for (i, (name, shape)) in shapes.iter().enumerate() {
                let archetype = grass::GrassArchetype {
                    debug_name: name.to_string(),
                    blade_height: Vec2::new(0.4, 0.8),
                    blade_width: Vec2::new(0.04, 0.08),
                    root_color: Color::srgb(0.15, 0.35, 0.10),
                    tip_color: Color::srgb(0.42, 0.80, 0.28),
                    blade_shape: *shape,
                    ..default()
                };
                world.spawn((
                    Name::new(name.to_string()),
                    grass::GrassPatch {
                        half_size: Vec2::new(2.5, 2.5),
                        seed: 500 + i as u64,
                        ..default()
                    },
                    grass::GrassConfig {
                        density_per_square_unit: 20.0,
                        archetypes: vec![archetype],
                        ..default()
                    },
                    Transform::from_xyz(-20.0 + i as f32 * 7.0, 0.0, 18.0),
                    GlobalTransform::default(),
                    Visibility::default(),
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
                ));
            }

            support::move_camera(
                world,
                Vec3::new(-8.0, 6.0, 30.0),
                Vec3::new(-4.0, 0.5, 18.0),
            );
        })))
        .then(Action::WaitFrames(90))
        .then(assertions::custom("Strip shape generates blades", |world| {
            support::patch_has_blades(world, "Shape: Strip")
        }))
        .then(assertions::custom("CrossBillboard shape generates blades", |world| {
            support::patch_has_blades(world, "Shape: Cross")
        }))
        .then(assertions::custom("FlatCard shape generates blades", |world| {
            support::patch_has_blades(world, "Shape: Card")
        }))
        .then(assertions::custom("SingleTriangle shape generates blades", |world| {
            support::patch_has_blades(world, "Shape: Triangle")
        }))
        .then(Action::Screenshot("grass_blade_shapes".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("grass_blade_shapes summary"))
        .build()
}

fn build_stylized() -> Scenario {
    Scenario::builder("grass_stylized")
        .description(
            "Spawn anime-style grass with ground-normal projection and single-triangle blades. \
             Verify it generates and renders visible geometry.",
        )
        .then(Action::Custom(Box::new(|world| {
            let anime_archetype = grass::GrassArchetype {
                debug_name: "Anime".into(),
                blade_height: Vec2::new(0.6, 1.2),
                blade_width: Vec2::new(0.08, 0.16),
                root_color: Color::srgb(0.12, 0.52, 0.18),
                tip_color: Color::srgb(0.40, 0.88, 0.32),
                color_variation: 0.04,
                blade_shape: grass::BladeShape::SingleTriangle,
                normal_source: grass::GrassNormalSource::GroundNormal,
                ..default()
            };
            world.spawn((
                Name::new("Anime Grass"),
                grass::GrassPatch {
                    half_size: Vec2::new(4.0, 4.0),
                    seed: 999,
                    ..default()
                },
                grass::GrassConfig {
                    density_per_square_unit: 16.0,
                    archetypes: vec![anime_archetype],
                    ..default()
                },
                Transform::from_xyz(-25.0, 0.0, 18.0),
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ));

            support::move_camera(
                world,
                Vec3::new(-25.0, 5.0, 28.0),
                Vec3::new(-25.0, 0.5, 18.0),
            );
        })))
        .then(Action::WaitFrames(90))
        .then(assertions::custom("anime grass generates visible blades", |world| {
            support::patch_has_blades(world, "Anime Grass")
                && support::patch(world, "Anime Grass")
                    .is_some_and(|p| p.visible_blade_count > 0)
        }))
        .then(Action::Screenshot("grass_stylized".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("grass_stylized summary"))
        .build()
}
