use crate::{ALL_LOWER_CASE, FatFileSystem, fat_layout::{DIRENT_SZ, ShortDirEntry}};

use super::{
    BlockDevice,
    // DiskInode,
    // DiskInodeType,
    // DirEntry,
    // EasyFileSystem,
    // DIRENT_SZ,
    get_block_cache,
};
use alloc::sync::Arc;
use alloc::string::String;
use alloc::vec::Vec;
use spin::{Mutex};
use crate::ROOT_DIR;

#[allow(unused)]
pub const ATTRIBUTE_READ_ONLY:u8 = 0x01;
#[allow(unused)]
pub const ATTRIBUTE_HIDDEN   :u8 = 0x02;
#[allow(unused)]
pub const ATTRIBUTE_SYSTEM   :u8 = 0x04;
#[allow(unused)]
pub const ATTRIBUTE_VOLUME_ID:u8 = 0x08;
#[allow(unused)]
pub const ATTRIBUTE_DIRECTORY:u8 = 0x10;
#[allow(unused)]
pub const ATTRIBUTE_ARCHIVE  :u8 = 0x20;
#[allow(unused)]
pub const ATTRIBUTE_LFN      :u8 = 0x0F;
/*
 inode -> short dir entry
*/
pub struct Inode {
    block_id: usize,
    block_offset: usize,
    fs: Arc<Mutex<FatFileSystem>>,
    block_device: Arc<dyn BlockDevice>,
}

impl Inode {
    /// We should not acquire efs lock here.
    pub fn new(
        block_id: u32,
        block_offset: usize,
        fs: Arc<Mutex<FatFileSystem>>,
        block_device: Arc<dyn BlockDevice>,
    ) -> Self {
        Self {
            block_id: block_id as usize,
            block_offset,
            fs,
            block_device,
        }
    }


    pub fn first_cluster(&self)->u32{
        self.read_dir_entry(|se:& ShortDirEntry|{
            se.first_cluster()
        })
    }

    pub fn set_first_cluster(&self, clu:u32){
        self.modify_dir_entry(|se:&mut ShortDirEntry|{
            se.set_first_cluster(clu);
        })
    }

     pub fn get_size(&self)->u32{
        self.read_dir_entry(|se:&ShortDirEntry|{
            se.get_size()
        })
    }

    pub fn is_dir(&self)->bool{
        let attribute = self.read_dir_entry(|se:&ShortDirEntry|{
            se.attribute()
        });
        if 0 != (attribute & ATTRIBUTE_DIRECTORY) {
            true
        }else{
            false   
        }
    }

    fn read_dir_entry<V>(&self,f: impl FnOnce(&ShortDirEntry) -> V) -> V {
        if self.block_id == 0 {
            let rr = ROOT_DIR.read();
            // let rr = root_dirent.lock();
            f(& rr)
        } else {
            get_block_cache(
                self.block_id,
                Arc::clone(&self.block_device)
            ).lock().read(self.block_offset, f)
        }
    }

    fn modify_dir_entry<V>(&self,f: impl FnOnce(&mut ShortDirEntry) -> V) -> V {
        if self.block_id == 0 {
            //println!("[fs]: modify vroot dent");
            let mut rw = ROOT_DIR.write();
            f(&mut rw)
        } else {
            get_block_cache(
                self.block_id,
                Arc::clone(&self.block_device)
            ).lock().modify(self.block_offset, f)
        }
    }

    /* 查找可用目录项，返回offset，簇不够也会返回相应的offset，caller需要及时分配 */
    fn find_free_dirent(&self,fs: &FatFileSystem)->Option<usize>{
        if !self.is_dir() {
            return None
        }
        let mut offset = 0;
        loop {
            let mut tmp_dirent = ShortDirEntry::empty();
            let read_sz = self.read_dir_entry(|short_ent:&ShortDirEntry|{
                short_ent.read_at(
                    offset, 
                    tmp_dirent.as_bytes_mut(), 
                   &fs,
                    &fs.get_fat(),
                    &self.block_device
                )
            });
            if tmp_dirent.is_empty() || read_sz == 0{
                return Some(offset)
            }
            offset += DIRENT_SZ;
        }
    }

       /* 返回sector和offset */
    pub fn get_pos(&self, offset:usize, fs: &FatFileSystem) -> ( usize, usize){
        let (_, sec, off) = self.read_dir_entry(|s_ent: &ShortDirEntry|{
            s_ent.get_pos(
                offset, 
                &fs, 
                &fs.get_fat(), 
                &self.block_device)
        });
        (sec, off)
    }

    // fn find_short_name(
    //     &self, 
    //     name:&str, 
    //     dir_ent: &ShortDirEntry,
    //     fs: FatFileSystem
    // ) -> Option<Inode> {
    //     let name_upper = name.to_ascii_uppercase();     
    //     let mut short_ent = ShortDirEntry::empty();
    //     let mut offset = 0;
    //     let mut read_sz:usize;
    //     loop {
    //         read_sz = dir_ent.read_at(
    //             offset, 
    //             short_ent.as_bytes_mut(), 
    //             &fs, 
    //             &fs.get_fat(), 
    //             &self.block_device
    //         );
    //         if read_sz != DIRENT_SZ || short_ent.is_empty() {
    //             return None
    //         }else{
    //             if short_ent.is_valid() && name_upper == short_ent.get_name_uppercase() {
    //                 let (short_sector, short_offset) = self.get_pos(offset);
    //                 let long_pos_vec:Vec<(usize, usize)> = Vec::new(); 
    //                 return Some(
    //                     VFile::new(
    //                         String::from(name),
    //                         short_sector, 
    //                         short_offset, 
    //                         long_pos_vec,
    //                         short_ent.attribute(),
    //                         short_ent.get_size(),
    //                         self.fs.clone(),
    //                         self.block_device.clone(),
    //                     )
    //                 )
    //             } else {
    //                 offset += DIRENT_SZ;
    //                 continue;
    //             }
    //         }
    //     }
    // }

    // fn find_inode_id(
    //     &self,
    //     name: &str,
    //     disk_inode: &DiskInode,
    // ) -> Option<u32> {
    //     // assert it is a directory
    //     assert!(disk_inode.is_dir());
    //     let file_count = (disk_inode.size as usize) / DIRENT_SZ;
    //     let mut dirent = DirEntry::empty();
    //     for i in 0..file_count {
    //         assert_eq!(
    //             disk_inode.read_at(
    //                 DIRENT_SZ * i,
    //                 dirent.as_bytes_mut(),
    //                 &self.block_device,
    //             ),
    //             DIRENT_SZ,
    //         );
    //         if dirent.name() == name {
    //             return Some(dirent.inode_number() as u32);
    //         }
    //     }
    //     None
    // }

    pub fn find(&self, name: &str) -> Option<Arc<Inode>> {
        let fs = self.fs.lock();
        self.read_dir_entry(|dir_ent:&ShortDirEntry|{
            let name_upper = name.to_ascii_uppercase();     
            let mut sub_entry = ShortDirEntry::empty();
            let mut offset = 0;
            let mut read_sz:usize;
            loop {
                read_sz = dir_ent.read_at(
                    offset, 
                    sub_entry.as_bytes_mut(), 
                    &fs, 
                    &fs.get_fat(), 
                    &self.block_device
                );
                if read_sz != DIRENT_SZ || sub_entry.is_empty() {
                    return None
                }else{
                    if sub_entry.is_valid() && name_upper == sub_entry.get_name_uppercase() {
                        let (short_sector, short_offset) = self.get_pos(offset, &fs);
                        return Some(Arc::new(
                            Inode::new(
                                short_sector as u32, 
                                short_offset,
                                self.fs.clone(),
                                self.block_device.clone(),
                            )
                        )
                        )
                    } else {
                        offset += DIRENT_SZ;
                        continue;
                    }
                }
            }
        })
    //     let fs = self.fs.lock();
    //     self.read_disk_inode(|disk_inode| {
    //         self.find_inode_id(name, disk_inode)
    //         .map(|inode_id| {
    //             let (block_id, block_offset) = fs.get_disk_inode_pos(inode_id);
    //             Arc::new(Self::new(
    //                 block_id,
    //                 block_offset,
    //                 self.fs.clone(),
    //                 self.block_device.clone(),
    //             ))
    //         })
    //     })
    }

    // fn increase_size(
    //     &self,
    //     new_size: u32,
    //     disk_inode: &mut DiskInode,
    //     fs: &mut MutexGuard<FatFileSystem>,
    // ) {
    //     if new_size < disk_inode.size {
    //         return;
    //     }
    //     let blocks_needed = disk_inode.blocks_num_needed(new_size);
    //     let mut v: Vec<u32> = Vec::new();
    //     for _ in 0..blocks_needed {
    //         v.push(fs.alloc_data());
    //     }
    //     disk_inode.increase_size(new_size, v, &self.block_device);
    // }

    // pub fn create(&self, name: &str) -> Option<Arc<Inode>> {
    //     let mut fs = self.fs.lock();
    //     if self.modify_disk_inode(|root_inode| {
    //         // assert it is a directory
    //         assert!(root_inode.is_dir());
    //         // has the file been created?
    //         self.find_inode_id(name, root_inode)
    //     }).is_some() {
    //         return None;
    //     }
    //     // create a new file
    //     // alloc a inode with an indirect block
    //     let new_inode_id = fs.alloc_inode();
    //     // initialize inode
    //     let (new_inode_block_id, new_inode_block_offset) 
    //         = fs.get_disk_inode_pos(new_inode_id);
    //     get_block_cache(
    //         new_inode_block_id as usize,
    //         Arc::clone(&self.block_device)
    //     ).lock().modify(new_inode_block_offset, |new_inode: &mut DiskInode| {
    //         new_inode.initialize(DiskInodeType::File);
    //     });
    //     self.modify_disk_inode(|root_inode| {
    //         // append file in the dirent
    //         let file_count = (root_inode.size as usize) / DIRENT_SZ;
    //         let new_size = (file_count + 1) * DIRENT_SZ;
    //         // increase size
    //         self.increase_size(new_size as u32, root_inode, &mut fs);
    //         // write dirent
    //         let dirent = DirEntry::new(name, new_inode_id);
    //         root_inode.write_at(
    //             file_count * DIRENT_SZ,
    //             dirent.as_bytes(),
    //             &self.block_device,
    //         );
    //     });

    //     let (block_id, block_offset) = fs.get_disk_inode_pos(new_inode_id);
    //     // return inode
    //     Some(Arc::new(Self::new(
    //         block_id,
    //         block_offset,
    //         self.fs.clone(),
    //         self.block_device.clone(),
    //     )))
    //     // release efs lock automatically by compiler
    // }

    pub fn ls(&self) -> Vec<(String,u8)> {
        // let _fs = self.fs.lock();
        // self.read_dir_entry(|disk_inode| {
        //     let file_count = (disk_inode.size as usize) / DIRENT_SZ;
        //     let mut v: Vec<String> = Vec::new();
        //     for i in 0..file_count {
        //         let mut dirent = DirEntry::empty();
        //         assert_eq!(
        //             disk_inode.read_at(
        //                 i * DIRENT_SZ,
        //                 dirent.as_bytes_mut(),
        //                 &self.block_device,
        //             ),
        //             DIRENT_SZ,
        //         );
        //         v.push(String::from(dirent.name()));
        //     }
        //     v
        // })
        let fs = self.fs.lock();
        let mut list:Vec<(String, u8)> = Vec::new();
        // DEBUG
        let mut offset:usize = 0;
        let mut short_ent =  ShortDirEntry::empty();
        loop {
            let read_sz = self.read_dir_entry(|curr_ent:&ShortDirEntry|{
                curr_ent.read_at(
                    offset, 
                    short_ent.as_bytes_mut(),
                    &fs,
                    &fs.get_fat(),
                    &self.block_device
                )
            });
            // 检测是否结束或被删除
            if read_sz != DIRENT_SZ || short_ent.is_empty() { 
                return list
            }
            list.push((short_ent.get_name_lowercase(), short_ent.attribute()));  
            offset += DIRENT_SZ;
        }
    }

    /* 在当前目录下创建文件 */ 
    pub fn create(& self, name: &str, attribute: u8) -> Option<Arc<Inode>> {
        
        assert!(self.is_dir());
        
        let manager_reader = self.fs.lock();
        // let (name_, ext_) = manager_reader.split_name_ext(name);
        // 搜索空处
        let dirent_offset:usize;
        if let Some(offset) = self.find_free_dirent(&manager_reader){
            dirent_offset = offset;
        } else {
            return None
        }
        let mut short_ent = ShortDirEntry::empty();
    
        let (name_bytes, ext_bytes) = manager_reader.short_name_format(name);
        short_ent.initialize(&name_bytes, &ext_bytes, attribute);
        short_ent.set_case(ALL_LOWER_CASE);
        // drop(manager_reader);
        
        // 写目录项
        assert_eq!(
            self._write_at(dirent_offset, short_ent.as_bytes_mut(),&manager_reader),
            DIRENT_SZ
        );
        // 
        let (block_id, block_offset) = self.get_pos(dirent_offset,&manager_reader);
        Some(Arc::new(Self::new(
            block_id as u32,
            block_offset,
            self.fs.clone(),
            self.block_device.clone(),
        )))
        
    }

    // pub fn read_at(&self, offset: usize, buf: &mut [u8]) -> usize {
    //     let _fs = self.fs.lock();
    //     self.read_disk_inode(|disk_inode| {
    //         disk_inode.read_at(offset, buf, &self.block_device)
    //     })
    // }

    // pub fn write_at(&self, offset: usize, buf: &[u8]) -> usize {
    //     let mut fs = self.fs.lock();
    //     self.modify_disk_inode(|disk_inode| {
    //         self.increase_size((offset + buf.len()) as u32, disk_inode, &mut fs);
    //         disk_inode.write_at(offset, buf, &self.block_device)
    //     })
    // }

    pub fn read_at(&self, offset: usize, buf: &mut [u8])->usize{
        let fs = self.fs.lock();
        self.read_dir_entry(|short_ent: &ShortDirEntry|{
            short_ent.read_at(
                offset, 
                buf, 
                &fs,
                &fs.get_fat(), 
                &self.block_device
            )
        })
    }   

    pub fn write_at(& self, offset: usize, buf: & [u8])->usize{
        let fs = self.fs.lock();
        self._write_at(offset, buf, &fs)
    }

    pub fn _write_at(& self, offset: usize, buf: & [u8],fs: &FatFileSystem)->usize{
        // let fs = self.fs.lock();
        self.increase_size((offset + buf.len()) as u32 ,&fs);
        self.modify_dir_entry(|short_ent: &mut ShortDirEntry|{
            short_ent.write_at(
                offset, 
                buf, 
                &fs, 
                &fs.get_fat(), 
                &self.block_device
            )
        })
    }

    fn increase_size(
        & self,
        new_size: u32,
        manager_writer: &FatFileSystem
    ) {  // TODO: return sth when cannot increase
        //println!("===================== in increase =======================");
        //println!("file: {}, newsz = {}", self.get_name(), new_size);
        //println!("try lock");
        let first_cluster = self.first_cluster();
        let old_size = self.get_size();
        // let manager_writer = self.fs.lock();
        //println!("get lock");
        if new_size <= old_size {
            //println!("oldsz > newsz");
            return;
        }
        let needed = manager_writer.cluster_num_needed(old_size, new_size, self.is_dir(), first_cluster);
        //println!("needed = {}", needed);
        if needed == 0{
            if !self.is_dir() {
                //self.size = new_size;
                self.modify_dir_entry(|se:&mut ShortDirEntry|{
                    se.set_size(new_size);
                });
            }  
            return;
        }   
        
        //println!("first cluster = {} nxt = {}", first_cluster, manager_writer.get_fat().read().get_next_cluster(first_cluster, self.block_device.clone()));
        if let Some(cluster) = manager_writer.alloc_cluster(needed) {
            //println!("*** cluster alloc = {}",cluster);
            if first_cluster == 0 { //未分配簇
                // drop(manager_writer);
                self.modify_dir_entry(|se:&mut ShortDirEntry|{
                    se.set_first_cluster(cluster);
                });
                //println!("fc = {}",self.first_cluster());
                //println!("================== increase end ====================");
            }else{  // 已经分配簇
                //let fs_reader = self.fs.read();
                //println!("[fs-inc]: file: {}, newsz = {}", self.get_name(), new_size);
                //println!("  cluster alloc = {}",cluster);
                let fat = manager_writer.get_fat();
                //println!("try lock1");
                let fat_writer = fat;
                //println!("get lock1");
                let final_cluster = fat_writer.final_cluster(first_cluster, self.block_device.clone());
                assert_ne!( cluster, 0);
                fat_writer.set_next_cluster(final_cluster, cluster, self.block_device.clone());
                //let allc = fat_writer.get_all_cluster_of(first_cluster, self.block_device.clone());
                //println!("  finish set next cluster, cluster chain:{:?}", allc);
                // drop(manager_writer);
            }
            //self.size = new_size;
            self.modify_dir_entry(|se:&mut ShortDirEntry|{
                se.set_size(new_size);
            });
        } else {
            panic!("SD Card no space!!!");
        }
    }
    // pub fn clear(&self) {
    //     let mut fs = self.fs.lock();
    //     self.modify_disk_inode(|disk_inode| {
    //         let size = disk_inode.size;
    //         let data_blocks_dealloc = disk_inode.clear_size(&self.block_device);
    //         assert!(data_blocks_dealloc.len() == DiskInode::total_blocks(size) as usize);
    //         for data_block in data_blocks_dealloc.into_iter() {
    //             fs.dealloc_data(data_block);
    //         }
    //     });
    // }
}
