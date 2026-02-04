# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Rein is a three-d style 3D rendering library built on wgpu for cross-platform GPU rendering. Primary use case is robotics visualization with URDF support.

## Build Commands

```bash
cargo build                      # Debug build (includes window feature)
cargo build --release            # Release build
cargo build --all-features       # All features (window + gui)
cargo build --no-default-features # Minimal build (no window)

cargo test --lib                 # Run library tests
cargo test test_name             # Run specific test
cargo test -- --nocapture        # Show test output

cargo check --all-features       # Type checking
cargo clippy --all-features      # Linting
cargo fmt                        # Format code
```

## Architecture

Six-layer design from low to high level:

1. **context/** - wgpu wrapper (`WgpuContext` with Arc-wrapped Device/Queue)
2. **core/** - GPU primitives (buffers, textures, pipelines, vertex types)
3. **renderer/** - High-level rendering (cameras, materials, geometry, lights, shadows, culling)
4. **window/** - winit abstraction (optional, feature="window")
5. **gui/** - Text rendering with glyphon (optional, feature="gui")
6. **urdf/** - URDF robot model loading

Shaders are in `src/shaders/` as WGSL files, compiled at runtime.

## Key Patterns

**Generic container pattern**: `Gm<G: Geometry, M: Material>` combines any geometry with any material.

**Uniform binding groups**:
- Group 0: Camera uniform
- Group 1: Model uniform
- Group 2: Material-specific uniforms

**Vertex types** (in order of complexity):
- `VertexP` - position only (shadow mapping)
- `VertexPC` - position + color (lines)
- `VertexPN` - position + normal (lighting)
- `VertexPNUC` - full (position, normal, UV, color)

**Render loop pattern**:
```rust
window.render_loop(state_init, |frame_input, state| {
    // Update and render
    FrameOutput { clear_color, effects, .. }
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
- `nalgebra` 0.34 - Advanced linear algebra (used in URDF kinematics)
- `urdf-rs` 0.9 - URDF parsing
- `winit` 0.30 - Window management (optional)
- `glyphon` 0.10 - Text rendering (optional)
