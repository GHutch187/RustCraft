use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Block {
    pub id: u16,
    pub meta: u8,
    pub block_light: u8,
    pub sky_light: u8,
}

#[allow(dead_code)]
impl Block {
    pub const AIR: Block = Block {
        id: 0,
        meta: 0,
        block_light: 0,
        sky_light: 0,
    };

    pub fn is_air(&self) -> bool {
        self.id == 0
    }

    pub fn name(&self) -> &'static str {
        match self.id {
            0 => "Air",
            1 => "Stone",
            2 => "Grass Block",
            3 => match self.meta {
                1 => "Coarse Dirt",
                2 => "Podzol",
                _ => "Dirt",
            },
            4 => "Cobblestone",
            5 => match self.meta {
                1 => "Spruce Wood Planks",
                2 => "Birch Wood Planks",
                3 => "Jungle Wood Planks",
                4 => "Acacia Wood Planks",
                5 => "Dark Oak Wood Planks",
                _ => "Oak Wood Planks",
            },
            6 => match self.meta & 7 {
                1 => "Spruce Sapling",
                2 => "Birch Sapling",
                3 => "Jungle Sapling",
                4 => "Acacia Sapling",
                5 => "Dark Oak Sapling",
                _ => "Oak Sapling",
            },
            7 => "Bedrock",
            8 => "Flowing Water",
            9 => "Still Water",
            10 => "Flowing Lava",
            11 => "Still Lava",
            12 => match self.meta {
                1 => "Red Sand",
                _ => "Sand",
            },
            13 => "Gravel",
            14 => "Gold Ore",
            15 => "Iron Ore",
            16 => "Coal Ore",
            17 => match self.meta & 3 {
                1 => "Spruce Wood",
                2 => "Birch Wood",
                3 => "Jungle Wood",
                _ => "Oak Wood",
            },
            18 => match self.meta & 3 {
                1 => "Spruce Leaves",
                2 => "Birch Leaves",
                3 => "Jungle Leaves",
                _ => "Oak Leaves",
            },
            19 => "Sponge",
            20 => "Glass",
            21 => "Lapis Lazuli Ore",
            22 => "Lapis Lazuli Block",
            23 => "Dispenser",
            24 => match self.meta {
                1 => "Chiseled Sandstone",
                2 => "Smooth Sandstone",
                _ => "Sandstone",
            },
            25 => "Note Block",
            26 => "Bed",
            27 => "Powered Rail",
            28 => "Detector Rail",
            29 => "Sticky Piston",
            30 => "Cobweb",
            31 => match self.meta {
                0 => "Dead Shrub",
                2 => "Fern",
                _ => "Tall Grass",
            },
            32 => "Dead Bush",
            33 => "Piston",
            34 => "Piston Head",
            35 => match self.meta & 15 {
                1 => "Orange Wool",
                2 => "Magenta Wool",
                3 => "Light Blue Wool",
                4 => "Yellow Wool",
                5 => "Lime Wool",
                6 => "Pink Wool",
                7 => "Gray Wool",
                8 => "Light Gray Wool",
                9 => "Cyan Wool",
                10 => "Purple Wool",
                11 => "Blue Wool",
                12 => "Brown Wool",
                13 => "Green Wool",
                14 => "Red Wool",
                15 => "Black Wool",
                _ => "White Wool",
            },
            36 => "Moving Piston",
            37 => "Dandelion",
            38 => match self.meta & 15 {
                1 => "Blue Orchid",
                2 => "Allium",
                3 => "Azure Bluet",
                4 => "Red Tulip",
                5 => "Orange Tulip",
                6 => "White Tulip",
                7 => "Pink Tulip",
                8 => "Oxeye Daisy",
                _ => "Poppy",
            },
            39 => "Brown Mushroom",
            40 => "Red Mushroom",
            41 => "Gold Block",
            42 => "Iron Block",
            43 => "Double Stone Slab",
            44 => match self.meta & 7 {
                1 => "Sandstone Slab",
                2 => "Wooden Slab",
                3 => "Cobblestone Slab",
                4 => "Bricks Slab",
                5 => "Stone Brick Slab",
                6 => "Nether Brick Slab",
                7 => "Quartz Slab",
                _ => "Stone Slab",
            },
            45 => "Bricks",
            46 => "TNT",
            47 => "Bookshelf",
            48 => "Mossy Cobblestone",
            49 => "Obsidian",
            50 => "Torch",
            51 => "Fire",
            52 => "Monster Spawner",
            53 => "Oak Wood Stairs",
            54 => "Chest",
            55 => "Redstone Wire",
            56 => "Diamond Ore",
            57 => "Diamond Block",
            58 => "Crafting Table",
            59 => "Wheat Crops",
            60 => "Farmland",
            61 => "Furnace",
            62 => "Burning Furnace",
            63 => "Standing Sign",
            64 => "Wooden Door",
            65 => "Ladder",
            66 => "Rail",
            67 => "Cobblestone Stairs",
            68 => "Wall Sign",
            69 => "Lever",
            70 => "Stone Pressure Plate",
            71 => "Iron Door",
            72 => "Wooden Pressure Plate",
            73 => "Redstone Ore",
            74 => "Glowing Redstone Ore",
            75 => "Redstone Torch (off)",
            76 => "Redstone Torch (on)",
            77 => "Stone Button",
            78 => "Snow Layer",
            79 => "Ice",
            80 => "Snow Block",
            81 => "Cactus",
            82 => "Clay",
            83 => "Sugar Canes",
            84 => "Jukebox",
            85 => "Fence",
            86 => "Pumpkin",
            87 => "Netherrack",
            88 => "Soul Sand",
            89 => "Glowstone",
            90 => "Nether Portal",
            91 => "Jack o'Lantern",
            92 => "Cake",
            93 => "Redstone Repeater (off)",
            94 => "Redstone Repeater (on)",
            95 => match self.meta & 15 {
                1 => "Orange Stained Glass",
                2 => "Magenta Stained Glass",
                3 => "Light Blue Stained Glass",
                4 => "Yellow Stained Glass",
                5 => "Lime Stained Glass",
                6 => "Pink Stained Glass",
                7 => "Gray Stained Glass",
                8 => "Light Gray Stained Glass",
                9 => "Cyan Stained Glass",
                10 => "Purple Stained Glass",
                11 => "Blue Stained Glass",
                12 => "Brown Stained Glass",
                13 => "Green Stained Glass",
                14 => "Red Stained Glass",
                15 => "Black Stained Glass",
                _ => "White Stained Glass",
            },
            96 => "Wooden Trapdoor",
            97 => "Monster Egg",
            98 => match self.meta {
                1 => "Mossy Stone Bricks",
                2 => "Cracked Stone Bricks",
                3 => "Chiseled Stone Bricks",
                _ => "Stone Bricks",
            },
            99 => "Huge Brown Mushroom",
            100 => "Huge Red Mushroom",
            101 => "Iron Bars",
            102 => "Glass Pane",
            103 => "Melon Block",
            104 => "Pumpkin Stem",
            105 => "Melon Stem",
            106 => "Vines",
            107 => "Fence Gate",
            108 => "Brick Stairs",
            109 => "Stone Brick Stairs",
            110 => "Mycelium",
            111 => "Lily Pad",
            112 => "Nether Brick",
            113 => "Nether Brick Fence",
            114 => "Nether Brick Stairs",
            115 => "Nether Wart",
            116 => "Enchantment Table",
            117 => "Brewing Stand",
            118 => "Cauldron",
            119 => "End Portal",
            120 => "End Portal Frame",
            121 => "End Stone",
            122 => "Dragon Egg",
            123 => "Redstone Lamp (off)",
            124 => "Redstone Lamp (on)",
            125 => "Double Wooden Slab",
            126 => match self.meta & 7 {
                1 => "Spruce Wood Slab",
                2 => "Birch Wood Slab",
                3 => "Jungle Wood Slab",
                4 => "Acacia Wood Slab",
                5 => "Dark Oak Wood Slab",
                _ => "Oak Wood Slab",
            },
            127 => "Cocoa Plant",
            128 => "Sandstone Stairs",
            129 => "Emerald Ore",
            130 => "Ender Chest",
            131 => "Tripwire Hook",
            132 => "Tripwire",
            133 => "Emerald Block",
            134 => "Spruce Wood Stairs",
            135 => "Birch Wood Stairs",
            136 => "Jungle Wood Stairs",
            137 => "Command Block",
            138 => "Beacon",
            139 => "Cobblestone Wall",
            140 => "Flower Pot",
            141 => "Carrot Crop",
            142 => "Potato Crop",
            143 => "Wooden Button",
            144 => "Mob Head",
            145 => "Anvil",
            146 => "Trapped Chest",
            147 => "Weighted Pressure Plate (light)",
            148 => "Weighted Pressure Plate (heavy)",
            149 => "Redstone Comparator (off)",
            150 => "Redstone Comparator (on)",
            151 => "Daylight Sensor",
            152 => "Redstone Block",
            153 => "Nether Quartz Ore",
            154 => "Hopper",
            155 => match self.meta {
                1 => "Chiseled Quartz Block",
                2 => "Pillar Quartz Block",
                _ => "Quartz Block",
            },
            156 => "Quartz Stairs",
            157 => "Activator Rail",
            158 => "Dropper",
            159 => match self.meta & 15 {
                1 => "Orange Stained Clay",
                2 => "Magenta Stained Clay",
                3 => "Light Blue Stained Clay",
                4 => "Yellow Stained Clay",
                5 => "Lime Stained Clay",
                6 => "Pink Stained Clay",
                7 => "Gray Stained Clay",
                8 => "Light Gray Stained Clay",
                9 => "Cyan Stained Clay",
                10 => "Purple Stained Clay",
                11 => "Blue Stained Clay",
                12 => "Brown Stained Clay",
                13 => "Green Stained Clay",
                14 => "Red Stained Clay",
                15 => "Black Stained Clay",
                _ => "White Stained Clay",
            },
            160 => match self.meta & 15 {
                1 => "Orange Stained Glass Pane",
                2 => "Magenta Stained Glass Pane",
                3 => "Light Blue Stained Glass Pane",
                4 => "Yellow Stained Glass Pane",
                5 => "Lime Stained Glass Pane",
                6 => "Pink Stained Glass Pane",
                7 => "Gray Stained Glass Pane",
                8 => "Light Gray Stained Glass Pane",
                9 => "Cyan Stained Glass Pane",
                10 => "Purple Stained Glass Pane",
                11 => "Blue Stained Glass Pane",
                12 => "Brown Stained Glass Pane",
                13 => "Green Stained Glass Pane",
                14 => "Red Stained Glass Pane",
                15 => "Black Stained Glass Pane",
                _ => "White Stained Glass Pane",
            },
            161 => match self.meta & 1 {
                1 => "Dark Oak Leaves",
                _ => "Acacia Leaves",
            },
            162 => match self.meta & 1 {
                1 => "Dark Oak Wood",
                _ => "Acacia Wood",
            },
            163 => "Acacia Wood Stairs",
            164 => "Dark Oak Wood Stairs",
            165 => "Slime Block",
            166 => "Barrier",
            167 => "Iron Trapdoor",
            170 => "Hay Bale",
            171 => match self.meta & 15 {
                1 => "Orange Carpet",
                2 => "Magenta Carpet",
                3 => "Light Blue Carpet",
                4 => "Yellow Carpet",
                5 => "Lime Carpet",
                6 => "Pink Carpet",
                7 => "Gray Carpet",
                8 => "Light Gray Carpet",
                9 => "Cyan Carpet",
                10 => "Purple Carpet",
                11 => "Blue Carpet",
                12 => "Brown Carpet",
                13 => "Green Carpet",
                14 => "Red Carpet",
                15 => "Black Carpet",
                _ => "White Carpet",
            },
            172 => "Hardened Clay",
            173 => "Block of Coal",
            174 => "Packed Ice",
            175 => match self.meta & 7 {
                1 => "Lilac",
                2 => "Double Tallgrass",
                3 => "Large Fern",
                4 => "Rose Bush",
                5 => "Peony",
                _ => "Sunflower",
            },
            _ => "Block",
        }
    }
}

pub fn get_item_name(id: i16) -> &'static str {
    if id > 0 && id <= 255 {
        return Block {
            id: id as u16,
            meta: 0,
            block_light: 0,
            sky_light: 0,
        }
        .name();
    }
    match id {
        256 => "Iron Shovel",
        257 => "Iron Pickaxe",
        258 => "Iron Axe",
        259 => "Flint and Steel",
        260 => "Apple",
        261 => "Bow",
        262 => "Arrow",
        263 => "Coal",
        264 => "Diamond",
        265 => "Iron Ingot",
        266 => "Gold Ingot",
        267 => "Iron Sword",
        268 => "Wooden Sword",
        269 => "Wooden Shovel",
        270 => "Wooden Pickaxe",
        271 => "Wooden Axe",
        272 => "Stone Sword",
        273 => "Stone Shovel",
        274 => "Stone Pickaxe",
        275 => "Stone Axe",
        276 => "Diamond Sword",
        277 => "Diamond Shovel",
        278 => "Diamond Pickaxe",
        279 => "Diamond Axe",
        280 => "Stick",
        281 => "Bowl",
        282 => "Mushroom Stew",
        283 => "Golden Sword",
        284 => "Golden Shovel",
        285 => "Golden Pickaxe",
        286 => "Golden Axe",
        287 => "String",
        288 => "Feather",
        289 => "Gunpowder",
        290 => "Wooden Hoe",
        291 => "Stone Hoe",
        292 => "Iron Hoe",
        293 => "Diamond Hoe",
        294 => "Golden Hoe",
        295 => "Wheat Seeds",
        296 => "Wheat",
        297 => "Bread",
        298 => "Leather Cap",
        299 => "Leather Tunic",
        300 => "Leather Pants",
        301 => "Leather Boots",
        302 => "Chainmail Helmet",
        303 => "Chainmail Chestplate",
        304 => "Chainmail Leggings",
        305 => "Chainmail Boots",
        306 => "Iron Helmet",
        307 => "Iron Chestplate",
        308 => "Iron Leggings",
        309 => "Iron Boots",
        310 => "Diamond Helmet",
        311 => "Diamond Chestplate",
        312 => "Diamond Leggings",
        313 => "Diamond Boots",
        314 => "Golden Helmet",
        315 => "Golden Chestplate",
        316 => "Golden Leggings",
        317 => "Golden Boots",
        318 => "Flint",
        319 => "Raw Porkchop",
        320 => "Cooked Porkchop",
        321 => "Painting",
        322 => "Golden Apple",
        323 => "Sign",
        324 => "Wooden Door",
        325 => "Bucket",
        326 => "Water Bucket",
        327 => "Lava Bucket",
        328 => "Minecart",
        329 => "Saddle",
        330 => "Iron Door",
        331 => "Redstone",
        332 => "Snowball",
        333 => "Boat",
        334 => "Leather",
        335 => "Milk Bucket",
        336 => "Brick",
        337 => "Clay Ball",
        338 => "Sugar Canes",
        339 => "Paper",
        340 => "Book",
        341 => "Slimeball",
        342 => "Minecart with Chest",
        343 => "Minecart with Furnace",
        344 => "Egg",
        345 => "Compass",
        346 => "Fishing Rod",
        347 => "Clock",
        348 => "Glowstone Dust",
        349 => "Raw Fish",
        350 => "Cooked Fish",
        351 => "Dye",
        352 => "Bone",
        353 => "Sugar",
        354 => "Cake",
        355 => "Bed",
        356 => "Redstone Repeater",
        357 => "Cookie",
        358 => "Map",
        359 => "Shears",
        360 => "Melon",
        361 => "Pumpkin Seeds",
        362 => "Melon Seeds",
        363 => "Raw Beef",
        364 => "Steak",
        365 => "Raw Chicken",
        366 => "Cooked Chicken",
        367 => "Rotten Flesh",
        368 => "Ender Pearl",
        369 => "Blaze Rod",
        370 => "Ghast Tear",
        371 => "Gold Nugget",
        372 => "Nether Wart",
        373 => "Potion",
        374 => "Glass Bottle",
        375 => "Spider Eye",
        376 => "Fermented Spider Eye",
        377 => "Blaze Powder",
        378 => "Magma Cream",
        379 => "Brewing Stand",
        380 => "Cauldron",
        381 => "Eye of Ender",
        382 => "Glistering Melon",
        383 => "Spawn Egg",
        384 => "Bottle o' Enchanting",
        385 => "Fire Charge",
        386 => "Book and Quill",
        387 => "Written Book",
        388 => "Emerald",
        389 => "Item Frame",
        390 => "Flower Pot",
        391 => "Carrot",
        392 => "Potato",
        393 => "Baked Potato",
        394 => "Poisonous Potato",
        395 => "Empty Map",
        396 => "Golden Carrot",
        397 => "Mob Head",
        398 => "Carrot on a Stick",
        399 => "Nether Star",
        400 => "Pumpkin Pie",
        401 => "Firework Rocket",
        402 => "Firework Star",
        403 => "Enchanted Book",
        404 => "Redstone Comparator",
        405 => "Nether Brick",
        406 => "Nether Quartz",
        407 => "Minecart with TNT",
        408 => "Minecart with Hopper",
        417 => "Iron Horse Armor",
        418 => "Golden Horse Armor",
        419 => "Diamond Horse Armor",
        420 => "Lead",
        421 => "Name Tag",
        422 => "Minecart with Command Block",
        2256 => "13 Disc",
        2257 => "Cat Disc",
        2258 => "Blocks Disc",
        2259 => "Chirp Disc",
        2260 => "Far Disc",
        2261 => "Mall Disc",
        2262 => "Mellohi Disc",
        2263 => "Stal Disc",
        2264 => "Strad Disc",
        2265 => "Ward Disc",
        2266 => "11 Disc",
        2267 => "Wait Disc",
        _ => "Item",
    }
}

pub fn block_emission(id: u16) -> u8 {
    match id {
        10 | 11 => 15, // Lava
        50 => 14,      // Torch
        51 => 15,      // Fire
        62 => 13,      // Lit Furnace
        76 => 7,       // Redstone Torch
        89 => 15,      // Glowstone
        90 => 11,      // Nether Portal
        91 => 15,      // Jack o'Lantern
        119 => 15,     // End Portal
        124 => 15,     // Redstone Lamp (on)
        138 => 15,     // Beacon
        169 => 15,     // Sea Lantern
        _ => 0,
    }
}

pub fn is_opaque_cube(id: u16) -> bool {
    match id {
        0 | 6 | 8 | 9 | 10 | 11 | 18 | 20 | 26 | 27 | 28 | 29 | 30 | 31 | 32 | 33 | 34 | 36 | 37 | 38
        | 39 | 40 | 44 | 50 | 51 | 52 | 53 | 54 | 55 | 59 | 60 | 63 | 64 | 65 | 66
        | 67 | 68 | 69 | 70 | 71 | 72 | 75 | 76 | 77 | 78 | 79 | 81 | 83 | 85 | 90
        | 92 | 93 | 94 | 95 | 96 | 101 | 102 | 104 | 105 | 106 | 107 | 108 | 109 | 111 | 113 | 114
        | 115 | 116 | 117 | 118 | 119 | 120 | 122 | 126 | 127 | 128 | 130 | 131 | 132 | 134 | 135
        | 136 | 138 | 139 | 140 | 141 | 142 | 143 | 144 | 145 | 146 | 147 | 148 | 149 | 150 | 151
        | 154 | 156 | 157 | 160 | 161 | 163 | 164 | 165 | 167 | 171 | 175 => false,
        _ => true,
    }
}

#[inline]
pub fn get_nibble(buffer: &[u8], index: usize) -> u8 {
    let byte = buffer[index / 2];
    if index % 2 == 0 {
        byte & 0x0F
    } else {
        (byte >> 4) & 0x0F
    }
}

#[inline]
pub fn set_nibble(buffer: &mut [u8], index: usize, val: u8) {
    let byte_idx = index / 2;
    let nibble = val & 0x0F;
    if index % 2 == 0 {
        buffer[byte_idx] = (buffer[byte_idx] & 0xF0) | nibble;
    } else {
        buffer[byte_idx] = (buffer[byte_idx] & 0x0F) | (nibble << 4);
    }
}

fn alloc_boxed_4096() -> Box<[u8; 4096]> {
    vec![0u8; 4096].into_boxed_slice().try_into().unwrap()
}

fn alloc_boxed_2048() -> Box<[u8; 2048]> {
    vec![0u8; 2048].into_boxed_slice().try_into().unwrap()
}

fn alloc_boxed_256() -> Box<[u8; 256]> {
    vec![0u8; 256].into_boxed_slice().try_into().unwrap()
}

#[derive(Clone)]
pub struct ChunkSection {
    pub block_ids: Box<[u8; 4096]>,
    pub block_meta: Box<[u8; 2048]>,
    pub block_light: Box<[u8; 2048]>,
    pub sky_light: Box<[u8; 2048]>,
    pub add_ids: Option<Box<[u8; 2048]>>,
}

impl ChunkSection {
    pub fn new() -> Self {
        Self {
            block_ids: alloc_boxed_4096(),
            block_meta: alloc_boxed_2048(),
            block_light: alloc_boxed_2048(),
            sky_light: alloc_boxed_2048(),
            add_ids: None,
        }
    }

    #[inline]
    pub fn block_index(x: usize, y: usize, z: usize) -> usize {
        (y << 8) | (z << 4) | x
    }

    pub fn get_block(&self, x: usize, y: usize, z: usize) -> Block {
        if x >= 16 || y >= 16 || z >= 16 {
            return Block::AIR;
        }
        let idx = Self::block_index(x, y, z);
        let base_id = self.block_ids[idx] as u16;
        let add_id = match &self.add_ids {
            Some(add) => (get_nibble(add.as_ref(), idx) as u16) << 8,
            None => 0,
        };
        let id = base_id | add_id;
        let meta = get_nibble(self.block_meta.as_ref(), idx);
        let block_light = get_nibble(self.block_light.as_ref(), idx);
        let sky_light = get_nibble(self.sky_light.as_ref(), idx);

        Block {
            id,
            meta,
            block_light,
            sky_light,
        }
    }

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: Block) {
        if x >= 16 || y >= 16 || z >= 16 {
            return;
        }
        let idx = Self::block_index(x, y, z);
        self.block_ids[idx] = (block.id & 0xFF) as u8;
        set_nibble(self.block_meta.as_mut(), idx, block.meta);
        set_nibble(self.block_light.as_mut(), idx, block.block_light);
        set_nibble(self.sky_light.as_mut(), idx, block.sky_light);

        let add_val = ((block.id >> 8) & 0x0F) as u8;
        if add_val != 0 {
            if self.add_ids.is_none() {
                self.add_ids = Some(alloc_boxed_2048());
            }
            if let Some(ref mut add) = self.add_ids {
                set_nibble(add.as_mut(), idx, add_val);
            }
        }
    }
}

#[derive(Clone)]
pub struct ChunkColumn {
    pub x: i32,
    pub z: i32,
    pub sections: [Option<Box<ChunkSection>>; 16],
    pub biomes: Box<[u8; 256]>,
}

#[allow(dead_code)]
impl ChunkColumn {
    pub fn new(x: i32, z: i32) -> Self {
        Self {
            x,
            z,
            sections: Default::default(),
            biomes: alloc_boxed_256(),
        }
    }

    pub fn parse_column_data(
        data: &[u8],
        x: i32,
        z: i32,
        primary_bitmap: u16,
        add_bitmap: u16,
        has_skylight: bool,
        ground_up_continuous: bool,
    ) -> Result<(Self, usize), &'static str> {
        let mut column = ChunkColumn::new(x, z);
        let mut offset = 0;

        let section_count = primary_bitmap.count_ones() as usize;
        let add_count = add_bitmap.count_ones() as usize;

        let blocks_total_size = section_count * 4096;
        let meta_total_size = section_count * 2048;
        let block_light_total_size = section_count * 2048;
        let sky_light_total_size = if has_skylight { section_count * 2048 } else { 0 };
        let add_total_size = add_count * 2048;
        let biome_total_size = if ground_up_continuous { 256 } else { 0 };

        let total_required = blocks_total_size
            + meta_total_size
            + block_light_total_size
            + sky_light_total_size
            + add_total_size
            + biome_total_size;

        if data.len() < total_required {
            return Err("Decompressed buffer too short for chunk column data");
        }

        let mut active_indices = Vec::new();
        for section_y in 0..16 {
            if (primary_bitmap & (1 << section_y)) != 0 {
                let mut section = ChunkSection::new();
                section.block_ids.copy_from_slice(&data[offset..offset + 4096]);
                offset += 4096;
                column.sections[section_y] = Some(Box::new(section));
                active_indices.push(section_y);
            }
        }

        for &section_y in &active_indices {
            if let Some(ref mut section) = column.sections[section_y] {
                section.block_meta.copy_from_slice(&data[offset..offset + 2048]);
                offset += 2048;
            }
        }

        for &section_y in &active_indices {
            if let Some(ref mut section) = column.sections[section_y] {
                section.block_light.copy_from_slice(&data[offset..offset + 2048]);
                offset += 2048;
            }
        }

        if has_skylight {
            for &section_y in &active_indices {
                if let Some(ref mut section) = column.sections[section_y] {
                    section.sky_light.copy_from_slice(&data[offset..offset + 2048]);
                    offset += 2048;
                }
            }
        }

        for section_y in 0..16 {
            if (add_bitmap & (1 << section_y)) != 0 {
                if let Some(ref mut section) = column.sections[section_y] {
                    let mut add_buf = alloc_boxed_2048();
                    add_buf.copy_from_slice(&data[offset..offset + 2048]);
                    section.add_ids = Some(add_buf);
                    offset += 2048;
                } else {
                    offset += 2048;
                }
            }
        }

        if ground_up_continuous {
            column.biomes.copy_from_slice(&data[offset..offset + 256]);
            offset += 256;
        }

        Ok((column, offset))
    }

    pub fn get_block(&self, x: usize, y: usize, z: usize) -> Block {
        if x >= 16 || y >= 256 || z >= 16 {
            return Block::AIR;
        }
        let section_y = y / 16;
        let local_y = y % 16;

        match &self.sections[section_y] {
            Some(section) => section.get_block(x, local_y, z),
            None => Block::AIR,
        }
    }

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: Block) {
        if x >= 16 || y >= 256 || z >= 16 {
            return;
        }
        let section_y = y / 16;
        let local_y = y % 16;

        if self.sections[section_y].is_none() && (!block.is_air() || block.block_light > 0 || block.sky_light > 0) {
            self.sections[section_y] = Some(Box::new(ChunkSection::new()));
        }

        if let Some(ref mut section) = self.sections[section_y] {
            section.set_block(x, local_y, z, block);
        }
    }

    pub fn get_biome(&self, x: usize, z: usize) -> u8 {
        if x >= 16 || z >= 16 {
            return 0;
        }
        self.biomes[z * 16 + x]
    }

    pub fn get_highest_block_y(&self, x: usize, z: usize) -> Option<(usize, Block)> {
        for y in (0..256).rev() {
            let block = self.get_block(x, y, z);
            if !block.is_air() {
                return Some((y, block));
            }
        }
        None
    }
}

#[derive(Default)]
pub struct World {
    chunks: HashMap<(i32, i32), ChunkColumn>,
    pub dirty_chunks: std::collections::HashSet<(i32, i32)>,
    pub enchanting_tables: std::collections::HashSet<(i32, i32, i32)>,
}

#[allow(dead_code)]
impl World {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            dirty_chunks: std::collections::HashSet::new(),
            enchanting_tables: std::collections::HashSet::new(),
        }
    }

    pub fn clear(&mut self) {
        self.chunks.clear();
        self.dirty_chunks.clear();
        self.enchanting_tables.clear();
    }

    pub fn insert_chunk(&mut self, column: ChunkColumn) {
        let (cx, cz) = (column.x, column.z);
        for (sy, sec_opt) in column.sections.iter().enumerate() {
            if let Some(sec) = sec_opt {
                for y in 0..16 {
                    for z in 0..16 {
                        for x in 0..16 {
                            let idx = ChunkSection::block_index(x, y, z);
                            if sec.block_ids[idx] == 116 {
                                self.enchanting_tables.insert((
                                    cx * 16 + x as i32,
                                    sy as i32 * 16 + y as i32,
                                    cz * 16 + z as i32,
                                ));
                            }
                        }
                    }
                }
            }
        }
        self.chunks.insert((cx, cz), column);
        self.mark_dirty_with_neighbors(cx, cz);
    }

    pub fn remove_chunk(&mut self, chunk_x: i32, chunk_z: i32) -> Option<ChunkColumn> {
        self.enchanting_tables.retain(|&(x, _, z)| x.div_euclid(16) != chunk_x || z.div_euclid(16) != chunk_z);
        self.mark_dirty_with_neighbors(chunk_x, chunk_z);
        self.chunks.remove(&(chunk_x, chunk_z))
    }

    pub fn get_chunk(&self, chunk_x: i32, chunk_z: i32) -> Option<&ChunkColumn> {
        self.chunks.get(&(chunk_x, chunk_z))
    }

    pub fn contains_chunk(&self, chunk_x: i32, chunk_z: i32) -> bool {
        self.chunks.contains_key(&(chunk_x, chunk_z))
    }

    pub fn get_chunk_mut(&mut self, chunk_x: i32, chunk_z: i32) -> Option<&mut ChunkColumn> {
        self.chunks.get_mut(&(chunk_x, chunk_z))
    }

    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    pub fn chunks_keys(&self) -> impl Iterator<Item = &(i32, i32)> {
        self.chunks.keys()
    }

    pub fn get_block(&self, world_x: i32, world_y: i32, world_z: i32) -> Block {
        if !(0..=255).contains(&world_y) {
            return Block::AIR;
        }

        let chunk_x = world_x.div_euclid(16);
        let chunk_z = world_z.div_euclid(16);
        let local_x = world_x.rem_euclid(16) as usize;
        let local_z = world_z.rem_euclid(16) as usize;

        match self.get_chunk(chunk_x, chunk_z) {
            Some(chunk) => chunk.get_block(local_x, world_y as usize, local_z),
            None => Block::AIR,
        }
    }

    pub fn set_block_raw(&mut self, world_x: i32, world_y: i32, world_z: i32, block: Block) {
        if !(0..=255).contains(&world_y) {
            return;
        }

        let old_block = self.get_block(world_x, world_y, world_z);
        if old_block.id == 116 {
            self.enchanting_tables.remove(&(world_x, world_y, world_z));
        }
        if block.id == 116 {
            self.enchanting_tables.insert((world_x, world_y, world_z));
        }

        let chunk_x = world_x.div_euclid(16);
        let chunk_z = world_z.div_euclid(16);
        let local_x = world_x.rem_euclid(16) as usize;
        let local_z = world_z.rem_euclid(16) as usize;

        if let Some(chunk) = self.get_chunk_mut(chunk_x, chunk_z) {
            chunk.set_block(local_x, world_y as usize, local_z, block);
            self.mark_dirty_with_neighbors(chunk_x, chunk_z);
        }
    }

    pub fn set_block(&mut self, world_x: i32, world_y: i32, world_z: i32, block: Block) {
        self.set_block_with_lighting(world_x, world_y, world_z, block);
    }

    pub fn set_block_with_lighting(&mut self, world_x: i32, world_y: i32, world_z: i32, mut block: Block) {
        if !(0..=255).contains(&world_y) {
            return;
        }

        let old_block = self.get_block(world_x, world_y, world_z);
        let old_emit = block_emission(old_block.id);
        let new_emit = block_emission(block.id);

        if new_emit > 0 {
            block.block_light = new_emit;
        } else if old_emit > 0 || is_opaque_cube(block.id) {
            block.block_light = 0;
        }

        self.set_block_raw(world_x, world_y, world_z, block);

        if old_emit > 0 && new_emit < old_emit {
            self.remove_block_light(world_x, world_y, world_z, old_emit);
            if new_emit > 0 {
                self.propagate_block_light(world_x, world_y, world_z, new_emit);
            }
        } else if new_emit > 0 {
            self.propagate_block_light(world_x, world_y, world_z, new_emit);
        } else if is_opaque_cube(block.id) && old_block.block_light > 0 {
            self.remove_block_light(world_x, world_y, world_z, old_block.block_light);
        } else if is_opaque_cube(old_block.id) && !is_opaque_cube(block.id) {
            let dirs = [(1, 0, 0), (-1, 0, 0), (0, 1, 0), (0, -1, 0), (0, 0, 1), (0, 0, -1)];
            let mut max_neighbor_light = 0u8;
            for (dx, dy, dz) in dirs {
                let nb = self.get_block(world_x + dx, world_y + dy, world_z + dz);
                if nb.block_light > max_neighbor_light {
                    max_neighbor_light = nb.block_light;
                }
            }
            if max_neighbor_light > 1 {
                self.propagate_block_light(world_x, world_y, world_z, max_neighbor_light - 1);
            }
        }
    }

    pub fn propagate_block_light(&mut self, start_x: i32, start_y: i32, start_z: i32, initial_light: u8) {
        if !(0..=255).contains(&start_y) || initial_light == 0 {
            return;
        }

        let mut start_b = self.get_block(start_x, start_y, start_z);
        if !is_opaque_cube(start_b.id) && start_b.block_light < initial_light {
            start_b.block_light = initial_light;
            self.set_block_raw(start_x, start_y, start_z, start_b);
        }

        let mut queue = std::collections::VecDeque::new();
        queue.push_back((start_x, start_y, start_z, initial_light));

        let dirs = [
            (1, 0, 0), (-1, 0, 0),
            (0, 1, 0), (0, -1, 0),
            (0, 0, 1), (0, 0, -1),
        ];

        while let Some((x, y, z, light)) = queue.pop_front() {
            if light <= 1 {
                continue;
            }
            let next_light = light - 1;

            for (dx, dy, dz) in dirs {
                let nx = x + dx;
                let ny = y + dy;
                let nz = z + dz;

                if !(0..=255).contains(&ny) {
                    continue;
                }

                let chunk_x = nx.div_euclid(16);
                let chunk_z = nz.div_euclid(16);
                if self.get_chunk(chunk_x, chunk_z).is_none() {
                    continue;
                }

                let mut b = self.get_block(nx, ny, nz);
                if is_opaque_cube(b.id) {
                    continue;
                }

                if b.block_light < next_light {
                    b.block_light = next_light;
                    self.set_block_raw(nx, ny, nz, b);
                    queue.push_back((nx, ny, nz, next_light));
                }
            }
        }
    }

    pub fn remove_block_light(&mut self, start_x: i32, start_y: i32, start_z: i32, old_light: u8) {
        if !(0..=255).contains(&start_y) || old_light == 0 {
            return;
        }

        let mut start_b = self.get_block(start_x, start_y, start_z);
        let emit = block_emission(start_b.id);
        if start_b.block_light != emit {
            start_b.block_light = emit;
            self.set_block_raw(start_x, start_y, start_z, start_b);
        }

        let mut remove_queue = std::collections::VecDeque::new();
        let mut propagate_queue = std::collections::VecDeque::new();

        remove_queue.push_back((start_x, start_y, start_z, old_light));

        let dirs = [
            (1, 0, 0), (-1, 0, 0),
            (0, 1, 0), (0, -1, 0),
            (0, 0, 1), (0, 0, -1),
        ];

        while let Some((x, y, z, val)) = remove_queue.pop_front() {
            for (dx, dy, dz) in dirs {
                let nx = x + dx;
                let ny = y + dy;
                let nz = z + dz;

                if !(0..=255).contains(&ny) {
                    continue;
                }

                let chunk_x = nx.div_euclid(16);
                let chunk_z = nz.div_euclid(16);
                if self.get_chunk(chunk_x, chunk_z).is_none() {
                    continue;
                }

                let mut nb = self.get_block(nx, ny, nz);
                if nb.block_light != 0 && nb.block_light < val {
                    remove_queue.push_back((nx, ny, nz, nb.block_light));
                    nb.block_light = 0;
                    self.set_block_raw(nx, ny, nz, nb);
                } else if nb.block_light >= val && nb.block_light > 0 {
                    propagate_queue.push_back((nx, ny, nz, nb.block_light));
                }
            }
        }

        while let Some((x, y, z, light)) = propagate_queue.pop_front() {
            if light <= 1 {
                continue;
            }
            let next_light = light - 1;

            for (dx, dy, dz) in dirs {
                let nx = x + dx;
                let ny = y + dy;
                let nz = z + dz;

                if !(0..=255).contains(&ny) {
                    continue;
                }

                let chunk_x = nx.div_euclid(16);
                let chunk_z = nz.div_euclid(16);
                if self.get_chunk(chunk_x, chunk_z).is_none() {
                    continue;
                }

                let mut b = self.get_block(nx, ny, nz);
                if is_opaque_cube(b.id) {
                    continue;
                }

                if b.block_light < next_light {
                    b.block_light = next_light;
                    self.set_block_raw(nx, ny, nz, b);
                    propagate_queue.push_back((nx, ny, nz, next_light));
                }
            }
        }
    }

    pub fn mark_dirty_with_neighbors(&mut self, chunk_x: i32, chunk_z: i32) {
        self.dirty_chunks.insert((chunk_x, chunk_z));
        self.dirty_chunks.insert((chunk_x + 1, chunk_z));
        self.dirty_chunks.insert((chunk_x - 1, chunk_z));
        self.dirty_chunks.insert((chunk_x, chunk_z + 1));
        self.dirty_chunks.insert((chunk_x, chunk_z - 1));
    }

    pub fn take_dirty_chunks(&mut self) -> std::collections::HashSet<(i32, i32)> {
        std::mem::take(&mut self.dirty_chunks)
    }

    pub fn get_highest_block(&self, world_x: i32, world_z: i32) -> Option<(i32, Block)> {
        let chunk_x = world_x.div_euclid(16);
        let chunk_z = world_z.div_euclid(16);
        let local_x = world_x.rem_euclid(16) as usize;
        let local_z = world_z.rem_euclid(16) as usize;

        let chunk = self.get_chunk(chunk_x, chunk_z)?;
        chunk
            .get_highest_block_y(local_x, local_z)
            .map(|(y, block)| (y as i32, block))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_torch_placement_and_removal() {
        let mut world = World::new();
        world.insert_chunk(ChunkColumn::new(0, 0));

        // Place a torch (id: 50) at (8, 64, 8)
        let torch = Block {
            id: 50,
            meta: 5,
            block_light: 14,
            sky_light: 0,
        };
        world.set_block(8, 64, 8, torch);

        assert_eq!(world.get_block(8, 64, 8).block_light, 14);
        assert_eq!(world.get_block(9, 64, 8).block_light, 13);
        assert_eq!(world.get_block(10, 64, 8).block_light, 12);
        assert_eq!(world.get_block(7, 64, 8).block_light, 13);
        assert_eq!(world.get_block(8, 65, 8).block_light, 13);
        assert_eq!(world.get_block(8, 63, 8).block_light, 13);

        // Remove the torch by setting to AIR
        world.set_block(8, 64, 8, Block::AIR);

        assert_eq!(world.get_block(8, 64, 8).block_light, 0);
        assert_eq!(world.get_block(9, 64, 8).block_light, 0);
        assert_eq!(world.get_block(10, 64, 8).block_light, 0);
        assert_eq!(world.get_block(7, 64, 8).block_light, 0);
        assert_eq!(world.get_block(8, 65, 8).block_light, 0);
        assert_eq!(world.get_block(8, 63, 8).block_light, 0);
    }

    #[test]
    fn test_torch_removal_with_adjacent_torch() {
        let mut world = World::new();
        world.insert_chunk(ChunkColumn::new(0, 0));

        // Place Torch 1 at (5, 64, 5)
        world.set_block(
            5,
            64,
            5,
            Block {
                id: 50,
                meta: 5,
                block_light: 14,
                sky_light: 0,
            },
        );

        // Place Torch 2 at (7, 64, 5)
        world.set_block(
            7,
            64,
            5,
            Block {
                id: 50,
                meta: 5,
                block_light: 14,
                sky_light: 0,
            },
        );

        // Midpoint (6, 64, 5) has light 13
        assert_eq!(world.get_block(6, 64, 5).block_light, 13);

        // Remove Torch 1 at (5, 64, 5)
        world.set_block(5, 64, 5, Block::AIR);

        // Torch 2 at (7, 64, 5) is still 14
        assert_eq!(world.get_block(7, 64, 5).block_light, 14);
        // (6, 64, 5) is still 13 from Torch 2
        assert_eq!(world.get_block(6, 64, 5).block_light, 13);
        // (5, 64, 5) is now illuminated by Torch 2 with light 12
        assert_eq!(world.get_block(5, 64, 5).block_light, 12);
        // (4, 64, 5) is now illuminated with light 11
        assert_eq!(world.get_block(4, 64, 5).block_light, 11);

        // Remove Torch 2 at (7, 64, 5)
        world.set_block(7, 64, 5, Block::AIR);

        // All positions should now be 0
        assert_eq!(world.get_block(7, 64, 5).block_light, 0);
        assert_eq!(world.get_block(6, 64, 5).block_light, 0);
        assert_eq!(world.get_block(5, 64, 5).block_light, 0);
        assert_eq!(world.get_block(4, 64, 5).block_light, 0);
    }

    #[test]
    fn test_packet_0x23_emulation_removes_light() {
        let mut world = World::new();
        world.insert_chunk(ChunkColumn::new(0, 0));

        // Place Torch
        world.set_block(
            8,
            64,
            8,
            Block {
                id: 50,
                meta: 5,
                block_light: 14,
                sky_light: 0,
            },
        );
        assert_eq!(world.get_block(8, 64, 8).block_light, 14);
        assert_eq!(world.get_block(9, 64, 8).block_light, 13);

        // Emulate Packet 0x23 server update replacing torch with air
        let block_id = 0u16;
        let block_meta = 0u8;
        let existing = world.get_block(8, 64, 8);
        world.set_block(
            8,
            64,
            8,
            Block {
                id: block_id,
                meta: block_meta,
                block_light: block_emission(block_id),
                sky_light: if is_opaque_cube(block_id) { 0 } else { existing.sky_light },
            },
        );

        assert_eq!(world.get_block(8, 64, 8).block_light, 0);
        assert_eq!(world.get_block(9, 64, 8).block_light, 0);
        assert_eq!(world.get_block(10, 64, 8).block_light, 0);
    }

    #[test]
    fn test_opaque_block_blocks_light() {
        let mut world = World::new();
        world.insert_chunk(ChunkColumn::new(0, 0));

        // Place Torch at (5, 64, 5)
        world.set_block(
            5,
            64,
            5,
            Block {
                id: 50,
                meta: 5,
                block_light: 14,
                sky_light: 0,
            },
        );
        assert_eq!(world.get_block(6, 64, 5).block_light, 13);
        assert_eq!(world.get_block(7, 64, 5).block_light, 12);

        // Place stone (id: 1) at (6, 64, 5)
        world.set_block(
            6,
            64,
            5,
            Block {
                id: 1,
                meta: 0,
                block_light: 0,
                sky_light: 0,
            },
        );

        assert_eq!(world.get_block(6, 64, 5).block_light, 0);
        // (7, 64, 5) dropped from 12 to 10 because direct path through (6, 64, 5) is blocked,
        // and light must travel 4 Manhattan steps around the stone block (14 - 4 = 10)
        assert_eq!(world.get_block(7, 64, 5).block_light, 10);
        // (5, 64, 5) torch still has light 14
        assert_eq!(world.get_block(5, 64, 5).block_light, 14);
    }
}

