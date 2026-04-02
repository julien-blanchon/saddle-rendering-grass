mod support;

use bevy::prelude::*;
use saddle_bevy_e2e::{
    action::Action,
    actions::{assertions, inspect},
    scenario::Scenario,
};
use grass::{GrassDiagnostics, GrassWind};

use crate::LabState;

pub fn list_scenarios() -> Vec<&'static str> {
    vec![
        "grass_smoke",
        "grass_wind_showcase",
        "grass_lod_showcase",
        "grass_interaction_strip",
    ]
}

pub fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "grass_smoke" => Some(build_smoke()),
        "grass_wind_showcase" => Some(build_wind_showcase()),
        "grass_lod_showcase" => Some(build_lod_showcase()),
        "grass_interaction_strip" => Some(build_interaction_strip()),
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
            support::patch(world, "Slope Meadow")
                .is_some_and(|patch| patch.chunk_count > 0 && patch.blade_count > 0)
        }))
        .then(assertions::custom("interaction strip generated", |world| {
            support::patch(world, "Interaction Strip")
                .is_some_and(|patch| patch.visible_chunk_count > 0)
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
        .then(Action::Custom(Box::new(|world| {
            support::move_camera(
                world,
                Vec3::new(-4.0, 5.5, 12.0),
                Vec3::new(0.0, 0.8, -6.0),
            );
        })))
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
        .then(Action::Custom(Box::new(|world| {
            support::move_camera(
                world,
                Vec3::new(0.0, 6.5, -36.0),
                Vec3::new(0.0, 0.0, -52.0),
            );
        })))
        .then(Action::WaitFrames(75))
        .then(Action::Custom(Box::new(|world| {
            let patch = support::patch(world, "Open Meadow LOD")
                .expect("Open Meadow LOD patch should exist");
            world.insert_resource(LodSnapshot {
                visible_blades: patch.visible_blade_count,
                lod0_chunks: patch.visible_lod_chunk_counts[0],
            });
        })))
        .then(Action::Screenshot("grass_lod_near".into()))
        .then(Action::WaitFrames(1))
        .then(Action::Custom(Box::new(|world| {
            support::move_camera(
                world,
                Vec3::new(0.0, 10.0, 16.0),
                Vec3::new(0.0, 0.0, -52.0),
            );
        })))
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
            patch.visible_lod_chunk_counts[0] < snapshot.lod0_chunks
                && patch.visible_lod_chunk_counts[2] > 0
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
        .then(Action::Custom(Box::new(|world| {
            support::move_camera(
                world,
                Vec3::new(14.0, 5.5, 9.0),
                Vec3::new(14.0, 0.5, -8.0),
            );
        })))
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
