mod lod;

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use std::time::Duration;
use iyes_perf_ui::prelude::*;
use lod::*;


#[derive(Resource)]
struct Animations {
    graph: Handle<AnimationGraph>,
    node_indices: Vec<AnimationNodeIndex>,
}

#[derive(Component)]
struct RatsSpawned;

#[derive(Component)]
struct Rat;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
enum AnimationLOD {
    High,    // Full animation quality + full mesh
    Medium,  // Reduced animation rate + full mesh
    Low,     // Very reduced animation rate + simplified mesh
    Culled,  // No animation, invisible
}

impl Default for AnimationLOD {
    fn default() -> Self {
        AnimationLOD::High
    }
}

impl LODLevel for AnimationLOD {
    fn distance_thresholds() -> Vec<f32> {
        vec![8.0, 15.0, 25.0]
    }

    fn from_distance(distance: f32) -> Self {
        match distance {
            d if d < 8.0 => AnimationLOD::High,
            d if d < 15.0 => AnimationLOD::Medium,
            d if d < 25.0 => AnimationLOD::Low,
            _ => AnimationLOD::Culled,
        }
    }

    fn update_frequency(&self) -> f32 {
        match self {
            AnimationLOD::High => 1.0 / 60.0,
            AnimationLOD::Medium => 1.0 / 10.0,
            AnimationLOD::Low => 1.0 / 2.0,
            AnimationLOD::Culled => 1.0 / 0.5,
        }
    }

    fn is_visible(&self) -> bool {
        !matches!(self, AnimationLOD::Culled)
    }

    fn needs_animation(&self) -> bool {
        matches!(self, AnimationLOD::High | AnimationLOD::Medium)
    }
}

impl LODEntity for Rat {}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PanOrbitCameraPlugin))
        .add_plugins(LODPlugin::<AnimationLOD, Rat>::default())
        .init_state::<MyStates>()
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .continue_to_state(MyStates::Next)
                .load_collection::<RatModels>()
                .load_collection::<WorldMaterial>(),
        )
        .add_systems(OnEnter(MyStates::Next), show_model)
        .add_systems(
            Update,
            (
                play_animation_when_ready.run_if(in_state(MyStates::Next)),
                restart_animations.run_if(in_state(MyStates::Next)),
                print_lod_stats.run_if(in_state(MyStates::Next)),
            )
        )
        //.add_observer(play_animation_when_ready)
        .run();
}

fn show_model(
    mut commands: Commands,
    rat_models: Res<RatModels>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    env: Res<WorldMaterial>,
) {
    commands.spawn(PerfUiAllEntries::default());

    // Also spawn a camera to view the model
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

    // Add a light to illuminate the scene
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, -0.5, 0.0)),
    ));

    let (graph, index) = AnimationGraph::from_clip(rat_models.animation_clip.clone());
    let graph_handle = graphs.add(graph);

    // Create a component that stores a reference to our animation.
    commands.insert_resource(Animations {
        graph: graph_handle.clone(),
        node_indices: vec![index],
    });

    let spawn_base = commands
        .spawn((Transform::default(), Visibility::default(), Rat))
        .id();

    let rat_handle = rat_models.rat_lod0.clone();
    commands.spawn_batch((0..50).flat_map(|x| (0..50).map(move |y| (x, y))).map(
        move |(_x, _y)| {
            (
                SceneRoot(rat_handle.clone()),
                Transform::from_xyz(_x as f32 / 10.0, _y as f32 / 10.0, 0.0)
                    .with_scale(Vec3::splat(1.0)),
                AnimationGraphHandle(graph_handle.clone()),
                AnimationLOD::default(),
                LODTimer::default(),
                LODDistance::default(),
                LODTransitionTimer::default(),
                Rat,
                ChildOf(spawn_base),
            )
        },
    ));
}

fn play_animation_when_ready(
    animations: Res<Animations>,
    mut commands: Commands,
    rats: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    for (entity, mut player) in rats {
        let mut animation_transitions = AnimationTransitions::new();
        animation_transitions
            .play(
                &mut player,
                animations.node_indices[0],
                Duration::from_millis(15),
            )
            .repeat();

        commands
            .entity(entity)
           .insert(AnimationGraphHandle(animations.graph.clone()))
           .insert(animation_transitions)
           .insert(AnimationLOD::default());  // Add LOD component to animated entities
    }
}

fn restart_animations(
    animations: Res<Animations>,
    mut commands: Commands,
    mut restart_query: Query<(Entity, Option<&mut AnimationPlayer>), With<NeedsAnimationRestart>>,
    rat_query: Query<(Entity, &Children, &AnimationLOD), With<Rat>>,
) {
    for (entity, player) in restart_query.iter_mut() {
        // Find if this entity is a child of a rat and get its LOD
        let mut should_animate = false;
        for (_, children, lod) in rat_query.iter() {
            if children.contains(&entity) {
                should_animate = lod.needs_animation();
                break;
            }
        }
        
        if should_animate {
                if let Some(mut player) = player {
                    // Entity has AnimationPlayer but needs AnimationTransitions
                    let mut animation_transitions = AnimationTransitions::new();
                    animation_transitions
                        .play(
                            &mut player,
                            animations.node_indices[0],
                            Duration::from_millis(15),
                        )
                        .repeat();

                    commands
                        .entity(entity)
                        .insert(animation_transitions)
                        .remove::<NeedsAnimationRestart>();
                } else {
                    // Entity needs both AnimationPlayer and AnimationTransitions
                    let mut new_player = AnimationPlayer::default();
                    let mut animation_transitions = AnimationTransitions::new();

                    animation_transitions
                        .play(
                            &mut new_player,
                            animations.node_indices[0],
                            Duration::from_millis(15),
                        )
                        .repeat();

                    commands
                        .entity(entity)
                        .insert(AnimationGraphHandle(animations.graph.clone()))
                        .insert(new_player)
                        .insert(animation_transitions)
                        .remove::<NeedsAnimationRestart>();
                }
        } else {
            // Remove the marker for entities that shouldn't animate
            commands.entity(entity).remove::<NeedsAnimationRestart>();
        }
    }
}

fn print_lod_stats(
    rat_query: Query<(&AnimationLOD, &LODDistance), With<Rat>>,
    animated_query: Query<&AnimationPlayer>,  // Count ALL animated entities, not just rats
    time: Res<Time>,
    mut last_print: Local<f32>,
) {
    // Print stats every 2 seconds
    if time.elapsed_secs() - *last_print > 2.0 {
        let mut high = 0;
        let mut medium = 0; 
        let mut low = 0;
        let mut culled = 0;
        let total = rat_query.iter().len();
        let animated_count = animated_query.iter().len();
        let mut min_distance = f32::MAX;
        let mut max_distance = 0.0f32;
        
        for (lod, distance) in rat_query.iter() {
            min_distance = min_distance.min(distance.0);
            max_distance = max_distance.max(distance.0);
            
            match lod {
                AnimationLOD::High => high += 1,
                AnimationLOD::Medium => medium += 1,
                AnimationLOD::Low => low += 1,
                AnimationLOD::Culled => culled += 1,
            }
        }
        
        println!("LOD Stats - Total: {}, High: {}, Medium: {}, Low: {}, Culled: {}", total, high, medium, low, culled);
        println!("Animated entities: {} / {} ({:.1}% performance saving)", animated_count, total, (1.0 - animated_count as f32 / total as f32) * 100.0);
        println!("Distance range: {:.1} - {:.1}", min_distance, max_distance);
        *last_print = time.elapsed_secs();
    }
}

#[derive(AssetCollection, Resource)]
struct RatModels {
    #[asset(path = "blackrat_free_glb/blackrat.glb#Scene0")]
    rat: Handle<Scene>,
    #[asset(path = "blackrat_furless/rat_without_furlod2.glb#Scene0")]
    rat_lod0: Handle<Scene>,
    #[asset(path = "blackrat_furless/rat_without_furlod2.glb#Animation1")]
    rat_lod0_animation: Handle<Scene>,
    #[asset(path = "blackrat_free_glb/blackrat.glb#Animation1")]
    animation_clip: Handle<AnimationClip>,
}
#[derive(AssetCollection, Resource)]
struct WorldMaterial {
    #[asset(path = "EnviromentMaps/pisa_diffuse_rgb9e5_zstd.ktx2")]
    diffuse_map: Handle<Image>,
    #[asset(path = "EnviromentMaps/pisa_specular_rgb9e5_zstd.ktx2")]
    specular_map: Handle<Image>,
}
#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyStates {
    #[default]
    AssetLoading,
    Next,
}
