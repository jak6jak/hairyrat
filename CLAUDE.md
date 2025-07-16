# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Bevy-based Rust application that renders and animates 3D rat models. The project is a single-file application that demonstrates:

- 3D model loading and animation using Bevy engine
- Asset management with environment maps and GLB models
- Pan-orbit camera controls
- Performance monitoring UI

## Common Commands

### Build and Run
```bash
cargo run
```

### Build (Debug/Release)
```bash
cargo build
cargo build --release
```

### Check Code
```bash
cargo check
```

### Run Tests
```bash
cargo test
```

### Format Code
```bash
cargo fmt
```

### Lint Code
```bash
cargo clippy
```

## Architecture

The application follows a Bevy ECS (Entity Component System) architecture with these key components:

### Core Systems
- **Asset Loading**: Uses `bevy_asset_loader` with state management (`MyStates::AssetLoading` → `MyStates::Next`)
- **Animation System**: Handles rat model animation with `AnimationGraph` and `AnimationPlayer`
- **Camera System**: Implements pan-orbit camera using `bevy_panorbit_camera`
- **Performance Monitoring**: Integrates `iyes_perf_ui` for runtime performance metrics

### Key Resources
- `Animations`: Stores animation graph handles and node indices
- `RatModels`: Asset collection for rat GLB scene and animation clips
- `WorldMaterial`: Environment map textures (diffuse and specular)

### Components
- `Rat`: Marker component for rat entities
- `RatsSpawned`: Component tracking spawned rats

### Asset Structure
- 3D models: `Assets/blackrat_free_glb/blackrat.glb`
- Environment maps: `Assets/EnviromentMaps/` (KTX2 format)
- Animation clips embedded in GLB files

## Development Notes

### Performance Configuration
The project uses optimized development builds:
- Dev profile: `opt-level = 1`
- Dependencies: `opt-level = 3`

### Animation Flow
1. Assets loaded during `AssetLoading` state
2. Scene setup in `show_model` system on state transition
3. Animation playback handled by `play_animation_when_ready` system
4. Uses `AnimationTransitions` for smooth animation blending

### Rendering Features
- Environment mapping for realistic lighting
- Directional lighting
- Pan-orbit camera controls
- Performance diagnostics display