use bevy::prelude::*;
use bevy::scene::SceneInstanceReady;
use bevy_asset_loader::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use std::time::Duration;

#[derive(Resource)]
struct Animations {
    graph: Handle<AnimationGraph>,
    node_indices: Vec<AnimationNodeIndex>,
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PanOrbitCameraPlugin))
        .init_state::<MyStates>()
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .continue_to_state(MyStates::Next)
                .load_collection::<RatModels>()
                .load_collection::<WorldMaterial>(),
        )
        .add_systems(OnEnter(MyStates::Next), (show_model))
        .run();
}

fn show_model(
    mut commands: Commands,
    rat_models: Res<RatModels>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    env: Res<WorldMaterial>,
) {
    // Spawn the scene directly using the loaded handle

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
        graph: graph_handle,
        node_indices: vec![index],
    });

    let spawn_base = commands
        .spawn((Transform::default(), Visibility::default()))
        .id();
    for rat_x in 0..100 {
        for rat_y in 0..100 {
            commands.entity(spawn_base).with_children(|builder| {
                builder
                    .spawn((
                        SceneRoot(rat_models.rat.clone()),
                        Transform::from_xyz((rat_x as f32 / 10.0), (rat_y as f32 / 10.0), 0.0)
                            .with_scale(Vec3::splat(1.0)),
                    ))
                    .observe(play_animation_when_ready);
            });
        }
    }
}

fn play_animation_when_ready(
    trigger: Trigger<SceneInstanceReady>,
    animations: Res<Animations>,
    mut commands: Commands,
    children: Query<&Children>,
    mut player: Query<(Entity, &mut AnimationPlayer)>,
) {
    for child in children.iter_descendants(trigger.target()) {
        if let Ok((entity, mut player)) = player.get_mut(child) {
            let mut animation_transitions = AnimationTransitions::new();
            let playing_animation = animation_transitions
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
            break;
        }
    }
}

#[derive(AssetCollection, Resource)]
struct RatModels {
    #[asset(path = "blackrat_free_glb\\blackrat.glb#Scene0")]
    rat: Handle<Scene>,
    #[asset(path = "blackrat_free_glb\\blackrat.glb#Animation1")]
    animation_clip: Handle<AnimationClip>,
}
#[derive(AssetCollection, Resource)]
struct WorldMaterial {
    #[asset(path = "EnviromentMaps\\pisa_diffuse_rgb9e5_zstd.ktx2")]
    diffuse_map: Handle<Image>,
    #[asset(path = "EnviromentMaps\\pisa_specular_rgb9e5_zstd.ktx2")]
    specular_map: Handle<Image>,
}
#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyStates {
    #[default]
    AssetLoading,
    Next,
}
