use core::any::Any;
// 块设备trait
pub trait BlockDevice: Send + Sync + Any {
    // 读取块到buffer中
    fn read_block(&self, block_id: usize, buf: &mut [u8]);
    // 写入一个块
    fn write_block(&self, block_id: usize, buf: &[u8]);
}