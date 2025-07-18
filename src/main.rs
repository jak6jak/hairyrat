mod lod;
mod lod_system;

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_inspector_egui::egui::debug_text::print;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use std::time::Duration;
use iyes_perf_ui::prelude::*;

// Import the new LOD system
use lod_system::prelude::*;
use lod_system::strategies::{MeshSwapLODConfig, HybridLODStrategy, HybridLODConfig, HybridLODData, AnimationLODConfig};

#[derive(Resource)]
struct Animations {
    graph: Handle<AnimationGraph>,
    node_indices: Vec<AnimationNodeIndex>,
}

// Rat entity marker
#[derive(Component)]
pub struct Rat;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PanOrbitCameraPlugin,
        ))
        // Use the hybrid LOD system that combines multiple strategies
        .add_plugins(LODPlugin::<Rat, HybridLODStrategy>::default())
        .insert_resource(LODLevels::<Rat>::new(create_standard_lod_levels()))
        .init_state::<AppState>()
        .add_loading_state(
            LoadingState::new(AppState::Loading)
                .continue_to_state(AppState::InGame)
                .load_collection::<RatAssets>()
                .load_collection::<EnvironmentAssets>(),
        )
        .add_systems(OnEnter(AppState::Loading), show_loading_screen)
        .add_systems(OnExit(AppState::Loading), hide_loading_screen)
        .add_systems(OnEnter(AppState::InGame), setup_scene)
        .add_systems(
            Update,
            (
                setup_initial_animations,
                handle_animation_lod,
                debug_lod_stats,
            ).run_if(in_state(AppState::InGame))
        )
        .run();
}

fn setup_scene(
    mut commands: Commands,
    rat_assets: Res<RatAssets>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    env: Res<EnvironmentAssets>,
) {
    commands.spawn(PerfUiAllEntries::default());

    // Setup Hybrid LOD configuration that combines multiple strategies
    let hybrid_config = HybridLODConfig {
        animation_config: AnimationLODConfig {
            high_quality_distance: 10.0,
            medium_quality_distance: 25.0,
            low_quality_distance: 50.0,
        },
        vat_config: Default::default(), // Not using VAT for now
        mesh_swap_config: MeshSwapLODConfig {
            mesh_handles: vec![],
            material_handles: vec![],
            scene_handles: vec![
                // Level 0: High quality furry rat
                rat_assets.rat.clone(),
                // Level 1: Medium quality - use furless rat
                rat_assets.rat_lod0.clone(),
                // Level 2: Low quality - use furless rat again
                rat_assets.rat_lod0.clone(),
                // Level 3: Hidden
            ],
        },
        use_vat_at_level: 99, // Never switch to VAT (you can change this)
    };
    commands.insert_resource(hybrid_config);

    // Camera setup - position it to see the LOD transitions
    commands.spawn((
        Transform::from_translation(Vec3::new(0.0, 15.0, 30.0))
            .looking_at(Vec3::ZERO, Vec3::Y),
        PanOrbitCamera::default(),
        EnvironmentMapLight {
            diffuse_map: env.diffuse_map.clone(),
            specular_map: env.specular_map.clone(),
            intensity: 900.0,
            ..default()
        },
    ));

    // Lighting
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, -0.5, 0.0)),
    ));

    // Setup animation - use animation from the main animated GLB file
    let (graph, index) = AnimationGraph::from_clip(rat_assets.animation_clip.clone());
    let graph_handle = graphs.add(graph);
    let spawn_base = commands
        .spawn((Transform::default(), Visibility::default(), Rat))
        .id();


    commands.insert_resource(Animations {
        graph: graph_handle.clone(),
        node_indices: vec![index],
    });

    // Spawn rats with strategy-based LOD system
    // The MeshSwapLODStrategy will handle swapping between scenes based on distance
    let high_quality_scene = rat_assets.rat.clone();
    
    let lod_levels = create_standard_lod_levels();
    let initial_lod = lod_levels[0]; // Start with highest quality
    
    commands.spawn_batch((0..100).flat_map(|x| (0..100).map(move |y| (x, y))).map(
        move |(x, y)| {
            // Start with high quality scene for all entities
            // The LOD system will swap to appropriate scenes based on distance
            // Spread rats out more to test LOD levels
            (
                SceneRoot(high_quality_scene.clone()),
                Transform::from_xyz((x as f32 - 25.0) / 4.0, (y as f32 - 25.0) / 4.0, 0.0)
                    .with_scale(Vec3::splat(1.0)),
                AnimationGraphHandle(graph_handle.clone()),
                Rat,
                ChildOf(spawn_base),
                // Components for hybrid LOD system
                LODState::new(initial_lod),
                LODDistance::default(),
                HybridLODData::default(),
            )
        },
    ));
}

// System to setup initial animations for all rats
fn setup_initial_animations(
    animations: Res<Animations>,
    mut commands: Commands,
    query: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) { 
    for (_entity, mut player) in query {

         let mut animation_transitions = AnimationTransitions::new();
        animation_transitions
            .play(
                &mut player,
                animations.node_indices[0],
                Duration::from_millis(15),
            )
            .repeat();

        commands
            .entity(_entity)
           .insert(AnimationGraphHandle(animations.graph.clone()))
           .insert(animation_transitions);
    }
}

// System to handle animation based on hybrid LOD data
fn handle_animation_lod(
    animations: Res<Animations>,
    mut commands: Commands,
    mut query: Query<(Entity, &HybridLODData, &Children), (With<Rat>, Changed<HybridLODData>)>,
    mut animation_players: Query<&mut AnimationPlayer>,
) {
    for (_entity, hybrid_lod_data, children) in query.iter_mut() {
        // Access the animation data from the hybrid strategy
        let lod_data = &hybrid_lod_data.animation_data;
        
        // Find animation player in children
        for child in children.iter() {
            if let Ok(mut player) = animation_players.get_mut(child) {
                if lod_data.animation_enabled {
                    // Ensure animation is playing
                    if !player.is_playing_animation(animations.node_indices[0]) {
                        let mut transitions = AnimationTransitions::new();
                        transitions
                            .play(&mut player, animations.node_indices[0], Duration::from_millis(15))
                            .repeat();
                        commands.entity(child).insert(transitions);
                    }
                    
                    // Adjust playback speed based on LOD
                    // Note: In Bevy 0.16, AnimationPlayer doesn't have set_speed method
                    // Speed control would need to be implemented differently
                } else {
                    // Stop animation for low LOD
                    player.stop_all();
                }
            }
        }
    }
}

fn debug_lod_stats(
    query: Query<(&LODState, &LODDistance, &HybridLODData, Option<&AnimationPlayer>), With<Rat>>,
    time: Res<Time>,
    mut last_print: Local<f32>,
) {
    if time.elapsed_secs() - *last_print > 2.0 {
        let mut level_counts = [0; 4];
        let mut animated_count = 0;
        let mut strategy_counts = [0; 3]; // Animation, VAT, MeshSwap
        let total = query.iter().len();
        
        for (lod_state, _distance, hybrid_data, animation_player) in query.iter() {
            let level = lod_state.current_level.level as usize;
            if level < level_counts.len() {
                level_counts[level] += 1;
            }
            if animation_player.is_some() {
                animated_count += 1;
            }
            
            // Count strategy usage
            match hybrid_data.current_strategy {
                lod_system::strategies::LODStrategyType::Animation => strategy_counts[0] += 1,
                lod_system::strategies::LODStrategyType::VAT => strategy_counts[1] += 1,
                lod_system::strategies::LODStrategyType::MeshSwap => strategy_counts[2] += 1,
            }
        }
        
        println!("\n=== LOD Stats (Hybrid Strategy) ===");
        println!("Total entities: {}", total);
        for (level, count) in level_counts.iter().enumerate() {
            if *count > 0 {
                let level_name = match level {
                    0 => "High   ",
                    1 => "Medium ",
                    2 => "Low    ",
                    3 => "Culled ",
                    _ => "Unknown",
                };
                println!("  {} (L{}): {} entities ({:.1}%)", 
                    level_name, level, count, (*count as f32 / total as f32) * 100.0);
            }
        }
        
        println!("Strategy Usage:");
        println!("  Animation: {} ({:.1}%)", strategy_counts[0], (strategy_counts[0] as f32 / total as f32) * 100.0);
        println!("  VAT:       {} ({:.1}%)", strategy_counts[1], (strategy_counts[1] as f32 / total as f32) * 100.0);
        println!("  MeshSwap:  {} ({:.1}%)", strategy_counts[2], (strategy_counts[2] as f32 / total as f32) * 100.0);
        
        println!("Animated: {} / {} ({:.1}% performance saving)", 
            animated_count, total, (1.0 - animated_count as f32 / total as f32) * 100.0);
        
        *last_print = time.elapsed_secs();
    }
}

#[derive(AssetCollection, Resource)]
struct RatAssets {
    #[asset(path = "blackrat_free_glb/blackrat.glb#Scene0")]
    rat: Handle<Scene>,
    #[asset(path = "blackrat_furless/rat_without_fur.glb#Scene0")]
    rat_lod0: Handle<Scene>,
    #[asset(path = "blackrat_furless/rat_without_fur.glb#Animation1")]
    rat_lod0_animation: Handle<AnimationClip>,
    #[asset(path = "blackrat_free_glb/blackrat.glb#Animation1")]
    animation_clip: Handle<AnimationClip>,
}

#[derive(AssetCollection, Resource)]
struct EnvironmentAssets {
    #[asset(path = "EnviromentMaps/pisa_diffuse_rgb9e5_zstd.ktx2")]
    diffuse_map: Handle<Image>,
    #[asset(path = "EnviromentMaps/pisa_specular_rgb9e5_zstd.ktx2")]
    specular_map: Handle<Image>,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum AppState {
    #[default]
    Loading,
    InGame,
}

// Loading screen functions
fn show_loading_screen(mut commands: Commands) {
    commands.spawn((
        Text::new("Loading Assets..."),
        TextFont {
            font_size: 40.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(50.0),
            left: Val::Percent(50.0),
            ..default()
        },
        LoadingScreenMarker,
    ));
}

fn hide_loading_screen(
    mut commands: Commands,
    loading_text: Query<Entity, With<LoadingScreenMarker>>,
) {
    for entity in loading_text.iter() {
        commands.entity(entity).despawn();
    }
}

#[derive(Component)]
struct LoadingScreenMarker;
