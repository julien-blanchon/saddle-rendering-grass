use bevy::prelude::*;

#[derive(Message, Clone, Copy, Debug)]
pub struct GrassRebuildRequest {
    pub patch: Entity,
}
