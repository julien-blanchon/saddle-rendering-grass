use bevy::ecs::schedule::ScheduleLabel;
use bevy::mesh::Mesh3d;
use bevy::prelude::*;
use bevy::shader::Shader;

use grass::{
    GrassChunking, GrassConfig, GrassDiagnostics, GrassPatch, GrassPlugin, GrassRebuildRequest,
    GrassSurface, GrassSystems,
};

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct Activate;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct Deactivate;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct Tick;

fn test_app() -> App {
    let mut app = App::new();
    app.init_schedule(Activate);
    app.init_schedule(Deactivate);
    app.init_schedule(Tick);
    app.add_plugins((
        MinimalPlugins,
        bevy::asset::AssetPlugin::default(),
        bevy::gizmos::GizmoPlugin,
    ));
    app.init_asset::<Shader>();
    app.init_asset::<Mesh>();
    app.init_asset::<Image>();
    app.configure_sets(Tick, GrassSystems::Prepare.before(GrassSystems::Animate));
    app.add_plugins(GrassPlugin::new(Activate, Deactivate, Tick));
    app
}

fn patch_diagnostics(app: &App, name: &str) -> grass::GrassPatchDiagnostics {
    app.world()
        .resource::<GrassDiagnostics>()
        .patches
        .iter()
        .find(|patch| patch.name == name)
        .cloned()
        .unwrap_or_else(|| panic!("missing diagnostics for patch '{name}'"))
}

#[test]
fn plugin_builds_with_custom_schedules() {
    let mut app = test_app();
    app.world_mut()
        .spawn((GrassPatch::default(), GrassConfig::default()));

    app.world_mut().run_schedule(Activate);
    app.world_mut().run_schedule(Tick);

    assert!(app.world().contains_resource::<grass::GrassDiagnostics>());
}

#[test]
fn deactivate_schedule_cleans_up_generated_children() {
    let mut app = test_app();

    let patch = app
        .world_mut()
        .spawn((GrassPatch::default(), GrassConfig::default()))
        .id();

    app.world_mut().run_schedule(Activate);
    app.world_mut().run_schedule(Tick);

    let child_count = app
        .world()
        .get::<Children>(patch)
        .map_or(0, |children| children.len());
    assert!(
        child_count > 0,
        "grass patch should generate chunk children"
    );

    app.world_mut().run_schedule(Deactivate);

    let remaining_children = app
        .world()
        .get::<Children>(patch)
        .map_or(0, |children| children.len());
    assert_eq!(
        remaining_children, 0,
        "deactivate schedule should despawn generated chunk children"
    );
}

#[test]
fn rebuild_request_respawns_generated_children() {
    let mut app = test_app();
    let patch = app
        .world_mut()
        .spawn((
            Name::new("Rebuild Patch"),
            GrassPatch::default(),
            GrassConfig::default(),
        ))
        .id();

    app.world_mut().run_schedule(Activate);
    app.world_mut().run_schedule(Tick);

    let initial_children = app
        .world()
        .get::<Children>(patch)
        .expect("grass patch should generate child chunks")
        .iter()
        .collect::<Vec<_>>();

    app.world_mut().write_message(GrassRebuildRequest { patch });
    app.world_mut().run_schedule(Tick);

    let rebuilt_children = app
        .world()
        .get::<Children>(patch)
        .expect("grass patch should regenerate child chunks")
        .iter()
        .collect::<Vec<_>>();

    assert!(!initial_children.is_empty());
    assert!(!rebuilt_children.is_empty());
    assert_ne!(
        initial_children, rebuilt_children,
        "manual rebuild requests should respawn chunk children"
    );
}

#[test]
fn mesh_surface_patch_rebuilds_when_source_mesh_handle_changes() {
    let mut app = test_app();

    let (large_surface, small_surface) = {
        let mut meshes = app.world_mut().resource_mut::<Assets<Mesh>>();
        (
            meshes.add(Mesh::from(Plane3d::default().mesh().size(8.0, 8.0))),
            meshes.add(Mesh::from(Plane3d::default().mesh().size(2.0, 2.0))),
        )
    };

    let source = app
        .world_mut()
        .spawn((
            Name::new("Surface Source"),
            Mesh3d(large_surface.clone()),
            Transform::default(),
            GlobalTransform::default(),
        ))
        .id();

    app.world_mut().spawn((
        Name::new("Mesh Patch"),
        GrassPatch {
            surface: GrassSurface::Mesh(source),
            chunking: GrassChunking {
                chunk_size: Vec2::splat(4.0),
            },
            ..default()
        },
        GrassConfig {
            density_per_square_unit: 3.0,
            max_blades_per_chunk: 2_500,
            ..default()
        },
    ));

    app.world_mut().run_schedule(Activate);
    app.world_mut().run_schedule(Tick);
    let initial_blades = patch_diagnostics(&app, "Mesh Patch").blade_count;

    app.world_mut()
        .entity_mut(source)
        .insert(Mesh3d(small_surface.clone()));
    app.world_mut().run_schedule(Tick);
    let rebuilt_blades = patch_diagnostics(&app, "Mesh Patch").blade_count;

    assert!(
        rebuilt_blades < initial_blades,
        "swapping to a smaller mesh surface should rebuild with fewer blades"
    );
}
