# Grass Lab

Crate-local verification app for [`grass`](../..). It keeps the shared crate runnable, visually inspectable, and E2E-testable without relying on project-level sandboxes.

## Run

```bash
cd examples && cargo run -p grass_lab
```

## Run E2E scenarios

```bash
cd examples && cargo run -p grass_lab --features e2e -- grass_smoke
cd examples && cargo run -p grass_lab --features e2e -- grass_wind_showcase
cd examples && cargo run -p grass_lab --features e2e -- grass_lod_showcase
cd examples && cargo run -p grass_lab --features e2e -- grass_interaction_strip
```

## BRP / live inspection

```bash
cd examples && cargo run -p grass_lab
uv run --project .codex/skills/bevy-brp/script brp resource list | rg GrassDiagnostics
uv run --project .codex/skills/bevy-brp/script brp resource get grass::resources::GrassDiagnostics
uv run --project .codex/skills/bevy-brp/script brp world query bevy_ecs::name::Name
uv run --project .codex/skills/bevy-brp/script brp extras screenshot /tmp/grass_lab.png
```

The lab scene includes:

- a dense planar meadow for wind and LOD checks
- a mesh-aligned slope patch for surface-following verification
- a mixed-archetype courtyard patch
- a moving interaction sphere for bend / flatten hooks
