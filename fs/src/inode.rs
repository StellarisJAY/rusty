use super::block_cache::{BLOCK_SIZE, get_block_cache, BlockCache};
use super::block_device::BlockDevice;
use alloc::sync::Arc;
use spin::Mutex;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum INodeType {
    File,
    Directory,
}
// 一个inode的大小
pub const INODE_SIZE: u32 = 128;
// 一个block中的inode数量
pub const INODES_PER_BLOCK: u32 = BLOCK_SIZE as u32 / INODE_SIZE;

// 直接索引的块数量
const DIRECT_INDEX_BLOCKS: u32 = 12;
// 可直接索引的文件大小上限 48KiB
const DIRECT_SIZE_LIMIT: u32 = DIRECT_INDEX_BLOCKS * BLOCK_SIZE as u32;
// 一级索引的块数量
const INDIRECT1_BLOCK_LIMIT: u32 = BLOCK_SIZE as u32 / 4;
// 可一级索引的文件大小上限
const INDIRECT1_SIZE_LIMIT: u32 = DIRECT_SIZE_LIMIT + BLOCK_SIZE as u32 * INDIRECT1_BLOCK_LIMIT;

// inode，大小对齐128字节
#[repr(align(128))]
pub struct DiskINode {
    pub size: u32,                           // 单个文件大小不超过4GiB
    pub indexes: [u32; DIRECT_INDEX_BLOCKS as usize], // 12个直接指针，直接指向数据块，最多48KiB
    pub indirect1: u32,     // 一级间接索引，指向一个全索引块，全索引块的4KiB全部记录数据块指针，共1024个指针，索引1024*4KiB = 4MiB数据
    pub indirect2: u32,     // 二级间接索引，指向一个二级全索引块，共1024个指针指向一级索引，所以共1024 * 1024 * 4KiB = 4GiB数据
    pub _type: INodeType,
}

impl DiskINode {
    pub fn init(&mut self, _type: INodeType) {
        self._type = _type;
        self.indexes = [0u32; DIRECT_INDEX_BLOCKS as usize];
        self.size = 0;
        self.indirect1 = 0;
        self.indirect2 = 0;
    }
    // 文件占用的数据块总数，由文件大小对数据块大小向上取整获得
    pub fn data_blocks(&self) -> u32 {
        return Self::data_blocks_for_size(self.size);
    }

    pub fn data_blocks_for_size(size: u32) -> u32 {
        // 向上取整
        return (size + BLOCK_SIZE as u32 - 1) / BLOCK_SIZE as u32;
    }

    pub fn index_blocks_for_size(size: u32) -> u32 {
        let mut data_blocks = (size + BLOCK_SIZE as u32 - 1) / BLOCK_SIZE as u32;
        let mut total = 0;
        // 大小在直接索引范围内
        if data_blocks <= DIRECT_INDEX_BLOCKS {
            return total;
        }
        data_blocks -= DIRECT_INDEX_BLOCKS ;
        // 加上一个一级索引块
        total += 1;
        // 大小在一级索引范围内
        if data_blocks <= INDIRECT1_BLOCK_LIMIT {
            return total;
        }else {
            data_blocks -= INDIRECT1_BLOCK_LIMIT;
            // 一个二级索引块和若干个一级索引块
            total += data_blocks / INDIRECT1_BLOCK_LIMIT + 1;
            // 有余数，需要额外分配一个一级索引块
            if data_blocks % INDIRECT1_BLOCK_LIMIT == 0 {
                total += 1;
            }
            return total;
        }
    }

    // 文件占用的磁盘块总数 = inode + 索引块 + 数据块
    pub fn total_blocks(&self) -> u32 {
        return self.data_blocks() + Self::index_blocks_for_size(self.size) + 1;
    }

    // 根据块顺序获取第seq个块的磁盘块id
    pub fn get_block_id(&self, seq: u32, block_dev: Arc<dyn BlockDevice>) -> u32 {
        assert!(self.data_blocks() > seq);
        let mut blocks = seq + 1;
        if blocks <= DIRECT_INDEX_BLOCKS {
            return self.indexes[blocks as usize - 1];
        }
        // 减去直接索引的节点数量
        blocks -= DIRECT_INDEX_BLOCKS;
        if blocks <= INDIRECT1_BLOCK_LIMIT {
            // 找到一级索引节点
            let indirect1 = get_block_cache(self.indirect1 as usize, Arc::clone(&block_dev));
            let cache1 = indirect1.lock();
            // 索引节点转换成u32数组，从数组获取对应序号的id
            let id = cache1.u32_array()[blocks as usize - 1];
            drop(cache1);
            drop(indirect1);
            return id;
        }else {
            // 减去一级的block数量
            blocks -= INDIRECT1_BLOCK_LIMIT;
            // 二级索引节点
            let indirect2 = get_block_cache(self.indirect2 as usize, Arc::clone(&block_dev));
            let cache2 = indirect2.lock();
            // 序号在二级数组的位置获得一级索引块id
            let id = cache2.u32_array()[((blocks - 1)/INDIRECT1_BLOCK_LIMIT) as usize];
            drop(cache2);
            drop(indirect2);
            let indirect1 = get_block_cache(id as usize, Arc::clone(&block_dev));
            let cache1 = indirect1.lock();
            let id = cache1.u32_array()[((blocks-1) % INDIRECT1_BLOCK_LIMIT) as usize];
            drop(cache1);
            drop(indirect1);
            return id;
        }
    }

    // 获取文件中偏移位置offset所对应的磁盘块编号
    pub fn get_block_from_offset(&self, offset: u32, block_dev: Arc<dyn BlockDevice>) -> u32 {
        // 计算offset在第几个块中
        let block_seq = offset / BLOCK_SIZE as u32;
        // 获取该块序号的块id
        return self.get_block_id(block_seq, Arc::clone(&block_dev));
    }
    // 获取偏移位置的块缓存
    pub fn get_block_cache_from_offset(&self, offset: u32, block_dev: Arc<dyn BlockDevice>) -> Arc<Mutex<BlockCache>> {
        let block_id = self.get_block_from_offset(offset, Arc::clone(&block_dev));
        return get_block_cache(block_id as usize, Arc::clone(&block_dev));
    }

    // 从offset读取文件数据到buf中
    pub fn read(&self, offset: u32, buf: &mut [u8], block_dev: Arc<dyn BlockDevice>) {
        let mut len = buf.len() as u32;
        assert!(self.size > offset && self.size >= offset + len);
        // 读取结束位置的偏移量
        let end_off = offset + len - 1;
        // 结束位置所在的块序号
        let end_block_seq = end_off / BLOCK_SIZE as u32;
        // 初始的块内偏移
        let mut inner_start = offset % BLOCK_SIZE as u32;
        // 初始的块内结束位置
        let mut inner_end = BLOCK_SIZE;
        // 顺序读取的最后一个块序号
        let mut current_block_seq = offset / BLOCK_SIZE as u32;
        // buf数组写入位置
        let mut idx = 0;
        loop {
            // 通过inode索引获取块id
            // 读取文件的io瓶颈，尽量顺序读来减少索引块的IO
            let block_id = self.get_block_id(current_block_seq, Arc::clone(&block_dev));
            let block = get_block_cache(block_id as usize, Arc::clone(&block_dev));
            let cache = block.lock();
            // 最后一个block
            if current_block_seq == end_block_seq {
                inner_end = end_off as usize % BLOCK_SIZE;
                // start到end的数据写入buf
                buf[idx..].copy_from_slice(&cache.cache[inner_start as usize..=inner_end]);
                break;
            }else {
                buf[idx..].copy_from_slice(&cache.cache[inner_start as usize..=inner_end]);
                // 本次读取的长度
                let read_len = inner_end - inner_start as usize + 1;
                // 已读取的长度 = 块大小 - 块内偏移
                idx += read_len;
                // 下一个block
                current_block_seq += 1;
                inner_start = 0;
            }
        }
    }

    pub fn write(&mut self, offset: u32, buf: &[u8], block_dev: Arc<dyn BlockDevice>) {
        //todo write at offset
    }

    // 向文件添加数据块来增大文件大小
    // index_blocks为预先计算出来需要的一级和二级索引块
    pub fn increse_size(&mut self, new_size: u32, new_blocks: Vec<u32>, mut index_blocks: Vec<u32>, block_dev: Arc<dyn BlockDevice>) {
        self.size = new_size;
        let mut current_blocks = self.total_blocks() + 1;

        for new_block in new_blocks {
            // 可以直接索引
            if current_blocks <= DIRECT_INDEX_BLOCKS {
                self.indexes[current_blocks as usize - 1] = new_block;
            }else if current_blocks <= INDIRECT1_BLOCK_LIMIT + DIRECT_INDEX_BLOCKS { // 一级索引
                let indirect1_seq = current_blocks - DIRECT_INDEX_BLOCKS - 1;
                // 一级索引不存在，通过参数的idx_blocks传入
                if self.indirect1 == 0 {
                    self.indirect1 = index_blocks.pop().unwrap();
                }
                // 在一级索引块中修改第n个block_id
                get_block_cache(self.indirect1 as usize, Arc::clone(&block_dev))
                .lock()
                .modify(indirect1_seq as usize * 4, |block_id: &mut u32| {
                    *block_id = new_block;
                })
            }else {
                // 二级索引不存在，通过参数创建
                if self.indirect2 == 0 {
                    self.indirect2 = index_blocks.pop().unwrap();
                }
                // 在二级索引中的序号
                let indirect2_seq = current_blocks - INDIRECT1_BLOCK_LIMIT - DIRECT_INDEX_BLOCKS - 1;
                get_block_cache(self.indirect2 as usize, Arc::clone(&block_dev))
                .lock()
                .modify(self.indirect2 as usize, |block_ids: &mut [u32;INDIRECT1_BLOCK_LIMIT as usize]| {
                    let sec_level_seq = indirect2_seq / INDIRECT1_BLOCK_LIMIT;
                    let first_level_seq = indirect2_seq % INDIRECT1_BLOCK_LIMIT;
                    // 二级索引中没有该一级索引编号，分配新的块号
                    if block_ids[sec_level_seq as usize] == 0 {
                        block_ids[sec_level_seq as usize] = index_blocks.pop().unwrap();
                    }
                    // 找到一级索引块，修改对应序号的block id
                    get_block_cache(block_ids[sec_level_seq as usize] as usize, Arc::clone(&block_dev))
                    .lock()
                    .modify(first_level_seq as usize, |block_id: &mut u32| {
                        *block_id = new_block;
                    })
                })
            }
            current_blocks += 1;
        }
    }
}

