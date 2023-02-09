const FS_MAGIC: u32 = 0xf3fc;

// 超级块，管理磁盘中的所有块
// 磁盘块布局：| super | inode bitmaps | inodes | data bitmaps | data blks |
#[repr(C)]
pub struct SuperBlock {
    magic: u32,               // 超级块验证magic num
    pub inode_bitmap_blocks: u32, // inode bitmap的block数量
    pub inode_blocks: u32,        // inode块数量
    pub data_bitmap_blocks: u32,  // 数据bitmap块数量
    pub data_blocks: u32,         // 数据块数量
}

impl SuperBlock {
    pub fn new(inode_bitmaps: u32, inodes: u32, data_bitmaps: u32, data_blocks: u32) -> Self {
        return Self { magic: FS_MAGIC, inode_bitmap_blocks: inode_bitmaps,
            inode_blocks: inodes, data_bitmap_blocks: data_bitmaps, data_blocks: data_blocks };
    }

    pub fn init(&mut self, inode_bitmaps: u32, inodes: u32, data_bitmaps: u32, data_blocks: u32) {
         self.magic = FS_MAGIC;
         self.inode_bitmap_blocks = inode_bitmaps;
         self.inode_blocks = inodes;
         self.data_bitmap_blocks = data_bitmaps;
         self.data_blocks = data_blocks;
    }
    pub fn is_valid(&self) -> bool {
        return self.magic == FS_MAGIC;
    }
}



