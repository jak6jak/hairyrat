use bevy::prelude::*;
use crate::lod_system::core::*;

// Animation LOD Strategy (similar to your current implementation)
pub struct AnimationLODStrategy;

#[derive(Resource, Default)]
pub struct AnimationLODConfig {
    pub high_quality_distance: f32,
    pub medium_quality_distance: f32,
    pub low_quality_distance: f32,
}

#[derive(Component, Default)]
pub struct AnimationLODData {
    pub animation_enabled: bool,
    pub update_rate: f32,
}

impl LODStrategy for AnimationLODStrategy {
    type Config = AnimationLODConfig;
    type ComponentData = AnimationLODData;
    
    fn transition(
        commands: &mut Commands,
        entity: Entity,
        _from_level: u8,
        to_level: u8,
        _config: &Self::Config,
        component_data: &mut Self::ComponentData,
    ) {
        match to_level {
            0 => { // High quality
                component_data.animation_enabled = true;
                component_data.update_rate = 60.0;
            }
            1 => { // Medium quality
                component_data.animation_enabled = true;
                component_data.update_rate = 10.0;
            }
            2 => { // Low quality
                component_data.animation_enabled = false;
                component_data.update_rate = 2.0;
            }
            _ => { // Culled
                component_data.animation_enabled = false;
                commands.entity(entity).insert(Visibility::Hidden);
            }
        }
        
        if to_level < 3 {
            commands.entity(entity).insert(Visibility::Visible);
        }
    }
    
    fn update(
        _time: &Time,
        _entity: Entity,
        _current_level: u8,
        _component_data: &mut Self::ComponentData,
    ) {
        // Animation updates would be handled by the animation system
    }
}

// Vertex Animation Texture (VAT) LOD Strategy
pub struct VATLODStrategy;

#[derive(Resource)]
pub struct VATLODConfig {
    pub texture_handles: Vec<Handle<Image>>, // Different resolution VAT textures
    pub frame_counts: Vec<u32>,              // Number of frames in each VAT
    pub playback_speeds: Vec<f32>,           // Playback speed for each LOD
}

impl Default for VATLODConfig {
    fn default() -> Self {
        Self {
            texture_handles: Vec::new(),
            frame_counts: vec![60, 30, 15, 1], // Example frame counts
            playback_speeds: vec![1.0, 0.5, 0.25, 0.0],
        }
    }
}

#[derive(Component)]
pub struct VATLODData {
    pub current_frame: f32,
    pub frame_count: u32,
    pub playback_speed: f32,
    pub texture_index: usize,
}

impl Default for VATLODData {
    fn default() -> Self {
        Self {
            current_frame: 0.0,
            frame_count: 60,
            playback_speed: 1.0,
            texture_index: 0,
        }
    }
}

impl LODStrategy for VATLODStrategy {
    type Config = VATLODConfig;
    type ComponentData = VATLODData;
    
    fn transition(
        commands: &mut Commands,
        entity: Entity,
        _from_level: u8,
        to_level: u8,
        config: &Self::Config,
        component_data: &mut Self::ComponentData,
    ) {
        let level_index = to_level as usize;
        
        if level_index < config.texture_handles.len() {
            // Update VAT data
            component_data.texture_index = level_index;
            component_data.frame_count = config.frame_counts[level_index];
            component_data.playback_speed = config.playback_speeds[level_index];
            
            // Update material with new VAT texture
            if let Some(texture) = config.texture_handles.get(level_index) {
                commands.entity(entity).insert(VATTexture(texture.clone()));
            }
            
            // Show entity
            commands.entity(entity).insert(Visibility::Visible);
        } else {
            // Hide entity if beyond LOD range
            commands.entity(entity).insert(Visibility::Hidden);
        }
    }
    
    fn update(
        time: &Time,
        _entity: Entity,
        _current_level: u8,
        component_data: &mut Self::ComponentData,
    ) {
        if component_data.playback_speed > 0.0 {
            component_data.current_frame += time.delta_secs() * component_data.playback_speed * 30.0;
            component_data.current_frame %= component_data.frame_count as f32;
        }
    }
    
    fn requires_update() -> bool {
        true // VAT needs per-frame updates to advance animation
    }
}

// Mesh Swap LOD Strategy - Enhanced to support both meshes and scenes
pub struct MeshSwapLODStrategy;

#[derive(Resource)]
pub struct MeshSwapLODConfig {
    pub mesh_handles: Vec<Handle<Mesh>>,
    pub material_handles: Vec<Handle<StandardMaterial>>,
    // Add scene support for complete model swapping
    pub scene_handles: Vec<Handle<Scene>>,
}

impl Default for MeshSwapLODConfig {
    fn default() -> Self {
        Self {
            mesh_handles: Vec::new(),
            material_handles: Vec::new(),
            scene_handles: Vec::new(),
        }
    }
}

#[derive(Component, Default)]
pub struct MeshSwapLODData {
    pub current_mesh_index: usize,
    pub current_scene_index: usize,
}

impl LODStrategy for MeshSwapLODStrategy {
    type Config = MeshSwapLODConfig;
    type ComponentData = MeshSwapLODData;
    
    fn transition(
        commands: &mut Commands,
        entity: Entity,
        _from_level: u8,
        to_level: u8,
        config: &Self::Config,
        component_data: &mut Self::ComponentData,
    ) {
        let level_index = to_level as usize;
        
        // Prioritize scene swapping if scene handles are available
        if !config.scene_handles.is_empty() && level_index < config.scene_handles.len() {
            component_data.current_scene_index = level_index;
            
            // Swap scene
            if let Some(scene) = config.scene_handles.get(level_index) {
                commands.entity(entity).insert(SceneRoot(scene.clone()));
            }
            
            commands.entity(entity).insert(Visibility::Visible);
        }
        // Fall back to mesh swapping if no scenes available
        else if level_index < config.mesh_handles.len() {
            component_data.current_mesh_index = level_index;
            
            // Swap mesh
            if let Some(mesh) = config.mesh_handles.get(level_index) {
                commands.entity(entity).insert(Mesh3d(mesh.clone()));
            }
            
            // Optionally swap material
            if let Some(material) = config.material_handles.get(level_index) {
                commands.entity(entity).insert(MeshMaterial3d(material.clone()));
            }
            
            commands.entity(entity).insert(Visibility::Visible);
        } else {
            commands.entity(entity).insert(Visibility::Hidden);
        }
    }
    
    fn update(
        _time: &Time,
        _entity: Entity,
        _current_level: u8,
        _component_data: &mut Self::ComponentData,
    ) {
        // No per-frame updates needed for mesh swapping
    }
}

// Hybrid LOD Strategy (combines multiple strategies)
pub struct HybridLODStrategy;

#[derive(Resource, Default)]
pub struct HybridLODConfig {
    pub animation_config: AnimationLODConfig,
    pub vat_config: VATLODConfig,
    pub mesh_swap_config: MeshSwapLODConfig,
    pub use_vat_at_level: u8, // At which LOD level to switch to VAT
}

#[derive(Component, Default)]
pub struct HybridLODData {
    pub animation_data: AnimationLODData,
    pub vat_data: VATLODData,
    pub mesh_swap_data: MeshSwapLODData,
    pub current_strategy: LODStrategyType,
}

#[derive(Default, Clone, Copy, PartialEq)]
pub enum LODStrategyType {
    #[default]
    Animation,
    VAT,
    MeshSwap,
}

impl LODStrategy for HybridLODStrategy {
    type Config = HybridLODConfig;
    type ComponentData = HybridLODData;
    
    fn transition(
        commands: &mut Commands,
        entity: Entity,
        from_level: u8,
        to_level: u8,
        config: &Self::Config,
        component_data: &mut Self::ComponentData,
    ) {
        // Determine which strategy to use based on LOD level
        let new_strategy = if to_level == 0 {
            // Level 0: Use animation with high-quality scene
            LODStrategyType::Animation
        } else if to_level == 1 {
            // Level 1: Use animation with medium-quality scene (mesh swap)
            LODStrategyType::MeshSwap
        } else if to_level < config.use_vat_at_level {
            // Level 2+: Use mesh swapping for different quality scenes
            LODStrategyType::MeshSwap
        } else if to_level < 3 {
            // Optional VAT level (if use_vat_at_level is set low)
            LODStrategyType::VAT
        } else {
            // Level 3+: Hidden/culled
            LODStrategyType::MeshSwap
        };
        
        // Clean up previous strategy if switching
        if component_data.current_strategy != new_strategy {
            match component_data.current_strategy {
                LODStrategyType::Animation => {
                    commands.entity(entity).remove::<AnimationPlayer>();
                    commands.entity(entity).remove::<AnimationTransitions>();
                }
                LODStrategyType::VAT => {
                    commands.entity(entity).remove::<VATTexture>();
                }
                _ => {}
            }
        }
        
        component_data.current_strategy = new_strategy;
        
        // Apply new strategy (can apply multiple strategies)
        match new_strategy {
            LODStrategyType::Animation => {
                // Apply animation strategy
                AnimationLODStrategy::transition(
                    commands,
                    entity,
                    from_level,
                    to_level,
                    &config.animation_config,
                    &mut component_data.animation_data,
                );
                
                // Always apply mesh swap to ensure correct scene for level 0
                MeshSwapLODStrategy::transition(
                    commands,
                    entity,
                    from_level,
                    to_level,
                    &config.mesh_swap_config,
                    &mut component_data.mesh_swap_data,
                );
            }
            LODStrategyType::MeshSwap => {
                // Apply mesh swap strategy
                MeshSwapLODStrategy::transition(
                    commands,
                    entity,
                    from_level,
                    to_level,
                    &config.mesh_swap_config,
                    &mut component_data.mesh_swap_data,
                );
                
                // Also apply animation strategy for levels that need it
                if to_level <= 1 {
                    AnimationLODStrategy::transition(
                        commands,
                        entity,
                        from_level,
                        to_level,
                        &config.animation_config,
                        &mut component_data.animation_data,
                    );
                }
            }
            LODStrategyType::VAT => {
                VATLODStrategy::transition(
                    commands,
                    entity,
                    from_level,
                    to_level,
                    &config.vat_config,
                    &mut component_data.vat_data,
                );
            }
        }
    }
    
    fn update(
        time: &Time,
        entity: Entity,
        current_level: u8,
        component_data: &mut Self::ComponentData,
    ) {
        match component_data.current_strategy {
            LODStrategyType::VAT => {
                VATLODStrategy::update(time, entity, current_level, &mut component_data.vat_data);
            }
            _ => {}
        }
    }
    
    fn requires_update() -> bool {
        true // Because VAT might need updates
    }
}

// Marker component for VAT texture
#[derive(Component)]
pub struct VATTexture(pub Handle<Image>);

// Helper functions for setting up LOD levels
pub fn create_standard_lod_levels() -> Vec<LODLevel> {
    vec![
        LODLevel::new(0, 0.0, 10.0, 1.0 / 60.0),      // High quality
        LODLevel::new(1, 10.0, 25.0, 1.0 / 30.0),     // Medium quality
        LODLevel::new(2, 25.0, 50.0, 1.0 / 10.0),     // Low quality
        LODLevel::new(3, 50.0, f32::MAX, 1.0),        // Culled/Static
    ]
}

pub fn create_aggressive_lod_levels() -> Vec<LODLevel> {
    vec![
        LODLevel::new(0, 0.0, 5.0, 1.0 / 60.0),       // High quality
        LODLevel::new(1, 5.0, 15.0, 1.0 / 20.0),      // Medium quality
        LODLevel::new(2, 15.0, 30.0, 1.0 / 5.0),      // Low quality
        LODLevel::new(3, 30.0, f32::MAX, 1.0),        // Culled/Static
    ]
}
