use super::block_device::BlockDevice;
use super::bitmap::{Bitmap, BLOCK_BITS};
use super::block_layout::SuperBlock;
use super::block_cache::{get_block_cache};
use super::inode::{INODES_PER_BLOCK};
use alloc::sync::Arc;
pub struct FileSystem {
    pub block_dev: Arc<dyn BlockDevice>, // 文件系统块设备
    pub inode_bitmap: Bitmap,            // inode分配表
    pub data_bitmap: Bitmap,             // data分配表
    inode_area_start: u32,               // inode区域起始块号
    data_area_start: u32,                // data区域起始块号
}

impl FileSystem {
    pub fn create(block_dev: Arc<dyn BlockDevice>, total_blocks: u32, inode_bitmap_blocks: u32) -> Self {
        // 因为一个block可以存多个inode，所以inode块总数 = bit总数（inode总数） /  一个块中能容纳的inode数
        let inode_blocks = inode_bitmap_blocks * BLOCK_BITS as u32 / INODES_PER_BLOCK;
        // 去除超级块、inode块后剩余的交给数据块和数据bitmap
        let remaining = total_blocks - (inode_blocks + inode_bitmap_blocks + 1);
        // data bitmap块数量 = 剩余块 / （一个bitmap块和若干数据块） 向上取整
        let data_bitmap_blocks = (remaining + BLOCK_BITS as u32 + 1) / (BLOCK_BITS as u32 + 1);
        let data_blocks = data_bitmap_blocks * (BLOCK_BITS as u32);

        // 清空缓存
        for i in 0..total_blocks {
            let cache = get_block_cache(i as usize, Arc::clone(&block_dev));
            let mut block = cache.lock();
            block.cache.fill(0u8);
            block.modified = true;
        }

        // 获取块号为0的超级块，修改后drop，自动写回磁盘
        let block = get_block_cache(0, Arc::clone(&block_dev));
        let mut cache = block.lock();
        let super_block: &mut SuperBlock = cache.get_mut(0);
        super_block.init(inode_bitmap_blocks, inode_blocks, data_bitmap_blocks, data_blocks);
        drop(super_block);
        drop(cache);

        return Self{
            block_dev: block_dev,
            inode_bitmap: Bitmap::new(1, inode_bitmap_blocks),
            data_bitmap: Bitmap::new(1 + inode_bitmap_blocks + inode_blocks, data_bitmap_blocks),
            inode_area_start: 1 + inode_bitmap_blocks,
            data_area_start: 1 + inode_bitmap_blocks + inode_blocks + data_bitmap_blocks,
        };
    }

    // 获取一个inode的全局块号和块内编号
    pub fn get_inode_block_id(&self, inode_id: u32) -> (u32, u32) {
        let inode_block = self.inode_area_start + inode_id / INODES_PER_BLOCK;
        let inner_inode_id = inode_id % INODES_PER_BLOCK;
        return (inode_block, inner_inode_id);
    }
}

