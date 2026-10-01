use std::borrow::Cow;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub struct ResourcePack {
    pub name: String,
    pub description: Option<String>,
    pub textures: HashMap<String, Vec<u8>>,
}

impl ResourcePack {
    pub fn normalize_key(path: &str) -> String {
        let mut clean = path.replace('\\', "/");
        while clean.starts_with("./") {
            clean = clean[2..].to_string();
        }
        while clean.starts_with('/') {
            clean = clean[1..].to_string();
        }
        if clean.starts_with("assets/minecraft/") {
            clean = clean["assets/minecraft/".len()..].to_string();
        } else if clean.starts_with("minecraft/") {
            clean = clean["minecraft/".len()..].to_string();
        }
        clean.to_lowercase()
    }

    pub fn from_zip(zip_path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let file = File::open(zip_path)?;
        let mut archive = zip::ZipArchive::new(file)?;
        let mut textures = HashMap::new();
        let mut description = None;

        let pack_name = zip_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Custom Pack")
            .to_string();

        for i in 0..archive.len() {
            let mut entry = archive.by_index(i)?;
            if entry.is_dir() {
                continue;
            }
            let entry_name = entry.name().to_string();

            if entry_name.ends_with("pack.mcmeta") {
                let mut buf = String::new();
                if entry.read_to_string(&mut buf).is_ok() {
                    description = parse_mcmeta_description(&buf);
                }
                continue;
            }

            if entry_name.to_lowercase().ends_with(".png") {
                let norm = Self::normalize_key(&entry_name);
                let mut bytes = Vec::new();
                if entry.read_to_end(&mut bytes).is_ok() {
                    textures.insert(norm, bytes);
                }
            }
        }

        Ok(Self {
            name: pack_name,
            description,
            textures,
        })
    }

    pub fn from_directory(dir_path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let mut textures = HashMap::new();
        let mut description = None;

        let pack_name = dir_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("Custom Pack")
            .to_string();

        let mcmeta_path = dir_path.join("pack.mcmeta");
        if mcmeta_path.is_file() {
            if let Ok(meta_str) = std::fs::read_to_string(&mcmeta_path) {
                description = parse_mcmeta_description(&meta_str);
            }
        }

        scan_directory(dir_path, dir_path, &mut textures);

        Ok(Self {
            name: pack_name,
            description,
            textures,
        })
    }
}

fn scan_directory(root: &Path, current: &Path, textures: &mut HashMap<String, Vec<u8>>) {
    let entries = match std::fs::read_dir(current) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_directory(root, &path, textures);
        } else if path.is_file() {
            let path_str = path.to_string_lossy();
            if path_str.to_lowercase().ends_with(".png") {
                if let Ok(rel) = path.strip_prefix(root) {
                    let norm = ResourcePack::normalize_key(&rel.to_string_lossy());
                    if let Ok(bytes) = std::fs::read(&path) {
                        textures.insert(norm, bytes);
                    }
                }
            }
        }
    }
}

fn parse_mcmeta_description(content: &str) -> Option<String> {
    if let Some(pos) = content.find("\"description\"") {
        let after = &content[pos + "\"description\"".len()..];
        if let Some(colon) = after.find(':') {
            let val_part = after[colon + 1..].trim_start();
            if val_part.starts_with('"') {
                let rest = &val_part[1..];
                let mut desc = String::new();
                let mut chars = rest.chars();
                while let Some(ch) = chars.next() {
                    if ch == '\\' {
                        if let Some(next_ch) = chars.next() {
                            match next_ch {
                                '"' => desc.push('"'),
                                '\\' => desc.push('\\'),
                                'n' => desc.push('\n'),
                                't' => desc.push('\t'),
                                'u' => {
                                    let hex: String = chars.by_ref().take(4).collect();
                                    if let Ok(code) = u32::from_str_radix(&hex, 16) {
                                        if let Some(c) = char::from_u32(code) {
                                            desc.push(c);
                                        }
                                    }
                                }
                                other => desc.push(other),
                            }
                        }
                    } else if ch == '"' {
                        break;
                    } else {
                        desc.push(ch);
                    }
                }
                return Some(desc);
            }
        }
    }
    None
}

fn parse_options_resource_pack(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("resourcePacks:") {
            let after = trimmed["resourcePacks:".len()..].trim();
            if let Some(start) = after.find('"') {
                if let Some(end) = after[start + 1..].find('"') {
                    let mut pack = after[start + 1..start + 1 + end].to_string();
                    if pack.starts_with("file/") {
                        pack = pack["file/".len()..].to_string();
                    }
                    if !pack.is_empty() {
                        return Some(pack);
                    }
                }
            }
        } else if !trimmed.is_empty() && !trimmed.starts_with('#') && !trimmed.contains(':') {
            return Some(trimmed.to_string());
        }
    }
    None
}

pub struct ResourcePackManager {
    pub active_pack: Option<ResourcePack>,
}

impl ResourcePackManager {
    pub fn load(custom_arg: Option<&str>) -> Self {
        let rp_dir = Path::new("resourcepacks");
        if !rp_dir.exists() {
            let _ = std::fs::create_dir_all(rp_dir);
            let readme_path = rp_dir.join("README.txt");
            if !readme_path.exists() {
                let _ = std::fs::write(
                    &readme_path,
                    "Place Minecraft 1.7.10 Resource Packs (.zip or extracted folders) here!\r\n\
                     Rustcraft will automatically load textures from active packs.\r\n",
                );
            }
        }

        // 1. Try explicit argument if provided
        if let Some(arg) = custom_arg {
            let direct_path = PathBuf::from(arg);
            if direct_path.is_file() {
                if let Ok(pack) = ResourcePack::from_zip(&direct_path) {
                    println!("[RESOURCE PACK] Loaded zip pack: '{}' ({} custom textures)", pack.name, pack.textures.len());
                    if let Some(desc) = &pack.description {
                        println!("[RESOURCE PACK] Description: {}", desc);
                    }
                    return Self { active_pack: Some(pack) };
                }
            } else if direct_path.is_dir() {
                if let Ok(pack) = ResourcePack::from_directory(&direct_path) {
                    println!("[RESOURCE PACK] Loaded folder pack: '{}' ({} custom textures)", pack.name, pack.textures.len());
                    if let Some(desc) = &pack.description {
                        println!("[RESOURCE PACK] Description: {}", desc);
                    }
                    return Self { active_pack: Some(pack) };
                }
            }

            let in_rp = rp_dir.join(arg);
            if in_rp.is_file() {
                if let Ok(pack) = ResourcePack::from_zip(&in_rp) {
                    println!("[RESOURCE PACK] Loaded zip pack: '{}' ({} custom textures)", pack.name, pack.textures.len());
                    if let Some(desc) = &pack.description {
                        println!("[RESOURCE PACK] Description: {}", desc);
                    }
                    return Self { active_pack: Some(pack) };
                }
            } else if in_rp.is_dir() {
                if let Ok(pack) = ResourcePack::from_directory(&in_rp) {
                    println!("[RESOURCE PACK] Loaded folder pack: '{}' ({} custom textures)", pack.name, pack.textures.len());
                    if let Some(desc) = &pack.description {
                        println!("[RESOURCE PACK] Description: {}", desc);
                    }
                    return Self { active_pack: Some(pack) };
                }
            }

            let in_rp_zip = rp_dir.join(format!("{}.zip", arg));
            if in_rp_zip.is_file() {
                if let Ok(pack) = ResourcePack::from_zip(&in_rp_zip) {
                    println!("[RESOURCE PACK] Loaded zip pack: '{}' ({} custom textures)", pack.name, pack.textures.len());
                    if let Some(desc) = &pack.description {
                        println!("[RESOURCE PACK] Description: {}", desc);
                    }
                    return Self { active_pack: Some(pack) };
                }
            }
        }

        // 2. Check options.txt
        for opt_candidate in &["options.txt", "resourcepacks/options.txt"] {
            let opt_path = Path::new(opt_candidate);
            if opt_path.is_file() {
                if let Ok(content) = std::fs::read_to_string(opt_path) {
                    if let Some(selected) = parse_options_resource_pack(&content) {
                        let candidate_path = rp_dir.join(&selected);
                        if candidate_path.is_file() {
                            if let Ok(pack) = ResourcePack::from_zip(&candidate_path) {
                                println!("[RESOURCE PACK] Loaded from options.txt: '{}' ({} custom textures)", pack.name, pack.textures.len());
                                return Self { active_pack: Some(pack) };
                            }
                        } else if candidate_path.is_dir() {
                            if let Ok(pack) = ResourcePack::from_directory(&candidate_path) {
                                println!("[RESOURCE PACK] Loaded from options.txt: '{}' ({} custom textures)", pack.name, pack.textures.len());
                                return Self { active_pack: Some(pack) };
                            }
                        }
                    }
                }
            }
        }

        println!("[RESOURCE PACK] Default Minecraft 1.7.10 textures active.");
        Self { active_pack: None }
    }
}

static MANAGER: OnceLock<ResourcePackManager> = OnceLock::new();

pub fn init(custom_pack_arg: Option<&str>) {
    let _ = MANAGER.set(ResourcePackManager::load(custom_pack_arg));
}

pub fn get_manager() -> &'static ResourcePackManager {
    MANAGER.get_or_init(|| ResourcePackManager::load(None))
}

pub fn get_candidate_keys(path: &str) -> Vec<String> {
    let norm = ResourcePack::normalize_key(path);
    let mut candidates = Vec::with_capacity(8);
    candidates.push(norm.clone());

    if norm.starts_with("textures/blocks/") {
        let sub = &norm["textures/blocks/".len()..];
        candidates.push(format!("textures/block/{}", sub));
    } else if norm.starts_with("textures/block/") {
        let sub = &norm["textures/block/".len()..];
        candidates.push(format!("textures/blocks/{}", sub));
    }

    if norm.starts_with("textures/items/") {
        let sub = &norm["textures/items/".len()..];
        candidates.push(format!("textures/item/{}", sub));
    } else if norm.starts_with("textures/item/") {
        let sub = &norm["textures/item/".len()..];
        candidates.push(format!("textures/items/{}", sub));
    }

    let file_name = norm.rsplit('/').next().unwrap_or(&norm);
    let modern_name = match file_name {
        "grass_top.png" => Some("grass_block_top.png"),
        "grass_side.png" => Some("grass_block_side.png"),
        "grass_side_overlay.png" => Some("grass_block_side_overlay.png"),
        "planks_oak.png" => Some("oak_planks.png"),
        "planks_spruce.png" => Some("spruce_planks.png"),
        "planks_birch.png" => Some("birch_planks.png"),
        "planks_jungle.png" => Some("jungle_planks.png"),
        "planks_acacia.png" => Some("acacia_planks.png"),
        "planks_big_oak.png" => Some("dark_oak_planks.png"),
        "log_oak.png" => Some("oak_log.png"),
        "log_oak_top.png" => Some("oak_log_top.png"),
        "log_spruce.png" => Some("spruce_log.png"),
        "log_spruce_top.png" => Some("spruce_log_top.png"),
        "log_birch.png" => Some("birch_log.png"),
        "log_birch_top.png" => Some("birch_log_top.png"),
        "log_jungle.png" => Some("jungle_log.png"),
        "log_jungle_top.png" => Some("jungle_log_top.png"),
        "log_acacia.png" => Some("acacia_log.png"),
        "log_acacia_top.png" => Some("acacia_log_top.png"),
        "log_big_oak.png" => Some("dark_oak_log.png"),
        "log_big_oak_top.png" => Some("dark_oak_log_top.png"),
        "leaves_oak.png" => Some("oak_leaves.png"),
        "leaves_spruce.png" => Some("spruce_leaves.png"),
        "leaves_birch.png" => Some("birch_leaves.png"),
        "leaves_jungle.png" => Some("jungle_leaves.png"),
        "leaves_acacia.png" => Some("acacia_leaves.png"),
        "leaves_big_oak.png" => Some("dark_oak_leaves.png"),
        "sandstone_normal.png" => Some("sandstone.png"),
        "wool_colored_white.png" => Some("white_wool.png"),
        "torch_on.png" => Some("torch.png"),
        "flower_rose.png" => Some("poppy.png"),
        "flower_dandelion.png" => Some("dandelion.png"),
        "tallgrass.png" => Some("grass.png"),
        "stonebrick.png" => Some("stone_bricks.png"),
        "brick.png" => Some("bricks.png"),
        "hardened_clay.png" => Some("terracotta.png"),
        "reeds.png" => Some("sugar_cane.png"),
        "deadbush.png" => Some("dead_bush.png"),
        "mushroom_brown.png" => Some("brown_mushroom.png"),
        "mushroom_red.png" => Some("red_mushroom.png"),
        "flower_blue_orchid.png" => Some("blue_orchid.png"),
        "flower_allium.png" => Some("allium.png"),
        "flower_houstonia.png" => Some("azure_bluet.png"),
        "flower_tulip_red.png" => Some("red_tulip.png"),
        "flower_tulip_orange.png" => Some("orange_tulip.png"),
        "flower_tulip_white.png" => Some("white_tulip.png"),
        "flower_tulip_pink.png" => Some("pink_tulip.png"),
        "flower_oxeye_daisy.png" => Some("oxeye_daisy.png"),
        "sapling_oak.png" => Some("oak_sapling.png"),
        "sapling_spruce.png" => Some("spruce_sapling.png"),
        "sapling_birch.png" => Some("birch_sapling.png"),
        "sapling_jungle.png" => Some("jungle_sapling.png"),
        "sapling_acacia.png" => Some("acacia_sapling.png"),
        "sapling_roofed_oak.png" => Some("dark_oak_sapling.png"),
        "furnace_front_off.png" => Some("furnace_front.png"),
        "pumpkin_face_off.png" => Some("carved_pumpkin.png"),
        "redstone_dust_cross.png" => Some("redstone_dust_dot.png"),
        "redstone_torch_on.png" => Some("redstone_torch.png"),
        "piston_top_normal.png" => Some("piston_top.png"),
        "door_wood_upper.png" => Some("oak_door_top.png"),
        "door_wood_lower.png" => Some("oak_door_bottom.png"),
        "door_iron_upper.png" => Some("iron_door_top.png"),
        "door_iron_lower.png" => Some("iron_door_bottom.png"),
        "trapdoor.png" => Some("oak_trapdoor.png"),
        "rail_normal.png" => Some("rail.png"),
        "rail_golden.png" => Some("powered_rail.png"),
        "rail_golden_powered.png" => Some("powered_rail_on.png"),
        "nether_brick.png" => Some("nether_bricks.png"),
        _ => None,
    };

    if let Some(modern) = modern_name {
        candidates.push(format!("textures/block/{}", modern));
        candidates.push(format!("textures/blocks/{}", modern));
        if modern == "grass.png" {
            candidates.push("textures/block/short_grass.png".to_string());
        }
        if modern == "oak_sapling.png" {
            candidates.push("textures/block/oak_sapling_stage_0.png".to_string());
        }
        if modern == "acacia_sapling.png" {
            candidates.push("textures/block/acacia_sapling_stage_0.png".to_string());
        }
        if modern == "carved_pumpkin.png" {
            candidates.push("textures/block/pumpkin_face.png".to_string());
        }
    }

    candidates
}

pub fn get_texture(path: &'static str, fallback: &'static [u8]) -> Cow<'static, [u8]> {
    let manager = get_manager();
    let candidates = get_candidate_keys(path);

    if let Some(pack) = &manager.active_pack {
        for candidate in &candidates {
            if let Some(bytes) = pack.textures.get(candidate) {
                return Cow::Borrowed(bytes.as_slice());
            }
        }
    }

    for candidate in &candidates {
        let on_disk = Path::new("assets/minecraft").join(candidate);
        if on_disk.is_file() {
            if let Ok(bytes) = std::fs::read(&on_disk) {
                return Cow::Owned(bytes);
            }
        }
    }

    Cow::Borrowed(fallback)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_key() {
        assert_eq!(
            ResourcePack::normalize_key("assets/minecraft/textures/blocks/stone.png"),
            "textures/blocks/stone.png"
        );
        assert_eq!(
            ResourcePack::normalize_key("assets\\minecraft\\textures\\blocks\\stone.png"),
            "textures/blocks/stone.png"
        );
        assert_eq!(
            ResourcePack::normalize_key("minecraft/textures/blocks/stone.png"),
            "textures/blocks/stone.png"
        );
        assert_eq!(
            ResourcePack::normalize_key("textures/blocks/stone.png"),
            "textures/blocks/stone.png"
        );
        assert_eq!(
            ResourcePack::normalize_key("TEXTURES/BLOCKS/STONE.PNG"),
            "textures/blocks/stone.png"
        );
        assert_eq!(
            ResourcePack::normalize_key("./assets/minecraft/textures/blocks/dirt.png"),
            "textures/blocks/dirt.png"
        );
    }

    #[test]
    fn test_parse_mcmeta_description() {
        let meta = r#"{"pack":{"pack_format":1,"description":"Faithful 32x for 1.7.10"}}"#;
        assert_eq!(
            parse_mcmeta_description(meta).as_deref(),
            Some("Faithful 32x for 1.7.10")
        );

        let meta_escaped = r#"{"pack":{"pack_format":1,"description":"Line 1\nLine 2 \"quoted\""}}"#;
        assert_eq!(
            parse_mcmeta_description(meta_escaped).as_deref(),
            Some("Line 1\nLine 2 \"quoted\"")
        );
    }

    #[test]
    fn test_parse_options_resource_pack() {
        let opt1 = "resourcePacks:[\"Faithful32.zip\"]";
        assert_eq!(parse_options_resource_pack(opt1).as_deref(), Some("Faithful32.zip"));

        let opt2 = "resourcePacks:[\"file/SphaxPureBDcraft.zip\"]";
        assert_eq!(parse_options_resource_pack(opt2).as_deref(), Some("SphaxPureBDcraft.zip"));

        let opt3 = "MyTexturePack";
        assert_eq!(parse_options_resource_pack(opt3).as_deref(), Some("MyTexturePack"));
    }

    #[test]
    fn test_fallback_texture() {
        let fallback = b"dummy_fallback";
        let res = get_texture("textures/blocks/non_existent_texture_xyz.png", fallback);
        assert_eq!(res.as_ref(), fallback);
    }

    #[test]
    fn test_candidate_keys() {
        let candidates = get_candidate_keys("textures/blocks/leaves_oak.png");
        assert!(candidates.contains(&"textures/blocks/leaves_oak.png".to_string()));
        assert!(candidates.contains(&"textures/block/leaves_oak.png".to_string()));
        assert!(candidates.contains(&"textures/block/oak_leaves.png".to_string()));

        let plank_candidates = get_candidate_keys("textures/blocks/planks_oak.png");
        assert!(plank_candidates.contains(&"textures/block/oak_planks.png".to_string()));

        let dirt_candidates = get_candidate_keys("textures/blocks/dirt.png");
        assert!(dirt_candidates.contains(&"textures/block/dirt.png".to_string()));
    }
}

