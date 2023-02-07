use super::block_cache::BLOCK_SIZE;
use super::block_device::BlockDevice;
use super::block_cache::get_block_cache;
use alloc::sync::Arc;

// 一个块的bit数量
const BLOCK_BITS: usize = BLOCK_SIZE * 8;

// bitmap块，物理大小BLOCK_SIZE的u64数组
type BitmapBlock = [u64; BLOCK_SIZE / 8];

// Bitmap块集合，只需要记录第一个块的id和块数量
pub struct Bitmap {
    first_block: usize,
    blocks: usize,
}

impl Bitmap {
    pub fn new(first_block: usize, blocks: usize) -> Self {
        return Self {first_block, blocks};
    }
    // 分配一个块，返回block id
    pub fn alloc_block(&self, block_device: Arc<dyn BlockDevice>) -> Option<usize> {
        for block in 0..self.blocks {
            let pos = block + self.first_block;
            let cache = get_block_cache(pos, Arc::clone(&block_device));
            let mut locked = cache.lock();
            let bitmap_block: &mut BitmapBlock = locked.get_mut(0);
            let res = bitmap_block.iter()
            .enumerate()
            .find(|(_, m)| {**m != u64::MAX})       // 找到还有0的u64数字
            .map(|(idx, bits64)| {(idx, (*bits64).trailing_ones())}); // 找到第一个为0的二进制位的位置
            if let Some((idx, inner_pos)) = res {
                // 将该位置设置1
                bitmap_block[idx] = bitmap_block[idx] | (1u64 << inner_pos);
                // 分配的block的顺序序号
                return Some(block * BLOCK_BITS + idx * 64 + inner_pos as usize);
            }
        }
        return None;
    }
    // 回收一个块，参数seq为块的序号，即从bitmap第一个block开始到目标块的序号
    pub fn dealloc(&self, seq: usize, block_device: Arc<dyn BlockDevice>) {
        let (block, idx, u64_offset) = decompose_bits(seq);
        let cache = get_block_cache(block + self.first_block, Arc::clone(&block_device));
        let mut locked = cache.lock();
        locked.modify(0, |bitmap_block: &mut BitmapBlock|{
            // 将二进制位设置为0
            bitmap_block[idx] &= !((1u64<<u64_offset) - 1);
        })
    }
}

// 从bit序号计算block序号, idx, u64 offset
fn decompose_bits(mut bits: usize) -> (usize, usize, usize) {
    let block = bits / BLOCK_BITS;
    bits = bits % BLOCK_SIZE;
    return (block, bits / 64, bits % 64);
}


