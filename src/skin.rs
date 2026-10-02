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

pub fn extract_json_string_field(json: &str, field: &str) -> Option<String> {
    let key = format!("\"{}\"", field);
    let key_pos = json.find(&key)?;
    let after_key = &json[key_pos + key.len()..];
    let colon_pos = after_key.find(':')?;
    let after_colon = after_key[colon_pos + 1..].trim_start();
    let quote_start = after_colon.find('"')?;
    let inner = &after_colon[quote_start + 1..];
    let quote_end = inner.find('"')?;
    Some(inner[..quote_end].to_string())
}

pub fn extract_skin_url_from_decoded_textures(json: &str) -> Option<String> {
    let skin_pos = json.find("\"SKIN\"")?;
    extract_json_string_field(&json[skin_pos..], "url")
}

pub fn http_get_bytes(url: &str) -> Result<Vec<u8>, std::io::Error> {
    let resp = ureq::get(url)
        .call()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    let mut bytes = Vec::new();
    let mut reader = resp.into_body().into_reader();
    reader
        .read_to_end(&mut bytes)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    Ok(bytes)
}

/// Official Minecraft 1.7.10 skin resolution via Mojang Session Service.
pub fn fetch_mojang_skin_url(player_name: &str) -> Option<String> {
    // 1. Resolve UUID from username
    let profile_url = format!("https://api.mojang.com/users/profiles/minecraft/{}", player_name);
    let profile_bytes = http_get_bytes(&profile_url).ok()?;
    let profile_json = String::from_utf8(profile_bytes).ok()?;
    let uuid = extract_json_string_field(&profile_json, "id")?;

    // 2. Fetch session profile with textures
    let session_url = format!("https://sessionserver.mojang.com/session/minecraft/profile/{}", uuid);
    let session_bytes = http_get_bytes(&session_url).ok()?;
    let session_json = String::from_utf8(session_bytes).ok()?;
    let base64_val = extract_json_string_field(&session_json, "value")?;

    // 3. Base64-decode textures payload
    use base64::Engine;
    let decoded = base64::engine::general_purpose::STANDARD.decode(base64_val.as_bytes()).ok()?;
    let decoded_str = String::from_utf8(decoded).ok()?;
    extract_skin_url_from_decoded_textures(&decoded_str)
}

/// Downloads skin PNG bytes with full fallback chain:
/// 1. Explicit skin_url (if provided)
/// 2. Official Mojang Session Service (textures.minecraft.net)
/// 3. Minotar fallback
/// 4. MC-Heads fallback
pub fn download_skin_bytes(player_name: &str, skin_url: Option<&str>) -> Option<Vec<u8>> {
    if let Some(u) = skin_url {
        if let Ok(b) = http_get_bytes(u) {
            return Some(b);
        }
    }

    if let Some(mojang_url) = fetch_mojang_skin_url(player_name) {
        if let Ok(b) = http_get_bytes(&mojang_url) {
            println!("[SKIN] Fetched official Mojang skin for {} from {}", player_name, mojang_url);
            return Some(b);
        }
    }

    let minotar_url = format!("https://minotar.net/skin/{}", player_name);
    if let Ok(b) = http_get_bytes(&minotar_url) {
        println!("[SKIN] Fetched fallback skin for {} from Minotar", player_name);
        return Some(b);
    }

    let mcheads_url = format!("https://mc-heads.net/skin/{}", player_name);
    if let Ok(b) = http_get_bytes(&mcheads_url) {
        println!("[SKIN] Fetched fallback skin for {} from MC-Heads", player_name);
        return Some(b);
    }

    None
}

pub fn get_cached_skin_bytes(player_name: &str) -> Option<Vec<u8>> {
    let cache_dir = std::path::Path::new(".skin_cache");
    let path = cache_dir.join(format!("{}.png", player_name));
    std::fs::read(path).ok()
}

pub fn save_skin_to_cache(player_name: &str, bytes: &[u8]) {
    let cache_dir = std::path::Path::new(".skin_cache");
    let _ = std::fs::create_dir_all(cache_dir);
    let path = cache_dir.join(format!("{}.png", player_name));
    let _ = std::fs::write(path, bytes);
}

pub fn load_local_skin_file(player_name: &str) -> Option<Vec<u8>> {
    let candidates = [
        format!("skins/{}.png", player_name),
        format!("assets/skins/{}.png", player_name),
    ];
    for c in &candidates {
        if let Ok(bytes) = std::fs::read(c) {
            if let Some(rgba) = normalize_skin_to_64x64(&bytes) {
                return Some(rgba);
            }
        }
    }
    if player_name == "local" {
        return load_local_custom_skin();
    }
    None
}

/// Asynchronously fetch a skin with local file, disk cache, and official Mojang network lookups.
pub fn fetch_skin_async(
    player_key: String,
    skin_url: Option<String>,
    player_name: String,
    pending: Arc<Mutex<Vec<(String, Vec<u8>)>>>,
) {
    if let Some(rgba) = load_local_skin_file(&player_name) {
        println!("[SKIN] Using local skin file for {}", player_name);
        let mut p = pending.lock().unwrap();
        p.push((player_key, rgba));
        return;
    }

    if let Some(cached_bytes) = get_cached_skin_bytes(&player_name) {
        if let Some(rgba) = normalize_skin_to_64x64(&cached_bytes) {
            println!("[SKIN] Loaded cached skin for {}", player_name);
            let mut p = pending.lock().unwrap();
            p.push((player_key.clone(), rgba));
        }
    }

    std::thread::spawn(move || {
        if let Some(bytes) = download_skin_bytes(&player_name, skin_url.as_deref()) {
            if let Some(rgba) = normalize_skin_to_64x64(&bytes) {
                save_skin_to_cache(&player_name, &bytes);
                println!("[SKIN] Successfully resolved and cached skin for {}", player_name);
                let mut p = pending.lock().unwrap();
                p.push((player_key, rgba));
            }
        } else {
            println!("[SKIN] Could not resolve skin for {}, defaulting to Steve", player_name);
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

    #[test]
    fn test_extract_json_string_field() {
        let json = r#"{"id" : "45b08726522145eeb2186c83fcd9f1e5", "name" : "GHutch187"}"#;
        assert_eq!(extract_json_string_field(json, "id"), Some("45b08726522145eeb2186c83fcd9f1e5".to_string()));
        assert_eq!(extract_json_string_field(json, "name"), Some("GHutch187".to_string()));
        assert_eq!(extract_json_string_field(json, "missing"), None);
    }

    #[test]
    fn test_mojang_skin_resolution_ghutch() {
        let skin_url = fetch_mojang_skin_url("GHutch187");
        assert!(skin_url.is_some(), "Must resolve official skin URL for GHutch187");
        let url = skin_url.unwrap();
        assert!(url.contains("textures.minecraft.net"), "URL should be from textures.minecraft.net: {}", url);
        let bytes = http_get_bytes(&url).expect("Must download skin bytes from official URL");
        let normalized = normalize_skin_to_64x64(&bytes);
        assert!(normalized.is_some(), "Must normalize official skin to 64x64");
    }
}
