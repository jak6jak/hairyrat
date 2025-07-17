# Design Document: Skeletal Mesh Optimization System

## Overview

This design implements a comprehensive skeletal mesh optimization system for the Bevy-based rat simulation. The system addresses performance bottlenecks when rendering thousands of animated rats by implementing Level of Detail (LOD) systems, animation time slicing, and vertex animations for distant objects.

The current implementation spawns 2500 rats in a grid formation, each with full skeletal animation. This design will optimize performance while maintaining visual quality through intelligent distance-based rendering strategies.

## Architecture

### Core Components

The optimization system consists of several interconnected components:

1. **LOD Manager**: Central system that manages distance calculations and LOD level assignments
2. **Animation Time Slicer**: Distributes animation updates across multiple frames
3. **Vertex Animation System**: Handles pre-baked vertex animations for distant objects
4. **Spatial Partitioning**: Efficient distance calculation and culling system
5. **Performance Monitor**: Adaptive quality adjustment based on framerate

### System Integration

The optimization system integrates with Bevy's existing animation and rendering pipeline:

- Hooks into Bevy's `AnimationPlayer` system for skeletal animations
- Utilizes Bevy's transform and visibility systems for culling
- Leverages Bevy's asset system for animation data management
- Integrates with Bevy's rendering pipeline for batched vertex animations

## Components and Interfaces

### LOD Manager Component

```rust
#[derive(Component)]
pub struct LodLevel {
    pub current_level: u8,
    pub last_update_frame: u64,
    pub distance_to_camera: f32,
}

#[derive(Resource)]
pub struct LodSettings {
    pub close_distance: f32,      // < 10 units
    pub medium_distance: f32,     // 10-25 units
    pub far_distance: f32,        // > 25 units
    pub cull_distance: f32,       // Beyond this, don't render
    pub hysteresis_factor: f32,   // Prevent LOD thrashing
}
```

### Animation Time Slicing

```rust
#[derive(Component)]
pub struct AnimationTimeSlice {
    pub update_interval: u8,      // Frames between updates
    pub frame_offset: u8,         // Stagger updates across rats
    pub last_update_frame: u64,
}

#[derive(Resource)]
pub struct AnimationScheduler {
    pub current_frame: u64,
    pub active_animations: Vec<Entity>,
    pub frame_buckets: HashMap<u8, Vec<Entity>>,
}
```

### Vertex Animation System

```rust
#[derive(Component)]
pub struct VertexAnimation {
    pub animation_data: Handle<VertexAnimationClip>,
    pub current_frame: f32,
    pub playback_speed: f32,
}

#[derive(Asset)]
pub struct VertexAnimationClip {
    pub frames: Vec<VertexFrame>,
    pub frame_rate: f32,
    pub loop_animation: bool,
}

#[derive(Clone)]
pub struct VertexFrame {
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
}
```

### Spatial Partitioning

```rust
#[derive(Resource)]
pub struct SpatialGrid {
    pub cell_size: f32,
    pub cells: HashMap<IVec2, Vec<Entity>>,
    pub dirty_cells: HashSet<IVec2>,
}

#[derive(Component)]
pub struct SpatialCell {
    pub grid_position: IVec2,
    pub last_position: Vec3,
}
```

## Data Models

### LOD Levels

- **Level 0 (Close)**: Full skeletal animation with all bones, updated every frame
- **Level 1 (Medium)**: Reduced bone count, updated every 2-3 frames
- **Level 2 (Far)**: Vertex animation or simplified skeletal animation, updated every 4-8 frames
- **Level 3 (Very Far)**: Static pose or culled entirely

### Animation Data Hierarchy

```
AnimationAssets
├── FullSkeletalAnimation (Level 0)
│   ├── All bones active
│   └── Full animation fidelity
├── ReducedSkeletalAnimation (Level 1)
│   ├── Key bones only (spine, limbs)
│   └── Simplified animation curves
├── VertexAnimation (Level 2)
│   ├── Pre-baked vertex positions
│   └── Compressed animation data
└── StaticPose (Level 3)
    └── Single frame representation
```

### Memory Management

- **Shared Animation Clips**: Multiple rats at the same LOD level share animation data
- **Lazy Loading**: Animation data loaded on-demand based on current LOD requirements
- **LRU Cache**: Least recently used animation data evicted when memory pressure increases
- **Compression**: Vertex animations use compressed formats to reduce memory footprint

## Error Handling

### Graceful Degradation

1. **Missing Animation Data**: Fall back to lower LOD level or static pose
2. **Performance Issues**: Automatically reduce LOD distances and increase time slicing
3. **Memory Pressure**: Aggressively cull distant objects and unload unused animations
4. **Asset Loading Failures**: Use placeholder animations until assets are available

### Error Recovery Strategies

```rust
pub enum OptimizationError {
    AnimationDataMissing(Entity),
    PerformanceThresholdExceeded,
    MemoryLimitReached,
    SpatialPartitioningFailed,
}

impl OptimizationError {
    pub fn handle(&self, world: &mut World) {
        match self {
            AnimationDataMissing(entity) => {
                // Fall back to static pose
                world.entity_mut(*entity).insert(StaticPose::default());
            }
            PerformanceThresholdExceeded => {
                // Reduce LOD distances by 20%
                let mut settings = world.resource_mut::<LodSettings>();
                settings.scale_distances(0.8);
            }
            MemoryLimitReached => {
                // Trigger aggressive cleanup
                world.run_system_once(cleanup_unused_animations);
            }
            SpatialPartitioningFailed => {
                // Rebuild spatial grid
                world.run_system_once(rebuild_spatial_grid);
            }
        }
    }
}
```

## Testing Strategy

### Unit Tests

1. **LOD Level Calculation**: Test distance-based LOD assignment with various camera positions
2. **Animation Time Slicing**: Verify correct frame distribution and update intervals
3. **Vertex Animation Playback**: Test animation frame interpolation and looping
4. **Spatial Partitioning**: Validate grid cell assignment and neighbor queries
5. **Memory Management**: Test LRU cache behavior and memory cleanup

### Integration Tests

1. **End-to-End Performance**: Measure framerate with 2500+ animated rats
2. **LOD Transitions**: Verify smooth transitions between LOD levels without popping
3. **Animation Synchronization**: Ensure vertex and skeletal animations stay in sync
4. **Adaptive Quality**: Test automatic quality adjustment under performance pressure
5. **Memory Usage**: Monitor memory consumption over extended runtime

### Performance Benchmarks

```rust
#[cfg(test)]
mod benchmarks {
    use criterion::{black_box, criterion_group, criterion_main, Criterion};
    
    fn bench_lod_calculation(c: &mut Criterion) {
        c.bench_function("lod_calculation_2500_rats", |b| {
            b.iter(|| {
                // Benchmark LOD calculation for 2500 rats
                calculate_lod_levels(black_box(&rat_positions), black_box(&camera_pos))
            })
        });
    }
    
    fn bench_animation_time_slicing(c: &mut Criterion) {
        c.bench_function("animation_scheduling", |b| {
            b.iter(|| {
                // Benchmark animation update scheduling
                schedule_animation_updates(black_box(&animation_entities))
            })
        });
    }
    
    criterion_group!(benches, bench_lod_calculation, bench_animation_time_slicing);
    criterion_main!(benches);
}
```

### Visual Testing

1. **LOD Debug Visualization**: Color-code rats by current LOD level
2. **Animation Frame Indicators**: Show which rats are updating animations each frame
3. **Performance Overlay**: Display real-time performance metrics and optimization status
4. **Distance Rings**: Visualize LOD distance thresholds around camera

## Implementation Considerations

### Bevy-Specific Integration

- **System Ordering**: Ensure LOD calculation runs before animation updates
- **Change Detection**: Use Bevy's change detection to minimize unnecessary calculations
- **Asset Management**: Leverage Bevy's asset system for efficient animation data loading
- **Component Queries**: Optimize queries using Bevy's query filters and iteration patterns

### Performance Optimizations

- **SIMD Operations**: Use SIMD for batch distance calculations where possible
- **Parallel Processing**: Utilize Bevy's parallel query iteration for LOD updates
- **Memory Layout**: Structure components for cache-friendly access patterns
- **Batch Operations**: Group similar operations to reduce system overhead

### Scalability Considerations

- **Dynamic Rat Count**: System should handle varying numbers of rats efficiently
- **Camera Movement**: Optimize for frequent camera position changes
- **Scene Complexity**: Account for varying scene complexity and occlusion
- **Hardware Adaptation**: Automatically adjust settings based on detected hardware capabilities