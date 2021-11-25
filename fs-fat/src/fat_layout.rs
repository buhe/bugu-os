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