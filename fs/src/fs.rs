use super::block_device::BlockDevice;
use super::bitmap::{Bitmap, BLOCK_BITS};
use super::block_layout::SuperBlock;
use super::block_cache::{get_block_cache, BLOCK_SIZE};
use super::inode::{INODES_PER_BLOCK, DiskINode, INodeType::Directory, INODE_SIZE};
use super::vfs::INode;
use alloc::sync::Arc;
use spin::Mutex;

pub struct FileSystem {
    pub block_dev: Arc<dyn BlockDevice>, // 文件系统块设备
    pub inode_bitmap: Bitmap,            // inode分配表
    pub data_bitmap: Bitmap,             // data分配表
    inode_area_start: u32,               // inode区域起始块号
    data_area_start: u32,                // data区域起始块号
}

impl FileSystem {
    // 在块设备上创建文件系统
    pub fn create(block_dev: Arc<dyn BlockDevice>, total_blocks: u32, inode_bitmap_blocks: u32) -> Self {
        // 因为一个block可以存多个inode，所以inode块总数 = bit总数（inode总数） /  一个块中能容纳的inode数
        let inode_blocks = inode_bitmap_blocks * BLOCK_BITS as u32 / INODES_PER_BLOCK;
        // 去除超级块、inode块后剩余的交给数据块和数据bitmap
        let remaining = total_blocks - (inode_blocks + inode_bitmap_blocks + 1);
        // data bitmap块数量 = 剩余块 / （一个bitmap块和若干数据块） 向上取整
        let data_bitmap_blocks = (remaining + BLOCK_BITS as u32 + 1) / (BLOCK_BITS as u32 + 1);
        let data_blocks = data_bitmap_blocks * (BLOCK_BITS as u32);

        // 清空缓存
        for i in 0..(total_blocks-1) {
            get_block_cache(i as usize, Arc::clone(&block_dev))
            .lock()
            .modify(0, |cache: &mut [u8;BLOCK_SIZE as usize]| {
                cache.fill(0);
            });
        }
        // 初始化 超级块
        get_block_cache(0, Arc::clone(&block_dev))
        .lock()
        .modify(0, |super_block: &mut SuperBlock| {
            super_block.init(inode_bitmap_blocks, inode_blocks, data_bitmap_blocks, data_blocks);
        });

        return Self{
            block_dev: block_dev,
            inode_bitmap: Bitmap::new(1, inode_bitmap_blocks),
            data_bitmap: Bitmap::new(1 + inode_bitmap_blocks + inode_blocks, data_bitmap_blocks),
            inode_area_start: 1 + inode_bitmap_blocks,
            data_area_start: 1 + inode_bitmap_blocks + inode_blocks + data_bitmap_blocks,
        };
    }

    // 从块设备上打开一个文件系统
    pub fn open(block_dev: Arc<dyn BlockDevice>) -> Arc<Mutex<Self>> {
        // 读取超级块，闭包处理后返回文件系统实例
        return get_block_cache(0, Arc::clone(&block_dev))
        .lock()
        .read(0, |super_block: &SuperBlock| {
            if super_block.is_valid() {
                // 根据超级块的信息，获取文件系统数据块、inode块位置
                let inode_blocks = super_block.inode_blocks;
                let inode_bitmap_blocks = super_block.inode_bitmap_blocks;
                // 获取bitmap区域
                let inode_bitmap = Bitmap::new(1, inode_bitmap_blocks);
                let data_bitmap = Bitmap::new(1 + inode_bitmap_blocks + inode_blocks, super_block.data_bitmap_blocks);
                let fs =  Self {
                    block_dev: block_dev,
                    inode_bitmap: inode_bitmap,
                    inode_area_start: 1 + inode_bitmap_blocks,
                    data_bitmap: data_bitmap,
                    data_area_start: 1 + inode_bitmap_blocks + inode_blocks + super_block.data_bitmap_blocks,
                };
                return Arc::new(Mutex::new(fs));
            }else {
                panic!("invalid file system super block");
            }
        });
    }

    // 获取一个inode的全局块号、块内编号 和 块内偏移
    pub fn get_inode_block_id(&self, inode_id: u32) -> (u32, u32, u32) {
        let inode_block = self.inode_area_start + inode_id / INODES_PER_BLOCK;
        let inner_inode_id = inode_id % INODES_PER_BLOCK;
        return (inode_block, inner_inode_id, inner_inode_id * INODE_SIZE);
    }

    // 获取一个数据块的全局块号
    pub fn get_data_block_id(&self, data_id: u32) -> u32 {
        return self.data_area_start + data_id;
    }
    // 从inode bitmap分配一个inode块，返回inode区域局部块号
    pub fn alloc_inode(&mut self) -> u32 {
        return self.inode_bitmap.alloc_block(Arc::clone(&self.block_dev)).unwrap();
    }

    // 分配data块，获取全局块号
    pub fn alloc_data_block(&mut self) -> u32 {
        return self.data_bitmap.alloc_block(Arc::clone(&self.block_dev)).unwrap() + self.data_area_start;
    }

    // 回收一个data块
    pub fn dealloc_data_block(&mut self, block_id: u32) {
        let block_cache = get_block_cache(block_id as usize, Arc::clone(&self.block_dev));
        let mut locked_cache = block_cache.lock();
        // 清空缓存数据
        locked_cache.clear();
        // bitmap回收data_block
        self.data_bitmap.dealloc(block_id - self.data_area_start, Arc::clone(&self.block_dev));
    }

    // 创建root目录inode
    pub fn create_root_inode(&mut self) -> u32 {
        let inode_seq = self.alloc_inode();
        let (block_id, _, block_off) = self.get_inode_block_id(inode_seq);
        get_block_cache(block_id as usize, Arc::clone(&self.block_dev))
        .lock()
        .modify(block_off as usize, |disk_inode: &mut DiskINode| {
            disk_inode._type = Directory;
        });
        return inode_seq;
    }

    // 根inode节点，inode编号为0
    pub fn root_inode(fs: Arc<Mutex<Self>>) -> INode {
        let fs_locked = fs.lock();
        let (block_id, _, inode_offset) = fs_locked.get_inode_block_id(0);
        return INode::new(block_id, inode_offset, Arc::clone(&fs), Arc::clone(&fs_locked.block_dev));
    }
}

