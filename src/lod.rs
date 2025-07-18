use bevy::prelude::*;
use bevy_inspector_egui::egui::debug_text::print;
use std::time::Duration;
use std::marker::PhantomData;

// Generic LOD System
pub trait LODLevel: Component<Mutability = bevy::ecs::component::Mutable> + Clone + Copy + PartialEq + std::fmt::Debug + Default {
    fn distance_thresholds() -> Vec<f32>;
    fn from_distance(distance: f32) -> Self;
    fn update_frequency(&self) -> f32;
    fn is_visible(&self) -> bool;
    fn needs_animation(&self) -> bool;
}

pub trait LODEntity: Component {}

#[derive(Component)]
pub struct LODDistance(pub f32);

#[derive(Component)]
pub struct LODTimer {
    pub timer: Timer,
}

#[derive(Component)]
pub struct LODTransitionTimer {
    pub timer: Timer,
}

#[derive(Component)]
pub struct NeedsAnimationRestart;

#[derive(Resource)]
pub struct LODConfig {
    pub transition_delay: Duration,
    pub max_operations_per_frame: usize,
}

impl Default for LODConfig {
    fn default() -> Self {
        Self {
            transition_delay: Duration::from_millis(10),
            max_operations_per_frame: 1150,
        }
    }
}

#[derive(Resource)]
pub struct LODProcessingBudget {
    pub current_operations: usize,
}

impl Default for LODProcessingBudget {
    fn default() -> Self {
        Self {
            current_operations: 0,
        }
    }
}

pub struct LODPlugin<T: LODLevel, E: LODEntity> {
    _phantom: PhantomData<(T, E)>,
}

impl<T: LODLevel, E: LODEntity> Default for LODPlugin<T, E> {
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T: LODLevel, E: LODEntity> Plugin for LODPlugin<T, E> {
    fn build(&self, app: &mut App) {
        app.init_resource::<LODConfig>()
            .init_resource::<LODProcessingBudget>()
            .add_systems(Update, (
                reset_lod_budget::<T, E>,
                update_distance_from_camera::<T, E>,
                update_lod::<T, E>,
                //apply_lod_updates::<T, E>,
            ).chain());
    }
}

// Generic LOD Systems
pub fn reset_lod_budget<T: LODLevel, E: LODEntity>(
    _config: Res<LODConfig>,
    mut budget: ResMut<LODProcessingBudget>,
) {
    budget.current_operations = 0;
}

pub fn update_distance_from_camera<T: LODLevel, E: LODEntity>(
    camera_query: Query<&Transform, (With<Camera>, Without<E>)>,
    mut entity_query: Query<(&Transform, &mut LODDistance), (With<E>, Without<Camera>)>,
) {
    if let Ok(camera_transform) = camera_query.single() {
        for (entity_transform, mut distance) in entity_query.iter_mut() {
            let to_entity = entity_transform.translation - camera_transform.translation;
            let camera_forward = camera_transform.forward();
            let view_distance = to_entity.dot(camera_forward.as_vec3());
            distance.0 = view_distance.abs();
        }
    }
}

pub fn update_lod<T: LODLevel, E: LODEntity>(
    time: Res<Time>,
    mut entity_query: Query<(&LODDistance, &mut T, &mut LODTimer, &mut LODTransitionTimer), With<E>>,
) {
    for (distance, mut lod, mut lod_timer, mut transition_timer) in entity_query.iter_mut() {
        let target_lod = T::from_distance(distance.0);

        if *lod != target_lod {
            transition_timer.timer.tick(time.delta());

            if transition_timer.timer.finished() {
                *lod = target_lod;
                transition_timer.timer.reset();

                let frequency = lod.update_frequency();
                lod_timer.timer.set_duration(Duration::from_secs_f32(frequency));
                lod_timer.timer.reset();
            }
        } else {
            transition_timer.timer.reset();
        }
    }
}

pub fn apply_lod_updates<T: LODLevel, E: LODEntity>(
    time: Res<Time>,
    mut commands: Commands,
    config: Res<LODConfig>,
    mut budget: ResMut<LODProcessingBudget>,
    mut entity_query: Query<(Entity, &T, &mut LODTimer, &mut Visibility, &Children), With<E>>,
    animation_query: Query<Entity, With<AnimationPlayer>>,
) {
    for (_entity, lod, mut lod_timer, mut visibility, children) in entity_query.iter_mut() {
        // if budget.current_operations >= config.max_operations_per_frame {
        //     break;
        // }

        lod_timer.timer.tick(time.delta());

        // Handle visibility
        *visibility = if lod.is_visible() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        // Handle animation components on child entities
        for child in children.iter() {
            if let Ok(_) = animation_query.get(child) {
                println!("Child {:?} has AnimationPlayer, LOD: {:?}, needs_animation: {}", 
                         child, lod, lod.needs_animation());
                
                if !lod.needs_animation() {
                    // Remove animation components for LOD levels that don't need them
                    println!("Removing animation components from entity {:?}", child);
                    commands.entity(child).remove::<AnimationPlayer>();
                    commands.entity(child).remove::<AnimationTransitions>();
                    budget.current_operations += 1;
                }
            } else if lod.needs_animation() {
                // Child doesn't have animation but should - mark for restart
                println!("Child {:?} needs animation restart, LOD: {:?}", child, lod);
                commands.entity(child).insert(NeedsAnimationRestart);
            }
        }

        budget.current_operations += 1;
    }
}

impl Default for LODTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(1.0/60.0, TimerMode::Repeating),
        }
    }
}

impl Default for LODTransitionTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.1, TimerMode::Once),
        }
    }
}

impl Default for LODDistance {
    fn default() -> Self {
        Self(0.0)
    }
}
