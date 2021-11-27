use alloc::{string::String, sync::Arc, vec::Vec};

use crate::{FatFileSystem, clone_into_array};

use super::{
    BLOCK_SZ,
    BlockDevice,
    get_block_cache,
};

const LEAD_SIGNATURE:u32 = 0x41615252;
const SECOND_SIGNATURE:u32 = 0x61417272;
pub const FREE_CLUSTER:u32 = 0x00000000;
pub const END_CLUSTER:u32  = 0x0FFFFFF8;
pub const BAD_CLUSTER:u32  = 0x0FFFFFF7;
const FATENTRY_PER_SEC:u32 = BLOCK_SZ as u32/4;
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
pub const DIRENT_SZ:usize = 32;
#[allow(unused)]
pub const SHORT_NAME_LEN:u32 = 8;
#[allow(unused)]
pub const SHORT_EXT_LEN:u32 = 3;
pub const LONG_NAME_LEN:u32 = 13;

pub const ALL_UPPER_CASE:u8 = 0x00;
pub const ALL_LOWER_CASE:u8 = 0x08;

type DataBlock = [u8; BLOCK_SZ];

#[repr(packed)]
#[derive(Clone, Copy, Debug)]
pub struct FatBS {
    pub unused:           [u8;11],
    pub bytes_per_sector:      u16,
    pub sectors_per_cluster:   u8,
    pub reserved_sector_count: u16,
    pub table_count:      u8,
    pub root_entry_count: u16,//FAT32必须等于0
    pub total_sectors_16: u16,
    pub media_type:       u8,
    pub table_size_16:    u16,// 无用
    pub sectors_per_track:u16,
    pub head_side_count:  u16,    
    pub hidden_sector_count: u32,  
    pub total_sectors_32:    u32,    
}

impl FatBS {
    pub fn total_sectors(&self) -> u32 {
        if self.total_sectors_16 == 0 {
            self.total_sectors_32
        } else {
            self.total_sectors_16 as u32
        }
    }

    /*第一个FAT表所在的扇区*/
    pub fn first_fat_sector(&self) -> u32 {
        self.reserved_sector_count as u32
    }
}

#[repr(packed)]
#[derive(Clone, Copy)]
#[allow(unused)]
pub struct FatExtBS{
    table_size_32:u32,
    extended_flags:u16,   
    fat_version:u16,
    root_clusters:u32,   
    fat_info:u16,
    backup_bs_sector:u16,
    reserved_0:[u8;12],
    drive_number:u8,
    reserved_1:u8,
    boot_signature:u8,  //0x28 or 0x29
}

impl FatExtBS{
    // FAT占用的扇区数
    pub fn fat_size(&self) -> u32{
        self.table_size_32
    }

    pub fn fat_info_sec(&self)->u32{
        self.fat_info as u32
    }

    #[allow(unused)]
    pub fn root_clusters(&self)->u32{
        self.root_clusters
    }
}

// 该结构体不对Buffer作结构映射，仅保留位置信息
// 但是为其中信息的获取和修改提供了接口
pub struct FSInfo{
    sector_num: u32,
}

impl FSInfo{
    pub fn new(sector_num: u32)->Self {
        Self{
            sector_num
        }
    }

    fn check_lead_signature(&self, block_device: Arc<dyn BlockDevice>) -> bool {
        get_block_cache(self.sector_num as usize, block_device)
        .lock()
        .read(0,|&lead_sig: &u32|{
            lead_sig == LEAD_SIGNATURE
        })
    }

    fn check_another_signature(&self, block_device: Arc<dyn BlockDevice>) -> bool {
        get_block_cache(self.sector_num as usize, block_device)
        .lock()
        .read(484,|&sec_sig: &u32|{
            sec_sig == SECOND_SIGNATURE
        })
    }

    /*对签名进行校验*/
    pub fn check_signature(&self, block_device: Arc<dyn BlockDevice>) -> bool {
        return self.check_lead_signature(block_device.clone()) 
            && self.check_another_signature(block_device.clone())
    }

    /*读取空闲簇数*/
    pub fn read_free_clusters(&self, block_device: Arc<dyn BlockDevice>) -> u32{
        get_block_cache(self.sector_num as usize, block_device)
        .lock()
        .read(488,|&free_cluster_count: &u32|{
            free_cluster_count
        })
    }

    /*写空闲块数*/
    pub fn write_free_clusters(&self, free_clusters: u32, block_device: Arc<dyn BlockDevice>) {
        get_block_cache(self.sector_num as usize, block_device)
        .lock()
        .modify(488,|free_cluster_count: &mut u32|{
            *free_cluster_count = free_clusters;
        });
    }   

    /*读起始空闲块*/
    pub fn first_free_cluster(&self, block_device: Arc<dyn BlockDevice>) ->  u32{
        get_block_cache(self.sector_num as usize, block_device)
        .lock()
        .read(492,|&start_cluster: &u32|{
            start_cluster
        })
    }

    /*写起始空闲块*/
    pub fn write_first_free_cluster(&self, start_cluster:u32, block_device: Arc<dyn BlockDevice>){
        //println!("sector_num = {}, start_c = {}", self.sector_num, start_cluster);
        get_block_cache(self.sector_num as usize, block_device)
        .lock()
        .modify(492,|start_clu: &mut u32|{
            *start_clu = start_cluster;
        });
    }
}

// 常驻内存，不作一一映射
#[allow(unused)]
#[derive(Clone, Copy)]
pub struct FAT{
    fat1_sector: u32, //FAT1和FAT2的起始扇区
    fat2_sector: u32, 
    n_sectors: u32,   //大小
    n_entry: u32,     //表项数量 
}

// TODO: 防越界处理（虽然可能这辈子都遇不到）
impl FAT{
    pub fn new(fat1_sector:u32, fat2_sector:u32, n_sectors: u32, n_entry:u32)->Self{
        Self{
            fat1_sector,
            fat2_sector,
            n_sectors,
            n_entry,
        }
    }

    /* 计算簇对应表项的位置：sector和offset */
    fn calculate_pos(&self, cluster: u32)->(u32,u32,u32){
        // 返回sector号和offset
        // 前为FAT1的扇区号，后为FAT2的扇区号，最后为offset
        // DEBUG 
        let fat1_sec = self.fat1_sector + cluster / FATENTRY_PER_SEC;
        let fat2_sec = self.fat2_sector + cluster / FATENTRY_PER_SEC;
        let offset = 4 * (cluster % FATENTRY_PER_SEC);
        (fat1_sec,fat2_sec,offset)
    }

    /* 搜索下一个可用簇 */
    // caller需要确定有足够的空闲簇，这里不作越界检查
    pub fn next_free_cluster(&self, current_cluster:u32, block_device: Arc<dyn BlockDevice>)->u32{
        // DEBUG
        let mut curr_cluster = current_cluster + 1;
        loop{
            #[allow(unused)]
            let (fat1_sec,fat2_sec,offset) = self.calculate_pos(curr_cluster);
            // 查看当前cluster的表项
            let entry_val = get_block_cache(
                fat1_sec as usize, 
                block_device.clone())
            .lock()
            .read(offset as usize,|&entry_val: &u32|{
                entry_val
            });
            if entry_val == FREE_CLUSTER { 
                break;
            }else{
                curr_cluster += 1;
            }

        }
        curr_cluster & 0x0FFFFFFF
    }

    /* 查询当前簇的下一个簇 */
    pub fn get_next_cluster(&self, cluster: u32, block_device: Arc<dyn BlockDevice>) -> u32{
        // 需要对损坏簇作出判断
        // 及时使用备用表
        // 无效或未使用返回0
        let (fat1_sec,_fat2_sec,offset) = self.calculate_pos(cluster);
        //println!("fat1_sec={} offset = {}", fat1_sec, offset);
        let fat1_rs = get_block_cache(fat1_sec as usize, block_device.clone())
        .lock()
        .read(offset as usize,|&next_cluster: &u32|{
            next_cluster
        });
        // let fat2_rs = get_block_cache(fat2_sec as usize, block_device.clone())
        // .lock()
        // .read(offset as usize,|&next_cluster: &u32|{
        //     next_cluster
        // });
        if fat1_rs == BAD_CLUSTER {
            0
            // if fat2_rs == BAD_CLUSTER {
            //     0
            // } else {
            //     fat2_rs & 0x0FFFFFFF
            // }
        } else {
            fat1_rs & 0x0FFFFFFF
        }
    }

    pub fn set_end(&self, cluster:u32, block_device: Arc<dyn BlockDevice>){
        self.set_next_cluster(cluster, END_CLUSTER, block_device);
    }

    /* 设置当前簇的下一个簇 */
    pub fn set_next_cluster(&self, cluster:u32, next_cluster:u32, block_device: Arc<dyn BlockDevice>){
        // 同步修改两个FAT
        // 注意设置末尾项为 0x0FFFFFF8 
        //assert_ne!(next_cluster, 0);
        let (fat1_sec,fat2_sec,offset) = self.calculate_pos(cluster);
        get_block_cache( fat1_sec as usize, block_device.clone())
        .lock()
        .modify(offset as usize,|old_clu: &mut u32|{
            *old_clu = next_cluster;
        });
        get_block_cache( fat2_sec as usize, block_device.clone())
        .lock()
        .modify(offset as usize,|old_clu: &mut u32|{
            *old_clu = next_cluster;
        });
    }

    /* 获取某个文件的指定cluster */
    pub fn get_cluster_at(&self, start_cluster:u32, index: u32, block_device: Arc<dyn BlockDevice>) -> u32{
        // 如果有异常，返回0
        //println!("** get_cluster_at index = {}",index);
        let mut cluster = start_cluster;
        #[allow(unused)]
        for i in 0..index {
            //print!("in fat curr cluster = {}", cluster);
            cluster = self.get_next_cluster(cluster, block_device.clone());
            //println!(", next cluster = {:X}", cluster);
            if cluster == 0 {
                break;
            }
        }
        cluster & 0x0FFFFFFF
    }


    pub fn final_cluster(&self, start_cluster:u32, block_device: Arc<dyn BlockDevice>)->u32 {
        let mut curr_cluster = start_cluster;
        assert_ne!(start_cluster, 0);
        loop{
            
            let next_cluster = self.get_next_cluster(curr_cluster, block_device.clone());
            //println!("in fianl cl {};{}", curr_cluster, next_cluster);
            //assert_ne!(next_cluster, 0);
            if next_cluster >= END_CLUSTER || next_cluster == 0 {
                return curr_cluster & 0x0FFFFFFF
            } else {
                curr_cluster = next_cluster;
            }
        }
    }

    pub fn get_all_cluster_of(&self, start_cluster:u32, block_device: Arc<dyn BlockDevice>)->Vec<u32>{
        let mut curr_cluster = start_cluster;
        let mut v_cluster:Vec<u32> = Vec::new();
        loop{
            v_cluster.push( curr_cluster & 0x0FFFFFFF );
            let next_cluster = self.get_next_cluster(curr_cluster, block_device.clone());
            //println!("in all, curr = {}, next = {}", curr_cluster, next_cluster);
            //assert_ne!(next_cluster, 0);
            if next_cluster >= END_CLUSTER || next_cluster == 0{
                return v_cluster
            } else {
                curr_cluster = next_cluster;
            }
        }
    }

    pub fn count_claster_num(&self, start_cluster:u32, block_device: Arc<dyn BlockDevice>)->u32{
        if start_cluster == 0{
            return 0;
        }
        let mut curr_cluster = start_cluster;
        let mut count:u32 = 0; 
        loop{
            count += 1;
            let next_cluster = self.get_next_cluster(curr_cluster, block_device.clone());
            //println!("next_cluster = {:X}",next_cluster);
            if next_cluster >= END_CLUSTER || next_cluster > 0xF000000{
                return count
            } else {
                curr_cluster = next_cluster;
            }
        }
    }
}
#[derive(Clone, Copy, Debug)]
#[repr(packed)]
#[allow(unused)]
pub struct ShortDirEntry{
    name: [u8;8],        // 删除时第0位为0xE5，未使用时为0x00. 有多余可以用0x20填充
    extension: [u8;3],
    attribute: u8,       //可以用于判断是目录还是文件
    winnt_reserved: u8,
    creation_tenths: u8, //精确到0.1s
    creation_time: u16,
    creation_date: u16,
    last_acc_date: u16,
    cluster_high: u16,
    modification_time: u16,
    modification_date: u16,
    cluster_low: u16,
    size: u32,
}

impl ShortDirEntry{
    /* K210上无法获取现实的时间，因此没有修改时间字段的必要 */

    /* 建一个空的，一般读取时用到 */
    // QUES 真的用得到？
    pub fn empty()->Self{
        Self{
            name: [0;8],        // 删除时第0位为0xE5，未使用时为0x00. 有多余可以用0x20填充
            extension: [0;3],
            attribute: 0,       //可以用于判断是目录还是文件
            winnt_reserved: 0,
            creation_tenths: 0, //精确到0.1s
            creation_time: 0,
            creation_date: 0,
            last_acc_date: 0,
            cluster_high:  0,
            modification_time: 0,
            modification_date: 0,
            cluster_low: 0,
            size: 0,
        }
    }

    /* 创建文件时调用 
    * 新建时不必分配块。写时检测初始簇是否为0，为0则需要分配。
    */
    pub fn new(
        name_: &[u8],        
        extension_: &[u8],
        attribute: u8,       
    ) -> Self{  
        let name:[u8;8] = clone_into_array(&name_[0..8]);
        let extension:[u8;3] = clone_into_array(&extension_[0..3]);
        Self{
            name,        
            extension,
            attribute,       
            winnt_reserved: 0,
            creation_tenths: 0, 
            creation_time: 0,
            creation_date: 0x529c,
            last_acc_date: 0,
            cluster_high:  0,
            modification_time: 0,
            modification_date: 0,
            cluster_low: 0,   
            size: 0,
        }
    }

    pub fn initialize(
        &mut self,
        name_: &[u8],        
        extension_: &[u8],
        attribute: u8,       
    ){  
        let name:[u8;8] = clone_into_array(&name_[0..8]);
        let extension:[u8;3] = clone_into_array(&extension_[0..3]);
        *self = Self{
            name,        
            extension,
            attribute,       
            winnt_reserved: 0,
            creation_tenths: 0, 
            creation_time: 0,
            creation_date: 0x529c,
            last_acc_date: 0,
            cluster_high:  0,
            modification_time: 0,
            modification_date: 0,
            cluster_low: 0,   
            size: 0,
        };
    }

    /* 返回目前使用的簇的数量 */
    pub fn data_clusters(&self, bytes_per_cluster: u32)->u32{
        // size为0的时候就是0
        (self.size + bytes_per_cluster -1)/ bytes_per_cluster
    }

    pub fn is_dir(&self)->bool{
        if 0 != (self.attribute & ATTRIBUTE_DIRECTORY) {
            true
        }else{
            false   
        }
    }

    pub fn is_valid(&self)->bool {
        if self.name[0] == 0xE5 {
            false
        }else{
            true
        }
    }

    pub fn is_deleted(&self)->bool{
        if self.name[0] == 0xE5 {
            true
        }else{
            false
        }
    }

    pub fn is_empty(&self)->bool{
        if self.name[0] == 0x00 {
            true
        } else {
            false
        }
    }

    pub fn is_file(&self)->bool{
        if 0 != (self.attribute & ATTRIBUTE_DIRECTORY) {
            false
        }else{
            true   
        }
    }

    pub fn is_long(&self)->bool{
        if self.attribute == ATTRIBUTE_LFN {
            true
        }else{
            false
        }
    }

    pub fn attribute(&self)->u8{
        self.attribute
    }

    pub fn get_creation_time(&self) -> (u32,u32,u32,u32,u32,u32,u64) {
        // year-month-day-Hour-min-sec-long_sec
        let year: u32  = ((self.creation_date & 0xFE00)>>9) as u32 + 1980;
        let month:u32  = ((self.creation_date & 0x01E0)>>5) as u32 ;    
        let day:u32  = (self.creation_date & 0x001F) as u32 ;    
        let hour:u32  = ((self.creation_time & 0xF800)>>11) as u32;    
        let min:u32  = ((self.creation_time & 0x07E0)>>5) as u32;    
        let sec:u32  = ((self.creation_time & 0x001F)<<1) as u32; // 秒数需要*2 
        let long_sec: u64 = ((((year - 1970) * 365 + month * 30 + day) * 24 + hour) * 3600 + min*60 + sec) as u64;
        (year,month,day,hour,min,sec,long_sec)
    }

    pub fn get_modification_time(&self) -> (u32,u32,u32,u32,u32,u32,u64) {
        // year-month-day-Hour-min-sec
        let year: u32  = ((self.modification_date & 0xFE00)>>9) as u32 + 1980;
        let month:u32  = ((self.modification_date & 0x01E0)>>5) as u32 ;    
        let day:u32  = (self.modification_date & 0x001F) as u32 ;    
        let hour:u32  = ((self.modification_time & 0xF800)>>11) as u32;    
        let min:u32  = ((self.modification_time & 0x07E0)>>5) as u32;    
        let sec:u32  = ((self.modification_time & 0x001F)<<1) as u32; // 秒数需要*2   
        let long_sec: u64 = ((((year - 1970) * 365 + month * 30 + day) * 24 + hour) * 3600 + min*60 + sec) as u64;
        (year,month,day,hour,min,sec,long_sec)
    }

    pub fn get_accessed_time(&self) -> (u32,u32,u32,u32,u32,u32,u64) {
        // year-month-day-Hour-min-sec
        let year: u32  = ((self.last_acc_date & 0xFE00)>>9) as u32 + 1980;
        let month:u32  = ((self.last_acc_date & 0x01E0)>>5) as u32 ;    
        let day:u32  = (self.last_acc_date & 0x001F) as u32 ;    
        let hour:u32 = 0;    
        let min:u32  = 0;    
        let sec:u32  = 0; // 没有相关信息，默认0 
        let long_sec: u64 = ((((year - 1970) * 365 + month * 30 + day) * 24 + hour) * 3600 + min*60 + sec) as u64;
        (year,month,day,hour,min,sec,long_sec)
    }

    /*获取文件起始簇号*/
    pub fn first_cluster(&self) -> u32 {
        ((self.cluster_high as u32) << 16) + (self.cluster_low as u32)
    }

    /*获取短文件名*/
    pub fn get_name_uppercase(&self)-> String {
        let mut name: String = String::new();
        for i in 0..8 {  // 记录文件名
            if self.name[i] == 0x20 {
                break;
            } else {
                name.push(self.name[i] as char);
            }
        }
        for i in 0..3 { // 记录扩展名
            if self.extension[i] == 0x20 {
                break;
            } else {
                if i == 0 {name.push('.'); }
                name.push(self.extension[i] as char);
            }
        }
        name
    }

    pub fn get_name_lowercase(&self) -> String {
        let mut name: String = String::new();
        for i in 0..8 {  // 记录文件名
            if self.name[i] == 0x20 {
                break;
            } else {
                name.push((self.name[i] as char).to_ascii_lowercase());
            }
        }
        for i in 0..3 { // 记录扩展名
            if self.extension[i] == 0x20 {
                break;
            } else {
                if i == 0 {name.push('.'); }
                name.push((self.extension[i] as char).to_ascii_lowercase());
            }
        }
        name
    }

    /* 计算校验和 */ // DEBUG
    pub fn checksum(&self)->u8{
        let mut name_buff:[u8;11] = [0u8;11]; 
        let mut sum:u8 = 0;
        for i in 0..8 { name_buff[i] = self.name[i]; }
        for i in 0..3 { name_buff[i+8] = self.extension[i]; }
        for i in 0..11{ 
            if (sum & 1) != 0 {
                sum = 0x80 + (sum>>1) + name_buff[i];
            }else{
                sum = (sum>>1) + name_buff[i];
            }
        }
        sum
    }

    /* 设置当前文件的大小 */ 
    // 簇的分配和回收实际要对FAT表操作
    pub fn set_size(
        &mut self,
        size: u32, 
    ) {
        self.size = size;
    }

    pub fn get_size(&self)->u32{
        self.size
    }

    pub fn set_case(&mut self, case: u8){
        self.winnt_reserved = case;
    }

    // QUES 分配和回收块都应该对整个文件系统上锁?

    /* 设置文件起始簇 */
    pub fn set_first_cluster(&mut self, cluster: u32){
        self.cluster_high = ((cluster & 0xFFFF0000)>>16) as u16;
        self.cluster_low = (cluster & 0x0000FFFF) as u16;
    }

    /* 清空文件，删除时使用 */
    pub fn clear(&mut self){
        self.size = 0;
        //self.name[0] = 0xE5;
        self.set_first_cluster(0);
    }

    pub fn delete(&mut self){
        self.size = 0;
        self.name[0] = 0xE5;
        self.set_first_cluster(0);
        //TODO:回收cluster?
    }

    /* 获取文件偏移量所在的簇、扇区和偏移 */
    pub fn get_pos(
        &self,
        offset: usize,
        manager: &FatFileSystem, 
        fat: &Arc<FAT>,
        block_device: &Arc<dyn BlockDevice>,
    )->(u32, usize, usize) {
        let manager_reader = manager;
        let fat_reader = fat;
        let bytes_per_sector = manager_reader.bytes_per_sector() as usize;
        let bytes_per_cluster = manager_reader.bytes_per_cluster() as usize;
        let cluster_index = manager_reader.cluster_of_offset(offset);
        let current_cluster = fat_reader.get_cluster_at(
            self.first_cluster(),
            cluster_index , 
            Arc::clone(block_device)
        );
        //println!("*** in get pos, cluster index = {}, current cluster = {}, first_cluster = {}", cluster_index, current_cluster, self.first_cluster());
        let current_sector = manager_reader.first_sector_of_cluster(current_cluster) 
                                + (offset - cluster_index as usize * bytes_per_cluster) 
                                / bytes_per_sector;
        (current_cluster, current_sector, offset % bytes_per_sector)
    }   

    /* 以偏移量读取文件，这里会对fat和manager加读锁 */
    pub fn read_at(
        &self,
        offset: usize,
        buf: &mut [u8],
        manager: &FatFileSystem, 
        fat: &Arc<FAT>,
        block_device: &Arc<dyn BlockDevice>,
    ) -> usize {
        //println!("========================================================\nin read_at self.first_cluster={}", self.first_cluster());
        // 获取共享锁
        let manager_reader = manager;
        let fat_reader = fat;
        let bytes_per_sector = manager_reader.bytes_per_sector() as usize;
        let bytes_per_cluster = manager_reader.bytes_per_cluster() as usize;
        let mut current_off = offset;
        //println!("size = {}",self.size);
        let end:usize;
        if self.is_dir() {
            let size =  bytes_per_cluster * fat_reader.count_claster_num(self.first_cluster() as u32, block_device.clone()) as usize;
            end = offset + buf.len().min(size );// DEBUG:约束上界
        } else {
            end = (offset + buf.len()).min(self.size as usize);
        }
        //println!("in read_at offset ={}; end={}",current_off, end);
        if current_off >= end {
            return 0;
        }
        //println!("first cluster = {}",self.first_cluster());
        // DEBUG: 如果一开始就不在第一个簇，如果buffer不大，会多次进入函数，这里可能会有问题
        // let cluster_index = manager_reader.cluster_of_offset(offset);
        let (c_clu, c_sec, _) = self.get_pos(
            offset, manager, 
            fat, 
            block_device
        );
        //println!("curr_clu = {} sec = {}", c_clu, c_sec);
        if c_clu >= END_CLUSTER {return 0};
        let mut current_cluster = c_clu;
        let mut current_sector = c_sec;
        /*
        let mut current_cluster = fat_reader.get_cluster_at(
            self.first_cluster(),
            manager_reader.cluster_of_offset(offset) , 
            Arc::clone(block_device)
        );
        println!("in read_at, after get_cluster_at, current_cluster = {}", current_cluster);

        //这里出了问题！！！
        let mut current_sector = manager_reader.first_sector_of_cluster(current_cluster) + current_off / bytes_per_sector;
        println!("in read_at current_sector={}", current_sector);
        */
        
        let mut read_size = 0usize;
        loop {
            // 将偏移量向上对齐扇区大小（一般是512
            let mut end_current_block = (current_off / bytes_per_sector + 1) * bytes_per_sector;
            end_current_block = end_current_block.min(end);
            // 读
            let block_read_size = end_current_block - current_off;
            let dst = &mut buf[read_size..read_size + block_read_size];
            // if self.is_dir() {
            //     get_info_cache(  // 目录项通过Infocache访问
            //         current_sector,
            //         Arc::clone(block_device),
            //         CacheMode::READ,
            //     )
            //     .read()
            //     .read(0, |data_block: &DataBlock| {
            //         let src = &data_block[current_off % BLOCK_SZ..current_off % BLOCK_SZ + block_read_size];
            //         dst.copy_from_slice(src);
            //     });
            // } else {
                get_block_cache(
                    current_sector,
                    Arc::clone(block_device)
                )
                .lock()
                .read(0, |data_block: &DataBlock| {
                    let src = &data_block[current_off % BLOCK_SZ..current_off % BLOCK_SZ + block_read_size];
                    dst.copy_from_slice(src);
                });
            // }
            // 更新读取长度
            read_size += block_read_size;
            if end_current_block == end { break; }
            // 更新索引参数
            current_off = end_current_block;
            if current_off % bytes_per_cluster == 0 {
                // 读完一个簇
                //println!("in read_at");
                current_cluster = fat_reader.get_next_cluster(current_cluster, Arc::clone(block_device));
                if current_cluster >= END_CLUSTER { break; } //没有下一个簇
                //println!("read at current_cluster = {}", current_cluster);
                // 计算所在扇区
                current_sector = manager_reader.first_sector_of_cluster(current_cluster);
                //println!("read at current_sector = {}", current_sector);
                //let mut guess = String::new();
                //std::io::stdin().read_line(&mut guess).expect("Failed to read line");
            } else {
                current_sector += 1; //没读完一个簇，直接进入下一扇区
            }   
        }
        read_size
    }

    /* 以偏移量写文件，这里会对fat和manager加读锁 */
    pub fn write_at(
        &self,
        offset: usize,
        buf: & [u8],
        manager: &FatFileSystem, 
        fat: &Arc<FAT>,
        block_device: &Arc<dyn BlockDevice>,
    ) -> usize {
        //println!("in w_short");
        // 获取共享锁
        let manager_reader = manager;
        let fat_reader = fat;
        let bytes_per_sector = manager_reader.bytes_per_sector() as usize;
        let bytes_per_cluster = manager_reader.bytes_per_cluster() as usize;
        let mut current_off = offset;
        let end:usize;
        if self.is_dir() {
            let size =  bytes_per_cluster * fat_reader.count_claster_num(self.first_cluster() as u32, block_device.clone()) as usize;
            end = offset + buf.len().min(size );// DEBUG:约束上界
        } else {
            end = (offset + buf.len()).min(self.size as usize);
        }
        //println!("current offset:{}; end = {}",offset, end);
        //if current_off >= end {
        //    return 0;
        //}
        //println!("in write_at first_cluster:{}",self.first_cluster());
        // DEBUG: 如果一开始就不在第一个簇！！
        /*
        let mut current_cluster = fat_reader.get_cluster_at(
            self.first_cluster(),
            manager_reader.cluster_of_offset(offset) , 
            Arc::clone(block_device)
        );
        //println!("in write_at curr_cluster:{}",current_cluster);
        let mut current_sector = manager_reader.first_sector_of_cluster(current_cluster) + current_off / bytes_per_sector;
        */
        let (c_clu, c_sec, _) = self.get_pos(
            offset, manager, 
            &manager_reader.get_fat(), 
            block_device
        );
        let mut current_cluster = c_clu;
        let mut current_sector = c_sec;
        let mut write_size = 0usize;
        //println!("in write_at curr_sec:{}",current_sector);
        
        loop {
            // 将偏移量向上对齐扇区大小（一般是512
            let mut end_current_block = (current_off / bytes_per_sector + 1) * bytes_per_sector;
            end_current_block = end_current_block.min(end);

            // 写
            let block_write_size = end_current_block - current_off;
            //println!("write cache: current_sector = {}", current_sector);
            // if self.is_dir() {
            //     get_info_cache(  // 目录项通过infocache访问
            //         current_sector,
            //         Arc::clone(block_device),
            //         CacheMode::READ,
            //     )
            //     .write()
            //     .modify(0, |data_block: &mut DataBlock| {
            //         let src = &buf[write_size..write_size + block_write_size];
            //         let dst = &mut data_block[current_off % BLOCK_SZ..current_off % BLOCK_SZ + block_write_size];
            //         dst.copy_from_slice(src);
            //     });
            // } else {
                get_block_cache(
                    current_sector,
                    Arc::clone(block_device)
                )
                .lock()
                .modify(0, |data_block: &mut DataBlock| {
                    let src = &buf[write_size..write_size + block_write_size];
                    let dst = &mut data_block[current_off % BLOCK_SZ..current_off % BLOCK_SZ + block_write_size];
                    dst.copy_from_slice(src);
                });
            // }
            // 更新读取长度
            write_size += block_write_size;
            if end_current_block == end { break; }
            // 更新索引参数
            current_off = end_current_block;
            if current_off % bytes_per_cluster == 0 {
                // 读完一个簇
                //println!("finish writing a cluster");
                current_cluster = fat_reader.get_next_cluster(current_cluster, Arc::clone(block_device));
                if current_cluster >= END_CLUSTER { panic!("END_CLUSTER"); } //没有下一个簇
                // 计算所在扇区
                //println!("write at current_cluster = {}", current_cluster);
                current_sector = manager_reader.first_sector_of_cluster(current_cluster);
                //println!("write at current_sector = {}", current_sector);
                //let mut guess = String::new();
                //std::io::stdin().read_line(&mut guess).expect("Failed to read line");
            } else {
                current_sector += 1; //没读完一个簇，直接进入下一扇区
            }   
        }
        write_size
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self as *const _ as usize as *const u8,
                DIRENT_SZ,
            )
        }
    }
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(
                self as *mut _ as usize as *mut u8,
                DIRENT_SZ,
            )
        }
    }
}
