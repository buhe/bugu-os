#![no_std]

extern crate alloc;

mod block_dev;
// mod layout;
// mod efs;
// mod bitmap;
mod vfs;
mod block_cache;
mod fat;
mod fat_layout;

pub const BLOCK_SZ: usize = 512;
pub use block_dev::BlockDevice;
// pub use efs::EasyFileSystem;
pub use fat::FatFileSystem;
pub use vfs::Inode;
pub use fat_layout::*;
// use bitmap::Bitmap;
use block_cache::get_block_cache;
pub use fat::ROOT_DIR;

pub fn clone_into_array<A, T>(slice: &[T]) -> A
where
    A: Default + AsMut<[T]>,
    T: Clone,
{
    let mut a = Default::default();
    <A as AsMut<[T]>>::as_mut(&mut a).clone_from_slice(slice);
    a
}