use crate::raycast::BlockFace;

pub const ATLAS_WIDTH: u32 = 256;
pub const ATLAS_HEIGHT: u32 = 256;
pub const TILE_SIZE: u32 = 16;
pub const TILES_PER_ROW: u32 = ATLAS_WIDTH / TILE_SIZE;

const STONE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/stone.png");
const GRASS_TOP_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/grass_top.png");
const GRASS_SIDE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/grass_side.png");
const GRASS_SIDE_OVERLAY_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/grass_side_overlay.png");
const DIRT_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/dirt.png");
const COBBLESTONE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/cobblestone.png");
const PLANKS_OAK_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/planks_oak.png");
const BEDROCK_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/bedrock.png");
const SAND_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/sand.png");
const GRAVEL_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/gravel.png");
const LOG_OAK_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/log_oak.png");
const LOG_OAK_TOP_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/log_oak_top.png");
const LEAVES_OAK_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/leaves_oak.png");
const GLASS_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/glass.png");
const COAL_ORE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/coal_ore.png");
const IRON_ORE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/iron_ore.png");
const GOLD_ORE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/gold_ore.png");
const DIAMOND_ORE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/diamond_ore.png");
const LAPIS_ORE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/lapis_ore.png");
const REDSTONE_ORE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/redstone_ore.png");
const OBSIDIAN_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/obsidian.png");
const SNOW_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/snow.png");
const ICE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/ice.png");
const ICE_PACKED_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/ice_packed.png");
const SPONGE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/sponge.png");
const SANDSTONE_NORMAL_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/sandstone_normal.png");
const SANDSTONE_TOP_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/sandstone_top.png");
const SANDSTONE_BOTTOM_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/sandstone_bottom.png");
const WOOL_WHITE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/wool_colored_white.png");
const TORCH_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/torch_on.png");
const WATER_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/water_still.png");
const WATER_FLOW_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/water_flow.png");
const LAVA_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/lava_still.png");
const DANDELION_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/flower_dandelion.png");
const POPPY_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/flower_rose.png");
const TALLGRASS_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/tallgrass.png");
const LAPIS_BLOCK_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/lapis_block.png");

const PLANKS_SPRUCE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/planks_spruce.png");
const PLANKS_BIRCH_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/planks_birch.png");
const PLANKS_JUNGLE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/planks_jungle.png");
const PLANKS_ACACIA_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/planks_acacia.png");
const PLANKS_BIG_OAK_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/planks_big_oak.png");

const LOG_SPRUCE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/log_spruce.png");
const LOG_SPRUCE_TOP_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/log_spruce_top.png");
const LOG_BIRCH_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/log_birch.png");
const LOG_BIRCH_TOP_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/log_birch_top.png");
const LOG_JUNGLE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/log_jungle.png");
const LOG_JUNGLE_TOP_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/log_jungle_top.png");
const LOG_ACACIA_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/log_acacia.png");
const LOG_ACACIA_TOP_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/log_acacia_top.png");
const LOG_BIG_OAK_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/log_big_oak.png");
const LOG_BIG_OAK_TOP_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/log_big_oak_top.png");

const CRAFTING_TABLE_TOP_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/crafting_table_top.png");
const CRAFTING_TABLE_SIDE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/crafting_table_side.png");
const CRAFTING_TABLE_FRONT_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/crafting_table_front.png");

const FURNACE_TOP_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/furnace_top.png");
const FURNACE_SIDE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/furnace_side.png");
const FURNACE_FRONT_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/furnace_front_off.png");

const BRICK_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/brick.png");
const STONEBRICK_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/stonebrick.png");
const BOOKSHELF_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/bookshelf.png");

const IRON_BLOCK_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/iron_block.png");
const GOLD_BLOCK_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/gold_block.png");
const DIAMOND_BLOCK_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/diamond_block.png");
const EMERALD_BLOCK_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/emerald_block.png");
const EMERALD_ORE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/emerald_ore.png");

const NETHERRACK_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/netherrack.png");
const GLOWSTONE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/glowstone.png");
const CLAY_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/clay.png");
const HARDENED_CLAY_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/hardened_clay.png");

const TNT_TOP_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/tnt_top.png");
const TNT_BOTTOM_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/tnt_bottom.png");
const TNT_SIDE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/tnt_side.png");

const CACTUS_TOP_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/cactus_top.png");
const CACTUS_BOTTOM_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/cactus_bottom.png");
const CACTUS_SIDE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/cactus_side.png");

const PUMPKIN_TOP_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/pumpkin_top.png");
const PUMPKIN_SIDE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/pumpkin_side.png");
const PUMPKIN_FACE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/pumpkin_face_off.png");

const MELON_TOP_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/melon_top.png");
const MELON_SIDE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/melon_side.png");

const MYCELIUM_TOP_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/mycelium_top.png");
const MYCELIUM_SIDE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/mycelium_side.png");
const REEDS_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/reeds.png");

const SAPLING_OAK_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/sapling_oak.png");
const SAPLING_SPRUCE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/sapling_spruce.png");
const SAPLING_BIRCH_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/sapling_birch.png");
const SAPLING_JUNGLE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/sapling_jungle.png");
const SAPLING_ACACIA_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/sapling_acacia.png");
const SAPLING_ROOFED_OAK_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/sapling_roofed_oak.png");

const DEADBUSH_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/deadbush.png");
const MUSHROOM_BROWN_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/mushroom_brown.png");
const MUSHROOM_RED_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/mushroom_red.png");
const FERN_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/fern.png");

const FLOWER_ORCHID_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/flower_blue_orchid.png");
const FLOWER_ALLIUM_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/flower_allium.png");
const FLOWER_HOUSTONIA_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/flower_houstonia.png");
const FLOWER_TULIP_RED_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/flower_tulip_red.png");
const FLOWER_TULIP_ORANGE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/flower_tulip_orange.png");
const FLOWER_TULIP_WHITE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/flower_tulip_white.png");
const FLOWER_TULIP_PINK_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/flower_tulip_pink.png");
const FLOWER_DAISY_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/flower_oxeye_daisy.png");

const REDSTONE_DUST_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/redstone_dust_cross.png");
const REDSTONE_TORCH_OFF_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/redstone_torch_off.png");
const REDSTONE_TORCH_ON_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/redstone_torch_on.png");
const REPEATER_OFF_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/repeater_off.png");
const REPEATER_ON_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/repeater_on.png");
const COMPARATOR_OFF_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/comparator_off.png");
const COMPARATOR_ON_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/comparator_on.png");
const REDSTONE_BLOCK_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/redstone_block.png");
const REDSTONE_LAMP_OFF_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/redstone_lamp_off.png");
const REDSTONE_LAMP_ON_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/redstone_lamp_on.png");

const PISTON_TOP_NORMAL_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/piston_top_normal.png");
const PISTON_TOP_STICKY_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/piston_top_sticky.png");
const PISTON_SIDE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/piston_side.png");
const PISTON_BOTTOM_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/piston_bottom.png");
const PISTON_INNER_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/piston_inner.png");

const DOOR_WOOD_UPPER_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/door_wood_upper.png");
const DOOR_WOOD_LOWER_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/door_wood_lower.png");
const DOOR_IRON_UPPER_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/door_iron_upper.png");
const DOOR_IRON_LOWER_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/door_iron_lower.png");
const TRAPDOOR_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/trapdoor.png");

const RAIL_NORMAL_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/rail_normal.png");
const RAIL_GOLDEN_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/rail_golden.png");
const RAIL_GOLDEN_POWERED_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/rail_golden_powered.png");
const RAIL_DETECTOR_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/rail_detector.png");
const RAIL_DETECTOR_POWERED_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/rail_detector_powered.png");
const RAIL_ACTIVATOR_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/rail_activator.png");
const RAIL_ACTIVATOR_POWERED_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/rail_activator_powered.png");
const LADDER_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/ladder.png");
const IRON_BARS_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/iron_bars.png");

const QUARTZ_SIDE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/quartz_block_side.png");
const QUARTZ_TOP_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/quartz_block_top.png");
const QUARTZ_BOTTOM_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/quartz_block_bottom.png");
const QUARTZ_CHISELED_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/quartz_block_chiseled.png");
const QUARTZ_CHISELED_TOP_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/quartz_block_chiseled_top.png");
const QUARTZ_LINES_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/quartz_block_lines.png");
const QUARTZ_LINES_TOP_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/quartz_block_lines_top.png");
const QUARTZ_ORE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/quartz_ore.png");

const WOOL_ORANGE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/wool_colored_orange.png");
const WOOL_MAGENTA_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/wool_colored_magenta.png");
const WOOL_LIGHT_BLUE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/wool_colored_light_blue.png");
const WOOL_YELLOW_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/wool_colored_yellow.png");
const WOOL_LIME_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/wool_colored_lime.png");
const WOOL_PINK_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/wool_colored_pink.png");
const WOOL_GRAY_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/wool_colored_gray.png");
const WOOL_SILVER_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/wool_colored_silver.png");
const WOOL_CYAN_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/wool_colored_cyan.png");
const WOOL_PURPLE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/wool_colored_purple.png");
const WOOL_BLUE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/wool_colored_blue.png");
const WOOL_BROWN_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/wool_colored_brown.png");
const WOOL_GREEN_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/wool_colored_green.png");
const WOOL_RED_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/wool_colored_red.png");
const WOOL_BLACK_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/wool_colored_black.png");

const CLAY_WHITE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/hardened_clay_stained_white.png");
const CLAY_ORANGE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/hardened_clay_stained_orange.png");
const CLAY_MAGENTA_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/hardened_clay_stained_magenta.png");
const CLAY_LIGHT_BLUE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/hardened_clay_stained_light_blue.png");
const CLAY_YELLOW_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/hardened_clay_stained_yellow.png");
const CLAY_LIME_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/hardened_clay_stained_lime.png");
const CLAY_PINK_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/hardened_clay_stained_pink.png");
const CLAY_GRAY_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/hardened_clay_stained_gray.png");
const CLAY_SILVER_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/hardened_clay_stained_silver.png");
const CLAY_CYAN_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/hardened_clay_stained_cyan.png");
const CLAY_PURPLE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/hardened_clay_stained_purple.png");
const CLAY_BLUE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/hardened_clay_stained_blue.png");
const CLAY_BROWN_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/hardened_clay_stained_brown.png");
const CLAY_GREEN_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/hardened_clay_stained_green.png");
const CLAY_RED_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/hardened_clay_stained_red.png");
const CLAY_BLACK_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/hardened_clay_stained_black.png");

const HAY_SIDE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/hay_block_side.png");
const HAY_TOP_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/hay_block_top.png");
const COAL_BLOCK_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/coal_block.png");
const NETHER_BRICK_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/nether_brick.png");
const SOUL_SAND_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/soul_sand.png");
const END_STONE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/end_stone.png");
const REDSTONE_DUST_LINE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/redstone_dust_line.png");
const LEVER_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/lever.png");

const GLASS_WHITE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/glass_white.png");
const GLASS_ORANGE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/glass_orange.png");
const GLASS_MAGENTA_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/glass_magenta.png");
const GLASS_LIGHT_BLUE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/glass_light_blue.png");
const GLASS_YELLOW_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/glass_yellow.png");
const GLASS_LIME_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/glass_lime.png");
const GLASS_PINK_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/glass_pink.png");
const GLASS_GRAY_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/glass_gray.png");
const GLASS_SILVER_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/glass_silver.png");
const GLASS_CYAN_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/glass_cyan.png");
const GLASS_PURPLE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/glass_purple.png");
const GLASS_BLUE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/glass_blue.png");
const GLASS_BROWN_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/glass_brown.png");
const GLASS_GREEN_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/glass_green.png");
const GLASS_RED_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/glass_red.png");
const GLASS_BLACK_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/glass_black.png");

const BED_FEET_END_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/bed_feet_end.png");
const BED_FEET_SIDE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/bed_feet_side.png");
const BED_FEET_TOP_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/bed_feet_top.png");
const BED_HEAD_END_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/bed_head_end.png");
const BED_HEAD_SIDE_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/bed_head_side.png");
const BED_HEAD_TOP_BYTES: &[u8] = include_bytes!("../assets/minecraft/textures/blocks/bed_head_top.png");

fn copy_tile(atlas: &mut [u8], slot: u8, png_bytes: &[u8]) {
    let img = match image::load_from_memory_with_format(png_bytes, image::ImageFormat::Png) {
        Ok(i) => i.to_rgba8(),
        Err(_) => return,
    };

    let (src_w, src_h) = img.dimensions();
    if src_w == 0 || src_h == 0 {
        return;
    }

    let frame_img = if src_h > src_w {
        image::imageops::crop_imm(&img, 0, 0, src_w, src_w).to_image()
    } else {
        img
    };

    let final_tile = if frame_img.width() == TILE_SIZE && frame_img.height() == TILE_SIZE {
        frame_img
    } else {
        image::imageops::resize(&frame_img, TILE_SIZE, TILE_SIZE, image::imageops::FilterType::Lanczos3)
    };

    let col = (slot as u32) % TILES_PER_ROW;
    let row = (slot as u32) / TILES_PER_ROW;
    let base_x = col * TILE_SIZE;
    let base_y = row * TILE_SIZE;

    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            let px = final_tile.get_pixel(x, y);
            let atlas_x = base_x + x;
            let atlas_y = base_y + y;
            let dst_idx = ((atlas_y * ATLAS_WIDTH + atlas_x) * 4) as usize;
            atlas[dst_idx..dst_idx + 4].copy_from_slice(&px.0);
        }
    }
}

const BLOCK_TILES: &[(u8, &str, &[u8])] = &[
    (1, "textures/blocks/stone.png", STONE_BYTES),
    (2, "textures/blocks/grass_top.png", GRASS_TOP_BYTES),
    (3, "textures/blocks/grass_side.png", GRASS_SIDE_BYTES),
    (4, "textures/blocks/dirt.png", DIRT_BYTES),
    (5, "textures/blocks/cobblestone.png", COBBLESTONE_BYTES),
    (6, "textures/blocks/planks_oak.png", PLANKS_OAK_BYTES),
    (7, "textures/blocks/bedrock.png", BEDROCK_BYTES),
    (8, "textures/blocks/sand.png", SAND_BYTES),
    (9, "textures/blocks/gravel.png", GRAVEL_BYTES),
    (10, "textures/blocks/log_oak.png", LOG_OAK_BYTES),
    (11, "textures/blocks/log_oak_top.png", LOG_OAK_TOP_BYTES),
    (12, "textures/blocks/leaves_oak.png", LEAVES_OAK_BYTES),
    (13, "textures/blocks/glass.png", GLASS_BYTES),
    (14, "textures/blocks/coal_ore.png", COAL_ORE_BYTES),
    (15, "textures/blocks/iron_ore.png", IRON_ORE_BYTES),
    (16, "textures/blocks/gold_ore.png", GOLD_ORE_BYTES),
    (17, "textures/blocks/diamond_ore.png", DIAMOND_ORE_BYTES),
    (18, "textures/blocks/lapis_ore.png", LAPIS_ORE_BYTES),
    (19, "textures/blocks/redstone_ore.png", REDSTONE_ORE_BYTES),
    (20, "textures/blocks/obsidian.png", OBSIDIAN_BYTES),
    (21, "textures/blocks/snow.png", SNOW_BYTES),
    (22, "textures/blocks/ice.png", ICE_BYTES),
    (23, "textures/blocks/sponge.png", SPONGE_BYTES),
    (24, "textures/blocks/sandstone_normal.png", SANDSTONE_NORMAL_BYTES),
    (25, "textures/blocks/sandstone_top.png", SANDSTONE_TOP_BYTES),
    (26, "textures/blocks/sandstone_bottom.png", SANDSTONE_BOTTOM_BYTES),
    (27, "textures/blocks/wool_colored_white.png", WOOL_WHITE_BYTES),
    (28, "textures/blocks/torch_on.png", TORCH_BYTES),
    (29, "textures/blocks/water_still.png", WATER_BYTES),
    (30, "textures/blocks/lava_still.png", LAVA_BYTES),
    (31, "textures/blocks/flower_dandelion.png", DANDELION_BYTES),
    (32, "textures/blocks/flower_rose.png", POPPY_BYTES),
    (33, "textures/blocks/tallgrass.png", TALLGRASS_BYTES),
    (34, "textures/blocks/lapis_block.png", LAPIS_BLOCK_BYTES),
    (35, "textures/blocks/planks_spruce.png", PLANKS_SPRUCE_BYTES),
    (36, "textures/blocks/planks_birch.png", PLANKS_BIRCH_BYTES),
    (37, "textures/blocks/planks_jungle.png", PLANKS_JUNGLE_BYTES),
    (38, "textures/blocks/planks_acacia.png", PLANKS_ACACIA_BYTES),
    (39, "textures/blocks/planks_big_oak.png", PLANKS_BIG_OAK_BYTES),
    (40, "textures/blocks/log_spruce.png", LOG_SPRUCE_BYTES),
    (41, "textures/blocks/log_spruce_top.png", LOG_SPRUCE_TOP_BYTES),
    (42, "textures/blocks/log_birch.png", LOG_BIRCH_BYTES),
    (43, "textures/blocks/log_birch_top.png", LOG_BIRCH_TOP_BYTES),
    (44, "textures/blocks/log_jungle.png", LOG_JUNGLE_BYTES),
    (45, "textures/blocks/log_jungle_top.png", LOG_JUNGLE_TOP_BYTES),
    (46, "textures/blocks/log_acacia.png", LOG_ACACIA_BYTES),
    (47, "textures/blocks/log_acacia_top.png", LOG_ACACIA_TOP_BYTES),
    (48, "textures/blocks/log_big_oak.png", LOG_BIG_OAK_BYTES),
    (49, "textures/blocks/log_big_oak_top.png", LOG_BIG_OAK_TOP_BYTES),
    (50, "textures/blocks/crafting_table_top.png", CRAFTING_TABLE_TOP_BYTES),
    (51, "textures/blocks/crafting_table_side.png", CRAFTING_TABLE_SIDE_BYTES),
    (52, "textures/blocks/crafting_table_front.png", CRAFTING_TABLE_FRONT_BYTES),
    (53, "textures/blocks/furnace_top.png", FURNACE_TOP_BYTES),
    (54, "textures/blocks/furnace_side.png", FURNACE_SIDE_BYTES),
    (55, "textures/blocks/furnace_front_off.png", FURNACE_FRONT_BYTES),
    (56, "textures/blocks/brick.png", BRICK_BYTES),
    (57, "textures/blocks/stonebrick.png", STONEBRICK_BYTES),
    (58, "textures/blocks/bookshelf.png", BOOKSHELF_BYTES),
    (59, "textures/blocks/iron_block.png", IRON_BLOCK_BYTES),
    (60, "textures/blocks/gold_block.png", GOLD_BLOCK_BYTES),
    (61, "textures/blocks/diamond_block.png", DIAMOND_BLOCK_BYTES),
    (62, "textures/blocks/emerald_block.png", EMERALD_BLOCK_BYTES),
    (63, "textures/blocks/emerald_ore.png", EMERALD_ORE_BYTES),
    (64, "textures/blocks/netherrack.png", NETHERRACK_BYTES),
    (65, "textures/blocks/glowstone.png", GLOWSTONE_BYTES),
    (66, "textures/blocks/clay.png", CLAY_BYTES),
    (67, "textures/blocks/hardened_clay.png", HARDENED_CLAY_BYTES),
    (68, "textures/blocks/tnt_top.png", TNT_TOP_BYTES),
    (69, "textures/blocks/tnt_bottom.png", TNT_BOTTOM_BYTES),
    (70, "textures/blocks/tnt_side.png", TNT_SIDE_BYTES),
    (71, "textures/blocks/cactus_top.png", CACTUS_TOP_BYTES),
    (72, "textures/blocks/cactus_bottom.png", CACTUS_BOTTOM_BYTES),
    (73, "textures/blocks/cactus_side.png", CACTUS_SIDE_BYTES),
    (74, "textures/blocks/pumpkin_top.png", PUMPKIN_TOP_BYTES),
    (75, "textures/blocks/pumpkin_side.png", PUMPKIN_SIDE_BYTES),
    (76, "textures/blocks/pumpkin_face_off.png", PUMPKIN_FACE_BYTES),
    (77, "textures/blocks/melon_top.png", MELON_TOP_BYTES),
    (78, "textures/blocks/melon_side.png", MELON_SIDE_BYTES),
    (79, "textures/blocks/mycelium_top.png", MYCELIUM_TOP_BYTES),
    (80, "textures/blocks/mycelium_side.png", MYCELIUM_SIDE_BYTES),
    (81, "textures/blocks/reeds.png", REEDS_BYTES),
    (82, "textures/blocks/water_flow.png", WATER_FLOW_BYTES),
    (83, "textures/blocks/sapling_oak.png", SAPLING_OAK_BYTES),
    (84, "textures/blocks/sapling_spruce.png", SAPLING_SPRUCE_BYTES),
    (85, "textures/blocks/sapling_birch.png", SAPLING_BIRCH_BYTES),
    (86, "textures/blocks/sapling_jungle.png", SAPLING_JUNGLE_BYTES),
    (87, "textures/blocks/sapling_acacia.png", SAPLING_ACACIA_BYTES),
    (88, "textures/blocks/sapling_roofed_oak.png", SAPLING_ROOFED_OAK_BYTES),
    (89, "textures/blocks/deadbush.png", DEADBUSH_BYTES),
    (90, "textures/blocks/mushroom_brown.png", MUSHROOM_BROWN_BYTES),
    (91, "textures/blocks/mushroom_red.png", MUSHROOM_RED_BYTES),
    (92, "textures/blocks/fern.png", FERN_BYTES),
    (93, "textures/blocks/flower_blue_orchid.png", FLOWER_ORCHID_BYTES),
    (94, "textures/blocks/flower_allium.png", FLOWER_ALLIUM_BYTES),
    (95, "textures/blocks/flower_houstonia.png", FLOWER_HOUSTONIA_BYTES),
    (96, "textures/blocks/flower_tulip_red.png", FLOWER_TULIP_RED_BYTES),
    (97, "textures/blocks/flower_tulip_orange.png", FLOWER_TULIP_ORANGE_BYTES),
    (98, "textures/blocks/flower_tulip_white.png", FLOWER_TULIP_WHITE_BYTES),
    (99, "textures/blocks/flower_tulip_pink.png", FLOWER_TULIP_PINK_BYTES),
    (100, "textures/blocks/flower_oxeye_daisy.png", FLOWER_DAISY_BYTES),
    (101, "textures/blocks/redstone_dust_cross.png", REDSTONE_DUST_BYTES),
    (102, "textures/blocks/redstone_torch_off.png", REDSTONE_TORCH_OFF_BYTES),
    (103, "textures/blocks/redstone_torch_on.png", REDSTONE_TORCH_ON_BYTES),
    (104, "textures/blocks/repeater_off.png", REPEATER_OFF_BYTES),
    (105, "textures/blocks/repeater_on.png", REPEATER_ON_BYTES),
    (106, "textures/blocks/comparator_off.png", COMPARATOR_OFF_BYTES),
    (107, "textures/blocks/comparator_on.png", COMPARATOR_ON_BYTES),
    (108, "textures/blocks/redstone_block.png", REDSTONE_BLOCK_BYTES),
    (109, "textures/blocks/redstone_lamp_off.png", REDSTONE_LAMP_OFF_BYTES),
    (110, "textures/blocks/redstone_lamp_on.png", REDSTONE_LAMP_ON_BYTES),
    (111, "textures/blocks/piston_top_normal.png", PISTON_TOP_NORMAL_BYTES),
    (112, "textures/blocks/piston_top_sticky.png", PISTON_TOP_STICKY_BYTES),
    (113, "textures/blocks/piston_side.png", PISTON_SIDE_BYTES),
    (114, "textures/blocks/piston_bottom.png", PISTON_BOTTOM_BYTES),
    (115, "textures/blocks/piston_inner.png", PISTON_INNER_BYTES),
    (116, "textures/blocks/door_wood_upper.png", DOOR_WOOD_UPPER_BYTES),
    (117, "textures/blocks/door_wood_lower.png", DOOR_WOOD_LOWER_BYTES),
    (118, "textures/blocks/door_iron_upper.png", DOOR_IRON_UPPER_BYTES),
    (119, "textures/blocks/door_iron_lower.png", DOOR_IRON_LOWER_BYTES),
    (120, "textures/blocks/trapdoor.png", TRAPDOOR_BYTES),
    (121, "textures/blocks/rail_normal.png", RAIL_NORMAL_BYTES),
    (122, "textures/blocks/rail_golden.png", RAIL_GOLDEN_BYTES),
    (123, "textures/blocks/rail_golden_powered.png", RAIL_GOLDEN_POWERED_BYTES),
    (124, "textures/blocks/rail_detector.png", RAIL_DETECTOR_BYTES),
    (125, "textures/blocks/rail_detector_powered.png", RAIL_DETECTOR_POWERED_BYTES),
    (126, "textures/blocks/rail_activator.png", RAIL_ACTIVATOR_BYTES),
    (127, "textures/blocks/rail_activator_powered.png", RAIL_ACTIVATOR_POWERED_BYTES),
    (128, "textures/blocks/ladder.png", LADDER_BYTES),
    (129, "textures/blocks/iron_bars.png", IRON_BARS_BYTES),
    (130, "textures/blocks/quartz_block_side.png", QUARTZ_SIDE_BYTES),
    (131, "textures/blocks/quartz_block_top.png", QUARTZ_TOP_BYTES),
    (132, "textures/blocks/quartz_block_bottom.png", QUARTZ_BOTTOM_BYTES),
    (133, "textures/blocks/quartz_block_chiseled.png", QUARTZ_CHISELED_BYTES),
    (134, "textures/blocks/quartz_block_lines.png", QUARTZ_LINES_BYTES),
    (135, "textures/blocks/quartz_ore.png", QUARTZ_ORE_BYTES),
    (136, "textures/blocks/quartz_block_chiseled_top.png", QUARTZ_CHISELED_TOP_BYTES),
    (137, "textures/blocks/quartz_block_lines_top.png", QUARTZ_LINES_TOP_BYTES),
    (138, "textures/blocks/wool_colored_orange.png", WOOL_ORANGE_BYTES),
    (139, "textures/blocks/wool_colored_magenta.png", WOOL_MAGENTA_BYTES),
    (140, "textures/blocks/wool_colored_light_blue.png", WOOL_LIGHT_BLUE_BYTES),
    (141, "textures/blocks/wool_colored_yellow.png", WOOL_YELLOW_BYTES),
    (142, "textures/blocks/wool_colored_lime.png", WOOL_LIME_BYTES),
    (143, "textures/blocks/wool_colored_pink.png", WOOL_PINK_BYTES),
    (144, "textures/blocks/wool_colored_gray.png", WOOL_GRAY_BYTES),
    (145, "textures/blocks/wool_colored_silver.png", WOOL_SILVER_BYTES),
    (146, "textures/blocks/wool_colored_cyan.png", WOOL_CYAN_BYTES),
    (147, "textures/blocks/wool_colored_purple.png", WOOL_PURPLE_BYTES),
    (148, "textures/blocks/wool_colored_blue.png", WOOL_BLUE_BYTES),
    (149, "textures/blocks/wool_colored_brown.png", WOOL_BROWN_BYTES),
    (150, "textures/blocks/wool_colored_green.png", WOOL_GREEN_BYTES),
    (151, "textures/blocks/wool_colored_red.png", WOOL_RED_BYTES),
    (152, "textures/blocks/wool_colored_black.png", WOOL_BLACK_BYTES),
    (153, "textures/blocks/hardened_clay_stained_white.png", CLAY_WHITE_BYTES),
    (154, "textures/blocks/hardened_clay_stained_orange.png", CLAY_ORANGE_BYTES),
    (155, "textures/blocks/hardened_clay_stained_magenta.png", CLAY_MAGENTA_BYTES),
    (156, "textures/blocks/hardened_clay_stained_light_blue.png", CLAY_LIGHT_BLUE_BYTES),
    (157, "textures/blocks/hardened_clay_stained_yellow.png", CLAY_YELLOW_BYTES),
    (158, "textures/blocks/hardened_clay_stained_lime.png", CLAY_LIME_BYTES),
    (159, "textures/blocks/hardened_clay_stained_pink.png", CLAY_PINK_BYTES),
    (160, "textures/blocks/hardened_clay_stained_gray.png", CLAY_GRAY_BYTES),
    (161, "textures/blocks/hardened_clay_stained_silver.png", CLAY_SILVER_BYTES),
    (162, "textures/blocks/hardened_clay_stained_cyan.png", CLAY_CYAN_BYTES),
    (163, "textures/blocks/hardened_clay_stained_purple.png", CLAY_PURPLE_BYTES),
    (164, "textures/blocks/hardened_clay_stained_blue.png", CLAY_BLUE_BYTES),
    (165, "textures/blocks/hardened_clay_stained_brown.png", CLAY_BROWN_BYTES),
    (166, "textures/blocks/hardened_clay_stained_green.png", CLAY_GREEN_BYTES),
    (167, "textures/blocks/hardened_clay_stained_red.png", CLAY_RED_BYTES),
    (168, "textures/blocks/hardened_clay_stained_black.png", CLAY_BLACK_BYTES),
    (169, "textures/blocks/hay_block_side.png", HAY_SIDE_BYTES),
    (170, "textures/blocks/hay_block_top.png", HAY_TOP_BYTES),
    (171, "textures/blocks/coal_block.png", COAL_BLOCK_BYTES),
    (172, "textures/blocks/nether_brick.png", NETHER_BRICK_BYTES),
    (173, "textures/blocks/soul_sand.png", SOUL_SAND_BYTES),
    (174, "textures/blocks/end_stone.png", END_STONE_BYTES),
    (175, "textures/blocks/redstone_dust_line.png", REDSTONE_DUST_LINE_BYTES),
    (176, "textures/blocks/lever.png", LEVER_BYTES),
    (177, "textures/blocks/glass_white.png", GLASS_WHITE_BYTES),
    (178, "textures/blocks/glass_orange.png", GLASS_ORANGE_BYTES),
    (179, "textures/blocks/glass_magenta.png", GLASS_MAGENTA_BYTES),
    (180, "textures/blocks/glass_light_blue.png", GLASS_LIGHT_BLUE_BYTES),
    (181, "textures/blocks/glass_yellow.png", GLASS_YELLOW_BYTES),
    (182, "textures/blocks/glass_lime.png", GLASS_LIME_BYTES),
    (183, "textures/blocks/glass_pink.png", GLASS_PINK_BYTES),
    (184, "textures/blocks/glass_gray.png", GLASS_GRAY_BYTES),
    (185, "textures/blocks/glass_silver.png", GLASS_SILVER_BYTES),
    (186, "textures/blocks/glass_cyan.png", GLASS_CYAN_BYTES),
    (187, "textures/blocks/glass_purple.png", GLASS_PURPLE_BYTES),
    (188, "textures/blocks/glass_blue.png", GLASS_BLUE_BYTES),
    (189, "textures/blocks/glass_brown.png", GLASS_BROWN_BYTES),
    (190, "textures/blocks/glass_green.png", GLASS_GREEN_BYTES),
    (191, "textures/blocks/glass_red.png", GLASS_RED_BYTES),
    (192, "textures/blocks/glass_black.png", GLASS_BLACK_BYTES),
    (193, "textures/blocks/bed_feet_end.png", BED_FEET_END_BYTES),
    (194, "textures/blocks/bed_feet_side.png", BED_FEET_SIDE_BYTES),
    (195, "textures/blocks/bed_feet_top.png", BED_FEET_TOP_BYTES),
    (196, "textures/blocks/bed_head_end.png", BED_HEAD_END_BYTES),
    (197, "textures/blocks/bed_head_side.png", BED_HEAD_SIDE_BYTES),
    (198, "textures/blocks/bed_head_top.png", BED_HEAD_TOP_BYTES),
    (199, "textures/blocks/ice_packed.png", ICE_PACKED_BYTES),
];

pub fn build_block_atlas() -> Vec<u8> {
    let mut atlas = vec![0u8; (ATLAS_WIDTH * ATLAS_HEIGHT * 4) as usize];

    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            let idx = ((y * ATLAS_WIDTH + x) * 4) as usize;
            atlas[idx..idx + 4].copy_from_slice(&[255, 255, 255, 255]);
        }
    }

    for &(slot, path, fallback) in BLOCK_TILES {
        let bytes = crate::resource_pack::get_texture(path, fallback);
        copy_tile(&mut atlas, slot, &bytes);
    }

    let overlay_bytes = crate::resource_pack::get_texture(
        "textures/blocks/grass_side_overlay.png",
        GRASS_SIDE_OVERLAY_BYTES,
    );
    if let Ok(overlay) = image::load_from_memory_with_format(&overlay_bytes, image::ImageFormat::Png) {
        let overlay_rgba = overlay.to_rgba8();
        let overlay_tile = if overlay_rgba.width() == TILE_SIZE && overlay_rgba.height() == TILE_SIZE {
            overlay_rgba
        } else {
            image::imageops::resize(&overlay_rgba, TILE_SIZE, TILE_SIZE, image::imageops::FilterType::Lanczos3)
        };
        let col = 3 % TILES_PER_ROW;
        let row = 3 / TILES_PER_ROW;
        let base_x = col * TILE_SIZE;
        let base_y = row * TILE_SIZE;
        let tint = [0.48f32, 0.75f32, 0.35f32];

        for y in 0..TILE_SIZE {
            for x in 0..TILE_SIZE {
                let opx = overlay_tile.get_pixel(x, y);
                let alpha = opx.0[3] as f32 / 255.0;
                if alpha > 0.0 {
                    let atlas_x = base_x + x;
                    let atlas_y = base_y + y;
                    let idx = ((atlas_y * ATLAS_WIDTH + atlas_x) * 4) as usize;

                    let src_r = opx.0[0] as f32 * tint[0];
                    let src_g = opx.0[1] as f32 * tint[1];
                    let src_b = opx.0[2] as f32 * tint[2];

                    let dst_r = atlas[idx] as f32;
                    let dst_g = atlas[idx + 1] as f32;
                    let dst_b = atlas[idx + 2] as f32;

                    atlas[idx] = (src_r * alpha + dst_r * (1.0 - alpha)).round() as u8;
                    atlas[idx + 1] = (src_g * alpha + dst_g * (1.0 - alpha)).round() as u8;
                    atlas[idx + 2] = (src_b * alpha + dst_b * (1.0 - alpha)).round() as u8;
                }
            }
        }
    }

    atlas
}

fn get_piston_face_slot(meta: u8, face: BlockFace, top_slot: u8) -> u8 {
    let orientation = match meta & 7 {
        0 => BlockFace::Bottom,
        1 => BlockFace::Top,
        2 => BlockFace::North,
        3 => BlockFace::South,
        4 => BlockFace::West,
        _ => BlockFace::East,
    };
    let opposite = match orientation {
        BlockFace::Bottom => BlockFace::Top,
        BlockFace::Top => BlockFace::Bottom,
        BlockFace::North => BlockFace::South,
        BlockFace::South => BlockFace::North,
        BlockFace::West => BlockFace::East,
        BlockFace::East => BlockFace::West,
    };
    if face == orientation {
        top_slot
    } else if face == opposite {
        114
    } else {
        113
    }
}

pub fn get_block_slot(block_id: u16, meta: u8, face: BlockFace) -> (u8, Option<[f32; 3]>) {
    match block_id {
        1 => (1, None),
        2 => match face {
            BlockFace::Top => (2, Some([0.48, 0.75, 0.35])),
            BlockFace::Bottom => (4, None),
            _ => (3, None),
        },
        3 => (4, None),
        4 => (5, None),
        5 => match meta & 7 {
            1 => (35, None),
            2 => (36, None),
            3 => (37, None),
            4 => (38, None),
            5 => (39, None),
            _ => (6, None),
        },
        6 => match meta & 7 {
            1 => (84, None),
            2 => (85, None),
            3 => (86, None),
            4 => (87, None),
            5 => (88, None),
            _ => (83, None),
        },
        7 => (7, None),
        8 => (82, None),
        9 => (29, None),
        10 | 11 => (30, None),
        12 => (8, None),
        13 => (9, None),
        14 => (16, None),
        15 => (15, None),
        16 => (14, None),
        17 => match meta & 3 {
            1 => match face {
                BlockFace::Top | BlockFace::Bottom => (41, None),
                _ => (40, None),
            },
            2 => match face {
                BlockFace::Top | BlockFace::Bottom => (43, None),
                _ => (42, None),
            },
            3 => match face {
                BlockFace::Top | BlockFace::Bottom => (45, None),
                _ => (44, None),
            },
            _ => match face {
                BlockFace::Top | BlockFace::Bottom => (11, None),
                _ => (10, None),
            },
        },
        18 => (12, Some([0.30, 0.68, 0.20])),
        19 => (23, None),
        20 => (13, None),
        21 => (18, None),
        22 => (34, None),
        24 => match face {
            BlockFace::Top => (25, None),
            BlockFace::Bottom => (26, None),
            _ => (24, None),
        },
        31 => match meta & 3 {
            2 => (92, Some([0.48, 0.75, 0.35])),
            _ => (33, Some([0.48, 0.75, 0.35])),
        },
        32 => (89, None),
        27 => if (meta & 8) != 0 { (123, None) } else { (122, None) },
        28 => if (meta & 8) != 0 { (125, None) } else { (124, None) },
        29 => (get_piston_face_slot(meta, face, 112), None),
        33 => (get_piston_face_slot(meta, face, 111), None),
        34 => {
            let top_slot = if (meta & 8) != 0 { 112 } else { 111 };
            (get_piston_face_slot(meta, face, top_slot), None)
        },
        35 => if meta == 0 {
            (27, None)
        } else {
            (137 + (meta.min(15) as u8), None)
        },
        44 => match meta & 7 {
            1 => match face {
                BlockFace::Top => (25, None),
                BlockFace::Bottom => (26, None),
                _ => (24, None),
            },
            2 => (6, None),
            3 => (5, None),
            4 => (56, None),
            5 => (57, None),
            6 => (172, None),
            7 => (131, None),
            _ => (1, None),
        },
        37 => (31, None),
        38 => match meta {
            1 => (93, None),
            2 => (94, None),
            3 => (95, None),
            4 => (96, None),
            5 => (97, None),
            6 => (98, None),
            7 => (99, None),
            8 => (100, None),
            _ => (32, None),
        },
        39 => (90, None),
        40 => (91, None),
        41 => (60, None),
        42 => (59, None),
        45 => (56, None),
        46 => match face {
            BlockFace::Top => (68, None),
            BlockFace::Bottom => (69, None),
            _ => (70, None),
        },
        47 => match face {
            BlockFace::Top | BlockFace::Bottom => (6, None),
            _ => (58, None),
        },
        49 => (20, None),
        50 => (28, None),
        55 => {
            let p = (meta.min(15) as f32) / 15.0;
            (101, Some([0.3 + 0.7 * p, 0.0, 0.0]))
        },
        64 => if (meta & 8) != 0 { (116, None) } else { (117, None) },
        65 => (128, None),
        66 => (121, None),
        69 => (176, None),
        70 => (1, None),
        71 => if (meta & 8) != 0 { (118, None) } else { (119, None) },
        72 => (6, None),
        75 => (102, None),
        76 => (103, None),
        77 => (1, None),
        143 => (6, None),
        147 => (60, None),
        148 => (59, None),
        56 => (17, None),
        57 => (61, None),
        58 => match face {
            BlockFace::Top => (50, None),
            BlockFace::Bottom => (6, None),
            BlockFace::North => (52, None),
            _ => (51, None),
        },
        61 | 62 => match face {
            BlockFace::Top | BlockFace::Bottom => (53, None),
            BlockFace::North => (55, None),
            _ => (54, None),
        },
        73 | 74 => (19, None),
        78 | 80 => (21, None),
        79 => (22, None),
        81 => match face {
            BlockFace::Top => (71, None),
            BlockFace::Bottom => (72, None),
            _ => (73, None),
        },
        82 => (66, None),
        83 => (81, Some([0.48, 0.75, 0.35])),
        86 | 91 => match face {
            BlockFace::Top | BlockFace::Bottom => (74, None),
            BlockFace::North => (76, None),
            _ => (75, None),
        },
        87 => (64, None),
        88 => (173, None),
        89 => (65, None),
        93 => match face { BlockFace::Top => (104, None), _ => (1, None) },
        94 => match face { BlockFace::Top => (105, None), _ => (1, None) },
        96 | 167 => (120, None),
        101 => (129, None),
        112 => (172, None),
        121 => (174, None),
        123 => (109, None),
        124 => (110, None),
        126 => match meta & 7 {
            1 => (35, None),
            2 => (36, None),
            3 => (37, None),
            4 => (38, None),
            5 => (39, None),
            _ => (6, None),
        },
        98 => (57, None),
        103 => match face {
            BlockFace::Top | BlockFace::Bottom => (77, None),
            _ => (78, None),
        },
        110 => match face {
            BlockFace::Top => (79, None),
            BlockFace::Bottom => (4, None),
            _ => (80, None),
        },
        129 => (63, None),
        133 => (62, None),
        149 => match face { BlockFace::Top => (106, None), _ => (1, None) },
        150 => match face { BlockFace::Top => (107, None), _ => (1, None) },
        152 => (108, None),
        153 => (135, None),
        155 => match meta {
            1 => match face {
                BlockFace::Top | BlockFace::Bottom => (136, None),
                _ => (133, None),
            },
            2..=4 => match face {
                BlockFace::Top | BlockFace::Bottom => (137, None),
                _ => (134, None),
            },
            _ => match face {
                BlockFace::Top => (131, None),
                BlockFace::Bottom => (132, None),
                _ => (130, None),
            },
        },
        157 => if (meta & 8) != 0 { (127, None) } else { (126, None) },
        159 => (153 + (meta.min(15) as u8), None),
        170 => match face {
            BlockFace::Top | BlockFace::Bottom => (170, None),
            _ => (169, None),
        },
        171 => if meta == 0 {
            (27, None)
        } else {
            (137 + (meta.min(15) as u8), None)
        },
        172 => (67, None),
        173 => (171, None),
        162 => match meta & 3 {
            1 => match face {
                BlockFace::Top | BlockFace::Bottom => (49, None),
                _ => (48, None),
            },
            _ => match face {
                BlockFace::Top | BlockFace::Bottom => (47, None),
                _ => (46, None),
            },
        },
        // Additional Minecraft 1.7.10 blocks
        23 => match face {
            BlockFace::North => (55, None),
            BlockFace::Top | BlockFace::Bottom => (53, None),
            _ => (54, None),
        },
        25 => (58, None),
        26 => {
            let is_head = (meta & 8) != 0;
            match face {
                BlockFace::Top => if is_head { (198, None) } else { (195, None) },
                BlockFace::Bottom => (6, None),
                _ => if is_head { (197, None) } else { (194, None) },
            }
        },
        30 => (13, None),
        36 => {
            let top_slot = if (meta & 8) != 0 { 112 } else { 111 };
            (get_piston_face_slot(meta, face, top_slot), None)
        },
        43 => match meta & 7 {
            1 => match face {
                BlockFace::Top => (25, None),
                BlockFace::Bottom => (26, None),
                _ => (24, None),
            },
            2 => (6, None),
            3 => (5, None),
            4 => (56, None),
            5 => (57, None),
            6 => (172, None),
            7 => (131, None),
            _ => (1, None),
        },
        48 => (5, None),
        52 => (129, None),
        53 => (6, None),
        54 | 146 => (6, None),
        59 => (81, Some([0.48, 0.75, 0.35])),
        60 => (4, None),
        67 => (5, None),
        84 => (58, None),
        85 => (6, None),
        92 => (27, None),
        95 => (177 + (meta.min(15) as u8), None),
        97 => (1, None),
        102 => (13, None),
        104 | 105 => (81, Some([0.48, 0.75, 0.35])),
        106 => (12, Some([0.30, 0.68, 0.20])),
        107 => (6, None),
        108 => (56, None),
        109 => (57, None),
        111 => (12, Some([0.30, 0.68, 0.20])),
        113 => (172, None),
        114 => (172, None),
        115 => (91, None),
        116 => (49, None),
        117 => (5, None),
        118 => (59, None),
        120 => (174, None),
        122 => (20, None),
        125 => match meta & 7 {
            1 => (35, None),
            2 => (36, None),
            3 => (37, None),
            4 => (38, None),
            5 => (39, None),
            _ => (6, None),
        },
        127 => (44, None),
        128 => (24, None),
        130 => (174, None),
        131 | 132 => (6, None),
        134 => (35, None),
        135 => (36, None),
        136 => (37, None),
        137 => (60, None),
        138 => (13, None),
        139 => (5, None),
        140 => (56, None),
        141 | 142 => (81, Some([0.48, 0.75, 0.35])),
        144 => (20, None),
        145 => (59, None),
        151 => (6, None),
        154 => (59, None),
        156 => match face {
            BlockFace::Top => (131, None),
            BlockFace::Bottom => (132, None),
            _ => (130, None),
        },
        158 => match face {
            BlockFace::North => (55, None),
            BlockFace::Top | BlockFace::Bottom => (53, None),
            _ => (54, None),
        },
        160 => (177 + (meta.min(15) as u8), None),
        163 => (38, None),
        164 => (39, None),
        165 => (12, Some([0.48, 0.75, 0.35])),
        174 => (199, None),
        175 => match meta & 7 {
            0 => (31, None),
            1 => (93, None),
            2 => (33, Some([0.48, 0.75, 0.35])),
            3 => (92, Some([0.48, 0.75, 0.35])),
            4 => (32, None),
            _ => (98, None),
        },
        _ => (5, None),
    }
}

pub fn get_slot_uv(slot: u8) -> (f32, f32, f32, f32) {
    let col = ((slot as u32) % TILES_PER_ROW) as f32;
    let row = ((slot as u32) / TILES_PER_ROW) as f32;
    let u0 = col / (TILES_PER_ROW as f32);
    let v0 = row / (TILES_PER_ROW as f32);
    let u1 = (col + 1.0) / (TILES_PER_ROW as f32);
    let v1 = (row + 1.0) / (TILES_PER_ROW as f32);
    let eps = 0.0001;
    (u0 + eps, v0 + eps, u1 - eps, v1 - eps)
}

pub fn get_water_frame_rgba(frame: usize, is_flow: bool) -> [u8; 16 * 16 * 4] {
    let bytes = if is_flow {
        crate::resource_pack::get_texture("textures/blocks/water_flow.png", WATER_FLOW_BYTES)
    } else {
        crate::resource_pack::get_texture("textures/blocks/water_still.png", WATER_BYTES)
    };
    let img = match image::load_from_memory_with_format(&bytes, image::ImageFormat::Png) {
        Ok(i) => i.to_rgba8(),
        Err(_) => return [0u8; 16 * 16 * 4],
    };

    let (src_w, src_h) = img.dimensions();
    if src_w == 0 || src_h == 0 {
        return [0u8; 16 * 16 * 4];
    }
    let total_frames = (src_h / src_w).max(1);
    let frame_idx = (frame as u32) % total_frames;
    let frame_y = frame_idx * src_w;
    let cropped = image::imageops::crop_imm(&img, 0, frame_y, src_w, src_w).to_image();
    let final_tile = if cropped.width() == 16 && cropped.height() == 16 {
        cropped
    } else {
        image::imageops::resize(&cropped, 16, 16, image::imageops::FilterType::Nearest)
    };

    let mut tile = [0u8; 16 * 16 * 4];
    for y in 0..16 {
        for x in 0..16 {
            let px = final_tile.get_pixel(x, y);
            let idx = ((y * 16 + x) * 4) as usize;
            tile[idx..idx + 4].copy_from_slice(&px.0);
        }
    }

    tile
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grass_side() {
        let img = image::load_from_memory_with_format(GRASS_SIDE_BYTES, image::ImageFormat::Png).unwrap().to_rgba8();
        println!("Grass side top-left pixel: {:?}", img.get_pixel(0, 0));
        println!("Grass side top-center pixel: {:?}", img.get_pixel(8, 0));
        println!("Grass side bottom pixel: {:?}", img.get_pixel(8, 15));
    }

    #[test]
    fn test_atlas_build() {
        let atlas = build_block_atlas();
        assert_eq!(atlas.len(), (ATLAS_WIDTH * ATLAS_HEIGHT * 4) as usize);
    }

    #[test]
    fn test_inspect_textures() {
        let reeds = image::load_from_memory_with_format(REEDS_BYTES, image::ImageFormat::Png).unwrap();
        assert_eq!(reeds.width(), 16);
        let overlay = image::load_from_memory_with_format(GRASS_SIDE_OVERLAY_BYTES, image::ImageFormat::Png).unwrap();
        assert_eq!(overlay.width(), 16);

        let torch = image::load_from_memory_with_format(TORCH_BYTES, image::ImageFormat::Png).unwrap().to_rgba8();
        println!("TORCH_ON:");
        for y in 0..16 {
            let mut line = String::new();
            for x in 0..16 {
                let p = torch.get_pixel(x, y).0;
                line.push(if p[3] > 10 { '#' } else { '.' });
            }
            println!("{:2}: {}", y, line);
        }

        let rstorch = image::load_from_memory_with_format(REDSTONE_TORCH_ON_BYTES, image::ImageFormat::Png).unwrap().to_rgba8();
        println!("REDSTONE_TORCH_ON:");
        for y in 0..16 {
            let mut line = String::new();
            for x in 0..16 {
                let p = rstorch.get_pixel(x, y).0;
                if p[3] > 10 {
                    if p[0] > 150 && p[1] < 100 {
                        line.push('R'); // Redstone red
                    } else {
                        line.push('#'); // wood stick
                    }
                } else {
                    line.push('.');
                }
            }
            println!("{:2}: {}", y, line);
        }

        let door_lower = image::load_from_memory_with_format(DOOR_WOOD_LOWER_BYTES, image::ImageFormat::Png).unwrap();
        assert_eq!(door_lower.width(), 16);

        let head_side = image::load_from_memory_with_format(BED_HEAD_SIDE_BYTES, image::ImageFormat::Png).unwrap();
        assert_eq!(head_side.width(), 16);
        let head_end = image::load_from_memory_with_format(BED_HEAD_END_BYTES, image::ImageFormat::Png).unwrap();
        assert_eq!(head_end.width(), 16);
        let feet_side = image::load_from_memory_with_format(BED_FEET_SIDE_BYTES, image::ImageFormat::Png).unwrap();
        assert_eq!(feet_side.width(), 16);
        let feet_end = image::load_from_memory_with_format(BED_FEET_END_BYTES, image::ImageFormat::Png).unwrap();
        assert_eq!(feet_end.width(), 16);
        let head_top = image::load_from_memory_with_format(BED_HEAD_TOP_BYTES, image::ImageFormat::Png).unwrap();
        assert_eq!(head_top.width(), 16);
        let feet_top = image::load_from_memory_with_format(BED_FEET_TOP_BYTES, image::ImageFormat::Png).unwrap();
        assert_eq!(feet_top.width(), 16);
    }

    #[test]
    fn test_sapling_slots() {
        for meta in 0..6 {
            let (slot, tint) = get_block_slot(6, meta, BlockFace::North);
            assert_ne!(slot, 5);
            assert!(slot >= 83 && slot <= 88);
            assert_eq!(tint, None);
        }
    }

    #[test]
    fn test_high_res_tile_downscale() {
        let mut img = image::RgbaImage::new(32, 32);
        for px in img.pixels_mut() {
            *px = image::Rgba([128, 64, 32, 255]);
        }
        let mut png_bytes = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut png_bytes), image::ImageFormat::Png).unwrap();

        let mut atlas = vec![0u8; (ATLAS_WIDTH * ATLAS_HEIGHT * 4) as usize];
        copy_tile(&mut atlas, 1, &png_bytes);

        let col = 1 % TILES_PER_ROW;
        let row = 1 / TILES_PER_ROW;
        let base_x = col * TILE_SIZE;
        let base_y = row * TILE_SIZE;
        let idx = ((base_y * ATLAS_WIDTH + base_x) * 4) as usize;
        assert_eq!(atlas[idx], 128);
        assert_eq!(atlas[idx + 1], 64);
        assert_eq!(atlas[idx + 2], 32);
        assert_eq!(atlas[idx + 3], 255);
    }

    #[test]
    fn test_new_block_slots() {
        // Redstone wire
        let (slot, tint) = get_block_slot(55, 0, BlockFace::Top);
        assert_eq!(slot, 101);
        assert!(tint.is_some());
        let (slot_p15, tint_p15) = get_block_slot(55, 15, BlockFace::Top);
        assert_eq!(slot_p15, 101);
        assert!(tint_p15.unwrap()[0] > tint.unwrap()[0]);

        // Pistons
        assert_eq!(get_block_slot(33, 1, BlockFace::Top).0, 111);
        assert_eq!(get_block_slot(33, 1, BlockFace::Bottom).0, 114);
        assert_eq!(get_block_slot(33, 1, BlockFace::North).0, 113);
        assert_eq!(get_block_slot(29, 2, BlockFace::North).0, 112);
        assert_eq!(get_block_slot(29, 2, BlockFace::South).0, 114);
        assert_eq!(get_block_slot(29, 2, BlockFace::Top).0, 113);

        // Doors
        assert_eq!(get_block_slot(64, 8, BlockFace::North).0, 116);
        assert_eq!(get_block_slot(64, 0, BlockFace::North).0, 117);
        assert_eq!(get_block_slot(71, 8, BlockFace::North).0, 118);
        assert_eq!(get_block_slot(71, 0, BlockFace::North).0, 119);

        // Rails & Ladders
        assert_eq!(get_block_slot(66, 0, BlockFace::Top).0, 121);
        assert_eq!(get_block_slot(27, 0, BlockFace::Top).0, 122);
        assert_eq!(get_block_slot(27, 8, BlockFace::Top).0, 123);
        assert_eq!(get_block_slot(65, 0, BlockFace::North).0, 128);
        assert_eq!(get_block_slot(101, 0, BlockFace::North).0, 129);

        // Repeaters & Comparators
        assert_eq!(get_block_slot(93, 0, BlockFace::Top).0, 104);
        assert_eq!(get_block_slot(94, 0, BlockFace::Top).0, 105);
        assert_eq!(get_block_slot(149, 0, BlockFace::Top).0, 106);
        assert_eq!(get_block_slot(150, 0, BlockFace::Top).0, 107);

        // Redstone Torches & Lamps
        assert_eq!(get_block_slot(75, 0, BlockFace::North).0, 102);
        assert_eq!(get_block_slot(76, 0, BlockFace::North).0, 103);
        assert_eq!(get_block_slot(123, 0, BlockFace::North).0, 109);
        assert_eq!(get_block_slot(124, 0, BlockFace::North).0, 110);
        assert_eq!(get_block_slot(152, 0, BlockFace::North).0, 108);

        // Wool 16 colors
        for meta in 0..16 {
            let (slot, _) = get_block_slot(35, meta, BlockFace::North);
            assert_ne!(slot, 5, "Wool meta {} fell back to cobblestone", meta);
        }

        // Stained clay 16 colors
        for meta in 0..16 {
            let (slot, _) = get_block_slot(159, meta, BlockFace::North);
            assert_ne!(slot, 5, "Stained clay meta {} fell back to cobblestone", meta);
        }

        // Quartz
        assert_eq!(get_block_slot(155, 0, BlockFace::Top).0, 131);
        assert_eq!(get_block_slot(155, 0, BlockFace::Bottom).0, 132);
        assert_eq!(get_block_slot(155, 0, BlockFace::North).0, 130);
        assert_eq!(get_block_slot(153, 0, BlockFace::North).0, 135);

        // Levers, Buttons, Pressure Plates
        assert_eq!(get_block_slot(69, 0, BlockFace::North).0, 176);
        assert_eq!(get_block_slot(70, 0, BlockFace::North).0, 1);
        assert_eq!(get_block_slot(72, 0, BlockFace::North).0, 6);
        assert_eq!(get_block_slot(77, 0, BlockFace::North).0, 1);
        assert_eq!(get_block_slot(143, 0, BlockFace::North).0, 6);
        assert_eq!(get_block_slot(147, 0, BlockFace::North).0, 60);
        assert_eq!(get_block_slot(148, 0, BlockFace::North).0, 59);

        // Stairs
        assert_eq!(get_block_slot(53, 0, BlockFace::North).0, 6);
        assert_eq!(get_block_slot(67, 0, BlockFace::North).0, 5);
        assert_eq!(get_block_slot(108, 0, BlockFace::North).0, 56);
        assert_eq!(get_block_slot(109, 0, BlockFace::North).0, 57);
        assert_eq!(get_block_slot(114, 0, BlockFace::North).0, 172);

        // Fences, Panes, Slabs
        assert_eq!(get_block_slot(85, 0, BlockFace::North).0, 6);
        assert_eq!(get_block_slot(102, 0, BlockFace::North).0, 13);
        assert_eq!(get_block_slot(43, 0, BlockFace::North).0, 1);
        assert_eq!(get_block_slot(125, 0, BlockFace::North).0, 6);

        // Stained Glass 16 colors (Block 95) & Panes (Block 160)
        assert_eq!(get_block_slot(95, 2, BlockFace::North).0, 179); // Magenta stained glass
        assert_eq!(get_block_slot(160, 2, BlockFace::North).0, 179); // Magenta stained glass pane
        for meta in 0..16 {
            let (slot_g, _) = get_block_slot(95, meta, BlockFace::North);
            assert_eq!(slot_g, 177 + meta as u8);
            let (slot_p, _) = get_block_slot(160, meta, BlockFace::North);
            assert_eq!(slot_p, 177 + meta as u8);
        }

        // Bed (Block 26)
        assert_eq!(get_block_slot(26, 0, BlockFace::Top).0, 195); // Foot top
        assert_eq!(get_block_slot(26, 0, BlockFace::North).0, 194); // Foot side
        assert_eq!(get_block_slot(26, 8, BlockFace::Top).0, 198); // Head top
        assert_eq!(get_block_slot(26, 8, BlockFace::North).0, 197); // Head side

        // Moving piston (Block 36)
        assert_eq!(get_block_slot(36, 1, BlockFace::Top).0, 111);
        assert_eq!(get_block_slot(36, 1, BlockFace::North).0, 113);

        // Packed Ice (Block 174)
        assert_eq!(get_block_slot(174, 0, BlockFace::North).0, 199);
    }
}

