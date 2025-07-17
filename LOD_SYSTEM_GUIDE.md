# Generic LOD System Guide

## Overview

The new LOD system has been redesigned to be more reusable and generic, supporting multiple LOD strategies including:
- Animation-based LOD (skeletal animation)
- Vertex Animation Texture (VAT) LOD
- Mesh swapping LOD
- Hybrid approaches combining multiple strategies

## Architecture

### Core Components (`lod_core.rs`)

1. **LODStrategy Trait**: The heart of the system. Implement this to create custom LOD behaviors.
```rust
pub trait LODStrategy: Send + Sync + 'static {
    type Config: Resource + Default;
    type ComponentData: Component + Default;
    
    fn transition(...);  // Called when LOD level changes
    fn update(...);      // Called per-frame if needed
    fn requires_update() -> bool;  // Whether per-frame updates are needed
}
```

2. **LODLevel**: Defines distance thresholds and update frequencies
```rust
LODLevel::new(
    level: u8,           // LOD level (0 = highest quality)
    min_distance: f32,   // Minimum distance for this LOD
    max_distance: f32,   // Maximum distance for this LOD
    update_frequency: f32 // How often to update (in seconds)
)
```

3. **LODState**: Tracks current LOD state and transitions
4. **LODPlugin<T, S>**: Generic plugin that works with any entity type T and strategy S

### Pre-built Strategies (`lod_strategies.rs`)

#### 1. AnimationLODStrategy
- Controls skeletal animation quality based on distance
- Reduces animation update rate for distant objects
- Disables animation entirely for very distant objects

#### 2. VATLODStrategy
- Uses Vertex Animation Textures for efficient animation
- Supports different texture resolutions for different LOD levels
- Can reduce frame count and playback speed at distance

#### 3. MeshSwapLODStrategy
- Swaps between different mesh resolutions
- Simple but effective for static geometry

#### 4. HybridLODStrategy
- Combines multiple strategies
- Example: Use skeletal animation up close, VAT at medium distance, static mesh far away

## Usage Examples

### Basic Setup with Animation LOD

```rust
use bevy::prelude::*;
use lod_core::*;
use lod_strategies::*;

// Define your entity type
#[derive(Component)]
struct MyEntity;

fn setup(mut commands: Commands) {
    // Define LOD levels
    let lod_levels = vec![
        LODLevel::new(0, 0.0, 10.0, 1.0/60.0),     // High quality
        LODLevel::new(1, 10.0, 25.0, 1.0/30.0),    // Medium quality
        LODLevel::new(2, 25.0, 50.0, 1.0/10.0),    // Low quality
        LODLevel::new(3, 50.0, f32::MAX, 1.0),     // Culled
    ];
    
    commands.insert_resource(LODLevels::<MyEntity>::new(lod_levels));
    
    // Spawn entity with LOD components
    commands.spawn((
        // Your mesh/scene components
        MyEntity,
        LODDistance::default(),
        LODState::new(LODLevel::new(0, 0.0, 10.0, 1.0/60.0)),
        AnimationLODData::default(),
    ));
}

// Add the plugin
app.add_plugins(LODPlugin::<MyEntity, AnimationLODStrategy>::default());
```

### Using VAT LOD

```rust
// Setup VAT configuration
let vat_config = VATLODConfig {
    texture_handles: vec![
        high_res_vat_texture,
        medium_res_vat_texture,
        low_res_vat_texture,
    ],
    frame_counts: vec![60, 30, 15],
    playback_speeds: vec![1.0, 0.5, 0.25],
};

commands.insert_resource(vat_config);

// Spawn VAT entity
commands.spawn((
    VATBundle {
        mesh: vat_mesh,
        material: vat_material,
        transform: Transform::default(),
        ..default()
    },
    MyEntity,
    LODDistance::default(),
    LODState::new(initial_lod_level),
    VATLODData::default(),
));

// Add plugins
app.add_plugins(LODPlugin::<MyEntity, VATLODStrategy>::default())
   .add_plugins(VATMaterialPlugin);
```

### Creating a Custom LOD Strategy

```rust
struct MyCustomStrategy;

#[derive(Resource, Default)]
struct MyCustomConfig {
    // Your configuration
}

#[derive(Component, Default)]
struct MyCustomData {
    // Per-entity data
}

impl LODStrategy for MyCustomStrategy {
    type Config = MyCustomConfig;
    type ComponentData = MyCustomData;
    
    fn transition(
        commands: &mut Commands,
        entity: Entity,
        from_level: u8,
        to_level: u8,
        config: &Self::Config,
        component_data: &mut Self::ComponentData,
    ) {
        // Handle LOD transitions
        match to_level {
            0 => { /* High quality setup */ }
            1 => { /* Medium quality setup */ }
            2 => { /* Low quality setup */ }
            _ => { /* Culled */ }
        }
    }
    
    fn update(
        time: &Time,
        entity: Entity,
        current_level: u8,
        component_data: &mut Self::ComponentData,
    ) {
        // Optional per-frame updates
    }
    
    fn requires_update() -> bool {
        false // Set to true if you need per-frame updates
    }
}
```

## Vertex Animation Texture (VAT) Implementation

### VAT Material Setup

The system includes a complete VAT implementation with:
- Custom shader (`vat_shader.wgsl`)
- Material definition (`VATMaterial`)
- Animation state management
- Helper functions for texture generation

### Creating VAT Textures

VAT textures store vertex positions and normals for each frame of animation:

```rust
// Calculate optimal texture dimensions
let (width, height) = calculate_vat_texture_dimensions(
    vertex_count,
    frame_count
);

// Create VAT material
let vat_material = create_vat_material(
    position_texture,
    normal_texture,
    base_color_texture,
    total_frames: 60,
    vertex_count: 5000,
    texture_width: 2048,
    bbox_min: Vec3::new(-1.0, -1.0, -1.0),
    bbox_max: Vec3::new(1.0, 1.0, 1.0),
);
```

### VAT Shader

The included shader supports:
- Position and normal animation from textures
- Multiple LOD levels with different frame counts
- Simplified static mode for distant objects
- Standard PBR material properties

## Best Practices

1. **Choose the Right Strategy**:
   - Use Animation LOD for characters with complex skeletal animations
   - Use VAT for crowds or repeated animated objects
   - Use Mesh Swap for static geometry with multiple detail levels
   - Use Hybrid for maximum flexibility

2. **Configure LOD Levels Appropriately**:
   - Test different distance thresholds for your use case
   - Balance quality vs performance
   - Consider your target hardware

3. **Optimize Update Frequencies**:
   - High LOD: 60 FPS updates
   - Medium LOD: 10-30 FPS updates
   - Low LOD: 1-5 FPS updates
   - Culled: No updates

4. **Memory Considerations**:
   - VAT textures can be large - use compression where possible
   - Share textures between similar objects
   - Unload unused LOD assets when possible

5. **Transition Smoothing**:
   - The system includes transition timers to prevent popping
   - Adjust transition delays based on your needs
   - Consider fade transitions for smoother changes

## Migration from Old System

To migrate from the old LOD system:

1. Replace `LODLevel` trait implementations with LOD level configurations
2. Replace `LODEntity` marker traits with simple marker components
3. Update system registration to use the new plugin syntax
4. Move LOD logic into strategy implementations

Example migration:
```rust
// Old
impl LODLevel for AnimationLOD { ... }
impl LODEntity for Rat {}

// New
#[derive(Component)]
struct Rat;

app.add_plugins(LODPlugin::<Rat, AnimationLODStrategy>::default());
```

## Performance Tips

1. **Budget Processing**: The system includes a processing budget to limit operations per frame
2. **Spatial Partitioning**: Consider adding spatial partitioning for very large numbers of entities
3. **GPU Instancing**: VAT is particularly well-suited for GPU instancing
4. **Texture Atlasing**: Combine multiple VAT animations into texture atlases

## Future Enhancements

The system is designed to be extensible. Potential additions:
- Impostor/billboard LOD strategy
- Procedural animation LOD
- Network-aware LOD for multiplayer
- Audio LOD integration
- Physics LOD integration
