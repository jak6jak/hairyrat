use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    reflect::TypePath,
    asset::Asset,
};

/// Material for Vertex Animation Textures
#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct VATMaterial {
    // Animation parameters
    #[uniform(0)]
    pub current_frame: f32,
    #[uniform(0)]
    pub total_frames: f32,
    #[uniform(0)]
    pub vertex_count: u32,
    #[uniform(0)]
    pub texture_width: u32,
    
    // Bounding box for position reconstruction
    #[uniform(0)]
    pub bbox_min: Vec3,
    #[uniform(0)]
    pub bbox_max: Vec3,
    
    // VAT textures
    #[texture(1)]
    #[sampler(3)]
    pub position_texture: Handle<Image>,
    
    #[texture(2)]
    #[sampler(4)]
    pub normal_texture: Handle<Image>,
    
    // Base color texture (optional)
    #[texture(5)]
    #[sampler(6)]
    pub base_color_texture: Option<Handle<Image>>,
    
    // Control parameters
    pub alpha_mode: AlphaMode,
}

impl Material for VATMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/vat_shader.wgsl".into()
    }

    fn vertex_shader() -> ShaderRef {
        "shaders/vat_shader.wgsl".into()
    }
    
    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
}

/// Component to track VAT animation state
#[derive(Component)]
pub struct VATAnimationState {
    pub current_frame: f32,
    pub playback_speed: f32,
    pub loop_animation: bool,
    pub is_playing: bool,
}

impl Default for VATAnimationState {
    fn default() -> Self {
        Self {
            current_frame: 0.0,
            playback_speed: 1.0,
            loop_animation: true,
            is_playing: true,
        }
    }
}

/// Bundle for entities using VAT
#[derive(Bundle)]
pub struct VATBundle {
    pub mesh_handle: MeshMaterial3d<VATMaterial>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
    pub animation_state: VATAnimationState,
}

impl Default for VATBundle {
    fn default() -> Self {
        Self {
            mesh_handle: Default::default(),
            transform: Default::default(),
            global_transform: Default::default(),
            visibility: Default::default(),
            inherited_visibility: Default::default(),
            view_visibility: Default::default(),
            animation_state: Default::default(),
        }
    }
}

/// System to update VAT materials with current animation frame
pub fn update_vat_materials(
    _time: Res<Time>,
    mut materials: ResMut<Assets<VATMaterial>>,
    query: Query<(&MeshMaterial3d<VATMaterial>, &VATAnimationState), Changed<VATAnimationState>>,
) {
    for (mesh_material, animation_state) in query.iter() {
        if let Some(material) = materials.get_mut(&mesh_material.0) {
            material.current_frame = animation_state.current_frame;
        }
    }
}

/// System to advance VAT animations
pub fn advance_vat_animations(
    time: Res<Time>,
    mut query: Query<(&mut VATAnimationState, &MeshMaterial3d<VATMaterial>)>,
    materials: Res<Assets<VATMaterial>>,
) {
    for (mut animation_state, mesh_material) in query.iter_mut() {
        if !animation_state.is_playing {
            continue;
        }
        
        if let Some(material) = materials.get(&mesh_material.0) {
            let delta = time.delta_secs() * animation_state.playback_speed * 30.0; // Assuming 30 fps base
            animation_state.current_frame += delta;
            
            if animation_state.loop_animation {
                animation_state.current_frame %= material.total_frames;
            } else {
                animation_state.current_frame = animation_state.current_frame.min(material.total_frames - 1.0);
            }
        }
    }
}

/// Plugin to add VAT material support
pub struct VATMaterialPlugin;

impl Plugin for VATMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<VATMaterial>::default())
            .add_systems(Update, (
                advance_vat_animations,
                update_vat_materials,
            ));
    }
}

/// Helper to create VAT material from textures
pub fn create_vat_material(
    position_texture: Handle<Image>,
    normal_texture: Handle<Image>,
    base_color_texture: Option<Handle<Image>>,
    total_frames: u32,
    vertex_count: u32,
    texture_width: u32,
    bbox_min: Vec3,
    bbox_max: Vec3,
) -> VATMaterial {
    VATMaterial {
        current_frame: 0.0,
        total_frames: total_frames as f32,
        vertex_count,
        texture_width,
        bbox_min,
        bbox_max,
        position_texture,
        normal_texture,
        base_color_texture,
        alpha_mode: AlphaMode::Opaque,
    }
}

/// Configuration for VAT texture generation
#[derive(Clone)]
pub struct VATConfig {
    pub texture_width: u32,
    pub frames_per_second: f32,
    pub total_frames: u32,
    pub include_normals: bool,
    pub compression_quality: f32, // 0.0 to 1.0
}

impl Default for VATConfig {
    fn default() -> Self {
        Self {
            texture_width: 2048,
            frames_per_second: 30.0,
            total_frames: 60,
            include_normals: true,
            compression_quality: 0.9,
        }
    }
}

/// Helper to calculate optimal texture dimensions for VAT
pub fn calculate_vat_texture_dimensions(vertex_count: usize, frame_count: usize) -> (u32, u32) {
    // Try to make texture as square as possible
    let total_pixels = vertex_count * frame_count;
    let width = (total_pixels as f32).sqrt().ceil() as u32;
    let height = ((total_pixels as f32) / width as f32).ceil() as u32;
    
    // Ensure power of 2 for better GPU compatibility
    let width = width.next_power_of_two();
    let height = height.next_power_of_two();
    
    (width, height)
}

/// Component to mark entities that should use simplified VAT (static frame)
#[derive(Component)]
pub struct SimplifiedVAT {
    pub static_frame: u32,
}

impl Default for SimplifiedVAT {
    fn default() -> Self {
        Self { static_frame: 0 }
    }
}
