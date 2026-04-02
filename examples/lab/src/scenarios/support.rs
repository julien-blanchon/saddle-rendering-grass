use bevy::prelude::*;
use grass::GrassPatchDiagnostics;

use crate::LabState;

pub fn move_camera(world: &mut World, translation: Vec3, look_at: Vec3) {
    let state = *world.resource::<LabState>();
    let mut entity = world.entity_mut(state.camera);
    entity.insert(Transform::from_translation(translation).looking_at(look_at, Vec3::Y));
}

pub fn patch(world: &World, name: &str) -> Option<GrassPatchDiagnostics> {
    world
        .resource::<grass::GrassDiagnostics>()
        .patches
        .iter()
        .find(|patch| patch.name == name)
        .cloned()
}

pub fn walker_translation(world: &World) -> Option<Vec3> {
    let state = *world.resource::<LabState>();
    world
        .get::<Transform>(state.strip_walker)
        .map(|transform| transform.translation)
}
