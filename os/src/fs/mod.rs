mod pipe;
mod stdio;
mod inode;

use crate::mm::UserBuffer;

pub use core::{future::Future, pin::Pin};

pub trait File : Send + Sync {
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
    fn read(&self, buf: UserBuffer) -> usize;
    fn write(&self, buf: UserBuffer) -> usize;
    
    fn aread(&self, buf: UserBuffer, space_id: usize, tid: usize, k: usize) -> Pin<Box<dyn Future<Output = ()> + 'static + Send + Sync>>;
}

use alloc::{boxed::Box, sync::Arc};
pub use pipe::{Pipe, make_pipe};
pub use stdio::{Stdin, Stdout};
pub use inode::{OSInode, open_file, OpenFlags, list_apps};