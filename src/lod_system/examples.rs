use bevy::prelude::*;
use crate::lod_system::core::*;
use crate::lod_system::strategies::*;
use crate::lod_system::vat::*;

/// Example of how to use the new generic LOD system with rats
/// This shows how to set up different LOD strategies

// Marker component for rats
#[derive(Component)]
pub struct Rat;

/// Example setup for Animation-based LOD
pub fn setup_animation_lod(
    mut commands: Commands,
    rat_models: Res<RatModels>,
    animations: Res<Animations>,
) {
    // Configure LOD levels
    let lod_levels = vec![
        LODLevel::new(0, 0.0, 10.0, 1.0 / 60.0),      // High - full animation
        LODLevel::new(1, 10.0, 25.0, 1.0 / 30.0),     // Medium - reduced rate
        LODLevel::new(2, 25.0, 50.0, 1.0 / 10.0),     // Low - very reduced rate
        LODLevel::new(3, 50.0, f32::MAX, 1.0),        // Culled
    ];
    
    commands.insert_resource(LODLevels::<Rat>::new(lod_levels));
    
    // Spawn rats with LOD components
    for x in 0..10 {
        for y in 0..10 {
            commands.spawn((
                SceneRoot(rat_models.rat_lod0.clone()),
                Transform::from_xyz(x as f32 * 2.0, 0.0, y as f32 * 2.0),
                Rat,
                LODDistance::default(),
                LODState::new(LODLevel::new(0, 0.0, 10.0, 1.0 / 60.0)),
                AnimationLODData::default(),
                AnimationGraphHandle(animations.graph.clone()),
            ));
        }
    }
}

/// Example setup for VAT-based LOD
pub fn setup_vat_lod(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<VATMaterial>>,
    mut images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
) {
    // Load VAT textures (these would be pre-generated from your rat animation)
    let position_texture_high = asset_server.load("vat/rat_positions_high.exr");
    let normal_texture_high = asset_server.load("vat/rat_normals_high.exr");
    let position_texture_low = asset_server.load("vat/rat_positions_low.exr");
    let normal_texture_low = asset_server.load("vat/rat_normals_low.exr");
    let base_color = asset_server.load("blackrat_free_glb/blackrat_color.png");
    
    // Create VAT materials for different LOD levels
    let vat_material_high = materials.add(create_vat_material(
        position_texture_high.clone(),
        normal_texture_high.clone(),
        Some(base_color.clone()),
        60,    // 60 frames
        5000,  // vertex count
        2048,  // texture width
        Vec3::new(-1.0, -1.0, -1.0),
        Vec3::new(1.0, 1.0, 1.0),
    ));
    
    let vat_material_low = materials.add(create_vat_material(
        position_texture_low.clone(),
        normal_texture_low.clone(),
        Some(base_color.clone()),
        30,    // 30 frames for lower LOD
        2500,  // reduced vertex count
        1024,  // smaller texture
        Vec3::new(-1.0, -1.0, -1.0),
        Vec3::new(1.0, 1.0, 1.0),
    ));
    
    // Configure VAT LOD
    let mut vat_config = VATLODConfig::default();
    vat_config.texture_handles = vec![
        position_texture_high,
        position_texture_low.clone(),
        position_texture_low, // Reuse low for very far
    ];
    vat_config.frame_counts = vec![60, 30, 1]; // Static frame for furthest
    vat_config.playback_speeds = vec![1.0, 0.5, 0.0];
    
    commands.insert_resource(vat_config);
    
    // Load base mesh
    let rat_mesh = asset_server.load("blackrat_furless/rat_without_fur.glb#Mesh0");
    
    // Spawn VAT rats
    for x in 0..10 {
        for y in 0..10 {
            commands.spawn((
                VATBundle {
                    mesh_handle: MeshMaterial3d(vat_material_high.clone()),
                    transform: Transform::from_xyz(x as f32 * 2.0, 0.0, y as f32 * 2.0),
                    ..default()
                },
                Mesh3d(rat_mesh.clone()),
                Rat,
                LODDistance::default(),
                LODState::new(LODLevel::new(0, 0.0, 10.0, 1.0 / 60.0)),
                VATLODData::default(),
            ));
        }
    }
}

/// Example setup for Hybrid LOD (Animation -> VAT -> Static)
pub fn setup_hybrid_lod(
    mut commands: Commands,
    rat_models: Res<RatModels>,
    animations: Res<Animations>,
    asset_server: Res<AssetServer>,
) {
    // Configure hybrid LOD
    let mut hybrid_config = HybridLODConfig::default();
    hybrid_config.use_vat_at_level = 2; // Switch to VAT at LOD level 2
    
    // Animation config for close distances
    hybrid_config.animation_config = AnimationLODConfig {
        high_quality_distance: 10.0,
        medium_quality_distance: 20.0,
        low_quality_distance: 30.0,
    };
    
    // VAT config for medium distances
    hybrid_config.vat_config.texture_handles = vec![
        asset_server.load("vat/rat_positions_med.exr"),
        asset_server.load("vat/rat_positions_low.exr"),
    ];
    hybrid_config.vat_config.frame_counts = vec![30, 15];
    hybrid_config.vat_config.playback_speeds = vec![0.5, 0.25];
    
    // Mesh swap config for far distances
    hybrid_config.mesh_swap_config.mesh_handles = vec![
        asset_server.load("blackrat_furless/rat_without_fur.glb#Mesh0"),
        asset_server.load("blackrat_furless/rat_without_furlod2.glb#Mesh0"),
    ];
    
    commands.insert_resource(hybrid_config);
    
    // Define LOD levels
    let lod_levels = vec![
        LODLevel::new(0, 0.0, 10.0, 1.0 / 60.0),      // High - full animation
        LODLevel::new(1, 10.0, 20.0, 1.0 / 30.0),     // Medium - reduced animation
        LODLevel::new(2, 20.0, 40.0, 1.0 / 10.0),     // Low - VAT
        LODLevel::new(3, 40.0, 60.0, 1.0 / 5.0),      // Very Low - Simple VAT
        LODLevel::new(4, 60.0, f32::MAX, 1.0),        // Static mesh
    ];
    
    commands.insert_resource(LODLevels::<Rat>::new(lod_levels));
    
    // Spawn hybrid LOD rats
    for x in 0..10 {
        for y in 0..10 {
            commands.spawn((
                SceneRoot(rat_models.rat_lod0.clone()),
                Transform::from_xyz(x as f32 * 2.0, 0.0, y as f32 * 2.0),
                Rat,
                LODDistance::default(),
                LODState::new(LODLevel::new(0, 0.0, 10.0, 1.0 / 60.0)),
                HybridLODData::default(),
                AnimationGraphHandle(animations.graph.clone()),
            ));
        }
    }
}

/// System to debug LOD states
pub fn debug_lod_system(
    query: Query<(&LODState, &LODDistance), With<Rat>>,
    time: Res<Time>,
    mut last_print: Local<f32>,
) {
    if time.elapsed_secs() - *last_print > 2.0 {
        let mut level_counts = [0; 5];
        let mut total = 0;
        
        for (lod_state, _distance) in query.iter() {
            let level = lod_state.current_level.level as usize;
            if level < level_counts.len() {
                level_counts[level] += 1;
            }
            total += 1;
        }
        
        println!("LOD Distribution:");
        for (level, count) in level_counts.iter().enumerate() {
            if *count > 0 {
                println!("  Level {}: {} entities ({:.1}%)", 
                    level, count, (*count as f32 / total as f32) * 100.0);
            }
        }
        
        *last_print = time.elapsed_secs();
    }
}

/// Example app setup showing different LOD strategies
pub fn setup_lod_example_app(app: &mut App) {
    // Choose which LOD strategy to use
    
    // Option 1: Animation-based LOD (similar to current implementation)
    app.add_plugins(LODPlugin::<Rat, AnimationLODStrategy>::default());
    
    // Option 2: VAT-based LOD
    // app.add_plugins(LODPlugin::<Rat, VATLODStrategy>::default())
    //    .add_plugins(VATMaterialPlugin);
    
    // Option 3: Hybrid LOD (most flexible)
    // app.add_plugins(LODPlugin::<Rat, HybridLODStrategy>::default())
    //    .add_plugins(VATMaterialPlugin);
    
    // Add debug system
    app.add_systems(Update, debug_lod_system);
}

// Resource definitions (these would come from your asset loading)
#[derive(Resource)]
pub struct RatModels {
    pub rat_lod0: Handle<Scene>,
    pub rat_lod1: Handle<Scene>,
    pub rat_lod2: Handle<Scene>,
}

#[derive(Resource)]
pub struct Animations {
    pub graph: Handle<AnimationGraph>,
    pub node_indices: Vec<AnimationNodeIndex>,
}
