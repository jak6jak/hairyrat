use bevy::{
    prelude::*
};
use bevy::scene::SceneInstanceReady;
use bevy_asset_loader::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};



#[derive(Component)]
struct AnimationToPlay {
    graph_handle: Handle<AnimationGraph>,
    index: AnimationNodeIndex,
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins,PanOrbitCameraPlugin))

        .init_state::<MyStates>()
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .continue_to_state(MyStates::Next)
                .load_collection::<RatModels>()
                .load_collection::<WorldMaterial>(),
        )
        .add_systems(OnEnter(MyStates::Next),show_model)
        .run();
}

fn show_model(mut commands: Commands, rat_models: Res<RatModels>, mut graphs: ResMut<Assets<AnimationGraph>>, env : Res<WorldMaterial>){
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
        }
    ));

    // Add a light to illuminate the scene
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, -0.5, 0.0)),
    ));

    let(graph,index) = AnimationGraph::from_clip(rat_models.animation_clip.clone());
    let graph_handle = graphs.add(graph);

    // Create a component that stores a reference to our animation.
    let animation_to_play = AnimationToPlay {
        graph_handle,
        index,
    };

    let mesh_scene = SceneRoot(rat_models.rat.clone());

    commands
        .spawn((animation_to_play, mesh_scene))
        .observe(play_animation_when_ready);
}

fn play_animation_when_ready(
    trigger: Trigger<SceneInstanceReady>,
    mut commands: Commands,
    children: Query<&Children>,
    animation_to_play: Query<&AnimationToPlay>,
    mut players: Query<&mut AnimationPlayer>,
){
    if let Ok(animation_to_play) = animation_to_play.get(trigger.target()){
        for child in children.iter_descendants(trigger.target()) {
            if let Ok(mut player) = players.get_mut(child) {
                player.play(animation_to_play.index).repeat();

                commands
                .entity(child)
                .insert(AnimationGraphHandle(animation_to_play.graph_handle.clone()));
            }
        }
    }
}

#[derive(AssetCollection,Resource)]
struct RatModels {
    #[asset(path = "blackrat_free_glb\\blackrat.glb#Scene0")]
    rat: Handle<Scene>,
    #[asset(path = "blackrat_free_glb\\blackrat.glb#Animation1")]
    animation_clip: Handle<AnimationClip>,

}
#[derive(AssetCollection,Resource)]
struct WorldMaterial {
    #[asset(path = "EnviromentMaps\\pisa_diffuse_rgb9e5_zstd.ktx2")]
    diffuse_map: Handle<Image>,
    #[asset(path = "EnviromentMaps\\pisa_specular_rgb9e5_zstd.ktx2")]
    specular_map: Handle<Image>,
}
#[derive(Clone,Eq,PartialEq, Debug,Hash,Default,States)]
enum MyStates {
    #[default]
    AssetLoading,
    Next,
}
