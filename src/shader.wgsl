struct CameraUniform {
    view_proj: mat4x4<f32>,
    sky_color: vec4<f32>,       // rgb = sky & fog color, a = sun_brightness
    light_factors: vec4<f32>,   // x = sun_brightness, y = min_ambient, z = fog_start, w = fog_end
    eye_pos: vec4<f32>,         // xyz = camera eye pos, w = elapsed_time_seconds
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
    @location(3) normal: vec3<f32>, // x = sky_light (0-1), y = block_light (0-1), z = ao (0-1)
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) fog_factor: f32,
    @location(3) light_data: vec3<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.uv = model.uv;
    out.color = model.color;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    out.light_data = model.normal;
    
    // Distance-based fog
    let dist = out.clip_position.w;
    let fog_start = camera.light_factors.z;
    let fog_end = camera.light_factors.w;
    out.fog_factor = clamp((dist - fog_start) / (fog_end - fog_start), 0.0, 1.0);
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var uv = in.uv;
    
    // Portal animation (Slot 220 -> col 12, row 13)
    let col = floor(uv.x * 16.0);
    let row = floor(uv.y * 32.0);
    if (col == 12.0 && row == 13.0) {
        let time_ticks = camera.eye_pos.w * 20.0;
        let frame = u32(time_ticks) % 32u;
        
        let dest_col = f32((384u + frame) % 16u);
        let dest_row = f32((384u + frame) / 16u);
        
        let local_u = fract(uv.x * 16.0);
        let local_v = fract(uv.y * 32.0);
        
        uv.x = (dest_col + local_u) / 16.0;
        uv.y = (dest_row + local_v) / 32.0;
    }
    
    let tex_color = textureSample(t_diffuse, s_diffuse, uv);
    if (tex_color.a < 0.1) {
        discard;
    }
    
    // Dynamic light computation:
    // in.light_data.x: sky_light (0.0 to 1.0)
    // in.light_data.y: block_light (0.0 to 1.0)
    // in.light_data.z: ambient occlusion factor (0.4 to 1.0)
    let sun_b = camera.light_factors.x;
    let sky_l = in.light_data.x * sun_b;
    let block_l = in.light_data.y;
    let max_l = clamp(max(sky_l, block_l), 0.0, 1.0);
    
    // Minecraft 1.7.10 brightness curve
    let f2 = max_l / (4.0 - 3.0 * max_l);
    let min_ambient = camera.light_factors.y;
    let ao = in.light_data.z;
    let brightness = mix(min_ambient, 1.0, f2) * ao;
    
    let shaded_rgb = in.color.rgb * brightness;
    let base_color = tex_color * vec4<f32>(shaded_rgb, in.color.a);
    let sky_color = camera.sky_color.rgb;
    
    let final_color = mix(base_color.rgb, sky_color, in.fog_factor);
    
    return vec4<f32>(final_color, base_color.a);
}

struct CrosshairInput {
    @location(0) position: vec2<f32>,
};

struct CrosshairOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vs_crosshair(model: CrosshairInput) -> CrosshairOutput {
    var out: CrosshairOutput;
    out.clip_position = vec4<f32>(model.position, 0.0, 1.0);
    return out;
}

@fragment
fn fs_crosshair(in: CrosshairOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 0.9);
}
