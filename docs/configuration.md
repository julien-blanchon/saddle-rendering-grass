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
| `bands` | `Vec<GrassLodBand>` | near / mid / far defaults (3 bands) | 1 to N authored bands, sorted by ascending `max_distance` | defines the near-to-far density and complexity ramp | major driver of far-field cost |

## `GrassArchetype`

| Field | Type | Default | Valid range / meaning | Visual effect | Perf impact |
|------|------|---------|------------------------|---------------|-------------|
| `debug_name` | `String` | `"Base"` | any label | shows up in chunk names / diagnostics | none |
| `weight` | `f32` | `1.0` | `> 0` to participate | relative frequency when multiple archetypes mix | more archetypes mean more generated chunk variants |
| `blade_height` | `Vec2` | `(0.55, 1.0)` | min/max > 0 | controls height variation | taller blades can read denser but do not change count |
| `blade_width` | `Vec2` | `(0.02, 0.045)` | min/max > 0 | controls blade thickness | wider blades can improve readability without more instances |
| `forward_curve` | `Vec2` | `(0.04, 0.18)` | small positive range typical | adds forward arc / droop | negligible |
| `lean` | `Vec2` | `(-0.12, 0.12)` | signed range | biases blades to lean left / right | negligible |
| `stiffness` | `Vec2` | `(0.9, 1.15)` | `> 0` | lower values bend more in wind | negligible |
| `interaction_strength` | `Vec2` | `(0.85, 1.15)` | `>= 0` | lower values resist trampling; higher values react more | negligible |
| `root_color` | `Color` | desaturated brown-grey | any color | base tint at blade root | no runtime cost |
| `tip_color` | `Color` | desaturated straw-grey | any color | gradient tint at blade tip | no runtime cost |
| `color_variation` | `f32` | `0.08` | `>= 0`, usually `0..0.4` | random hue / value jitter between blades | negligible |
| `roughness` | `f32` | `0.9` | `0.089..=1.0` practical | controls specular sharpness | no geometry cost |
| `reflectance` | `f32` | `0.16` | `0..=1` | broad surface energy response | no geometry cost |
| `diffuse_transmission` | `f32` | `0.18` | `0..=1` | brighter backlit blades | lighting cost only |

## `GrassConfig`

| Field | Type | Default | Valid range / meaning | Visual effect | Perf impact |
|------|------|---------|------------------------|---------------|-------------|
| `density_per_square_unit` | `f32` | `38.0` | `>= 0` | global blade density baseline before masks / per-patch scaling | primary blade-count driver |
| `max_blades_per_chunk` | `u32` | `1600` | `> 0` | clamps worst-case chunk density | strong safety valve against runaway chunk cost |
| `align_to_surface` | `f32` | `0.7` | `0..=1` typical | `0` keeps grass upright, `1` follows the surface normal closely | negligible |
| `normal_offset` | `f32` | `0.005` | small positive value | lifts blades slightly off the surface to avoid z-fighting | negligible |
| `density_map` | `Option<GrassDensityMap>` | `None` | optional density texture | carves empty / dense areas | rebuild-time texture sampling only |
| `density_layers` | `Vec<GrassDensityLayer>` | empty | additional density map layers with compositing | stack slope mask + painted mask + noise | rebuild-time texture sampling per layer |
| `lod` | `GrassLodConfig` | default 3-band config | authored LOD band set (1-N bands) | near/far density and complexity balance | major far-field cost control |
| `archetypes` | `Vec<GrassArchetype>` | one neutral base archetype | at least one enabled archetype recommended | mixes turf / meadow / flower variants in one patch | more archetypes multiply generated chunk meshes |
| `cast_shadows` | `bool` | `false` | enable only when needed | makes grass participate in shadow casting | can become very expensive on dense patches |
| `scatter_filter` | `GrassScatterFilter` | all filters disabled | slope, altitude, and exclusion zone placement filters | rejects blades on cliffs, above snow line, near buildings | rebuild-time per-blade checks |

## `GrassWind`

`GrassWind` is an explicit resource. The crate default is directionless and motionless, so callers insert or mutate it with the exact profile they want. The repository examples keep optional wind helpers in `examples/common/src/presets.rs`.

| Field | Type | Default | Valid range / meaning | Visual effect | Perf impact |
|------|------|---------|------------------------|---------------|-------------|
| `direction` | `Vec2` | `Vec2::ZERO` | any vector; normalized internally | world-space wind direction | negligible |
| `sway_strength` | `f32` | `0.0` | `>= 0` | overall low-frequency bend amplitude | negligible |
| `sway_frequency` | `f32` | `0.25` | `>= 0` | wavelength of macro sway across the field | negligible |
| `sway_speed` | `f32` | `0.35` | `>= 0` | time speed of macro sway | negligible |
| `gust_strength` | `f32` | `0.0` | `>= 0` | smooth rolling wind bursts | negligible |
| `gust_frequency` | `f32` | `0.12` | `>= 0` | spatial frequency of gust noise | negligible |
| `gust_speed` | `f32` | `0.08` | `>= 0` | temporal speed of gust evolution | negligible |
| `flutter_strength` | `f32` | `0.0` | `>= 0` | small high-frequency blade flutter | negligible |
| `flutter_speed` | `f32` | `2.5` | `> 0` | frequency of per-blade flutter oscillation | negligible |

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
| `flutter_speed_from_speed` | `f32` | `0.15` | `>= 0` | adds shared wind speed into flutter oscillation rate | negligible |

## `GrassInteractionMap`

World-space CPU texture that actors stamp into each frame. Sampled by the grass vertex shader per-vertex. Replaces the 4-zone uniform limit with unlimited actors and persistent trails.

| Field | Type | Default | Valid range / meaning | Visual effect | Perf impact |
|------|------|---------|------------------------|---------------|-------------|
| `center` | `Vec2` | `Vec2::ZERO` | world XZ center of the map region | where interaction is tracked | negligible |
| `half_extent` | `f32` | `30.0` | `> 0` in world units | how large the interaction region is | larger = more texels to update |
| `resolution` | `u32` | `256` | 64, 128, 256, 512 typical | finer detail with higher resolution | CPU cost scales with resolution² |
| `recovery_speed` | `f32` | `2.0` | `0` = permanent, `1` = ~1s recovery, `5` = very fast | how quickly trails fade | negligible |
| `follow_camera` | `bool` | `true` | auto-center on camera each frame | keeps interaction region near the player | negligible |
| `enabled` | `bool` | `true` | toggle the entire interaction map system | falls back to legacy zones when disabled | negligible |

## `GrassInteractionActor`

Attach to any entity with a `Transform` to make it affect nearby grass via the interaction map.

| Field | Type | Default | Valid range / meaning | Visual effect | Perf impact |
|------|------|---------|------------------------|---------------|-------------|
| `radius` | `f32` | `1.4` | `> 0` in world units | footprint size | more pixels stamped per frame |
| `policy` | `GrassInteractionPolicy` | `BendAndFlatten` | how the actor affects grass | see policy table below | negligible |
| `falloff` | `f32` | `2.0` | `> 0` exponent | `1` = linear, `2` = quadratic edge, `0.5` = very soft | negligible |

## `GrassInteractionPolicy`

| Variant | Parameters | Effect | Recovery |
|---------|-----------|--------|----------|
| `Bend { strength }` | strength: 0..1 | Push blades away from actor center | Yes (recovery_speed) |
| `Flatten { strength }` | strength: 0..1 | Push blades downward (trample) | Yes |
| `BendAndFlatten { bend_strength, flatten_strength }` | both: 0..1 | Combined bend + flatten (most common for characters) | Yes |
| `Hide { permanent }` | permanent: bool | Collapse blades to zero height (cut/destroy) | Only if permanent=false |

## `GrassInteractionZone` (Legacy)

Still works alongside the interaction map. Up to 4 zones are packed into shader uniforms.



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

## `GrassScatterFilter`

Controls scatter-time placement filters. All filters use AND logic — a blade must pass every enabled filter.

| Field | Type | Default | Valid range / meaning | Visual effect | Perf impact |
|------|------|---------|------------------------|---------------|-------------|
| `slope_range_degrees` | `Option<(f32, f32)>` | `None` | `(min, max)` in degrees; `0` = flat, `90` = vertical | rejects blades on surfaces steeper than max or flatter than min | rebuild-time per-blade check |
| `altitude_range` | `Option<(f32, f32)>` | `None` | `(min_y, max_y)` in world space | rejects blades above or below the Y range (snow line, water line) | rebuild-time per-blade check |
| `exclusion_zones` | `Vec<GrassExclusionZone>` | empty | world-space spherical exclusion zones | clears grass around buildings, roads, props | rebuild-time per-blade distance check |

## `GrassExclusionZone`

| Field | Type | Default | Valid range / meaning | Visual effect | Perf impact |
|------|------|---------|------------------------|---------------|-------------|
| `center` | `Vec3` | `Vec3::ZERO` | world-space center | center of the exclusion sphere | negligible |
| `radius` | `f32` | `2.0` | `> 0` | hard exclusion radius | negligible |
| `falloff` | `f32` | `0.0` | `>= 0` | soft density ramp beyond `radius`; `0` = hard cutoff | blends edge smoothly | negligible |

## `GrassDensityLayer`

Additional density map layers applied after the primary `density_map`. Each layer's result is combined with the running density via its blend mode.

| Field | Type | Default | Valid range / meaning | Visual effect | Perf impact |
|------|------|---------|------------------------|---------------|-------------|
| `image` | `Handle<Image>` | empty | any loaded 2D image | additional density sculpting layer | rebuild-time texture sampling |
| `channel` | `GrassTextureChannel` | `Luminance` | R / G / B / A / Luminance | which channel drives this layer's value | negligible |
| `mode` | `GrassDensityMapMode` | `PatchUv` | `PatchUv` or `SurfaceUv` | UV mapping mode | negligible |
| `invert` | `bool` | `false` | flip dense/sparse | reverses interpretation | negligible |
| `blend` | `GrassDensityBlendMode` | `Multiply` | `Multiply`, `Min`, `Max`, `Add` | how this layer composites with the running density | negligible |

## `BladeShape`

| Variant | Geometry | Best for | Vertex cost |
|---------|----------|----------|-------------|
| `Strip` | Multi-segment tapered ribbon | Realistic grass, default | 2 vertices per segment |
| `CrossBillboard` | Two perpendicular strips (X) | Volumetric fill, stylized | 2× strip cost |
| `FlatCard` | Single quad (2 triangles) | Cheap far LOD, texture cards | 4 vertices |
| `SingleTriangle` | One triangle, point tip | Anime / Zelda style | 3 vertices |

## `GrassNormalSource`

| Variant | Shading | Best for |
|---------|---------|----------|
| `BladeFacing` | Normal follows blade forward direction (standard PBR) | Realistic grass |
| `GroundNormal` | Normal projected from ground surface (flat unified shading) | Anime, cel-shaded, toon styles |

## `GrassArchetype` (new fields)

| Field | Type | Default | Valid range / meaning | Visual effect | Perf impact |
|------|------|---------|------------------------|---------------|-------------|
| `blade_shape` | `BladeShape` | `Strip` | any variant | controls blade geometry type | varies by shape |
| `tip_alpha` | `f32` | `1.0` | `0.0..=1.0` | vertex alpha at blade tip; `1.0` = opaque, `0.0` = transparent tip | enables alpha blending when < 1.0 |
| `normal_source` | `GrassNormalSource` | `BladeFacing` | any variant | controls how blade normals are computed for shading | negligible |
| `blade_texture` | `Option<Handle<Image>>` | `None` | any loaded 2D image | optional albedo texture applied to blade strip UVs | enables alpha masking |
| `alpha_cutoff` | `f32` | `0.5` | `0.01..=1.0` | alpha cutoff threshold when `blade_texture` is set | negligible |
