use super::block_device::BlockDevice;
use super::fs::FileSystem;
use super::inode::{DiskINode, INodeType};
use super::block_cache::get_block_cache;
use super::dir::{DIR_SIZE, DirEntry};
use spin::Mutex;
use alloc::sync::Arc;
use alloc::string::String;
use alloc::vec::Vec;

// 内存记录的INode信息
pub struct INode {
    pub block_id: u32,              // inode所在的块id
    pub block_offset: u32,          // inode在块内的偏移
    fs: Arc<Mutex<FileSystem>>,     // 文件系统引用
    block_dev: Arc<dyn BlockDevice>,// 块设备引用
}

impl INode {
    pub fn new(block_id: u32, block_offset: u32, fs: Arc<Mutex<FileSystem>>, block_dev: Arc<dyn BlockDevice>) -> Self {
        return Self {
            block_id,
            block_offset,
            fs,
            block_dev,
        };
    }
    // 读取磁盘inode并进行互斥操作
    pub fn read_disk_inode<V>(&self, f: impl FnOnce(&DiskINode)->V) -> V {
        get_block_cache(self.block_id as usize, Arc::clone(&self.block_dev))
        .lock()
        .read(self.block_offset as usize, |inode: &DiskINode| {
            f(inode)
        })
    }

    // 修改磁盘inode的互斥操作
    pub fn modify_disk_inode<V>(&mut self, f: impl FnOnce(&mut DiskINode)->V) -> V {
        get_block_cache(self.block_id as usize, Arc::clone(&self.block_dev))
        .lock()
        .modify(self.block_offset as usize, |inode: &mut DiskINode| {
            f(inode)
        })
    }

    // 在当前目录inode中寻找文件名为name的文件inode
    pub fn find(&self, name: &str) -> Option<INode> {
        return self.read_disk_inode(|disk_inode|{
            self.find_file_inode(name, disk_inode)
            .map(|id| {
                // 从文件系统找到inode id对应的inode块
                let fs = self.fs.lock();
                let (block_id, block_offset) = fs.get_inode_block_id(id);
                return INode::new(block_id, block_offset, Arc::clone(&self.fs), Arc::clone(&self.block_dev));
            })
        });
    }

    // 找到以当前inode为目录下的文件的inode id
    fn find_file_inode(&self, name: &str, disk_inode: &DiskINode) -> Option<u32> {
        // 该目录下的文件总数
        let file_count = disk_inode.size / DIR_SIZE;
        for i in 0..file_count {
            let mut dir = DirEntry::empty();
            // 读取目录inode的目录项的文件名
            disk_inode.read(i * DIR_SIZE, dir.to_bytes_mut(), Arc::clone(&self.block_dev));
            if dir.name() == name {
                return Some((&dir).inode_id());
            }
        }
        return None;
    }

    // 列举当前inode目录下的所有文件名
    pub fn ls(&self) -> Vec<String> {
        let mut files = Vec::new();
        // 对disk inode互斥只读操作
        self.read_disk_inode(|disk_inode| {
            let file_count = disk_inode.size / DIR_SIZE;
            for i in 0..file_count {
                let mut dir_entry = DirEntry::empty();
                // 将磁盘缓存数据读取到dir entry
                disk_inode.read(DIR_SIZE * i, dir_entry.to_bytes_mut(), Arc::clone(&self.block_dev));
                files.push(String::from(dir_entry.name()));
            }
        });
        return files;
    }
}

