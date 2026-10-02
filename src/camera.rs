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

pub fn calculate_sky_and_light(time_of_day: i64) -> ([f32; 4], [f32; 4]) {
    let (_celestial_angle, raw_sun, sun_brightness) = get_sun_factors(time_of_day);

    // Minecraft 1.7.10 fog color curve
    let fog_r = 0.7529f32 * raw_sun * 0.94 + 0.06;
    let fog_g = 0.8471f32 * raw_sun * 0.94 + 0.06;
    let fog_b = 1.0f32 * raw_sun * 0.91 + 0.09;

    // Minimum ambient light level in 1.7.10:
    let min_ambient = 0.12 + 0.08 * raw_sun;

    let fog_start = 100.0;
    let fog_end = 160.0;

    (
        [fog_r, fog_g, fog_b, sun_brightness],
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
}


