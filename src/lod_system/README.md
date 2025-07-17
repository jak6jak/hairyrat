# Generic LOD System

A flexible and extensible Level of Detail (LOD) system for Bevy that supports multiple rendering strategies.

## Features

- **Generic Architecture**: Works with any entity type and LOD strategy
- **Multiple Built-in Strategies**:
  - Animation LOD (skeletal animation quality control)
  - Vertex Animation Texture (VAT) LOD
  - Mesh swapping LOD
  - Hybrid LOD (combines multiple strategies)
- **Extensible**: Easy to create custom LOD strategies
- **Performance Optimized**: Processing budget system, transition smoothing
- **VAT Support**: Complete implementation with custom shaders

## Quick Start

```rust
use bevy::prelude::*;
use lod_system::prelude::*;

#[derive(Component)]
struct MyEntity;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(LODPlugin::<MyEntity, AnimationLODStrategy>::default())
        .run();
}
```

## File Structure

```
src/lod_system/
├── mod.rs          # Module exports and prelude
├── core.rs         # Core LOD traits and systems
├── strategies.rs   # Built-in LOD strategies
├── vat.rs          # Vertex Animation Texture support
└── examples.rs     # Usage examples

Assets/shaders/
└── vat_shader.wgsl # VAT shader implementation
```

## Key Components

### Core (`core.rs`)
- `LODStrategy` trait - Define custom LOD behaviors
- `LODLevel` - Distance thresholds and update frequencies
- `LODState` - Tracks current LOD state
- `LODPlugin<T, S>` - Generic plugin for any entity/strategy

### Strategies (`strategies.rs`)
- `AnimationLODStrategy` - Controls animation quality
- `VATLODStrategy` - Vertex Animation Texture support
- `MeshSwapLODStrategy` - Simple mesh replacement
- `HybridLODStrategy` - Combines multiple strategies

### VAT Support (`vat.rs`)
- `VATMaterial` - Custom material for VAT rendering
- `VATAnimationState` - Animation playback control
- `VATMaterialPlugin` - Plugin for VAT support
- Helper functions for texture generation

## Migration from Old System

The new system is more flexible and easier to extend:

**Old System:**
```rust
impl LODLevel for AnimationLOD { ... }
impl LODEntity for Rat {}
```

**New System:**
```rust
#[derive(Component)]
struct Rat;

app.add_plugins(LODPlugin::<Rat, AnimationLODStrategy>::default());
```

## Creating Custom Strategies

```rust
struct MyStrategy;

impl LODStrategy for MyStrategy {
    type Config = MyConfig;
    type ComponentData = MyData;
    
    fn transition(...) {
        // Handle LOD level changes
    }
    
    fn update(...) {
        // Optional per-frame updates
    }
}
```

## Performance Considerations

- Processing budget limits operations per frame
- Transition timers prevent visual popping
- Update frequencies reduce unnecessary calculations
- VAT is GPU-efficient for large crowds

## Future Enhancements

- Impostor/billboard rendering
- Procedural animation LOD
- Network-aware LOD
- Audio/Physics LOD integration

See `LOD_SYSTEM_GUIDE.md` for detailed documentation.
