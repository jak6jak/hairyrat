use bevy::{
    prelude::*
};
use bevy_asset_loader::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins,PanOrbitCameraPlugin))

        .init_state::<MyStates>()
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .continue_to_state(MyStates::Next)
                .load_collection::<RatModels>(),
        )
        .add_systems(OnEnter(MyStates::Next),show_model)
        .run();
}

fn show_model(mut commands: Commands, rat_models: Res<RatModels>){
    // Spawn the scene directly using the loaded handle
    commands.spawn(SceneRoot(rat_models.rat.clone()));

    // Also spawn a camera to view the model
    commands.spawn((
        Transform::from_translation(Vec3::new(0.0, 1.5, 5.0)),
        PanOrbitCamera::default(),
    ));

    // Add a light to illuminate the scene
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, -0.5, 0.0)),
    ));

}

#[derive(AssetCollection,Resource)]
struct RatModels {
    #[asset(path = "blackrat_free_glb\\blackrat.glb#Scene0")]
    rat: Handle<Scene>,
}
#[derive(Clone,Eq,PartialEq, Debug,Hash,Default,States)]
enum MyStates {
    #[default]
    AssetLoading,
    Next,
}
