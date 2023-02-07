use super::block_cache::{BLOCK_SIZE, get_block_cache};
use super::block_device::BlockDevice;
use alloc::sync::Arc;
use spin::Mutex;

const FS_MAGIC: u32 = 0xf3fc;

// 超级块，管理磁盘中的所有块
// 磁盘块布局：| super | inode bitmaps | inodes | data bitmaps | data blks |
#[repr(C)]
pub struct SuperBlock {
    magic: u32,               // 超级块验证magic num
    inode_bitmap_blocks: u32, // inode bitmap的block数量
    inode_blocks: u32,        // inode块数量
    data_bitmap_blocks: u32,  // 数据bitmap块数量
    data_blocks: u32,         // 数据块数量
}

impl SuperBlock {
    pub fn new(inode_bitmaps: u32, inodes: u32, data_bitmaps: u32, data_blocks: u32) -> Self {
        return Self { magic: FS_MAGIC, inode_bitmap_blocks: inode_bitmaps,
            inode_blocks: inodes, data_bitmap_blocks: data_bitmaps, data_blocks: data_blocks };
    }
    pub fn is_valid(&self) -> bool {
        return self.magic == FS_MAGIC;
    }
}

// 一个inode可以直接索引的数据块数量
const INODE_DIRECT_LIMIT: usize = 28;
const INODE_INDIRECT1_LIMIT: usize = 128;
const DIRECT_BOUND: usize = 32 * 1024;
const INDIRECT1_BOUND: usize = 96 * 1024;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum INodeType {
    File,
    Directory,
}

// 磁盘inode块
#[repr(C)]
pub struct DiskINode {
    pub size: u32,
    pub direct: [u32; INODE_DIRECT_LIMIT], // 直接索引，直接通过block_id索引的数据块，最多64个块，共64*512 = 32KiB
    pub indirect1: u32,                    // 一级索引，文件超过32KiB，一级索引块中所有的u32用来记录数据块，共512/4=128块，最大128 * 512 = 64KiB
    pub indirect2: u32,                    // 二级索引，二级索引中的每个u32指向一个一级索引，最大128 * 64KiB = 8MiB
    pub _type: INodeType,
}

impl DiskINode {
    pub fn init(&mut self, _type: INodeType) {
        self.direct = [0u32; INODE_DIRECT_LIMIT];
        self.size = 0;
        self.indirect1 = 0;
        self.indirect2 = 0;
        self._type = _type;
    }
    pub fn is_directory(&self) -> bool {
        return self._type == INodeType::Directory;
    }
    pub fn is_file(&self) -> bool {
        return self._type == INodeType::File;
    }

    // 获取一个文件中的pos位置所属的磁盘块编号
    pub fn get_block_id(&self, pos: u32, block_device: Arc<dyn BlockDevice>) -> u32 {
        let mut inner = pos as usize / BLOCK_SIZE;
        let pos = pos as usize;
        if pos < DIRECT_BOUND {
            return self.direct[inner];
        }
        // 超过了直接索引文件的上限，
        if pos < INDIRECT1_BOUND{
            // 从一级索引块获取u32数组
            let indirect1_block = get_block_cache(self.indirect1 as usize, Arc::clone(&block_device));
            let locked = indirect1_block.lock();
            let blocks: &[u32;BLOCK_SIZE/4] = locked.get_ref(0);
            return blocks[inner - INODE_DIRECT_LIMIT];
        }else {
            // 减去直接索引的块，剩下的从二级索引获取
            inner -= INODE_DIRECT_LIMIT;
            // 从二级索引找到u32数组，取u32作为一级索引块id，找到一级索引块的u32数组
            let indirect2_block = get_block_cache(self.indirect2 as usize, Arc::clone(&block_device));
            let locked = indirect2_block.lock();
            let i2_blocks: &[u32; BLOCK_SIZE/4] = locked.get_ref(0);
            let indirect1 = i2_blocks[inner / 128];
            drop(locked);
            // 找到二级索引中的一级编号对应的一级索引块，从一级索引块获取u32
            let indirect1_block = get_block_cache(indirect1 as usize, Arc::clone(&block_device));
            let locked = indirect1_block.lock();
            let i1_blocks: &[u32;BLOCK_SIZE/4] = locked.get_ref(0);
            return i1_blocks[inner % 128];
        }
    }
}







