use fs::block_device::BlockDevice;
use fs::block_cache::BLOCK_SIZE;
use fs::fs::FileSystem;
use std::io::{Seek, SeekFrom, Read, Write};
use std::sync::Mutex;
use std::fs::{File,OpenOptions};
use alloc::sync::Arc;
extern crate alloc;
struct BlockFile(Mutex<File>);


impl BlockDevice for BlockFile {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        let mut file = self.0.lock().unwrap();
        file.seek(SeekFrom::Start((block_id * BLOCK_SIZE) as u64)).expect("file seek failed");
        assert_eq!(file.read(buf).unwrap(), BLOCK_SIZE, "NOT a complete block");
    }

    fn write_block(&self, block_id: usize, buf: &[u8]) {
        let mut file = self.0.lock().unwrap();
        file.seek(SeekFrom::Start((block_id * BLOCK_SIZE) as u64)).expect("file seek failed");
        assert_eq!(file.write(buf).unwrap(), BLOCK_SIZE, "NOT a complete block");
    }
}

fn main() {
    let block_file = Arc::new(BlockFile(Mutex::new({
        let f = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("target/fs.img").unwrap();
        f.set_len(4096 * 4096).unwrap();
        f
    })));
    let mut fs = FileSystem::create(block_file.clone(),4096,1);
    fs.create_root_inode();
    let fs = FileSystem::open(block_file.clone());
    let mut root = FileSystem::root_inode(fs.clone());
    root.create("test-file1");
    root.create("test-file2");
    root.create("test-file3");

    let files = root.ls();
    for f in files.iter() {
        println!("{}", f);
    }
}
