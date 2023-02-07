use alloc::sync::Arc;
use super::block_device::BlockDevice;
use alloc::collections::VecDeque;
use spin::mutex::Mutex;
use lazy_static::lazy_static;

// 一个磁盘块的大小
pub const BLOCK_SIZE: usize = 512;
pub const BLOCK_CACHE_SIZE: usize = 16;

// 一个磁盘块缓存项
#[repr(C)]
pub struct BlockCache {
    pub cache: [u8; BLOCK_SIZE], // 缓存数组
    pub block_id: usize,         // 块ID
    pub modified: bool,          // 是否发生修改
    pub block_device: Arc<dyn BlockDevice> // 块设备引用
}

// 块缓存管理器
pub struct BlockCacheManager {
    caches: VecDeque<(usize, Arc<Mutex<BlockCache>>)>, // 互斥的共享所有权
}

// 懒加载 块缓存管理器 单例，Mutex包装保证互斥访问
lazy_static!{
    pub static ref BLOCK_CACHE_MANAGER: Mutex<BlockCacheManager> = Mutex::new(BlockCacheManager::new());
}

pub fn get_block_cache(block_id: usize, block_device: Arc<dyn BlockDevice>) -> Arc<Mutex<BlockCache>> {
    return BLOCK_CACHE_MANAGER.lock().get_block_cache(block_id, block_device);
}


impl BlockCache {
    // 创建新的缓存块，从块设备读取数据缓存
    pub fn new(block_id: usize, block_device: Arc<dyn BlockDevice>) -> Self {
        let mut buf = [0u8; BLOCK_SIZE];
        block_device.read_block(block_id, &mut buf);
        return Self {
            cache: buf,
            block_id: block_id,
            modified: false,
            block_device: block_device,
        }
    }

    // 将缓存同步到块设备中
    pub fn sync(&mut self) {
        if self.modified  {
            self.block_device.write_block(self.block_id, &self.cache);
            self.modified = false;
        }
    }

    // 块缓存地址
    pub fn addr(&self, offset: usize) -> usize {
        return &self.cache[offset] as *const _ as usize;
    }

    pub fn get_ref<T>(&self, offset: usize) -> &T where T: Sized {
        let size = core::mem::size_of::<T>();
        assert!(offset + size <= BLOCK_SIZE, "offset and size overflow");
        let addr = self.addr(offset);
        let ptr = addr as *const T;
        unsafe {
            return ptr.as_ref().unwrap();
        }
    }

    pub fn get_mut<T>(&mut self, offset: usize) -> &mut T where T: Sized {
        let size = core::mem::size_of::<T>();
        assert!(offset + size <= BLOCK_SIZE, "offset and size overflow");
        let addr = self.addr(offset);
        let ptr = addr as *mut T;
        self.modified = true;
        unsafe {
            return ptr.as_mut().unwrap();
        }
    }
    pub fn read<T, V>(&self, offset: usize, f: impl FnOnce(&T) -> V) -> V {
        f(self.get_ref(offset))
    }

    pub fn modify<T, V>(&mut self, offset:usize, f: impl FnOnce(&mut T) -> V) -> V {
        f(self.get_mut(offset))
    }
}

// 回收BlockCache时自动sync到块设备
impl Drop for BlockCache {
    fn drop(&mut self) {
        self.sync();
    }
}

impl BlockCacheManager {
    pub fn new() -> Self {
        return Self {caches: VecDeque::new()};
    }

    pub fn get_block_cache(&mut self, block_id: usize, block_device: Arc<dyn BlockDevice>) -> Arc<Mutex<BlockCache>> {
        // 从缓存找到block_id对应的块缓存
        if let Some(pair) = self.caches.iter().find(|pair|{pair.0 == block_id}) {
            return Arc::clone(&pair.1);
        }
        // 达到缓存上限，弹出一个块
        if self.caches.len() == BLOCK_CACHE_SIZE {
            // 弹出引用计数为1，即只被manager持有引用的块
            if let Some((idx, _)) = self.caches.iter().enumerate().find(|(_, pair)| {Arc::strong_count(&pair.1) == 1}) {
                self.caches.remove(idx);
            }else {
                // 没有空闲的块，缓存耗尽
                panic!("block cache full, cant push new block cache");
            }
        }
        // 创建缓存块
        let block_cache = Arc::new(Mutex::new(BlockCache::new(block_id, block_device)));
        self.caches.push_back((block_id, Arc::clone(&block_cache)));
        return block_cache;
    }
}