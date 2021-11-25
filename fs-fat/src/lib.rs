#![no_std]

extern crate alloc;

mod block_dev;
mod layout;
mod efs;
mod bitmap;
mod vfs;
mod block_cache;
mod fat;
mod fat_layout;

pub const BLOCK_SZ: usize = 512;
pub use block_dev::BlockDevice;
pub use efs::EasyFileSystem;
pub use fat::FatFileSystem;
pub use vfs::Inode;
use layout::*;
use bitmap::Bitmap;
use block_cache::get_block_cache;