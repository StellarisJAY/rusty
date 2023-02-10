use super::block_device::BlockDevice;
use super::fs::FileSystem;
use super::inode::{DiskINode, INodeType};
use super::block_cache::get_block_cache;
use super::dir::{DIR_SIZE, DirEntry};
use spin::{Mutex, MutexGuard};
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
    pub fn modify_disk_inode<V>(&self, f: impl FnOnce(&mut DiskINode)->V) -> V {
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
                let (block_id, _, block_offset) = fs.get_inode_block_id(id);
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
            assert!(disk_inode._type == INodeType::Directory);
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

    // 在当前目录下创建文件
    pub fn create(&mut self, name: &str) -> Option<Arc<INode>> {
        let (is_dir, file_exist) = self.read_disk_inode(|disk_inode| {
            if disk_inode.is_dir() {
                return (true, self.find_file_inode(name, disk_inode).is_some());
            }
            return (false, false);
        });
        assert!(is_dir);
        if file_exist {
            return None;
        }
        let mut fs = self.fs.lock();
        let inode_seq = fs.alloc_inode();

        let (block_id, _, block_offset) = fs.get_inode_block_id(inode_seq);
        // 初始化新文件的磁盘inode
        get_block_cache(block_id as usize, Arc::clone(&self.block_dev))
        .lock()
        .modify(block_offset as usize, |disk_inode: &mut DiskINode| {
            disk_inode.init(INodeType::File);
        });
        // 在当前目录inode中添加新文件的目录项
        self.modify_disk_inode(|dir_inode| {
            // 计算新目录项的偏移
            let offset = dir_inode.size;
            // 目录inode块扩容
            self.increase_size(dir_inode.size + DIR_SIZE, dir_inode, &mut fs);
            // 写入目录entry
            let dir_entry = DirEntry::new(name, inode_seq);
            dir_inode.write(offset, dir_entry.to_bytes(), Arc::clone(&self.block_dev));
        });
        let inode = Self::new(block_id, block_offset, Arc::clone(&self.fs), Arc::clone(&self.block_dev));
        return Some(Arc::new(inode));
    }

    // 从inode读取文件
    pub fn read_at(&self, offset: u32, buf: &mut [u8]) {
        // 互斥读
        let _fs = self.fs.lock();
        self.read_disk_inode(|disk_inode: &DiskINode| {
            disk_inode.read(offset, buf, Arc::clone(&self.block_dev));
        });
    }

    // 写入文件offset位置
    pub fn write_at(&mut self, offset: u32, buf: &[u8]) {
        // 互斥写
        let mut fs = self.fs.lock();
        self.modify_disk_inode(|disk_inode: &mut DiskINode| {
            self.increase_size(offset + buf.len() as u32, disk_inode, &mut fs);
            disk_inode.write(offset, buf, Arc::clone(&self.block_dev));
        });
    }
    // inode对应的文件扩容到新的大小
    fn increase_size(&self, new_size: u32, disk_inode: &mut DiskINode, fs: &mut MutexGuard<FileSystem>) {
        let old_size = disk_inode.size;
        // 分配需要的新data blocks
        let new_blocks_needed = DiskINode::data_blocks_for_size(new_size - old_size);
        let mut new_blocks: Vec<u32> = Vec::new();
        for _ in 0..new_blocks_needed {
            new_blocks.push(fs.alloc_data_block());
        }
        // 分配新的索引blocks
        let mut index_blocks: Vec<u32> = Vec::new();
        // 所需的新索引块 = 新大小索引总数 - 旧索引总数
        let index_blocks_needed = DiskINode::index_blocks_for_size(new_size) - DiskINode::index_blocks_for_size(old_size);
        for _ in 0..index_blocks_needed {
            index_blocks.push(fs.alloc_data_block());
        }
        // 磁盘inode扩容
        disk_inode.increse_size(new_size, new_blocks, index_blocks, Arc::clone(&self.block_dev));
    }
}

