use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
    pub sky_color: [f32; 4],      // rgb = sky & fog color, a = sun_brightness
    pub light_factors: [f32; 4],  // x = sun_brightness, y = min_ambient, z = fog_start, w = fog_end
    pub eye_pos: [f32; 4],        // xyz = camera eye pos, w = elapsed_time_seconds
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
            sky_color: [0.53, 0.81, 0.92, 1.0],
            light_factors: [1.0, 0.15, 100.0, 160.0],
            eye_pos: [0.0, 0.0, 0.0, 0.0],
        }
    }

    pub fn update_view_proj(
        &mut self,
        eye: Vec3,
        yaw_deg: f32,
        pitch_deg: f32,
        aspect: f32,
        time_of_day: i64,
        elapsed_seconds: f32,
    ) {
        let yaw_rad = yaw_deg.to_radians();
        let pitch_rad = pitch_deg.to_radians();

        let front = Vec3::new(
            -yaw_rad.sin() * pitch_rad.cos(),
            -pitch_rad.sin(),
            yaw_rad.cos() * pitch_rad.cos(),
        )
        .normalize();

        let target = eye + front;
        let up = Vec3::Y;

        let view = Mat4::look_at_rh(eye, target, up);
        let proj = Mat4::perspective_rh(70.0f32.to_radians(), aspect, 0.1, 1000.0);

        self.view_proj = (proj * view).to_cols_array_2d();
        self.eye_pos = [eye.x, eye.y, eye.z, elapsed_seconds];

        let (sky_col, light_facs) = calculate_sky_and_light(time_of_day);
        self.sky_color = sky_col;
        self.light_factors = light_facs;
    }
}

pub fn get_celestial_angle(time_of_day: i64) -> f32 {
    let t = (time_of_day.rem_euclid(24000)) as f32;

    let mut frac = t / 24000.0 - 0.25;
    if frac < 0.0 {
        frac += 1.0;
    }
    if frac > 1.0 {
        frac -= 1.0;
    }
    let f1 = frac;
    let curved = 1.0 - ((frac * std::f32::consts::PI).cos() + 1.0) / 2.0;
    f1 + (curved - f1) / 3.0
}

pub fn get_sun_factors(time_of_day: i64) -> (f32, f32, f32) {
    let celestial_angle = get_celestial_angle(time_of_day);
    let sun_factor = (celestial_angle * std::f32::consts::PI * 2.0).cos() * 2.0 + 0.5;
    let raw_sun = sun_factor.clamp(0.0, 1.0);
    let sun_brightness = raw_sun * 0.78 + 0.22;
    (celestial_angle, raw_sun, sun_brightness)
}

pub fn calc_sunrise_sunset_colors(celestial_angle: f32) -> Option<[f32; 4]> {
    let f = (celestial_angle * std::f32::consts::PI * 2.0).cos();
    if f >= -0.4 && f <= 0.4 {
        let f1 = f / 0.4 * 0.5 + 0.5;
        let mut f2 = 1.0 - (1.0 - (f1 * std::f32::consts::PI).sin()) * 0.99;
        f2 = f2 * f2;
        let r = f1 * 0.3 + 0.7;
        let g = f1 * f1 * 0.7 + 0.2;
        let b = 0.2;
        let a = f2;
        Some([r, g, b, a])
    } else {
        None
    }
}

pub fn get_star_brightness(celestial_angle: f32) -> f32 {
    let f1 = (celestial_angle * std::f32::consts::PI * 2.0).cos() * 2.0 + 0.25;
    let f2 = (1.0 - f1).clamp(0.0, 1.0);
    f2 * f2 * 0.5
}

pub fn hsb_to_rgb(hue: f32, saturation: f32, brightness: f32) -> [f32; 3] {
    if saturation <= 0.0 {
        return [brightness, brightness, brightness];
    }
    let h = (hue - hue.floor()) * 6.0;
    let f = h - h.floor();
    let p = brightness * (1.0 - saturation);
    let q = brightness * (1.0 - saturation * f);
    let t = brightness * (1.0 - saturation * (1.0 - f));
    match (h as i32) % 6 {
        0 => [brightness, t, p],
        1 => [q, brightness, p],
        2 => [p, brightness, t],
        3 => [p, q, brightness],
        4 => [t, p, brightness],
        _ => [brightness, p, q],
    }
}

pub fn get_sky_color_by_temp(temp: f32) -> [f32; 3] {
    let t = (temp / 3.0).clamp(-1.0, 1.0);
    let hue = 0.62222224 - t * 0.05;
    let sat = 0.5 + t * 0.1;
    let bri = 1.0;
    hsb_to_rgb(hue, sat, bri)
}

pub fn get_minecraft_sky_color(celestial_angle: f32, temp: f32) -> [f32; 3] {
    let f1 = ((celestial_angle * std::f32::consts::PI * 2.0).cos() * 2.0 + 0.5).clamp(0.0, 1.0);
    let base = get_sky_color_by_temp(temp);
    [base[0] * f1, base[1] * f1, base[2] * f1]
}

pub fn get_world_provider_fog_color(celestial_angle: f32) -> [f32; 3] {
    let f1 = ((celestial_angle * std::f32::consts::PI * 2.0).cos() * 2.0 + 0.5).clamp(0.0, 1.0);
    let mut r = 0.7529412f32;
    let mut g = 0.84705883f32;
    let mut b = 1.0f32;
    r *= f1 * 0.94 + 0.06;
    g *= f1 * 0.94 + 0.06;
    b *= f1 * 0.91 + 0.09;
    [r, g, b]
}

pub fn update_fog_color(celestial_angle: f32, sky_color: [f32; 3], render_distance_chunks: f32) -> [f32; 3] {
    let mut f = 0.25 + 0.75 * (render_distance_chunks / 16.0);
    f = 1.0 - (f as f64).powf(0.25) as f32;

    let base_fog = get_world_provider_fog_color(celestial_angle);
    let mut fog_r = base_fog[0];
    let mut fog_g = base_fog[1];
    let mut fog_b = base_fog[2];

    fog_r += (sky_color[0] - fog_r) * f;
    fog_g += (sky_color[1] - fog_g) * f;
    fog_b += (sky_color[2] - fog_b) * f;

    [fog_r, fog_g, fog_b]
}

pub fn calculate_sky_and_light(time_of_day: i64) -> ([f32; 4], [f32; 4]) {
    let (celestial_angle, raw_sun, sun_brightness) = get_sun_factors(time_of_day);
    let sky_rgb = get_minecraft_sky_color(celestial_angle, 0.8);
    let fog_rgb = update_fog_color(celestial_angle, sky_rgb, 10.0);

    // Minimum ambient light level in 1.7.10:
    let min_ambient = 0.12 + 0.08 * raw_sun;

    let fog_start = 100.0;
    let fog_end = 160.0;

    (
        [fog_rgb[0], fog_rgb[1], fog_rgb[2], sun_brightness],
        [sun_brightness, min_ambient, fog_start, fog_end],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_celestial_angles() {
        let noon_angle = get_celestial_angle(6000);
        assert!((noon_angle - 0.0).abs() < 0.01);

        let sunset_angle = get_celestial_angle(12000);
        assert!((sunset_angle - 0.215).abs() < 0.01);

        let midnight_angle = get_celestial_angle(18000);
        assert!((midnight_angle - 0.50).abs() < 0.01);

        let sunrise_angle = get_celestial_angle(0);
        assert!((sunrise_angle - 0.785).abs() < 0.01);
    }

    #[test]
    fn test_sunset_colors() {
        let sunset = calc_sunrise_sunset_colors(get_celestial_angle(12000));
        assert!(sunset.is_some());
        let noon = calc_sunrise_sunset_colors(get_celestial_angle(6000));
        assert!(noon.is_none());
    }

    #[test]
    fn test_star_brightness() {
        let noon_stars = get_star_brightness(get_celestial_angle(6000));
        assert_eq!(noon_stars, 0.0);

        let midnight_stars = get_star_brightness(get_celestial_angle(18000));
        assert!((midnight_stars - 0.50).abs() < 0.01);
    }

    #[test]
    fn test_camera_uniform_update_view_proj() {
        let mut uniform = CameraUniform::new();
        let eye = Vec3::new(10.0, 64.0, 10.0);
        uniform.update_view_proj(eye, 45.0, -20.0, 16.0 / 9.0, 6000, 12.34);
        assert_eq!(uniform.eye_pos[0], 10.0);
        assert_eq!(uniform.eye_pos[1], 64.0);
        assert_eq!(uniform.eye_pos[2], 10.0);
        assert_eq!(uniform.eye_pos[3], 12.34);
    }
}


