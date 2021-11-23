#![no_std]
mod block_dev;

pub const BLOCK_SZ: usize = 512;
pub use block_dev::BlockDevice;