use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
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
fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PanOrbitCameraPlugin))
        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
        .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin)
        .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
        .add_plugins(bevy::render::diagnostic::RenderDiagnosticsPlugin)
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(PerfUiPlugin)
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
            (play_animation_when_ready.run_if(in_state(MyStates::Next)),
            //test_system.run_if(in_state(MyStates::Next)),)
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
                //AnimationTransitions::new(),
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
