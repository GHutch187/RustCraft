use std::collections::HashMap;
use std::io::Read;
use std::sync::{Arc, Mutex};
use image::GenericImageView;

pub const DEFAULT_STEVE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/entity/steve.png");

/// Normalizes any 64x32 or 64x64 PNG skin into standard 64x64 RGBA8 raw bytes.
pub fn normalize_skin_to_64x64(png_bytes: &[u8]) -> Option<Vec<u8>> {
    let img = image::load_from_memory_with_format(png_bytes, image::ImageFormat::Png).ok()?;
    let (width, height) = img.dimensions();

    let mut rgba_out = vec![0u8; 64 * 64 * 4];

    if width == 64 && height == 64 {
        let rgba = img.to_rgba8();
        rgba_out.copy_from_slice(&rgba);
    } else if width == 64 && height == 32 {
        let rgba = img.to_rgba8();
        // 1. Copy top 32 rows (Head, Torso, Right Arm, Right Leg, Hat)
        for y in 0..32 {
            for x in 0..64 {
                let px = rgba.get_pixel(x, y);
                let out_idx = ((y * 64 + x) * 4) as usize;
                rgba_out[out_idx..out_idx + 4].copy_from_slice(&px.0);
            }
        }

        // 2. Mirror Right Leg (x: 0..16, y: 16..32) into Left Leg (x: 16..32, y: 48..64)
        for y in 0..16 {
            for x in 0..16 {
                let px = rgba.get_pixel(x, 16 + y);
                let dest_x = 16 + x;
                let dest_y = 48 + y;
                let out_idx = ((dest_y * 64 + dest_x) * 4) as usize;
                rgba_out[out_idx..out_idx + 4].copy_from_slice(&px.0);
            }
        }

        // 3. Mirror Right Arm (x: 40..56, y: 16..32) into Left Arm (x: 32..48, y: 48..64)
        for y in 0..16 {
            for x in 0..16 {
                let px = rgba.get_pixel(40 + x, 16 + y);
                let dest_x = 32 + x;
                let dest_y = 48 + y;
                let out_idx = ((dest_y * 64 + dest_x) * 4) as usize;
                rgba_out[out_idx..out_idx + 4].copy_from_slice(&px.0);
            }
        }
    } else {
        return None;
    }

    // Force opaque alpha on base layer to prevent invisible player bodies
    for y in 0..16 {
        for x in 0..32 {
            rgba_out[((y * 64 + x) * 4 + 3) as usize] = 255;
        }
    }
    for y in 16..32 {
        for x in 0..64 {
            rgba_out[((y * 64 + x) * 4 + 3) as usize] = 255;
        }
    }
    for y in 48..64 {
        for x in 16..32 {
            rgba_out[((y * 64 + x) * 4 + 3) as usize] = 255;
        }
        for x in 32..48 {
            rgba_out[((y * 64 + x) * 4 + 3) as usize] = 255;
        }
    }

    Some(rgba_out)
}

/// Checks if a custom skin file `skin.png` or `assets/skin.png` exists locally.
pub fn load_local_custom_skin() -> Option<Vec<u8>> {
    if let Ok(bytes) = std::fs::read("skin.png") {
        if let Some(rgba) = normalize_skin_to_64x64(&bytes) {
            println!("[SKIN] Loaded local custom skin from skin.png");
            return Some(rgba);
        }
    }
    if let Ok(bytes) = std::fs::read("assets/skin.png") {
        if let Some(rgba) = normalize_skin_to_64x64(&bytes) {
            println!("[SKIN] Loaded local custom skin from assets/skin.png");
            return Some(rgba);
        }
    }
    None
}

pub fn get_default_steve_rgba() -> Vec<u8> {
    let steve_bytes = crate::resource_pack::get_texture("textures/entity/steve.png", DEFAULT_STEVE_BYTES);
    normalize_skin_to_64x64(&steve_bytes).unwrap_or_else(|| vec![255; 64 * 64 * 4])
}

/// Holds skin textures on the GPU and pending downloaded skins.
pub struct SkinManager {
    /// Pending RGBA textures waiting to be uploaded to GPU: (player_key, rgba8)
    pub pending_skins: Arc<Mutex<Vec<(String, Vec<u8>)>>>,
    /// GPU BindGroups for each player skin
    pub skin_bind_groups: HashMap<String, wgpu::BindGroup>,
    /// GPU Textures
    pub skin_textures: HashMap<String, wgpu::Texture>,
    pub default_bind_group: Option<wgpu::BindGroup>,
}

impl SkinManager {
    pub fn new(pending_skins: Arc<Mutex<Vec<(String, Vec<u8>)>>>) -> Self {
        Self {
            pending_skins,
            skin_bind_groups: HashMap::new(),
            skin_textures: HashMap::new(),
            default_bind_group: None,
        }
    }

    pub fn init_default(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
        camera_buffer: &wgpu::Buffer,
        sampler: &wgpu::Sampler,
    ) {
        let initial_rgba = load_local_custom_skin().unwrap_or_else(get_default_steve_rgba);
        let (bg, tex) = Self::create_skin_gpu(device, queue, layout, camera_buffer, sampler, &initial_rgba);
        self.default_bind_group = Some(bg.clone());
        self.skin_bind_groups.insert("default".to_string(), bg.clone());
        self.skin_bind_groups.insert("local".to_string(), bg);
        self.skin_textures.insert("default".to_string(), tex);
    }

    pub fn create_skin_gpu(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
        camera_buffer: &wgpu::Buffer,
        sampler: &wgpu::Sampler,
        rgba_64x64: &[u8],
    ) -> (wgpu::BindGroup, wgpu::Texture) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Player Skin Texture"),
            size: wgpu::Extent3d {
                width: 64,
                height: 64,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            rgba_64x64,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(64 * 4),
                rows_per_image: Some(64),
            },
            wgpu::Extent3d {
                width: 64,
                height: 64,
                depth_or_array_layers: 1,
            },
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
            label: Some("Player Skin Bind Group"),
        });

        (bind_group, texture)
    }

    pub fn process_pending(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
        camera_buffer: &wgpu::Buffer,
        sampler: &wgpu::Sampler,
    ) {
        let mut pending = self.pending_skins.lock().unwrap();
        for (key, rgba) in pending.drain(..) {
            let (bg, tex) = Self::create_skin_gpu(device, queue, layout, camera_buffer, sampler, &rgba);
            self.skin_bind_groups.insert(key.clone(), bg.clone());
            self.skin_bind_groups.insert(key.to_lowercase(), bg);
            self.skin_textures.insert(key, tex);
        }
    }

    pub fn get_bind_group<'a>(&'a self, player_key: &str) -> Option<&'a wgpu::BindGroup> {
        self.skin_bind_groups
            .get(player_key)
            .or_else(|| self.skin_bind_groups.get(&player_key.to_lowercase()))
            .or(self.default_bind_group.as_ref())
    }
}

/// Asynchronously fetch a skin by URL or player username and push into pending_skins.
pub fn fetch_skin_async(player_key: String, skin_url: Option<String>, player_name: String, pending: Arc<Mutex<Vec<(String, Vec<u8>)>>>) {
    std::thread::spawn(move || {
        let url = if let Some(u) = skin_url {
            u
        } else {
            format!("https://minotar.net/skin/{}", player_name)
        };

        match ureq::get(&url).call() {
            Ok(resp) => {
                let mut bytes = Vec::new();
                let mut reader = resp.into_body().into_reader();
                if reader.read_to_end(&mut bytes).is_ok() {
                    if let Some(rgba) = normalize_skin_to_64x64(&bytes) {
                        println!("[SKIN] Successfully loaded skin for {}", player_name);
                        let mut p = pending.lock().unwrap();
                        p.push((player_key, rgba));
                    }
                }
            }
            Err(e) => {
                println!("[SKIN] Could not fetch skin for {}: {:?}", player_name, e);
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_steve() {
        let rgba = get_default_steve_rgba();
        assert_eq!(rgba.len(), 64 * 64 * 4);
    }

    #[test]
    fn test_normalize_64x64() {
        let mut img = image::RgbaImage::new(64, 64);
        for p in img.pixels_mut() {
            *p = image::Rgba([128, 64, 32, 255]);
        }
        let mut png_bytes = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut png_bytes), image::ImageFormat::Png).unwrap();
        let normalized = normalize_skin_to_64x64(&png_bytes).expect("Should normalize 64x64");
        assert_eq!(normalized.len(), 64 * 64 * 4);
    }

    #[test]
    fn test_normalize_64x32() {
        let mut img = image::RgbaImage::new(64, 32);
        for p in img.pixels_mut() {
            *p = image::Rgba([100, 150, 200, 255]);
        }
        let mut png_bytes = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut png_bytes), image::ImageFormat::Png).unwrap();
        let normalized = normalize_skin_to_64x64(&png_bytes).expect("Should normalize 64x32");
        assert_eq!(normalized.len(), 64 * 64 * 4);
    }
}
