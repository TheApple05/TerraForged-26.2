// MIT ReTerraForged Levels.java and Size.java, baseline
// 29239f1f164d19dbfcccaca4a8277d64e784207c. Descriptors never allocate a tile.
#[derive(Clone, Copy, Debug)]
pub struct Levels {
    pub world_height: i32,
    pub unit: f32,
    pub water_y: i32,
    ground_y: i32,
    pub ground_level: i32,
    pub water_level: i32,
    pub ground: f32,
    pub water: f32,
    elevation_range: f32,
}

impl Levels {
    pub fn new(height: i32, sea_level: i32) -> Self {
        let world_height = if height > 1 { height } else { 1 };
        let unit = 1i32 as f32 / world_height as f32;
        let water_level = sea_level;
        let ground_level = water_level.wrapping_add(1);
        let water_y = if water_level.wrapping_sub(1) < world_height {
            water_level.wrapping_sub(1)
        } else {
            world_height
        };
        let ground_y = if ground_level.wrapping_sub(1) < world_height {
            ground_level.wrapping_sub(1)
        } else {
            world_height
        };
        let ground = ground_y as f32 / world_height as f32;
        let water = water_y as f32 / world_height as f32;
        let elevation_range = 1.0f32 - water;
        Self {
            world_height,
            unit,
            water_y,
            ground_y,
            ground_level,
            water_level,
            ground,
            water,
            elevation_range,
        }
    }

    pub fn scale_f32(&self, value: f32) -> i32 {
        (value * self.world_height as f32) as i32
    }

    pub fn elevation_f32(&self, value: f32) -> f32 {
        if value <= self.water {
            return 0.0f32;
        }
        (value - self.water) / self.elevation_range
    }

    pub fn elevation_i32(&self, y: i32) -> f32 {
        if y <= self.water_y {
            return 0.0f32;
        }
        self.scale_i32(y.wrapping_sub(self.water_y)) / self.elevation_range
    }

    pub fn scale_i32(&self, level: i32) -> f32 {
        level as f32 / self.world_height as f32
    }

    pub fn water(&self, amount: i32) -> f32 {
        self.water_y.wrapping_add(amount) as f32 / self.world_height as f32
    }

    pub fn ground(&self, amount: i32) -> f32 {
        self.ground_y.wrapping_add(amount) as f32 / self.world_height as f32
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Size {
    pub size: i32,
    pub mask: i32,
    pub border: i32,
    pub total: i32,
    pub lower_border: i32,
    pub upper_border: i32,
    pub array_size: i32,
}

impl Size {
    pub fn mask_index(&self, i: i32) -> i32 {
        i & self.mask
    }

    pub fn index_of(&self, x: i32, z: i32) -> i32 {
        z.wrapping_mul(self.total).wrapping_add(x)
    }

    pub fn chunk_to_block(i: i32) -> i32 {
        i.wrapping_shl(4)
    }

    pub fn block_to_chunk(i: i32) -> i32 {
        i >> 4
    }

    pub fn count(min_x: i32, min_z: i32, max_x: i32, max_z: i32) -> i32 {
        let dx = max_x.wrapping_sub(min_x);
        let dz = max_z.wrapping_sub(min_z);
        dx.wrapping_mul(dz)
    }

    pub fn make(size: i32, border: i32) -> Self {
        let total = size.wrapping_add(2i32.wrapping_mul(border));
        Self {
            size,
            mask: size.wrapping_sub(1),
            border,
            total,
            lower_border: border,
            upper_border: border.wrapping_add(size),
            array_size: total.wrapping_mul(total),
        }
    }

    pub fn chunks(factor: i32, border_chunks: i32) -> Self {
        let chunks = 1i32.wrapping_shl((factor & 31) as u32);
        Self::make(chunks, border_chunks)
    }

    pub fn blocks(factor: i32, border_chunks: i32) -> Self {
        let chunks = 1i32.wrapping_shl((factor & 31) as u32);
        let blocks = chunks.wrapping_shl(4);
        let border_blocks = border_chunks.wrapping_shl(4);
        Self::make(blocks, border_blocks)
    }
}
