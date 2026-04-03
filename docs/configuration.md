# `grass` Configuration

This document lists the public tuning surface for `grass`. Defaults are the current crate defaults.

## `GrassPatch`

| Field | Type | Default | Valid range / meaning | Visual effect | Perf impact |
|------|------|---------|------------------------|---------------|-------------|
| `half_size` | `Vec2` | `Vec2::splat(6.0)` | positive X/Z half-extents in local patch space | larger patch footprint | larger patches generate more chunks and blades |
| `density_scale` | `f32` | `1.0` | `>= 0`; per-patch density multiplier | makes one patch denser / sparser without touching shared config | linear effect on blade count |
| `seed` | `u64` | `1` | any deterministic seed | changes scatter layout and per-blade variation | none at runtime; rebuild required |
| `chunking` | `GrassChunking` | `chunk_size = (8, 8)` | chunk partition settings | smaller chunks localize rebuilds and culling | more entities / meshes when smaller |
| `surface` | `GrassSurface` | `Planar` | planar rectangle or external mesh surface | switches between flat patches and terrain-following grass | mesh mode costs surface baking at rebuild time |

## `GrassChunking`

| Field | Type | Default | Valid range / meaning | Visual effect | Perf impact |
|------|------|---------|------------------------|---------------|-------------|
| `chunk_size` | `Vec2` | `Vec2::new(8.0, 8.0)` | `> 0` in patch-local units | does not directly change blade look | smaller chunks increase entity count; larger chunks increase rebuild granularity |

## `GrassSurface`

| Variant | Meaning | Visual effect | Perf impact |
|---------|---------|---------------|-------------|
| `Planar` | Use patch-local XZ plane | flat meadows, lawns, strips | cheapest path |
| `Mesh(Entity)` | Scatter on another entity's `Mesh3d` surface | terrain edges, ramps, arbitrary static mesh surfaces | rebuild bakes mesh triangles and UVs |

## `GrassDensityMap`

| Field | Type | Default | Valid range / meaning | Visual effect | Perf impact |
|------|------|---------|------------------------|---------------|-------------|
| `image` | `Handle<Image>` | empty handle | any loaded 2D image | sculpts density across the patch | texture sampling during rebuild only |
| `channel` | `GrassTextureChannel` | `Luminance` | `Red`, `Green`, `Blue`, `Alpha`, `Luminance` | selects which texture channel drives density | negligible |
| `mode` | `GrassDensityMapMode` | `PatchUv` | `PatchUv` or `SurfaceUv` | patch UV gives authored rectangle masks even on mesh surfaces; surface UV follows source mesh UVs | negligible |
| `invert` | `bool` | `false` | flip dense vs sparse areas | reverses mask interpretation | negligible |

## `GrassLodBand`

| Field | Type | Default | Valid range / meaning | Visual effect | Perf impact |
|------|------|---------|------------------------|---------------|-------------|
| `max_distance` | `f32` | band-dependent (`18`, `42`, `78`) | `> 0` camera distance | controls where the band remains visible | farther bands increase visible grass range |
| `density_scale` | `f32` | band-dependent (`1.0`, `0.42`, `0.15`) | `0..=1+` typical | reduces blade count in that band | strongest perf lever per distance band |
| `segments` | `u8` | band-dependent (`6`, `4`, `2`) | `>= 1` | fewer vertical blade segments make distant grass simpler | lower vertex cost in that band |
| `fade_distance` | `f32` | band-dependent (`4`, `6`, `10`) | `>= 0` | softens distance transitions via `VisibilityRange` | slightly more overlap between bands |

## `GrassLodConfig`

| Field | Type | Default | Valid range / meaning | Visual effect | Perf impact |
|------|------|---------|------------------------|---------------|-------------|
| `bands` | `[GrassLodBand; 3]` | near / mid / far defaults | three authored bands | defines the near-to-far density and complexity ramp | major driver of far-field cost |

## `GrassArchetype`

| Field | Type | Default | Valid range / meaning | Visual effect | Perf impact |
|------|------|---------|------------------------|---------------|-------------|
| `debug_name` | `String` | `"Meadow"` | any label | shows up in chunk names / diagnostics | none |
| `weight` | `f32` | `1.0` | `> 0` to participate | relative frequency when multiple archetypes mix | more archetypes mean more generated chunk variants |
| `blade_height` | `Vec2` | `(0.65, 1.25)` | min/max > 0 | controls height variation | taller blades can read denser but do not change count |
| `blade_width` | `Vec2` | `(0.025, 0.055)` | min/max > 0 | controls blade thickness | wider blades can improve readability without more instances |
| `forward_curve` | `Vec2` | `(0.08, 0.28)` | small positive range typical | adds forward arc / droop | negligible |
| `lean` | `Vec2` | `(-0.18, 0.18)` | signed range | biases blades to lean left / right | negligible |
| `stiffness` | `Vec2` | `(0.85, 1.2)` | `> 0` | lower values bend more in wind | negligible |
| `interaction_strength` | `Vec2` | `(0.8, 1.2)` | `>= 0` | lower values resist trampling; higher values react more | negligible |
| `root_color` | `Color` | dark green | any color | base tint at blade root | no runtime cost |
| `tip_color` | `Color` | lighter green | any color | gradient tint at blade tip | no runtime cost |
| `color_variation` | `f32` | `0.16` | `>= 0`, usually `0..0.4` | random hue / value jitter between blades | negligible |
| `roughness` | `f32` | `0.9` | `0.089..=1.0` practical | controls specular sharpness | no geometry cost |
| `reflectance` | `f32` | `0.16` | `0..=1` | broad surface energy response | no geometry cost |
| `diffuse_transmission` | `f32` | `0.2` | `0..=1` | brighter backlit blades | lighting cost only |

## `GrassConfig`

| Field | Type | Default | Valid range / meaning | Visual effect | Perf impact |
|------|------|---------|------------------------|---------------|-------------|
| `density_per_square_unit` | `f32` | `38.0` | `>= 0` | global blade density baseline before masks / per-patch scaling | primary blade-count driver |
| `max_blades_per_chunk` | `u32` | `1600` | `> 0` | clamps worst-case chunk density | strong safety valve against runaway chunk cost |
| `align_to_surface` | `f32` | `0.7` | `0..=1` typical | `0` keeps grass upright, `1` follows the surface normal closely | negligible |
| `normal_offset` | `f32` | `0.005` | small positive value | lifts blades slightly off the surface to avoid z-fighting | negligible |
| `density_map` | `Option<GrassDensityMap>` | `None` | optional density texture | carves empty / dense areas | rebuild-time texture sampling only |
| `lod` | `GrassLodConfig` | default 3-band config | authored LOD band set | near/far density and complexity balance | major far-field cost control |
| `archetypes` | `Vec<GrassArchetype>` | one default meadow archetype | at least one enabled archetype recommended | mixes turf / meadow / flower variants in one patch | more archetypes multiply generated chunk meshes |
| `cast_shadows` | `bool` | `false` | enable only when needed | makes grass participate in shadow casting | can become very expensive on dense patches |

## `GrassWind`

| Field | Type | Default | Valid range / meaning | Visual effect | Perf impact |
|------|------|---------|------------------------|---------------|-------------|
| `direction` | `Vec2` | `(0.85, 0.35)` | any vector; normalized internally | world-space wind direction | negligible |
| `sway_strength` | `f32` | `0.18` | `>= 0` | overall low-frequency bend amplitude | negligible |
| `sway_frequency` | `f32` | `0.35` | `>= 0` | wavelength of macro sway across the field | negligible |
| `sway_speed` | `f32` | `0.85` | `>= 0` | time speed of macro sway | negligible |
| `gust_strength` | `f32` | `0.08` | `>= 0` | local noisy wind bursts | negligible |
| `gust_frequency` | `f32` | `0.18` | `>= 0` | spatial frequency of gust noise | negligible |
| `gust_speed` | `f32` | `0.2` | `>= 0` | temporal speed of gust evolution | negligible |
| `flutter_strength` | `f32` | `0.04` | `>= 0` | small high-frequency blade flutter | negligible |

Updating `GrassWind` does not rebuild patches. It only refreshes the shared material uniform.

## `GrassWindBridge`

`GrassWindBridge` is the optional adapter from `saddle-world-wind` into the grass runtime. It keeps `GrassWind` as the authored fallback profile, then scales the sampled shared wind field into grass-specific sway / gust / flutter amplitudes.

| Field | Type | Default | Valid range / meaning | Visual effect | Perf impact |
|------|------|---------|------------------------|---------------|-------------|
| `enabled` | `bool` | `true` | when `false`, ignore `saddle-world-wind` even if it is present | forces the crate to use the standalone `GrassWind` profile only | none |
| `sample_height_offset` | `f32` | `0.35` | local height offset above each chunk center | samples wind slightly above the ground instead of at the exact patch plane | negligible |
| `sway_strength_scale` | `f32` | `1.35` | `>= 0` | multiplies sampled `WindSample::sway_factor` into bend amplitude | negligible |
| `sway_frequency_from_turbulence` | `f32` | `0.9` | `>= 0` | adds turbulence-driven variation to macro sway frequency | negligible |
| `sway_speed_from_speed` | `f32` | `0.18` | `>= 0` | adds shared wind speed into macro sway speed | negligible |
| `gust_strength_scale` | `f32` | `0.28` | `>= 0` | maps shared gust envelopes into the grass gust layer | negligible |
| `gust_frequency_from_turbulence` | `f32` | `0.45` | `>= 0` | increases gust noise detail under turbulent shared wind | negligible |
| `gust_speed_from_speed` | `f32` | `0.08` | `>= 0` | accelerates gust evolution under faster shared wind | negligible |
| `flutter_strength_scale` | `f32` | `0.2` | `>= 0` | maps sampled flutter detail into high-frequency blade motion | negligible |

## `GrassInteractionZone`

| Field | Type | Default | Valid range / meaning | Visual effect | Perf impact |
|------|------|---------|------------------------|---------------|-------------|
| `radius` | `f32` | `1.4` | `> 0` | area affected by the zone | more affected blades in view |
| `bend_strength` | `f32` | `0.45` | `>= 0` | lateral bend away from the zone center | negligible |
| `flatten_strength` | `f32` | `0.25` | `>= 0` | downward flattening amount | negligible |
| `falloff` | `f32` | `1.5` | `> 0` | exponent shaping edge softness | negligible |

Only the first four zones are packed into each material uniform in the current implementation.

## `GrassDebugSettings`

| Field | Type | Default | Meaning | Visual effect | Perf impact |
|------|------|---------|---------|---------------|-------------|
| `draw_patch_bounds` | `bool` | `false` | draw authored patch boxes | patch footprint visualization | low debug-only gizmo cost |
| `draw_chunk_bounds` | `bool` | `false` | draw generated chunk boxes | shows chunk partitioning and streaming granularity | low debug-only gizmo cost |
| `draw_lod_colors` | `bool` | `false` | color chunk-bound gizmos by LOD band when `draw_chunk_bounds` is enabled | helps visualize near / mid / far band transitions | low debug-only gizmo cost |
| `draw_interaction_zones` | `bool` | `false` | draw interaction radii | shows trample / bend coverage | low debug-only gizmo cost |

## `GrassRebuildRequest`

| Field | Type | Meaning | Effect |
|------|------|---------|--------|
| `patch` | `Entity` | target patch entity | marks one patch dirty so it rebuilds on the next update |
