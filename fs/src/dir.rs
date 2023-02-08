
pub const NAME_LIMIT: usize = 27;
pub const DIR_SIZE: u32 = 32;

// 一个目录项，大小32字节
// 记录目录项名字 和 对应的inode id
#[repr(C)]
pub struct DirEntry {
    name: [u8; NAME_LIMIT + 1], // 名字，C字符串，末尾\0
    inode_id: u32,              // 目录项对应的inode
}

impl DirEntry {
    pub fn empty() -> Self {
        return Self {name: [0u8; NAME_LIMIT + 1], inode_id: 0};
    }

    pub fn new(name: &str, inode_id: u32) -> Self {
        let mut entry = Self::empty();
        entry.name[..name.len()].copy_from_slice(name.as_bytes());
        entry.inode_id = inode_id;
        return entry;
    }

    pub fn to_bytes(&self) -> &[u8] {
        let ptr = self.name.as_ptr();
        unsafe {return core::slice::from_raw_parts(ptr, DIR_SIZE as usize);}
    }

    pub fn to_bytes_mut(&mut self) -> &mut [u8] {
        let ptr = self.name.as_mut_ptr();
        unsafe {return core::slice::from_raw_parts_mut(ptr, DIR_SIZE as usize);}
    }

    pub fn name(&self) -> &str {
        let len = self.name.iter().enumerate()
        .find(|(_, b)| {**b == 0})
        .map(|(idx, _)| {idx})
        .unwrap();
        return core::str::from_utf8(&self.name[..len]).unwrap();
    }
    
    pub fn inode_id(&self) -> u32 {
        return self.inode_id;
    }
}