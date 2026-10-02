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
            3 => "Dirt",
            4 => "Cobblestone",
            5 => "Oak Wood Planks",
            6 => "Sapling",
            7 => "Bedrock",
            8 => "Flowing Water",
            9 => "Still Water",
            10 => "Flowing Lava",
            11 => "Still Lava",
            12 => "Sand",
            13 => "Gravel",
            14 => "Gold Ore",
            15 => "Iron Ore",
            16 => "Coal Ore",
            17 => "Wood Log",
            18 => "Leaves",
            19 => "Sponge",
            20 => "Glass",
            21 => "Lapis Lazuli Ore",
            22 => "Lapis Lazuli Block",
            24 => "Sandstone",
            31 => "Tall Grass",
            35 => "Wool",
            37 => "Dandelion",
            38 => "Poppy",
            49 => "Obsidian",
            50 => "Torch",
            56 => "Diamond Ore",
            73 => "Redstone Ore",
            78 => "Snow Layer",
            79 => "Ice",
            80 => "Snow Block",
            81 => "Cactus",
            82 => "Clay",
            86 => "Pumpkin",
            111 => "Lily Pad",
            _ => "Block",
        }
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

        if self.sections[section_y].is_none() && !block.is_air() {
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
}

#[allow(dead_code)]
impl World {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            dirty_chunks: std::collections::HashSet::new(),
        }
    }

    pub fn insert_chunk(&mut self, column: ChunkColumn) {
        let (cx, cz) = (column.x, column.z);
        self.chunks.insert((cx, cz), column);
        self.mark_dirty_with_neighbors(cx, cz);
    }

    pub fn remove_chunk(&mut self, chunk_x: i32, chunk_z: i32) -> Option<ChunkColumn> {
        self.mark_dirty_with_neighbors(chunk_x, chunk_z);
        self.chunks.remove(&(chunk_x, chunk_z))
    }

    pub fn get_chunk(&self, chunk_x: i32, chunk_z: i32) -> Option<&ChunkColumn> {
        self.chunks.get(&(chunk_x, chunk_z))
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

    pub fn set_block(&mut self, world_x: i32, world_y: i32, world_z: i32, block: Block) {
        if !(0..=255).contains(&world_y) {
            return;
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

