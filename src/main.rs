use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use std::time::Duration;
use iyes_perf_ui::prelude::*;


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

#[derive(Component)]
struct LODTimer {
    timer: Timer,
}

impl Default for LODTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(1.0/60.0, TimerMode::Repeating),
        }
    }
}

#[derive(Component)]
struct DistanceFromCamera(f32);

impl Default for DistanceFromCamera {
    fn default() -> Self {
        Self(0.0)
    }
}
fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PanOrbitCameraPlugin))
        // .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
        // .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin)
        // .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
        // .add_plugins(bevy::render::diagnostic::RenderDiagnosticsPlugin)
        // .add_plugins(PerfUiPlugin)
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
                update_distance_from_camera.run_if(in_state(MyStates::Next)),
                update_animation_lod.run_if(in_state(MyStates::Next)),
                apply_lod_animation_updates.run_if(in_state(MyStates::Next)),
                print_lod_stats.run_if(in_state(MyStates::Next)),
            ).chain()
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
                DistanceFromCamera::default(),
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
        eprintln!("{entity}");
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
           .insert(animation_transitions);
    }
}

fn update_distance_from_camera(
    camera_query: Query<&Transform, (With<Camera>, Without<Rat>)>,
    mut rat_query: Query<(&Transform, &mut DistanceFromCamera), (With<Rat>, Without<Camera>)>,
) {
    if let Ok(camera_transform) = camera_query.single() {
        for (rat_transform, mut distance) in rat_query.iter_mut() {
            distance.0 = camera_transform.translation.distance(rat_transform.translation);
        }
    }
}

fn update_animation_lod(
    mut rat_query: Query<(&DistanceFromCamera, &mut AnimationLOD, &mut LODTimer)>,
) {
    #[cfg(feature = "trace_tracy")]
    let _span = bevy::utils::tracing::info_span!("update_animation_lod").entered();
    for (distance, mut lod, mut lod_timer) in rat_query.iter_mut() {
        let new_lod = match distance.0 {
            d if d < 10.0 => AnimationLOD::High,
            d if d < 25.0 => AnimationLOD::Medium, 
            d if d < 50.0 => AnimationLOD::Low,
            _ => AnimationLOD::Culled,
        };

        if *lod != new_lod {
            *lod = new_lod;
            
            let frequency = match *lod {
                AnimationLOD::High => 1.0 / 60.0,
                AnimationLOD::Medium => 1.0 / 20.0,
                AnimationLOD::Low => 1.0 / 5.0,
                AnimationLOD::Culled => 1.0 / 1.0,
            };
            print!("{:?}", new_lod);
            lod_timer.timer.set_duration(Duration::from_secs_f32(frequency));
            lod_timer.timer.reset();
        }
    }
}

fn apply_lod_animation_updates(
    time: Res<Time>,
    mut commands: Commands,
    mut rat_query: Query<(
        Entity,
        &AnimationLOD, 
        &mut LODTimer, 
        Option<&mut AnimationPlayer>,
        Option<&mut AnimationTransitions>,
        &mut Visibility,
    ), With<Rat>>,
) {
    #[cfg(feature = "trace_tracy")]
    let _span = bevy::utils::tracing::info_span!("apply_lod_animation_updates").entered();
    for (entity, lod, mut lod_timer, player, _transitions, mut visibility) in rat_query.iter_mut() {
        lod_timer.timer.tick(time.delta());
        
        match *lod {
            AnimationLOD::Culled => {
                // Make invisible and remove animation components for maximum performance
                *visibility = Visibility::Hidden;
                if player.is_some() {
                    commands.entity(entity).remove::<AnimationPlayer>();
                    commands.entity(entity).remove::<AnimationTransitions>();
                }
            }
            AnimationLOD::Low => {
                *visibility = Visibility::Visible;
                // Re-add animation components if they were removed
                if player.is_none() && lod_timer.timer.just_finished() {
                    // Only update animation every few frames
                    // Animation components will be re-added by play_animation_when_ready
                }
                if let Some(mut player) = player {
                    if lod_timer.timer.just_finished() {
                        // Allow animation update only on timer tick
                        if player.all_paused() {
                            player.resume_all();
                        }
                    } else {
                        // Pause between timer ticks for reduced update rate
                        player.pause_all();
                    }
                }
            }
            AnimationLOD::Medium => {
                *visibility = Visibility::Visible;
                if let Some(mut player) = player {
                    if lod_timer.timer.just_finished() {
                        if player.all_paused() {
                            player.resume_all();
                        }
                    } else if lod_timer.timer.elapsed_secs() > lod_timer.timer.duration().as_secs_f32() * 0.5 {
                        player.pause_all();
                    }
                }
            }
            AnimationLOD::High => {
                *visibility = Visibility::Visible;
                if let Some(mut player) = player {
                    if player.all_paused() {
                        player.resume_all();
                    }
                }
            }
        }
    }
}

fn print_lod_stats(
    rat_query: Query<(&AnimationLOD, &DistanceFromCamera), With<Rat>>,
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
