# Saddle Rendering Grass

GPU-friendly grass and low-lying foliage rendering for Bevy with chunked CPU scattering, vertex-stage wind animation, density maps, multi-band LOD, mesh-surface alignment, interaction zones, and crate-local E2E / BRP verification.

## Render Path Choice

`grass` uses **generated blade-strip geometry + `ExtendedMaterial<StandardMaterial, _>`**.

- Chosen for pragmatic Bevy integration: it keeps standard PBR lighting, shadows, and material batching without building a full custom render pipeline.
- Chosen for predictable authoring: patches rebuild deterministically from seeds, source meshes, and density maps.
- Chosen for scalable motion: placement happens on the CPU only when inputs change, while wind and interaction bending stay in the vertex shader every frame.

This is intentionally not a fully GPU-driven foliage system. The crate prioritizes a maintainable baseline that fits most real Bevy projects before jumping to a lower-level instance-buffer pipeline.

## Quick Start

```rust,no_run
use bevy::prelude::*;
use grass::{GrassConfig, GrassPatch, GrassPatchBundle, GrassPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GrassPlugin::default())
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-8.0, 6.0, 12.0).looking_at(Vec3::new(0.0, 0.5, -4.0), Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 25_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(12.0, 20.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        Name::new("Ground"),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(48.0, 48.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.22, 0.25, 0.18),
            perceptual_roughness: 0.95,
            ..default()
        })),
    ));

    commands.spawn((
        GrassPatchBundle {
            name: Name::new("Starter Meadow"),
            patch: GrassPatch {
                half_size: Vec2::new(8.0, 12.0),
                density_scale: 1.0,
                seed: 7,
                ..default()
            },
            config: GrassConfig {
                density_per_square_unit: 28.0,
                ..default()
            },
        },
        Transform::from_xyz(0.0, 0.0, -8.0),
    ));
}
```

## Required Scene Setup

- Use a 3D camera and at least one light.
- Spawn either:
  - a planar patch with the patch entity transform defining the patch space, or
  - a mesh-surface patch with `GrassSurface::Mesh(source_entity)` where `source_entity` has `Mesh3d` and a transform.
- Keep source meshes static or rebuild them intentionally; `grass` watches mesh and density-map asset changes and will regenerate affected patches.

`cast_shadows` defaults to `false` because dense grass shadows get expensive quickly. Enable it only where near-camera fidelity justifies the cost.

## Public API

| Type | Purpose |
|------|---------|
| `GrassPlugin` | Registers the grass runtime with injectable activate / deactivate / update schedules |
| `GrassSystems` | Public ordering hooks: `Prepare`, `Scatter`, `Upload`, `Animate`, `Debug` |
| `GrassPatch` | Authoring component for patch bounds, density scaling, seed, chunking, and planar vs mesh surface mode; auto-requires `GrassConfig` and core transform/visibility components |
| `GrassPatchBundle` | Minimal authoring bundle for `Name + GrassPatch + GrassConfig` |
| `GrassConfig` | Per-patch density, LOD, archetypes, surface alignment, density-map, and shadow controls |
| `GrassArchetype` | Blade dimensions, bend/lean/stiffness, root/tip colors, tint variance, and transmission |
| `GrassChunking` | Chunk size used to partition patch rebuilds and generated meshes |
| `GrassDensityMap` | Optional density texture plus channel / mapping selection |
| `GrassSurface` | `Planar` or `Mesh(Entity)` surface selection |
| `GrassLodConfig` / `GrassLodBand` | Distance bands that reduce blade density and blade segment count |
| `GrassWind` | Global wind direction and layered sway / gust / flutter controls |
| `GrassWindBridge` | Optional adapter that maps `saddle-world-wind` samples into the grass shader while preserving `GrassWind` as the standalone fallback profile |
| `GrassInteractionZone` | Reusable bend / flatten impulse zone for moving actors or debug proxies |
| `GrassDebugSettings` | Optional gizmo toggles for patch bounds, chunk bounds, LOD colors, and interaction radii |
| `GrassDiagnostics` / `GrassPatchDiagnostics` | Runtime counts for active + visible chunks/blades and per-patch LOD visibility |
| `GrassRebuildRequest` | Message-based manual rebuild trigger for a specific patch |
| `GrassMaterial` | Public material alias for downstream inspection or custom lab tooling |

## Supported

- Deterministic CPU scattering from patch seed
- Per-patch density scaling via `GrassPatch::density_scale`
- Planar patches and mesh-surface patches
- Per-blade height / width / lean / forward curve / color variation / phase variation
- Multi-archetype patches
- Density maps sampled in patch UV or source-mesh UV space (`GrassDensityMapMode`)
- Three-band LOD with density reduction, segment reduction, and Bevy `VisibilityRange`
- Vertex-stage wind with macro sway, gust noise, per-blade flutter, and local interaction zones
- Optional composition with `saddle-world-wind`: each generated chunk samples the shared wind field at runtime and falls back to `GrassWind` when the wind crate is not present
- Message-triggered rebuilds and asset-change-triggered rebuilds
- Diagnostics and BRP-friendly runtime inspection

## Intentionally Deferred

- Texture-atlas blade sprites
- Full GPU instance-buffer scattering
- Blue-noise / Poisson sampling
- Shadow-proxy meshes for far grass
- Automatic biome or terrain-generation integration

## Examples

| Example | Purpose | Run |
|---------|---------|-----|
| `basic` | Mixed turf + mesh-aligned slope patch | `cargo run -p grass --example basic` |
| `wind_showcase` | Different stiffness profiles under animated wind | `cargo run -p grass --example wind_showcase` |
| `lod_showcase` | Large distant meadow for density / LOD transitions | `cargo run -p grass --example lod_showcase` |
| `interaction_strip` | Moving bend / flatten zone without gameplay coupling | `cargo run -p grass --example interaction_strip` |
| `stress_field` | Heavier multi-patch field for diagnostics and perf checks | `cargo run -p grass --example stress_field` |

## Crate-Local Lab

The richer verification app lives at `shared/rendering/grass/examples/lab`:

```bash
cargo run -p grass_lab
```

Targeted E2E scenarios:

```bash
cargo run -p grass_lab --features e2e -- grass_smoke
cargo run -p grass_lab --features e2e -- grass_wind_showcase
cargo run -p grass_lab --features e2e -- grass_lod_showcase
cargo run -p grass_lab --features e2e -- grass_interaction_strip
```

BRP workflow:

```bash
cargo run -p grass_lab
uv run --project .codex/skills/bevy-brp/script brp resource list | rg GrassDiagnostics
uv run --project .codex/skills/bevy-brp/script brp resource get grass::resources::GrassDiagnostics
uv run --project .codex/skills/bevy-brp/script brp extras screenshot /tmp/grass_lab_brp.png
uv run --project .codex/skills/bevy-brp/script brp extras shutdown
```

## Performance Notes

- Cost scales first with `density_per_square_unit`, then with `half_size`, then with archetype count.
- `max_blades_per_chunk` is the hard backstop against runaway chunk cost. Lower it first if a patch spikes.
- Smaller `chunk_size` improves rebuild locality but increases generated entities and material state churn.
- `cast_shadows = true` can dominate cost on dense near-camera patches.
- Far-field cost is controlled by `GrassLodConfig`: reduce `density_scale` and `segments` before reducing patch size.
- `draw_chunk_bounds` now renders the true generated chunk footprint. Enable `draw_lod_colors` alongside it when you want per-LOD coloring instead of a neutral outline.

## Common Pitfalls

- If a patch is invisible, check the source mesh or patch transform first. Mesh-surface patches use the source entity's mesh and transform as the scatter surface.
- If grass looks too rigid, raise `GrassWind` sway / gust strength or lower the archetype stiffness range.
- If your project already uses `saddle-world-wind`, keep `GrassWind` as the simple fallback profile and tune `GrassWindBridge` instead of duplicating a second wind simulation.
- If grass hovers above the ground, lower `normal_offset`. If it z-fights, raise it slightly.
- If a patch rebuilds more often than expected, check whether some external system is mutating `GrassPatch`, `GrassConfig`, or the source mesh / density image every frame.
- Visible-count diagnostics are camera-dependent. They are meant for tuning and BRP inspection, not for deterministic gameplay logic.

## More Detail

- [Architecture](docs/architecture.md)
- [Configuration](docs/configuration.md)
