use bevy::camera::visibility::VisibilityRange;
use bevy::light::NotShadowCaster;
use bevy::mesh::Mesh3d;
use bevy::pbr::MeshMaterial3d;
use bevy::prelude::*;

use crate::components::{GrassChunkRuntime, GrassGenerated, GrassPatchState};
use crate::config::{GrassConfig, GrassSurface};
use crate::interaction::{GrassInteractionMap, InteractionMapState};
use crate::lod::resolve_lod_bands;
use crate::materials::build_material;
use crate::mesh::build_chunk_mesh;
use crate::resources::{
    GrassDebugSettings, GrassDiagnostics, GrassInteractionSample, GrassInteractionState,
    GrassPatchDiagnostics, GrassRuntimeState, GrassWind, GrassWindBridge,
};
use crate::scatter::{mesh_chunk_samples, planar_chunk_samples};
use crate::surface::{ChunkLayout, SurfaceBake, bake_mesh_surface};
use crate::wind::{WindConfig, WindZone, WindZoneSnapshot, sample_wind_with_zones, snapshot_zone};
use crate::{GrassInteractionZone, GrassMaterial, GrassPatch, GrassRebuildRequest};

pub(crate) fn runtime_is_active(runtime: Res<GrassRuntimeState>) -> bool {
    runtime.active
}

pub(crate) fn activate_runtime(
    mut runtime: ResMut<GrassRuntimeState>,
    mut patches: Query<&mut GrassPatchState, With<GrassPatch>>,
) {
    runtime.active = true;
    for mut state in &mut patches {
        state.dirty = true;
    }
}

pub(crate) fn deactivate_runtime(
    mut commands: Commands,
    mut runtime: ResMut<GrassRuntimeState>,
    mut materials: ResMut<Assets<GrassMaterial>>,
    mut patches: Query<&mut GrassPatchState, With<GrassPatch>>,
    generated: Query<Entity, With<GrassGenerated>>,
) {
    runtime.active = false;

    for entity in &generated {
        commands.entity(entity).despawn();
    }

    for mut state in &mut patches {
        for handle in state.material_handles.drain(..) {
            materials.remove(handle.id());
        }
        state.generated_chunks.clear();
        state.dirty = true;
    }
}

pub(crate) fn mark_dirty_from_requests(
    mut requests: MessageReader<GrassRebuildRequest>,
    mut patches: Query<&mut GrassPatchState>,
) {
    for request in requests.read() {
        if let Ok(mut state) = patches.get_mut(request.patch) {
            state.dirty = true;
        }
    }
}

pub(crate) fn mark_dirty_from_component_changes(
    mut patches: Query<
        (&GrassPatch, &GrassConfig, &mut GrassPatchState),
        Or<(Added<GrassPatch>, Changed<GrassPatch>, Changed<GrassConfig>)>,
    >,
) {
    for (_, _, mut state) in &mut patches {
        state.dirty = true;
    }
}

pub(crate) fn mark_dirty_from_surface_changes(
    changed_sources: Query<Entity, Changed<Mesh3d>>,
    source_meshes: Query<&Mesh3d>,
    mut patches: Query<(&GrassPatch, &mut GrassPatchState)>,
) {
    let changed_sources = changed_sources.iter().collect::<Vec<_>>();

    for (patch, mut state) in &mut patches {
        let GrassSurface::Mesh(surface_entity) = patch.surface else {
            continue;
        };

        let source_missing = source_meshes.get(surface_entity).is_err();
        if changed_sources.contains(&surface_entity)
            || (source_missing && !state.generated_chunks.is_empty())
        {
            state.dirty = true;
        }
    }
}

pub(crate) fn mark_dirty_from_asset_changes(
    mut mesh_events: MessageReader<AssetEvent<Mesh>>,
    mut image_events: MessageReader<AssetEvent<Image>>,
    source_meshes: Query<&Mesh3d>,
    mut patches: Query<(Entity, &GrassPatch, &GrassConfig, &mut GrassPatchState)>,
) {
    let changed_meshes = mesh_events.read().map(asset_event_id).collect::<Vec<_>>();
    let changed_images = image_events.read().map(asset_event_id).collect::<Vec<_>>();

    if changed_meshes.is_empty() && changed_images.is_empty() {
        return;
    }

    for (_, patch, config, mut state) in &mut patches {
        let mut dirty = false;
        if let GrassSurface::Mesh(surface_entity) = patch.surface {
            if let Ok(mesh3d) = source_meshes.get(surface_entity) {
                dirty |= changed_meshes.contains(&mesh3d.0.id().untyped());
            }
        }
        if let Some(density) = &config.density_map {
            dirty |= changed_images.contains(&density.image.id().untyped());
        }
        for layer in &config.density_layers {
            dirty |= changed_images.contains(&layer.image.id().untyped());
        }

        if dirty {
            state.dirty = true;
        }
    }
}

fn asset_event_id<T: Asset>(event: &AssetEvent<T>) -> bevy::asset::UntypedAssetId {
    match event {
        AssetEvent::Added { id }
        | AssetEvent::Modified { id }
        | AssetEvent::Unused { id }
        | AssetEvent::LoadedWithDependencies { id }
        | AssetEvent::Removed { id } => id.untyped(),
    }
}

pub(crate) fn collect_interaction_zones(
    zones: Query<(&GrassInteractionZone, &GlobalTransform)>,
    mut state: ResMut<GrassInteractionState>,
) {
    state.zones.clear();
    state.zones.extend(
        zones
            .iter()
            .map(|(zone, transform)| GrassInteractionSample {
                center: transform.translation(),
                radius: zone.radius.max(0.0),
                bend_strength: zone.bend_strength,
                flatten_strength: zone.flatten_strength,
                falloff: zone.falloff.max(0.01),
            }),
    );
}

pub(crate) fn rebuild_dirty_patches(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<GrassMaterial>>,
    images: Res<Assets<Image>>,
    fallback_wind: Res<GrassWind>,
    wind_bridge: Res<GrassWindBridge>,
    time: Res<Time>,
    wind_config: Option<Res<WindConfig>>,
    wind_zones: Query<(&WindZone, &GlobalTransform)>,
    interactions: Res<GrassInteractionState>,
    interaction_map: Option<Res<GrassInteractionMap>>,
    interaction_map_state: Option<Res<InteractionMapState>>,
    source_meshes: Query<(&Mesh3d, &GlobalTransform)>,
    mut patches: Query<(
        Entity,
        &GrassPatch,
        &GrassConfig,
        &GlobalTransform,
        &mut GrassPatchState,
    )>,
) {
    let zone_snapshots = world_wind_snapshots(&wind_zones);

    for (patch_entity, patch, config, patch_global, mut state) in &mut patches {
        if !state.dirty {
            continue;
        }

        clear_patch_outputs(&mut commands, &mut materials, &mut state);

        let Some(build_plan) =
            build_surface_plan(patch, patch_global, config, &source_meshes, &meshes)
        else {
            state.dirty = false;
            continue;
        };

        let density_image = config
            .density_map
            .as_ref()
            .and_then(|density| images.get(&density.image));
        let density_layer_images: Vec<Option<&Image>> = config
            .density_layers
            .iter()
            .map(|layer| images.get(&layer.image))
            .collect();
        let lods = resolve_lod_bands(&config.lod);

        for archetype_index in 0..config.archetypes.len() {
            let archetype = &config.archetypes[archetype_index];
            if archetype.weight <= 0.0 {
                continue;
            }
            for lod in &lods {
                for chunk in &build_plan.chunks {
                    let samples = match &build_plan.kind {
                        SurfaceKind::Planar { patch_half_size } => planar_chunk_samples(
                            *patch_half_size,
                            chunk.min,
                            chunk.max,
                            patch.density_scale,
                            config,
                            archetype,
                            &lod.band,
                            density_image,
                            &density_layer_images,
                            patch.surface,
                            &build_plan.surface_global,
                            patch.seed
                                ^ ((chunk.coord.x as i64 as u64) << 32)
                                ^ chunk.coord.y as i64 as u64
                                ^ lod.index as u64
                                ^ archetype_index as u64,
                        ),
                        SurfaceKind::Mesh { bake } => mesh_chunk_samples(
                            bake.triangle_indices(chunk.coord),
                            bake,
                            patch.density_scale,
                            config,
                            archetype,
                            &lod.band,
                            density_image,
                            &density_layer_images,
                            &build_plan.surface_global,
                            patch.seed
                                ^ ((chunk.coord.x as i64 as u64) << 32)
                                ^ chunk.coord.y as i64 as u64
                                ^ lod.index as u64
                                ^ archetype_index as u64,
                        ),
                    };

                    if samples.is_empty() {
                        continue;
                    }

                    let Some(mesh) = build_chunk_mesh(
                        &samples,
                        archetype,
                        config,
                        lod.band.segments,
                        chunk.center,
                    ) else {
                        continue;
                    };
                    let chunk_world_center = build_plan
                        .surface_global
                        .transform_point(chunk.center + Vec3::Y * wind_bridge.sample_height_offset);
                    let resolved_wind = resolve_grass_wind(
                        &fallback_wind,
                        &wind_bridge,
                        wind_config.as_deref(),
                        &zone_snapshots,
                        chunk_world_center,
                        time.elapsed_secs(),
                    );
                    let imap = interaction_map.as_deref();
                    let imap_texture = interaction_map_state
                        .as_ref()
                        .map(|s| s.texture_handle.clone());
                    let material = build_material(
                        archetype,
                        &resolved_wind,
                        &interactions.zones,
                        imap,
                        imap_texture,
                    );
                    let material_handle = materials.add(material);
                    state.material_handles.push(material_handle.clone());
                    let mesh_handle = meshes.add(mesh);
                    let local_transform = chunk_local_transform(
                        patch_global,
                        &build_plan.surface_global,
                        chunk.center,
                    );

                    let mut entity = commands.spawn((
                        Name::new(format!(
                            "Grass Chunk / {} / LOD {} / {}",
                            patch_entity.index(),
                            lod.index,
                            archetype.debug_name
                        )),
                        GrassGenerated,
                        GrassChunkRuntime {
                            patch: patch_entity,
                            source_entity: build_plan.source_entity,
                            center_local: chunk.center,
                            size_local: chunk.max - chunk.min,
                            lod_index: lod.index,
                            blade_count: samples.len() as u32,
                        },
                        Mesh3d(mesh_handle),
                        MeshMaterial3d(material_handle.clone()),
                        local_transform,
                        Visibility::default(),
                        InheritedVisibility::default(),
                        ViewVisibility::default(),
                        VisibilityRange {
                            start_margin: lod.visibility_range.start_margin.clone(),
                            end_margin: lod.visibility_range.end_margin.clone(),
                            use_aabb: lod.visibility_range.use_aabb,
                        },
                        ChildOf(patch_entity),
                    ));

                    if !config.cast_shadows {
                        entity.insert(NotShadowCaster);
                    }

                    let chunk_entity = entity.id();
                    state.generated_chunks.push(chunk_entity);
                }
            }
        }

        state.dirty = false;
    }
}

pub(crate) fn sync_chunk_transforms(
    patch_globals: Query<&GlobalTransform, With<GrassPatch>>,
    source_globals: Query<&GlobalTransform>,
    mut chunks: Query<(&GrassChunkRuntime, &mut Transform)>,
) {
    for (chunk, mut transform) in &mut chunks {
        let Some(source_entity) = chunk.source_entity else {
            continue;
        };
        let (Ok(patch_global), Ok(surface_global)) = (
            patch_globals.get(chunk.patch),
            source_globals.get(source_entity),
        ) else {
            continue;
        };
        *transform = chunk_local_transform(patch_global, surface_global, chunk.center_local);
    }
}

pub(crate) fn sync_material_uniforms(
    fallback_wind: Res<GrassWind>,
    wind_bridge: Res<GrassWindBridge>,
    time: Res<Time>,
    wind_config: Option<Res<WindConfig>>,
    wind_zones: Query<(&WindZone, &GlobalTransform)>,
    interactions: Res<GrassInteractionState>,
    interaction_map: Option<Res<GrassInteractionMap>>,
    interaction_map_state: Option<Res<InteractionMapState>>,
    mut materials: ResMut<Assets<GrassMaterial>>,
    chunks: Query<
        (
            &GlobalTransform,
            &MeshMaterial3d<GrassMaterial>,
            Option<&ViewVisibility>,
        ),
        With<GrassChunkRuntime>,
    >,
) {
    let map_changed = interaction_map.as_ref().is_some_and(|m| m.is_changed());
    if !fallback_wind.is_changed()
        && !wind_bridge.is_changed()
        && wind_config
            .as_ref()
            .is_none_or(|config| !config.is_changed())
        && !interactions.is_changed()
        && !map_changed
        && wind_zones.iter().len() == 0
    {
        return;
    }

    let zone_snapshots = world_wind_snapshots(&wind_zones);
    let imap = interaction_map.as_deref();
    let imap_texture = interaction_map_state
        .as_ref()
        .map(|s| s.texture_handle.clone());

    // Track already-updated material IDs to avoid redundant writes when
    // multiple chunks share the same material handle.
    let mut updated_materials =
        std::collections::HashSet::with_capacity(chunks.iter().len().min(128));

    for (transform, material_handle, visibility) in &chunks {
        // Skip invisible chunks — no point updating their material.
        if visibility.is_some_and(|v| !v.get()) {
            continue;
        }

        let mat_id = material_handle.0.id();
        if !updated_materials.insert(mat_id) {
            continue; // Already updated this material
        }

        let Some(material) = materials.get_mut(&material_handle.0) else {
            continue;
        };
        let sample_point =
            transform.translation() + Vec3::Y * wind_bridge.sample_height_offset.max(0.0);
        let resolved_wind = resolve_grass_wind(
            &fallback_wind,
            &wind_bridge,
            wind_config.as_deref(),
            &zone_snapshots,
            sample_point,
            time.elapsed_secs(),
        );
        material.extension.uniform = crate::materials::GrassMaterialUniform::from_wind_and_zones(
            &resolved_wind,
            &interactions.zones,
            imap,
        );
        if let Some(ref tex) = imap_texture {
            material.extension.interaction_map = Some(tex.clone());
        }
    }
}

pub(crate) fn publish_diagnostics(
    runtime: Res<GrassRuntimeState>,
    wind_bridge: Res<GrassWindBridge>,
    wind_config: Option<Res<WindConfig>>,
    wind_zones: Query<&WindZone>,
    interactions: Res<GrassInteractionState>,
    patch_query: Query<(Entity, Option<&Name>, &GrassPatchState), With<GrassPatch>>,
    chunk_query: Query<(&GrassChunkRuntime, Option<&ViewVisibility>)>,
    mut diagnostics: ResMut<GrassDiagnostics>,
) {
    diagnostics.runtime_active = runtime.active;
    diagnostics.active_patches = patch_query.iter().len() as u32;
    diagnostics.active_chunks = chunk_query.iter().len() as u32;
    diagnostics.active_blades = chunk_query.iter().map(|(chunk, _)| chunk.blade_count).sum();
    diagnostics.visible_chunks = chunk_query
        .iter()
        .filter(|(_, visibility)| visibility.is_some_and(|visibility| visibility.get()))
        .count() as u32;
    diagnostics.visible_blades = chunk_query
        .iter()
        .filter_map(|(chunk, visibility)| {
            visibility
                .is_some_and(|visibility| visibility.get())
                .then_some(chunk.blade_count)
        })
        .sum();
    diagnostics.interaction_zones = interactions.zones.len() as u32;
    diagnostics.using_world_wind = wind_bridge.enabled && wind_config.is_some();
    diagnostics.wind_zone_count = wind_zones.iter().count() as u32;
    diagnostics.patches.clear();

    for (entity, name, state) in &patch_query {
        let mut entry = GrassPatchDiagnostics {
            entity,
            name: name.map_or_else(|| format!("Patch {}", entity.index()), Name::to_string),
            dirty: state.dirty,
            ..default()
        };

        for (chunk, visibility) in &chunk_query {
            if chunk.patch != entity {
                continue;
            }
            // Grow per-LOD arrays on demand to accommodate any band count
            let idx = chunk.lod_index;
            if idx >= entry.lod_chunk_counts.len() {
                entry.lod_chunk_counts.resize(idx + 1, 0);
                entry.lod_blade_counts.resize(idx + 1, 0);
                entry.visible_lod_chunk_counts.resize(idx + 1, 0);
                entry.visible_lod_blade_counts.resize(idx + 1, 0);
            }
            entry.chunk_count += 1;
            entry.blade_count += chunk.blade_count;
            entry.lod_chunk_counts[idx] += 1;
            entry.lod_blade_counts[idx] += chunk.blade_count;
            if visibility.is_some_and(|visibility| visibility.get()) {
                entry.visible_chunk_count += 1;
                entry.visible_blade_count += chunk.blade_count;
                entry.visible_lod_chunk_counts[idx] += 1;
                entry.visible_lod_blade_counts[idx] += chunk.blade_count;
            }
        }

        diagnostics.patches.push(entry);
    }
}

pub(crate) fn draw_debug_gizmos(
    debug: Res<GrassDebugSettings>,
    patch_query: Query<(&GrassPatch, &GlobalTransform)>,
    chunk_query: Query<(&GrassChunkRuntime, &GlobalTransform)>,
    interactions: Res<GrassInteractionState>,
    mut gizmos: Gizmos,
) {
    if debug.draw_patch_bounds {
        for (patch, transform) in &patch_query {
            if !matches!(patch.surface, GrassSurface::Planar) {
                continue;
            }
            let (scale, rotation, translation) = transform.to_scale_rotation_translation();
            let size = scale * Vec3::new(patch.half_size.x * 2.0, 0.02, patch.half_size.y * 2.0);
            gizmos.cube(
                Transform {
                    translation,
                    rotation,
                    scale: size,
                },
                Color::srgb(0.2, 0.8, 0.3),
            );
        }
    }

    if debug.draw_chunk_bounds {
        for (chunk, transform) in &chunk_query {
            let (scale, rotation, translation) = transform.to_scale_rotation_translation();
            let color = if debug.draw_lod_colors {
                lod_color(chunk.lod_index)
            } else {
                Color::srgb(0.78, 0.82, 0.88)
            };
            gizmos.cube(
                Transform {
                    translation,
                    rotation,
                    scale: scale
                        * Vec3::new(
                            chunk.size_local.x.max(0.05),
                            0.02,
                            chunk.size_local.y.max(0.05),
                        ),
                },
                color,
            );
        }
    }

    if debug.draw_interaction_zones {
        for zone in &interactions.zones {
            gizmos.circle(zone.center, zone.radius, Color::srgb(0.85, 0.65, 0.15));
        }
    }
}

fn lod_color(index: usize) -> Color {
    match index {
        0 => Color::srgb(0.22, 0.78, 0.35),
        1 => Color::srgb(0.92, 0.78, 0.28),
        _ => Color::srgb(0.88, 0.42, 0.18),
    }
}

fn clear_patch_outputs(
    commands: &mut Commands,
    materials: &mut Assets<GrassMaterial>,
    state: &mut GrassPatchState,
) {
    for entity in state.generated_chunks.drain(..) {
        commands.entity(entity).despawn();
    }
    for handle in state.material_handles.drain(..) {
        materials.remove(handle.id());
    }
}

#[derive(Clone)]
enum SurfaceKind {
    Planar { patch_half_size: Vec2 },
    Mesh { bake: SurfaceBake },
}

#[derive(Clone)]
struct SurfaceChunk {
    coord: IVec2,
    min: Vec2,
    max: Vec2,
    center: Vec3,
}

#[derive(Clone)]
struct SurfaceBuildPlan {
    kind: SurfaceKind,
    surface_global: GlobalTransform,
    source_entity: Option<Entity>,
    chunks: Vec<SurfaceChunk>,
}

fn build_surface_plan(
    patch: &GrassPatch,
    patch_global: &GlobalTransform,
    config: &GrassConfig,
    source_meshes: &Query<(&Mesh3d, &GlobalTransform)>,
    mesh_assets: &Assets<Mesh>,
) -> Option<SurfaceBuildPlan> {
    match patch.surface {
        GrassSurface::Planar => {
            let layout = ChunkLayout {
                min: -patch.half_size,
                max: patch.half_size,
                chunk_size: patch.chunking.chunk_size.max(Vec2::splat(0.001)),
                dims: UVec2::new(
                    ((patch.half_size.x * 2.0) / patch.chunking.chunk_size.x.max(0.001))
                        .ceil()
                        .max(1.0) as u32,
                    ((patch.half_size.y * 2.0) / patch.chunking.chunk_size.y.max(0.001))
                        .ceil()
                        .max(1.0) as u32,
                ),
            };
            let mut chunks = Vec::new();
            for y in 0..layout.dims.y as i32 {
                for x in 0..layout.dims.x as i32 {
                    let coord = IVec2::new(x, y);
                    let (min, max) = layout.bounds_for_coord(coord);
                    chunks.push(SurfaceChunk {
                        coord,
                        min,
                        max,
                        center: layout.center_for_coord(coord),
                    });
                }
            }
            Some(SurfaceBuildPlan {
                kind: SurfaceKind::Planar {
                    patch_half_size: patch.half_size,
                },
                surface_global: *patch_global,
                source_entity: None,
                chunks,
            })
        }
        GrassSurface::Mesh(entity) => {
            let (mesh3d, surface_global) = source_meshes.get(entity).ok()?;
            let mesh = mesh_assets.get(&mesh3d.0)?;
            let bake = bake_mesh_surface(mesh, patch.chunking.chunk_size)?;
            let mut chunks = Vec::new();
            for y in 0..bake.layout.dims.y as i32 {
                for x in 0..bake.layout.dims.x as i32 {
                    let coord = IVec2::new(x, y);
                    let (min, max) = bake.layout.bounds_for_coord(coord);
                    chunks.push(SurfaceChunk {
                        coord,
                        min,
                        max,
                        center: bake.layout.center_for_coord(coord),
                    });
                }
            }
            let _ = config;
            Some(SurfaceBuildPlan {
                kind: SurfaceKind::Mesh { bake },
                surface_global: *surface_global,
                source_entity: Some(entity),
                chunks,
            })
        }
    }
}

fn chunk_local_transform(
    patch_global: &GlobalTransform,
    surface_global: &GlobalTransform,
    center_local: Vec3,
) -> Transform {
    let world_from_chunk = surface_global.to_matrix() * Mat4::from_translation(center_local);
    let local = patch_global.to_matrix().inverse() * world_from_chunk;
    Transform::from_matrix(local)
}

fn world_wind_snapshots(zones: &Query<(&WindZone, &GlobalTransform)>) -> Vec<WindZoneSnapshot> {
    let mut snapshots = zones
        .iter()
        .map(|(zone, transform)| snapshot_zone(zone, transform))
        .collect::<Vec<_>>();
    snapshots.sort_by_key(|snapshot| std::cmp::Reverse(snapshot.zone.priority));
    snapshots
}

fn resolve_grass_wind(
    fallback_wind: &GrassWind,
    wind_bridge: &GrassWindBridge,
    wind_config: Option<&WindConfig>,
    zone_snapshots: &[WindZoneSnapshot],
    sample_point: Vec3,
    time_secs: f32,
) -> GrassWind {
    let Some(wind_config) = wind_config else {
        return fallback_wind.clone();
    };
    if !wind_bridge.enabled {
        return fallback_wind.clone();
    }

    let sample = sample_wind_with_zones(sample_point, time_secs, wind_config, zone_snapshots);
    fallback_wind.resolved_from_world_sample(wind_bridge, &sample)
}

#[cfg(test)]
#[path = "systems_tests.rs"]
mod tests;
