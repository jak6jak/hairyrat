use bevy::prelude::*;
use std::time::Duration;
use std::marker::PhantomData;

// Core LOD traits and components

/// Represents a strategy for handling LOD transitions
pub trait LODStrategy: Send + Sync + 'static {
    type Config: Resource + Default;
    type ComponentData: Component<Mutability = bevy::ecs::component::Mutable> + Default + Send + Sync;
    
    /// Called when transitioning between LOD levels
    fn transition(
        commands: &mut Commands,
        entity: Entity,
        from_level: u8,
        to_level: u8,
        config: &Self::Config,
        component_data: &mut Self::ComponentData,
    );
    
    /// Called to update the LOD representation each frame (if needed)
    fn update(
        time: &Time,
        entity: Entity,
        current_level: u8,
        component_data: &mut Self::ComponentData,
    );
    
    /// Returns true if this strategy requires per-frame updates
    fn requires_update() -> bool {
        false
    }
}

/// Core LOD level definition
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct LODLevel {
    pub level: u8,
    pub min_distance: f32,
    pub max_distance: f32,
    pub update_frequency: f32,
}

impl LODLevel {
    pub fn new(level: u8, min_distance: f32, max_distance: f32, update_frequency: f32) -> Self {
        Self {
            level,
            min_distance,
            max_distance,
            update_frequency,
        }
    }
    
    pub fn from_distance(distance: f32, levels: &[LODLevel]) -> Option<LODLevel> {
        levels.iter()
            .find(|l| distance >= l.min_distance && distance < l.max_distance)
            .copied()
    }
}

/// Component to track current LOD state
#[derive(Component)]
pub struct LODState {
    pub current_level: LODLevel,
    pub target_level: Option<LODLevel>,
    pub transition_timer: Timer,
    pub update_timer: Timer,
}

impl LODState {
    pub fn new(initial_level: LODLevel) -> Self {
        Self {
            current_level: initial_level,
            target_level: None,
            transition_timer: Timer::from_seconds(0.1, TimerMode::Once),
            update_timer: Timer::from_seconds(initial_level.update_frequency, TimerMode::Repeating),
        }
    }
}

/// Component to store distance from camera
#[derive(Component, Default)]
pub struct LODDistance(pub f32);

/// Resource to define LOD levels for a specific entity type
#[derive(Resource)]
pub struct LODLevels<T> {
    pub levels: Vec<LODLevel>,
    _phantom: PhantomData<T>,
}

impl<T> LODLevels<T> {
    pub fn new(levels: Vec<LODLevel>) -> Self {
        Self {
            levels,
            _phantom: PhantomData,
        }
    }
}

/// Resource for LOD processing budget
#[derive(Resource)]
pub struct LODProcessingBudget {
    pub max_operations_per_frame: usize,
    pub current_operations: usize,
}

impl Default for LODProcessingBudget {
    fn default() -> Self {
        Self {
            max_operations_per_frame: 50,
            current_operations: 0,
        }
    }
}

/// Generic LOD plugin that can work with any strategy
pub struct LODPlugin<T: Component, S: LODStrategy> {
    _phantom: PhantomData<(T, S)>,
}

impl<T: Component, S: LODStrategy> Default for LODPlugin<T, S> {
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T: Component, S: LODStrategy> Plugin for LODPlugin<T, S> {
    fn build(&self, app: &mut App) {
        app.init_resource::<LODProcessingBudget>()
            .init_resource::<S::Config>()
            .add_systems(Update, (
                reset_lod_budget,
                update_distance_from_camera::<T>,
                calculate_target_lod::<T>,
                apply_lod_transitions::<T, S>,
            ).chain());
            
        if S::requires_update() {
            app.add_systems(Update, update_lod_representations::<T, S>);
        }
    }
}

// System implementations

fn reset_lod_budget(mut budget: ResMut<LODProcessingBudget>) {
    budget.current_operations = 0;
}

fn update_distance_from_camera<T: Component>(
    camera_query: Query<&Transform, (With<Camera>, Without<T>)>,
    mut entity_query: Query<(&Transform, &mut LODDistance), (With<T>, Without<Camera>)>,
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

fn calculate_target_lod<T: Component>(
    lod_levels: Res<LODLevels<T>>,
    mut entity_query: Query<(&LODDistance, &mut LODState), With<T>>,
) {
    for (distance, mut lod_state) in entity_query.iter_mut() {
        if let Some(target_level) = LODLevel::from_distance(distance.0, &lod_levels.levels) {
            if target_level != lod_state.current_level {
                lod_state.target_level = Some(target_level);
            }
        }
    }
}

fn apply_lod_transitions<T: Component, S: LODStrategy>(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<S::Config>,
    mut budget: ResMut<LODProcessingBudget>,
    mut entity_query: Query<(Entity, &mut LODState, &mut S::ComponentData), With<T>>,
) {
    for (entity, mut lod_state, mut component_data) in entity_query.iter_mut() {
        if budget.current_operations >= budget.max_operations_per_frame {
            break;
        }
        
        // Handle transitions
        if let Some(target_level) = lod_state.target_level {
            lod_state.transition_timer.tick(time.delta());
            
            if lod_state.transition_timer.finished() {
                let from_level = lod_state.current_level.level;
                let to_level = target_level.level;
                
                S::transition(
                    &mut commands,
                    entity,
                    from_level,
                    to_level,
                    &config,
                    &mut component_data,
                );
                
                lod_state.current_level = target_level;
                lod_state.target_level = None;
                lod_state.transition_timer.reset();
                lod_state.update_timer.set_duration(Duration::from_secs_f32(target_level.update_frequency));
                
                budget.current_operations += 1;
            }
        } else {
            lod_state.transition_timer.reset();
        }
    }
}

fn update_lod_representations<T: Component, S: LODStrategy>(
    time: Res<Time>,
    mut entity_query: Query<(Entity, &LODState, &mut S::ComponentData), With<T>>,
) {
    for (entity, lod_state, mut component_data) in entity_query.iter_mut() {
        S::update(&time, entity, lod_state.current_level.level, &mut component_data);
    }
}
