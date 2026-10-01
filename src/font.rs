use std::sync::OnceLock;

static FONT_DATA: &[u8] = include_bytes!("../assets/minecraft/textures/font/ascii.png");

pub struct FontInfo {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub char_widths: [f32; 256],
}

static FONT: OnceLock<FontInfo> = OnceLock::new();

pub fn get_font() -> &'static FontInfo {
    FONT.get_or_init(|| {
        let font_bytes = crate::resource_pack::get_texture("textures/font/ascii.png", FONT_DATA);
        let img = image::load_from_memory_with_format(&font_bytes, image::ImageFormat::Png)
            .expect("Failed to decode ascii.png")
            .to_rgba8();
        let (width, height) = img.dimensions();
        let raw = img.into_raw();

        let mut char_widths = [0.0f32; 256];
        let cell_w = (width / 16) as usize;
        let cell_h = (height / 16) as usize;
        let w_usize = width as usize;

        for char_idx in 0..256 {
            if char_idx == 32 {
                char_widths[char_idx] = 4.0;
                continue;
            }
            let col = char_idx % 16;
            let row = char_idx / 16;
            let mut k = 7i32;
            while k >= 0 {
                let px = col * cell_w + (k as usize);
                let mut has_pixel = false;
                for l in 0..cell_h {
                    let py = row * cell_h + l;
                    let idx = (py * w_usize + px) * 4 + 3;
                    if raw[idx] > 32 {
                        has_pixel = true;
                        break;
                    }
                }
                if has_pixel {
                    break;
                }
                k -= 1;
            }
            char_widths[char_idx] = (k + 2) as f32;
        }

        FontInfo {
            width,
            height,
            rgba: raw,
            char_widths,
        }
    })
}

pub fn get_char_width(c: char) -> f32 {
    let font = get_font();
    let idx = (c as u32).min(255) as usize;
    font.char_widths[idx]
}

pub fn get_char_uv(c: char) -> (f32, f32, f32, f32) {
    let font = get_font();
    let idx = (c as u32).min(255);
    let col = (idx % 16) as f32;
    let row = (idx / 16) as f32;
    let u0 = (col * 8.0) / (font.width as f32);
    let v0 = (row * 8.0) / (font.height as f32);
    let u1 = ((col + 1.0) * 8.0) / (font.width as f32);
    let v1 = ((row + 1.0) * 8.0) / (font.height as f32);
    (u0, v0, u1, v1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_widths() {
        let font = get_font();
        println!("Font dimensions: {}x{}", font.width, font.height);
        for &ch in &[' ', '!', '"', '1', 'A', 'I', 'M', 'a', 'i', 'l', 't', 'W', '.', ':', '/'] {
            println!("Char '{}' width: {}", ch, get_char_width(ch));
        }
    }
}

