// mod lod;
// mod lod_system;

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

    // ...LOD system removed...

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

    // Spawn rats in a grid, all using the high quality scene
    let high_quality_scene = rat_assets.rat.clone();
    commands.spawn_batch((0..100).flat_map(|x| (0..100).map(move |y| (x, y))).map(
        move |(x, y)| {
            (
                SceneRoot(high_quality_scene.clone()),
                Transform::from_xyz((x as f32 - 25.0) / 4.0, (y as f32 - 25.0) / 4.0, 0.0)
                    .with_scale(Vec3::splat(1.0)),
                AnimationGraphHandle(graph_handle.clone()),
                Rat,
                ChildOf(spawn_base),
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


#[derive(AssetCollection, Resource)]
struct RatAssets {
    #[asset(path = "blackrat_free_glb/blackrat.glb#Scene0")]
    rat: Handle<Scene>,
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
