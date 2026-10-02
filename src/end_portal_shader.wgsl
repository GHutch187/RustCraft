struct CameraUniform {
    view_proj: mat4x4<f32>,
    sky_color: vec4<f32>,
    light_factors: vec4<f32>,
    eye_pos: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(0) @binding(1)
var t_sky: texture_2d<f32>;

@group(0) @binding(2)
var t_portal: texture_2d<f32>;

@group(0) @binding(3)
var s_repeat: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(in.position, 1.0);
    out.world_pos = in.position;
    return out;
}

fn sample_portal_layer(
    pos_xz: vec2<f32>,
    delta: vec2<f32>,
    dy: f32,
    scroll: f32,
    depth: f32,
    scale: f32,
    rot_rad: f32,
    tint: vec3<f32>
) -> vec3<f32> {
    let p = pos_xz + delta * (depth / dy);
    let cos_a = cos(rot_rad);
    let sin_a = sin(rot_rad);
    let p_centered = p - vec2<f32>(0.5, 0.5);
    let p_rot = vec2<f32>(
        p_centered.x * cos_a - p_centered.y * sin_a,
        p_centered.x * sin_a + p_centered.y * cos_a
    );
    let uv = p_rot * scale + vec2<f32>(0.5, 0.5) + vec2<f32>(0.0, scroll);
    let samp = textureSample(t_portal, s_repeat, uv);
    return samp.rgb * tint;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let eye = camera.eye_pos.xyz;
    let time = camera.eye_pos.w;

    let dy = abs(eye.y - in.world_pos.y) + 0.001;
    let delta = in.world_pos.xz - eye.xz;

    let scroll = fract(time / 700.0);

    // Layer 0: end_sky.png
    let p0 = in.world_pos.xz + delta * (65.0 / dy);
    let uv0 = (p0 - vec2<f32>(0.5, 0.5)) * 0.125 + vec2<f32>(0.5, 0.5);
    let sky_sample = textureSample(t_sky, s_repeat, uv0);
    var color = sky_sample.rgb * 0.1;

    // Layers 1 to 15 matching Minecraft 1.7.10 PRNG (seed 31100L) and additive blending
    color += sample_portal_layer(in.world_pos.xz, delta, dy, scroll, 15.0, 0.5000, 8660.0 * 0.0174532925, vec3<f32>(0.0063, 0.0510, 0.0475));
    color += sample_portal_layer(in.world_pos.xz, delta, dy, scroll, 14.0, 0.0625, 34604.0 * 0.0174532925, vec3<f32>(0.0147, 0.0542, 0.0535));
    color += sample_portal_layer(in.world_pos.xz, delta, dy, scroll, 13.0, 0.0625, 77832.0 * 0.0174532925, vec3<f32>(0.0249, 0.0589, 0.0615));
    color += sample_portal_layer(in.world_pos.xz, delta, dy, scroll, 12.0, 0.0625, 138344.0 * 0.0174532925, vec3<f32>(0.0349, 0.0634, 0.0523));
    color += sample_portal_layer(in.world_pos.xz, delta, dy, scroll, 11.0, 0.0625, 216140.0 * 0.0174532925, vec3<f32>(0.0345, 0.0471, 0.0670));
    color += sample_portal_layer(in.world_pos.xz, delta, dy, scroll, 10.0, 0.0625, 311220.0 * 0.0174532925, vec3<f32>(0.0463, 0.0611, 0.0908));
    color += sample_portal_layer(in.world_pos.xz, delta, dy, scroll, 9.0, 0.0625, 423584.0 * 0.0174532925, vec3<f32>(0.0536, 0.0848, 0.0501));
    color += sample_portal_layer(in.world_pos.xz, delta, dy, scroll, 8.0, 0.0625, 553232.0 * 0.0174532925, vec3<f32>(0.0590, 0.0729, 0.1084));
    color += sample_portal_layer(in.world_pos.xz, delta, dy, scroll, 7.0, 0.0625, 700164.0 * 0.0174532925, vec3<f32>(0.0550, 0.0620, 0.1053));
    color += sample_portal_layer(in.world_pos.xz, delta, dy, scroll, 6.0, 0.0625, 864380.0 * 0.0174532925, vec3<f32>(0.0763, 0.0790, 0.0849));
    color += sample_portal_layer(in.world_pos.xz, delta, dy, scroll, 5.0, 0.0625, 1045880.0 * 0.0174532925, vec3<f32>(0.0408, 0.1419, 0.1375));
    color += sample_portal_layer(in.world_pos.xz, delta, dy, scroll, 4.0, 0.0625, 1244664.0 * 0.0174532925, vec3<f32>(0.1181, 0.0857, 0.1288));
    color += sample_portal_layer(in.world_pos.xz, delta, dy, scroll, 3.0, 0.0625, 1460732.0 * 0.0174532925, vec3<f32>(0.0296, 0.1971, 0.2012));
    color += sample_portal_layer(in.world_pos.xz, delta, dy, scroll, 2.0, 0.0625, 1694084.0 * 0.0174532925, vec3<f32>(0.1364, 0.2600, 0.2014));
    color += sample_portal_layer(in.world_pos.xz, delta, dy, scroll, 1.0, 0.0625, 1944720.0 * 0.0174532925, vec3<f32>(0.0607, 0.2361, 0.4961));

    return vec4<f32>(color, 1.0);
}
