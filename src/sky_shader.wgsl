struct CameraUniform {
    view_proj: mat4x4<f32>,
    sky_color: vec4<f32>,
    light_factors: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(0) @binding(1)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(2)
var s_diffuse: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vs_sky(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.uv = model.uv;
    out.color = model.color;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    out.clip_position.z = out.clip_position.w;
    return out;
}

@vertex
fn vs_clouds(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.uv = model.uv;
    out.color = model.color;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    return out;
}


@fragment
fn fs_sky_color(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}

@fragment
fn fs_sun(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex = textureSample(t_diffuse, s_diffuse, in.uv);
    let luma = max(tex.r, max(tex.g, tex.b));
    if (luma < 0.02) {
        discard;
    }
    // Minecraft renders the sun as pure white. The glow halo uses a soft
    // alpha falloff from the luma of the texture. Inner bright pixels are
    // fully white; outer glow pixels fade out via the smoothstep.
    let alpha = in.color.a * smoothstep(0.02, 0.35, luma);
    return vec4<f32>(1.0, 1.0, 1.0, alpha);
}

@fragment
fn fs_moon(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex = textureSample(t_diffuse, s_diffuse, in.uv);
    let luma = max(tex.r, max(tex.g, tex.b));
    if (luma < 0.01) {
        discard;
    }
    let glow_boost = select(1.0, 2.5, luma < 0.5);
    let rgb = tex.rgb * in.color.rgb * (in.color.a * glow_boost);
    return vec4<f32>(rgb, 1.0);
}

