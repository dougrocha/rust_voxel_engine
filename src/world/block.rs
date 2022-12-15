#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum BlockFace {
    FRONT,
    BACK,
    LEFT,
    RIGHT,
    TOP,
    BOTTOM,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockType {
    DEFAULT,
    AIR,
    GRASS,
    DIRT,
    STONE,

    WOOD,
    SAND,

    WATER,
    LAVA,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPosition {
    pub x: i32,
    pub z: i32,
    pub y: i32,
}

impl BlockPosition {
    pub fn new(x: i32, y: i32, z: i32) -> BlockPosition {
        BlockPosition { x, y, z }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Block {
    pub block_type: BlockType,

    pub is_transparent: bool,
}

impl Block {
    pub fn new(block_type: BlockType) -> Block {
        let is_transparent = match block_type {
            BlockType::DEFAULT => false,
            BlockType::AIR => true,
            BlockType::GRASS => false,
            BlockType::DIRT => false,
            BlockType::STONE => false,
            BlockType::WOOD => false,
            BlockType::SAND => false,
            BlockType::WATER => true,
            BlockType::LAVA => true,
        };

        Block {
            block_type,
            is_transparent,
        }
    }

    pub fn is_transparent(&self) -> bool {
        self.is_transparent
    }

    pub fn is_solid(&self) -> bool {
        !self.is_transparent
    }

    pub fn get_block_type(&self) -> BlockType {
        self.block_type
    }

    pub fn set_transparent(&mut self) {
        self.is_transparent = true;
    }
}
