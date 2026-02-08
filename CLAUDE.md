# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Rein is a 3D engine built on wgpu with ECS (hecs), physics simulation, and GPU compute. Primary use case is robotics visualization with URDF support.

## Build Commands

```bash
cargo build                        # Debug build (includes window feature)
cargo build --release              # Release build
cargo build --all-features         # All features
cargo build --no-default-features  # Minimal build (no window)
cargo build --features ecs         # ECS only
cargo build --features engine      # Engine (ECS + window)
cargo build --features full        # Everything

cargo test --lib                   # Run library tests
cargo test --all-features          # All tests
cargo test test_name               # Run specific test

cargo check --all-features         # Type checking
cargo clippy --all-features        # Linting
cargo fmt                          # Format code
```

## Architecture

Ten-layer design from low to high level:

1. **context/** - wgpu wrapper (`WgpuContext` with Arc-wrapped Device/Queue)
2. **core/** - GPU primitives (buffers, textures, pipelines, vertex types)
3. **compute/** - Compute shader dispatch, GPU-CPU readback
4. **renderer/** - High-level rendering (cameras, materials, geometry, lights, shadows, culling)
5. **physics/** - Rigid body simulation, collision detection (feature="physics")
6. **ecs/** - hecs ECS integration, components, systems (feature="ecs")
7. **engine/** - Game loop with App trait, fixed/variable timestep (feature="engine")
8. **window/** - winit abstraction (feature="window")
9. **gui/** - Text rendering with glyphon (feature="gui")
10. **urdf/** - URDF robot model loading

Shaders are in `src/shaders/` as WGSL files, compiled at runtime.

## Feature Flags

```toml
default = ["window"]
compute = []
ecs = ["dep:hecs"]
physics = ["ecs"]
gpu-physics = ["physics"]
engine = ["ecs", "window"]
full = ["engine", "physics", "gpu-physics", "gui"]
```

## Key Patterns

**Generic container pattern**: `Gm<G: Geometry, M: Material>` combines any geometry with any material.

**ECS game loop pattern** (feature="engine"):
```rust
struct MyApp;
impl App for MyApp {
    fn init(&mut self, ctx: &WgpuContext, world: &mut hecs::World) { /* setup */ }
    fn update(&mut self, world: &mut hecs::World, ctx: &SystemContext) { /* per-frame */ }
}
run_app(WindowSettings::default(), GameLoopConfig::default(), MyApp)?;
```

**Uniform binding groups**:
- Group 0: Camera uniform
- Group 1: Model uniform
- Group 2: Material-specific uniforms

**Vertex types** (in order of complexity):
- `VertexP` - position only (shadow mapping)
- `VertexPC` - position + color (lines)
- `VertexPN` - position + normal (lighting)
- `VertexPNUC` - full (position, normal, UV, color)

**Render loop pattern** (without engine):
```rust
window.render_loop(state, |state, frame| {
    // Update and render
    FrameOutput::default()
})
```

## Adding New Components

**New Material**:
1. Create `src/renderer/material/my_material.rs` implementing `Material` trait
2. Add shader `src/shaders/my_material.wgsl`
3. Export from `src/renderer/material/mod.rs`

**New Effect**:
1. Implement `Effect` trait in `src/effect/`
2. Add shader in `src/shaders/effects/`
3. Add to `EffectChain` for use

## Dependencies

- `wgpu` 28 - GPU backend
- `glam` 0.31 - Math (Vec3, Mat4, etc.)
- `hecs` 0.10 - ECS (optional)
- `urdf-rs` 0.9 - URDF parsing
- `winit` 0.30 - Window management (optional)
- `glyphon` 0.10 - Text rendering (optional)
