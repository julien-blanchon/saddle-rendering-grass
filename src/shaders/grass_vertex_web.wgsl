#import bevy_pbr::forward_io::{VertexOutput}
#import bevy_pbr::mesh_bindings::mesh
#import bevy_render::globals::Globals
#import bevy_render::maths::{affine3_to_square, mat2x4_f32_to_mat3x3_unpack}
#import bevy_render::view::View

struct GrassMaterial {
    wind_direction: vec2<f32>,
    sway_strength: f32,
    sway_frequency: f32,
    sway_speed: f32,
    gust_strength: f32,
    gust_frequency: f32,
    gust_speed: f32,
    flutter_strength: f32,
    flutter_speed: f32,
    interaction_count: u32,
    interaction_map_active: u32,
    interaction_map_region: vec4<f32>,
    zone_centers_radius: array<vec4<f32>, 4>,
    zone_behavior: array<vec4<f32>, 4>,
};

@group(0) @binding(0)
var<uniform> view: View;
@group(0) @binding(11)
var<uniform> globals: Globals;

@group(#{MATERIAL_BIND_GROUP}) @binding(100)
var<uniform> grass_material: GrassMaterial;

@group(#{MATERIAL_BIND_GROUP}) @binding(101)
var interaction_map_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(102)
var interaction_map_sampler: sampler;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(5) color: vec4<f32>,
    @location(8) root_phase: vec4<f32>,
    @location(9) variation: vec4<f32>,
};

fn get_world_from_local(instance_index: u32) -> mat4x4<f32> {
    return affine3_to_square(mesh[instance_index].world_from_local);
}

fn mesh_normal_local_to_world(vertex_normal: vec3<f32>, instance_index: u32) -> vec3<f32> {
    if any(vertex_normal != vec3<f32>(0.0)) {
        return normalize(
            mat2x4_f32_to_mat3x3_unpack(
                mesh[instance_index].local_from_world_transpose_a,
                mesh[instance_index].local_from_world_transpose_b,
            ) * vertex_normal
        );
    }
    return vertex_normal;
}

fn position_world_to_clip(world_pos: vec3<f32>) -> vec4<f32> {
    return view.clip_from_world * vec4<f32>(world_pos, 1.0);
}

fn hash12(p: vec2<f32>) -> f32 {
    let h = dot(p, vec2<f32>(127.1, 311.7));
    return fract(sin(h) * 43758.5453123);
}

fn value_noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    let a = hash12(i);
    let b = hash12(i + vec2<f32>(1.0, 0.0));
    let c = hash12(i + vec2<f32>(0.0, 1.0));
    let d = hash12(i + vec2<f32>(1.0, 1.0));
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

fn sample_interaction_map(world_xz: vec2<f32>) -> vec4<f32> {
    if (grass_material.interaction_map_active == 0u) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    let center = grass_material.interaction_map_region.xy;
    let inv_extent = grass_material.interaction_map_region.zw;
    let uv = (world_xz - center) * inv_extent + vec2<f32>(0.5);
    let clamped_uv = clamp(uv, vec2<f32>(0.001), vec2<f32>(0.999));
    let texel = textureSampleLevel(interaction_map_texture, interaction_map_sampler, clamped_uv, 0.0);
    let bend_x = (texel.r - 0.5) * 2.0;
    let bend_z = (texel.g - 0.5) * 2.0;
    return vec4<f32>(bend_x, bend_z, texel.b, texel.a);
}

fn world_displacement(
    root_world: vec3<f32>,
    height_factor: f32,
    phase: f32,
    stiffness: f32,
    interaction_strength: f32,
    random_value: f32,
) -> vec3<f32> {
    let raw_dir = vec3<f32>(
        grass_material.wind_direction.x,
        0.0,
        grass_material.wind_direction.y,
    );
    let dir_len = length(raw_dir);
    let wind_dir = select(raw_dir / dir_len, vec3<f32>(1.0, 0.0, 0.0), dir_len < 0.0001);
    let side_dir = vec3<f32>(-wind_dir.z, 0.0, wind_dir.x);
    let macro_wave = sin(
        dot(root_world.xz, grass_material.wind_direction * grass_material.sway_frequency)
            + globals.time * grass_material.sway_speed
            + phase
    );
    let gust_noise = value_noise(
        root_world.xz * grass_material.gust_frequency
            + globals.time * grass_material.gust_speed
    ) * 2.0 - 1.0;
    let flutter =
        sin(globals.time * grass_material.flutter_speed + random_value * 18.0 + phase)
            * grass_material.flutter_strength;
    var displacement =
        wind_dir * (macro_wave * grass_material.sway_strength + gust_noise * grass_material.gust_strength);
    displacement += side_dir * flutter;

    let imap = sample_interaction_map(root_world.xz);
    let map_bend = vec3<f32>(imap.x, 0.0, imap.y);
    let map_flatten = imap.z;
    displacement += map_bend * interaction_strength * 0.8;
    displacement.y -= map_flatten * interaction_strength * 0.6;

    let count = min(grass_material.interaction_count, 4u);
    for (var i = 0u; i < count; i += 1u) {
        let zone = grass_material.zone_centers_radius[i];
        let behavior = grass_material.zone_behavior[i];
        let to_root = root_world - zone.xyz;
        let horizontal = vec2<f32>(to_root.x, to_root.z);
        let distance = length(horizontal);
        if (distance >= zone.w || zone.w <= 0.0) {
            continue;
        }
        let normalized = 1.0 - distance / zone.w;
        let influence = pow(normalized, behavior.z) * interaction_strength;
        let away = select(vec2<f32>(1.0, 0.0), normalize(horizontal), distance > 0.0001);
        displacement += vec3<f32>(away.x, 0.0, away.y) * behavior.x * influence;
        displacement.y -= behavior.y * influence;
    }

    return displacement * height_factor * stiffness;
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    let world_from_local = get_world_from_local(vertex.instance_index);

    let base_world = world_from_local * vec4<f32>(vertex.position, 1.0);
    let root_world = world_from_local * vec4<f32>(vertex.root_phase.xyz, 1.0);

    let imap = sample_interaction_map(root_world.xz);
    let hide_factor = 1.0 - imap.w;

    let displacement = world_displacement(
        root_world.xyz,
        vertex.uv.y,
        vertex.root_phase.w,
        vertex.variation.x,
        vertex.variation.y,
        vertex.variation.z,
    );

    let final_pos = mix(root_world.xyz, base_world.xyz + displacement, hide_factor);

    out.position = position_world_to_clip(final_pos);
    out.world_position = vec4<f32>(final_pos, 1.0);
    out.world_normal =
        normalize(mesh_normal_local_to_world(vertex.normal, vertex.instance_index) + displacement * 0.08);
    out.uv = vertex.uv;
    out.color = vec4<f32>(vertex.color.rgb, vertex.color.a * hide_factor);

#ifdef VISIBILITY_RANGE_DITHER
    out.visibility_range_dither = -16;
#endif

    return out;
}
