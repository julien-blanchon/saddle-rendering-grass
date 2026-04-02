use bevy::prelude::*;
use bevy::shader::Shader;

use crate::{GrassConfig, GrassPatch, GrassPlugin};

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TestState {
    #[default]
    Active,
}

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        bevy::asset::AssetPlugin::default(),
        bevy::state::app::StatesPlugin,
        bevy::gizmos::GizmoPlugin,
    ));
    app.init_asset::<Shader>();
    app.init_asset::<Mesh>();
    app.init_asset::<Image>();
    app.init_state::<TestState>();
    app.add_plugins(GrassPlugin::new(
        OnEnter(TestState::Active),
        OnExit(TestState::Active),
        Update,
    ));
    app
}

#[test]
fn plugin_registers_runtime_resources() {
    let mut app = test_app();
    app.update();

    assert!(app.world().contains_resource::<crate::GrassWind>());
    assert!(app.world().contains_resource::<crate::GrassDiagnostics>());
}

#[test]
fn activate_marks_runtime_active() {
    let mut app = test_app();
    app.world_mut()
        .spawn((GrassPatch::default(), GrassConfig::default()));
    app.update();

    assert!(
        app.world()
            .resource::<crate::resources::GrassRuntimeState>()
            .active
    );
}

#[test]
fn grass_patch_requires_default_config() {
    let mut app = test_app();
    let patch = app.world_mut().spawn(GrassPatch::default()).id();

    assert!(
        app.world().get::<GrassConfig>(patch).is_some(),
        "spawning GrassPatch alone should provide a default GrassConfig"
    );
}
