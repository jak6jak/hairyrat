mod lod;
mod lod_system;

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use std::time::Duration;
use iyes_perf_ui::prelude::*;

// Import the new LOD system
use lod_system::prelude::*;
use lod_system::strategies::AnimationLODData;

#[derive(Resource)]
struct Animations {
    graph: Handle<AnimationGraph>,
    node_indices: Vec<AnimationNodeIndex>,
}

#[derive(Component)]
struct Rat;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PanOrbitCameraPlugin,
            // Performance UI plugin
            iyes_perf_ui::PerfUiPlugin,
        ))
        // Initialize LOD levels resource early
        .insert_resource(LODLevels::<Rat>::new(vec![
            LODLevel::new(0, 0.0, 8.0, 1.0 / 60.0),      // High - full animation
            LODLevel::new(1, 8.0, 15.0, 1.0 / 30.0),     // Medium - reduced rate
            LODLevel::new(2, 15.0, 25.0, 1.0 / 10.0),    // Low - very reduced rate
            LODLevel::new(3, 25.0, f32::MAX, 1.0),       // Culled
        ]))
        // Use the new generic LOD system with Animation strategy
        .add_plugins(LODPlugin::<Rat, AnimationLODStrategy>::default())
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

    // Camera setup
    commands.spawn((
        Transform::from_translation(Vec3::new(0.0, 1.5, 5.0)),
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

    // Setup animation
    let (graph, index) = AnimationGraph::from_clip(rat_assets.animation_clip.clone());
    let graph_handle = graphs.add(graph);

    commands.insert_resource(Animations {
        graph: graph_handle.clone(),
        node_indices: vec![index],
    });

    // Spawn rats with the new LOD system
    let rat_scene = rat_assets.rat_lod0.clone();
    commands.spawn_batch((0..50).flat_map(|x| (0..50).map(move |y| (x, y))).map(
        move |(x, y)| {
            (
                SceneRoot(rat_scene.clone()),
                Transform::from_xyz(x as f32 / 10.0, y as f32 / 10.0, 0.0)
                    .with_scale(Vec3::splat(1.0)),
                AnimationGraphHandle(graph_handle.clone()),
                Rat,
                LODDistance::default(),
                LODState::new(LODLevel::new(0, 0.0, 8.0, 1.0 / 60.0)),
                AnimationLODData::default(),
            )
        },
    ));
}

// System to handle animation based on LOD data
fn handle_animation_lod(
    animations: Res<Animations>,
    mut commands: Commands,
    mut query: Query<(Entity, &AnimationLODData, &Children), (With<Rat>, Changed<AnimationLODData>)>,
    mut animation_players: Query<&mut AnimationPlayer>,
) {
    for (_entity, lod_data, children) in query.iter_mut() {
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
    query: Query<(&LODState, &LODDistance, &AnimationLODData), With<Rat>>,
    time: Res<Time>,
    mut last_print: Local<f32>,
) {
    if time.elapsed_secs() - *last_print > 2.0 {
        let mut level_counts = [0; 4];
        let mut animated_count = 0;
        let total = query.iter().len();
        
        for (lod_state, _distance, lod_data) in query.iter() {
            let level = lod_state.current_level.level as usize;
            if level < level_counts.len() {
                level_counts[level] += 1;
            }
            if lod_data.animation_enabled {
                animated_count += 1;
            }
        }
        
        println!("\n=== LOD Stats (New System) ===");
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
    #[asset(path = "blackrat_furless/rat_without_furlod2.glb#Animation1")]
    rat_lod0_animation: Handle<Scene>,
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
