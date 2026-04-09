use bevy::prelude::*;
use grass::GrassPatchDiagnostics;
use saddle_bevy_e2e::action::Action;

use crate::LabState;

pub fn move_camera(world: &mut World, translation: Vec3, look_at: Vec3) {
    let state = *world.resource::<LabState>();
    let mut entity = world.entity_mut(state.camera);
    entity.insert(Transform::from_translation(translation).looking_at(look_at, Vec3::Y));
}

pub fn focus_camera_action(translation: Vec3, look_at: Vec3) -> Action {
    Action::Custom(Box::new(move |world| move_camera(world, translation, look_at)))
}

pub fn patch(world: &World, name: &str) -> Option<GrassPatchDiagnostics> {
    world
        .resource::<grass::GrassDiagnostics>()
        .patches
        .iter()
        .find(|patch| patch.name == name)
        .cloned()
}

pub fn patch_has_blades(world: &World, name: &str) -> bool {
    patch(world, name).is_some_and(|patch| patch.blade_count > 0)
}

pub fn patch_has_visible_chunks(world: &World, name: &str) -> bool {
    patch(world, name).is_some_and(|patch| patch.visible_chunk_count > 0)
}

pub fn patch_is_dirty(world: &World, name: &str) -> bool {
    patch(world, name).is_some_and(|patch| patch.dirty)
}

pub fn walker_translation(world: &World) -> Option<Vec3> {
    let state = *world.resource::<LabState>();
    world
        .get::<Transform>(state.strip_walker)
        .map(|transform| transform.translation)
}
