# `grass` Architecture

## Chosen Render Path

`grass` uses **CPU-built chunk meshes** plus an **`ExtendedMaterial<StandardMaterial, _>` vertex shader**.

This was chosen over a lower-level instance-buffer pipeline because it satisfies the required feature set with less render-graph complexity:

- standard Bevy PBR lighting stays intact
- no custom draw phase or extraction path is required
- rebuild logic remains ordinary ECS + asset work
- vertex animation still moves the expensive per-frame work onto the GPU

The tradeoff is that all LOD chunk meshes are prebuilt as regular mesh entities, so memory / entity count rises with patch size and archetype count. That is acceptable for the baseline shared crate because it keeps the runtime understandable and debuggable.

## Reference Decisions

This crate intentionally distills a few ideas from the reference stack instead of copying a whole production foliage pipeline:

- `shared/context/bevy_feronia` proved that chunking, density maps, wind-aware materials, and mesh-surface scattering all generalize well. `grass` keeps those ideas, but drops backend-specific asset workflows, observer-driven scatter orchestration, and content-hierarchy authoring so the shared API stays small and crate-local.
- GPU Gems and the Ghost of Tsushima / Horizon vegetation talks both push layered motion instead of one sine wave. `grass` follows that split directly: macro sway, gust noise, per-blade phase variation, and local interaction impulses all contribute to the final vertex offset.
- Cross-engine foliage workflows consistently separate authored placement density from runtime visibility reduction. `grass` mirrors that by rebuilding deterministic placement only when authoring inputs change, while LOD fading and wind remain runtime concerns.
- Production foliage talks also make the near-vs-far tradeoff explicit: geometry-heavy close grass is easier to light and silhouette correctly, while distant grass must reduce both density and per-blade complexity. That is why `grass` uses strip geometry up close and authored segment reduction at range instead of alpha cards or runtime re-scatter every frame.

## Data Flow

1. Authoring inserts `GrassPatch` + `GrassConfig` on a patch entity.
2. The runtime marks the patch dirty on:
   - activation
   - `Added<GrassPatch>`
   - `Changed<GrassPatch>`
   - `Changed<GrassConfig>`
   - `AssetEvent<Mesh>` for a referenced mesh surface
   - `AssetEvent<Image>` for a referenced density map
   - `GrassRebuildRequest`
3. `Prepare` systems collect interaction zones and dirty flags.
4. `Scatter` rebuilds only dirty patches:
   - choose planar or mesh-surface bake
   - divide the patch into chunk bounds
   - resolve LOD bands
   - sample deterministic blade roots per chunk / archetype / LOD
   - generate blade-strip meshes with per-vertex variation attributes
   - spawn generated chunk children with `Mesh3d`, `MeshMaterial3d<GrassMaterial>`, and `VisibilityRange`
5. `Upload` publishes diagnostics.
6. `Animate` keeps chunk transforms and material uniforms synchronized with source transforms, wind, and interaction zones.
7. `Debug` draws optional gizmos for authored patch bounds, chunk bounds, and interaction radii.

Generated chunk entities are children of the patch entity. Deactivation despawns them and clears the associated material handles.

## Public Runtime Surface

- `GrassPatch` and `GrassConfig` are the authoring surface.
- `GrassWind` is the shared runtime motion surface.
- `GrassWindBridge` is the optional bridge into `saddle-world-wind`.
- `GrassInteractionZone` is the reusable deformation hook.
- `GrassDiagnostics` is the public readback / BRP surface.
- `GrassRebuildRequest` is the public manual invalidation hook.

`GrassWind` remains the standalone fallback profile. When `saddle-world-wind` is present, `GrassWindBridge` samples the shared field per generated chunk and maps it into the grass material uniform. That keeps `grass` usable on its own while still letting it participate in a larger atmosphere stack.

Internal mesh payloads, scatter caches, surface bakes, and shader plumbing stay private.

## System Ordering

`GrassSystems` is intentionally split into five phases:

| Set | Purpose |
|-----|---------|
| `Prepare` | dirty detection, asset-change handling, interaction collection |
| `Scatter` | chunk rebuild and generated-entity spawn |
| `Upload` | publish diagnostics from the current runtime state |
| `Animate` | sync chunk transforms and wind / interaction uniforms |
| `Debug` | optional gizmo drawing |

The plugin chains these sets in the caller-provided update schedule. The default constructor uses an always-on `Update` path, but callers can wire custom activate / deactivate / update schedules with `GrassPlugin::new(...)`.

## Surface Strategy

### Planar patches

Planar patches use the patch entity's local XZ plane and `half_size` to define the authored rectangle.

### Mesh-surface patches

`GrassSurface::Mesh(entity)` treats another mesh entity as the scatter surface:

- the source mesh is baked into triangles
- local chunk bounds are derived from the mesh-surface bake
- density maps can optionally sample source UVs
- density maps can also stay in patch UV space even on mesh surfaces, which is useful when the source mesh UVs are tiled or unrelated to foliage placement
- generated chunk transforms stay synced to the source mesh transform

This keeps the crate terrain-agnostic: any static Bevy mesh can become a grass surface.

## LOD And Culling

Each `GrassLodBand` defines:

- `max_distance`
- `density_scale`
- `segments`
- `fade_distance`

At rebuild time the crate generates one chunk mesh per band. At runtime those chunk entities rely on Bevy `VisibilityRange` for distance fading / dithered visibility, with both fade-in and fade-out margins so adjacent bands overlap instead of popping.

The current design favors simple, explicit authoring:

- near bands keep more blades and more vertical blade segments
- far bands reduce both blade count and blade geometric complexity
- diagnostics report both total chunk counts and camera-visible chunk counts

Because visible counts come from `ViewVisibility`, they represent the live rendered state and are useful for BRP / E2E tuning.

## Wind Layering

The vertex shader combines four motion layers:

1. **global sway**: low-frequency directional wind from `GrassWind.direction`, `sway_frequency`, and `sway_speed`
2. **gust noise**: smooth value-noise rolling across the field, modulated by `gust_frequency` and `gust_speed`. Uses bilinear-interpolated hash noise to produce continuous, non-flickering gust waves.
3. **per-blade flutter**: high-frequency local motion with phase variation and configurable `flutter_speed`
4. **local interaction zones**: bend / flatten offsets from nearby `GrassInteractionZone`s

`GrassWind` defaults to neutral zero-motion data with no baked-in direction bias. Scene-specific meadow and wind profiles live in the repository's example-side preset module so the runtime resource stays plain and explicit.

If `saddle-world-wind` is active, those authored `GrassWind` values become the baseline profile and the runtime samples the shared wind field at each chunk center before updating the material uniform.

Per-blade variation is baked into the mesh as custom vertex attributes:

- root position
- per-blade phase
- stiffness multiplier
- interaction multiplier
- random value

That keeps the material uniform small while allowing every blade to move slightly differently.

## Rebuild Triggers

The crate does not rebuild every frame. Rebuilds happen only when:

- a patch or config is added / changed
- a watched mesh or density image changes
- the runtime is activated
- a caller sends `GrassRebuildRequest`

Wind animation and interaction-zone motion do **not** rebuild meshes. They only update the shared material uniform.

`GrassPatch::density_scale` is applied inside the scatter phase before any LOD or archetype weighting, so one authored patch can be thinned out or densified without cloning `GrassConfig`.

## Major Tradeoffs

### Geometry strips instead of alpha cards

- better close-range silhouettes
- no alpha-test foliage card sorting problems
- more vertices than a card-based approach

### Prebuilt LOD meshes instead of runtime re-scatter on distance change

- simpler runtime behavior
- deterministic diagnostics and BRP inspection
- more entities and mesh memory up front

### CPU scattering instead of fully GPU-driven generation

- easier to author and debug today
- deterministic seed handling is straightforward
- extremely large open-world fields may eventually want a lower-level GPU backend

## Debugging Guidance

Use `GrassDiagnostics` first:

- `active_chunks` / `active_blades` tell you what exists
- `visible_chunks` / `visible_blades` tell you what the camera is actually seeing
- per-patch `visible_lod_chunk_counts` show distance behavior

Then use BRP:

```bash
uv run --project .codex/skills/bevy-brp/script brp resource get grass::resources::GrassDiagnostics
uv run --project .codex/skills/bevy-brp/script brp world query bevy_ecs::name::Name
```

The crate-local `grass_lab` app is the intended integration target for that workflow.
