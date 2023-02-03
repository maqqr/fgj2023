#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings

// NOTE: Bindings must come before functions that use them!
#import bevy_pbr::mesh_functions

@group(1) @binding(0)
var<uniform> time: f32;
@group(1) @binding(1)
var<uniform> bending: f32;
@group(1) @binding(2)
var<uniform> cam_position: vec3<f32>;
@group(1) @binding(3)
var<uniform> color: vec3<f32>;

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
#ifdef VERTEX_UVS
    @location(2) uv: vec2<f32>,
#endif
#ifdef VERTEX_TANGENTS
    @location(3) tangent: vec4<f32>,
#endif
#ifdef VERTEX_COLORS
    @location(4) color: vec4<f32>,
#endif
#ifdef SKINNED
    @location(5) joint_indices: vec4<u32>,
    @location(6) joint_weights: vec4<f32>,
#endif
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    #import bevy_pbr::mesh_vertex_output
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
#ifdef SKINNED
    var model = skin_model(vertex.joint_indices, vertex.joint_weights);
    out.world_normal = skin_normals(model, vertex.normal);
#else
    var model = mesh.model;
    out.world_normal = mesh_normal_local_to_world(vertex.normal);
#endif
    out.world_position = mesh_position_local_to_world(model, vec4<f32>(vertex.position, 1.0));
#ifdef VERTEX_UVS
    out.uv = vertex.uv;
#endif
#ifdef VERTEX_TANGENTS
    out.world_tangent = mesh_tangent_local_to_world(model, vertex.tangent);
#endif
#ifdef VERTEX_COLORS
    out.color = vertex.color;
#endif

    out.world_position = mesh_position_local_to_world(model, vec4<f32>(vertex.position.x, vertex.position.y, vertex.position.z, 1.0));
    var dist_from_camera = (out.world_position.xyz - cam_position).z;
    out.world_position += vec4<f32>(0.0, pow(dist_from_camera, 2.0) * -bending, 0.0, 0.0);
    out.clip_position = mesh_position_world_to_clip(out.world_position);

    return out;
}

struct FragmentInput {
    #import bevy_pbr::mesh_vertex_output
};

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
#ifdef VERTEX_COLORS
    return in.color;
#else

    var ambient = vec3<f32>(0.1, 0.1, 0.1);
    var lightDir = vec3<f32>(0.5, -0.7, 0.2);

    var N = normalize(in.world_normal);
    var V = normalize(view.world_position.xyz - in.world_position.xyz);
    var diff = max(dot(N, V), 0.0001) * color;

    var result = ambient + diff;

    return vec4(result, 1.0);

#endif
}