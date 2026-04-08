use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

/// World-space interaction map that actors stamp into each frame.
///
/// The map covers a square region centered on `center` (typically the camera).
/// RGBA channels encode per-texel interaction state:
/// - **R**: bend direction X (signed, -1..1, 0.5 = neutral)
/// - **G**: bend direction Z (signed, -1..1, 0.5 = neutral)
/// - **B**: flatten amount (0 = none, 1 = fully flattened)
/// - **A**: hide mask (0 = visible, 1 = hidden/cut)
///
/// Each frame:
/// 1. The map decays toward neutral based on `recovery_speed`
/// 2. Active `GrassInteractionActor` entities stamp their footprint
/// 3. The texture is uploaded and sampled by the grass vertex shader
#[derive(Resource, Clone, Debug, Reflect)]
#[reflect(Resource, Default)]
pub struct GrassInteractionMap {
    /// World-space center of the map (typically follows the camera).
    pub center: Vec2,
    /// Half-size of the world region covered by the map (in world units).
    /// The map covers `center ± half_extent` on X and Z.
    pub half_extent: f32,
    /// Texture resolution (width = height). Higher = finer detail, more CPU cost.
    /// Typical values: 128, 256, 512.
    pub resolution: u32,
    /// How fast interaction effects recover per second.
    /// `0.0` = permanent trails, `1.0` = full recovery in ~1s, `5.0` = very fast snap-back.
    pub recovery_speed: f32,
    /// Whether the map center automatically follows the primary camera.
    pub follow_camera: bool,
    /// Whether the interaction map is active. When false, falls back to legacy zones.
    pub enabled: bool,
}

impl Default for GrassInteractionMap {
    fn default() -> Self {
        Self {
            center: Vec2::ZERO,
            half_extent: 30.0,
            resolution: 256,
            recovery_speed: 2.0,
            follow_camera: true,
            enabled: true,
        }
    }
}

impl GrassInteractionMap {
    /// Converts a world XZ position to UV coordinates in the map (0..1).
    /// Returns `None` if the position is outside the map region.
    pub fn world_to_uv(&self, world_xz: Vec2) -> Option<Vec2> {
        let relative = world_xz - self.center;
        let uv = (relative / (self.half_extent * 2.0)) + Vec2::splat(0.5);
        if uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0 {
            None
        } else {
            Some(uv)
        }
    }

    /// Converts a world XZ position to pixel coordinates.
    fn world_to_pixel(&self, world_xz: Vec2) -> Option<(usize, usize)> {
        let uv = self.world_to_uv(world_xz)?;
        let x = (uv.x * (self.resolution - 1) as f32).round() as usize;
        let y = (uv.y * (self.resolution - 1) as f32).round() as usize;
        Some((x.min(self.resolution as usize - 1), y.min(self.resolution as usize - 1)))
    }
}

/// How an interaction actor affects grass.
#[derive(Clone, Debug, Reflect, PartialEq)]
pub enum GrassInteractionPolicy {
    /// Push blades away from the actor center. Grass recovers when actor moves.
    Bend {
        /// Bend strength (0..1 typical).
        strength: f32,
    },
    /// Push blades downward (flatten/trample).
    Flatten {
        /// Flatten amount (0..1).
        strength: f32,
    },
    /// Combined bend + flatten (most common for characters/vehicles).
    BendAndFlatten {
        bend_strength: f32,
        flatten_strength: f32,
    },
    /// Completely hide blades (cut/destroy). Does not recover unless `permanent` is false.
    Hide {
        /// If true, the hide effect doesn't decay (permanent cut).
        permanent: bool,
    },
}

impl Default for GrassInteractionPolicy {
    fn default() -> Self {
        Self::BendAndFlatten {
            bend_strength: 0.6,
            flatten_strength: 0.3,
        }
    }
}

/// An entity that interacts with grass by stamping into the interaction map.
///
/// Attach this to any entity with a `Transform` to make it affect nearby grass.
/// The interaction is applied as a circular footprint centered on the entity's XZ position.
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
#[require(Transform, GlobalTransform)]
pub struct GrassInteractionActor {
    /// Radius of the interaction footprint in world units.
    pub radius: f32,
    /// How the actor affects grass.
    pub policy: GrassInteractionPolicy,
    /// Edge falloff exponent. Higher = sharper edge, lower = softer gradient.
    /// `1.0` = linear, `2.0` = quadratic (default), `0.5` = sqrt (very soft).
    pub falloff: f32,
}

impl Default for GrassInteractionActor {
    fn default() -> Self {
        Self {
            radius: 1.4,
            policy: GrassInteractionPolicy::default(),
            falloff: 2.0,
        }
    }
}

/// Internal: the CPU-side pixel buffer for the interaction map.
#[derive(Resource)]
pub(crate) struct InteractionMapState {
    /// RGBA pixel data (R=bend_x, G=bend_z, B=flatten, A=hide).
    /// Neutral values: R=128, G=128, B=0, A=0.
    pub data: Vec<u8>,
    pub resolution: u32,
    /// Handle to the GPU texture.
    pub texture_handle: Handle<Image>,
    /// Previous frame's center — used to shift the map when camera moves.
    pub prev_center: Vec2,
}

impl InteractionMapState {
    pub fn new(resolution: u32, images: &mut Assets<Image>) -> Self {
        let size = (resolution * resolution * 4) as usize;
        let mut data = vec![0u8; size];
        // Initialize R,G to 128 (neutral bend direction)
        for pixel in 0..(resolution * resolution) as usize {
            data[pixel * 4] = 128;     // R: bend_x neutral
            data[pixel * 4 + 1] = 128; // G: bend_z neutral
            // B=0 (no flatten), A=0 (no hide)
        }

        let image = Image::new(
            Extent3d {
                width: resolution,
                height: resolution,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            data.clone(),
            TextureFormat::Rgba8Unorm,
            RenderAssetUsages::default(),
        );
        let texture_handle = images.add(image);

        Self {
            data,
            resolution,
            texture_handle,
            prev_center: Vec2::ZERO,
        }
    }
}

/// System: ensure InteractionMapState exists when GrassInteractionMap is present.
pub(crate) fn ensure_interaction_map_state(
    map: Option<Res<GrassInteractionMap>>,
    state: Option<Res<InteractionMapState>>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(map) = map else { return };
    if !map.enabled {
        return;
    }
    if state.is_some() {
        return;
    }
    let new_state = InteractionMapState::new(map.resolution, &mut images);
    commands.insert_resource(new_state);
}

/// System: follow camera with the interaction map center.
pub(crate) fn update_interaction_map_center(
    map: Option<ResMut<GrassInteractionMap>>,
    cameras: Query<&GlobalTransform, With<Camera3d>>,
) {
    let Some(mut map) = map else { return };
    if !map.enabled || !map.follow_camera {
        return;
    }
    if let Some(camera_transform) = cameras.iter().next() {
        let cam_pos = camera_transform.translation();
        map.center = Vec2::new(cam_pos.x, cam_pos.z);
    }
}

/// System: decay the interaction map toward neutral, shift when camera moves, then stamp actors.
pub(crate) fn stamp_interaction_map(
    time: Res<Time>,
    map: Option<Res<GrassInteractionMap>>,
    state: Option<ResMut<InteractionMapState>>,
    mut images: ResMut<Assets<Image>>,
    actors: Query<(&GrassInteractionActor, &GlobalTransform)>,
) {
    let Some(map) = map else { return };
    let Some(mut state) = state else { return };
    if !map.enabled {
        return;
    }

    let dt = time.delta_secs();
    let res = state.resolution as usize;

    // Handle map center shift (camera moved)
    let center_delta = map.center - state.prev_center;
    if center_delta.length_squared() > 0.01 {
        shift_map_data(&mut state.data, res, center_delta, map.half_extent);
        state.prev_center = map.center;
    }

    // Decay toward neutral
    let decay = (map.recovery_speed * dt).min(1.0);
    if decay > 0.0 {
        for pixel in 0..res * res {
            let idx = pixel * 4;
            // R,G decay toward 128 (neutral)
            state.data[idx] = lerp_u8(state.data[idx], 128, decay);
            state.data[idx + 1] = lerp_u8(state.data[idx + 1], 128, decay);
            // B (flatten) decays toward 0
            state.data[idx + 2] = lerp_u8(state.data[idx + 2], 0, decay);
            // A (hide) — only decay non-permanent (we mark permanent as 255)
            if state.data[idx + 3] < 255 {
                state.data[idx + 3] = lerp_u8(state.data[idx + 3], 0, decay);
            }
        }
    }

    // Stamp each actor's footprint
    for (actor, transform) in &actors {
        let actor_pos = Vec2::new(transform.translation().x, transform.translation().z);
        stamp_actor(&map, &mut state.data, res, actor_pos, actor);
    }

    // Upload to GPU
    if let Some(image) = images.get_mut(&state.texture_handle) {
        if let Some(ref mut data) = image.data {
            data.copy_from_slice(&state.data);
        }
    }
}

fn stamp_actor(
    map: &GrassInteractionMap,
    data: &mut [u8],
    res: usize,
    actor_pos: Vec2,
    actor: &GrassInteractionActor,
) {
    let radius = actor.radius.max(0.0);
    if radius <= 0.0 {
        return;
    }

    // Calculate pixel bounds of the actor's footprint
    let min_world = actor_pos - Vec2::splat(radius);
    let max_world = actor_pos + Vec2::splat(radius);

    let Some(min_px) = map.world_to_pixel(min_world) else { return };
    let Some(max_px) = map.world_to_pixel(max_world) else { return };

    let extent = map.half_extent * 2.0;
    let texel_size = extent / res as f32;

    for py in min_px.1..=max_px.1.min(res - 1) {
        for px in min_px.0..=max_px.0.min(res - 1) {
            // Convert pixel back to world position
            let world = Vec2::new(
                map.center.x - map.half_extent + (px as f32 + 0.5) * texel_size,
                map.center.y - map.half_extent + (py as f32 + 0.5) * texel_size,
            );
            let delta = world - actor_pos;
            let dist = delta.length();
            if dist >= radius {
                continue;
            }

            // Falloff: 1.0 at center, 0.0 at edge
            let normalized = 1.0 - dist / radius;
            let influence = normalized.powf(actor.falloff);

            let idx = (py * res + px) * 4;

            match &actor.policy {
                GrassInteractionPolicy::Bend { strength } => {
                    let dir = if dist > 0.001 {
                        delta.normalize()
                    } else {
                        Vec2::X
                    };
                    // Encode direction: 0.5 = neutral, 0 = -1, 1 = +1
                    let bend_x = (128.0 + dir.x * influence * strength * 127.0) as u8;
                    let bend_z = (128.0 + dir.y * influence * strength * 127.0) as u8;
                    // Max blend (strongest wins)
                    let existing_x = (data[idx] as f32 - 128.0).abs();
                    let new_x = (bend_x as f32 - 128.0).abs();
                    if new_x > existing_x {
                        data[idx] = bend_x;
                    }
                    let existing_z = (data[idx + 1] as f32 - 128.0).abs();
                    let new_z = (bend_z as f32 - 128.0).abs();
                    if new_z > existing_z {
                        data[idx + 1] = bend_z;
                    }
                }
                GrassInteractionPolicy::Flatten { strength } => {
                    let flatten = (influence * strength * 255.0) as u8;
                    data[idx + 2] = data[idx + 2].max(flatten);
                }
                GrassInteractionPolicy::BendAndFlatten {
                    bend_strength,
                    flatten_strength,
                } => {
                    let dir = if dist > 0.001 {
                        delta.normalize()
                    } else {
                        Vec2::X
                    };
                    let bend_x = (128.0 + dir.x * influence * bend_strength * 127.0) as u8;
                    let bend_z = (128.0 + dir.y * influence * bend_strength * 127.0) as u8;
                    let existing_x = (data[idx] as f32 - 128.0).abs();
                    let new_x = (bend_x as f32 - 128.0).abs();
                    if new_x > existing_x {
                        data[idx] = bend_x;
                    }
                    let existing_z = (data[idx + 1] as f32 - 128.0).abs();
                    let new_z = (bend_z as f32 - 128.0).abs();
                    if new_z > existing_z {
                        data[idx + 1] = bend_z;
                    }
                    let flatten = (influence * flatten_strength * 255.0) as u8;
                    data[idx + 2] = data[idx + 2].max(flatten);
                }
                GrassInteractionPolicy::Hide { permanent } => {
                    let hide_val = if *permanent { 255u8 } else { 254u8 };
                    let hide = (influence * hide_val as f32) as u8;
                    data[idx + 3] = data[idx + 3].max(hide);
                }
            }
        }
    }
}

/// Shift map data when the camera (map center) moves. Pixels that scroll out
/// are lost; new pixels enter as neutral.
fn shift_map_data(data: &mut [u8], res: usize, center_delta: Vec2, half_extent: f32) {
    let extent = half_extent * 2.0;
    let texel_size = extent / res as f32;
    let shift_x = (center_delta.x / texel_size).round() as i32;
    let shift_y = (center_delta.y / texel_size).round() as i32;

    if shift_x.unsigned_abs() as usize >= res || shift_y.unsigned_abs() as usize >= res {
        // Entire map scrolled out — reset to neutral
        for pixel in 0..res * res {
            data[pixel * 4] = 128;
            data[pixel * 4 + 1] = 128;
            data[pixel * 4 + 2] = 0;
            data[pixel * 4 + 3] = 0;
        }
        return;
    }

    // Copy with shift (simple row-by-row copy to avoid allocation)
    let mut temp = vec![0u8; data.len()];
    for pixel in 0..res * res {
        temp[pixel * 4] = 128;
        temp[pixel * 4 + 1] = 128;
    }

    for y in 0..res {
        for x in 0..res {
            let src_x = x as i32 + shift_x;
            let src_y = y as i32 + shift_y;
            if src_x >= 0 && src_x < res as i32 && src_y >= 0 && src_y < res as i32 {
                let src_idx = (src_y as usize * res + src_x as usize) * 4;
                let dst_idx = (y * res + x) * 4;
                temp[dst_idx..dst_idx + 4].copy_from_slice(&data[src_idx..src_idx + 4]);
            }
        }
    }

    data.copy_from_slice(&temp);
}

fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    let result = a as f32 + (b as f32 - a as f32) * t;
    result.round().clamp(0.0, 255.0) as u8
}

#[cfg(test)]
#[path = "interaction_tests.rs"]
mod tests;
