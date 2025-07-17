// Vertex Animation Texture (VAT) Shader
// This shader reads vertex positions and normals from textures to animate meshes

struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

struct VATUniforms {
    // Animation parameters
    current_frame: f32,
    total_frames: f32,
    vertex_count: u32,
    texture_width: u32,
    // Bounding box for position reconstruction
    bbox_min: vec3<f32>,
    bbox_max: vec3<f32>,
}

@group(0) @binding(0) var<uniform> view: View;
@group(1) @binding(0) var<uniform> mesh: Mesh;
@group(2) @binding(0) var<uniform> vat: VATUniforms;
@group(2) @binding(1) var position_texture: texture_2d<f32>;
@group(2) @binding(2) var normal_texture: texture_2d<f32>;
@group(2) @binding(3) var position_sampler: sampler;
@group(2) @binding(4) var normal_sampler: sampler;

// Standard material bindings
@group(3) @binding(0) var base_color_texture: texture_2d<f32>;
@group(3) @binding(1) var base_color_sampler: sampler;

fn get_vat_uv(vertex_id: u32, frame: f32) -> vec2<f32> {
    let vertices_per_row = vat.texture_width;
    let vertex_x = vertex_id % vertices_per_row;
    let vertex_y = vertex_id / vertices_per_row;
    
    // Calculate UV coordinates for the current frame
    let frame_offset = frame / vat.total_frames;
    let u = (f32(vertex_x) + 0.5) / f32(vertices_per_row);
    let v = (f32(vertex_y) + frame_offset) / f32(vat.total_frames);
    
    return vec2<f32>(u, v);
}

fn decode_position(encoded: vec3<f32>) -> vec3<f32> {
    // Decode position from normalized texture values
    let bbox_size = vat.bbox_max - vat.bbox_min;
    return vat.bbox_min + encoded * bbox_size;
}

fn decode_normal(encoded: vec3<f32>) -> vec3<f32> {
    // Decode normal from texture (stored as 0-1, needs to be -1 to 1)
    return normalize(encoded * 2.0 - 1.0);
}

@vertex
fn vertex(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    
    // Calculate texture coordinates for VAT lookup
    let vat_uv = get_vat_uv(input.vertex_index, vat.current_frame);
    
    // Sample position and normal from VAT textures
    let encoded_position = textureSampleLevel(position_texture, position_sampler, vat_uv, 0.0).xyz;
    let encoded_normal = textureSampleLevel(normal_texture, normal_sampler, vat_uv, 0.0).xyz;
    
    // Decode the values
    let animated_position = decode_position(encoded_position);
    let animated_normal = decode_normal(encoded_normal);
    
    // Transform to world space
    let world_position = (mesh.model * vec4<f32>(animated_position, 1.0)).xyz;
    let world_normal = normalize((mesh.model * vec4<f32>(animated_normal, 0.0)).xyz);
    
    // Calculate clip position
    output.clip_position = view.view_proj * vec4<f32>(world_position, 1.0);
    output.world_position = world_position;
    output.world_normal = world_normal;
    output.uv = input.uv;
    
    return output;
}

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    // Simple lighting calculation
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let n_dot_l = max(dot(input.world_normal, light_dir), 0.0);
    let ambient = 0.2;
    let diffuse = n_dot_l * 0.8;
    
    // Sample base color texture
    let base_color = textureSample(base_color_texture, base_color_sampler, input.uv);
    
    // Apply lighting
    let final_color = base_color.rgb * (ambient + diffuse);
    
    return vec4<f32>(final_color, base_color.a);
}

// Simplified VAT shader for lower LOD levels
@vertex
fn vertex_simple(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    
    // For lowest LOD, just use the first frame (static pose)
    let vat_uv = get_vat_uv(input.vertex_index, 0.0);
    
    let encoded_position = textureSampleLevel(position_texture, position_sampler, vat_uv, 0.0).xyz;
    let static_position = decode_position(encoded_position);
    
    let world_position = (mesh.model * vec4<f32>(static_position, 1.0)).xyz;
    output.clip_position = view.view_proj * vec4<f32>(world_position, 1.0);
    output.world_position = world_position;
    output.world_normal = input.normal; // Use original normal for performance
    output.uv = input.uv;
    
    return output;
}
