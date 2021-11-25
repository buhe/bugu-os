use alloc::{sync::Arc, vec::Vec};

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
        let (fat1_sec,fat2_sec,offset) = self.calculate_pos(cluster);
        //println!("fat1_sec={} offset = {}", fat1_sec, offset);
        let fat1_rs = get_block_cache(fat1_sec as usize, block_device.clone())
        .lock()
        .read(offset as usize,|&next_cluster: &u32|{
            next_cluster
        });
        let fat2_rs = get_block_cache(fat2_sec as usize, block_device.clone())
        .lock()
        .read(offset as usize,|&next_cluster: &u32|{
            next_cluster
        });
        if fat1_rs == BAD_CLUSTER {
            if fat2_rs == BAD_CLUSTER {
                0
            } else {
                fat2_rs & 0x0FFFFFFF
            }
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