mod heap_allocator;
mod address;
mod frame_allocator;
mod page_table;
mod memory_set;

use page_table::PTEFlags;
use address::VPNRange;
pub use address::{PhysAddr, VirtAddr, PhysPageNum, VirtPageNum, StepByOne};
pub use frame_allocator::{FrameTracker, frame_alloc, frame_dealloc,};
pub use page_table::{
    PageTable,
    PageTableEntry,
    translated_byte_buffer,
    translated_str,
    translated_ref,
    translated_refmut,
    UserBuffer,
    UserBufferIterator,
    translated_context,
};
pub use memory_set::{
    MemorySet, 
    KERNEL_SPACE, 
    MapPermission, 
    kernel_token, 
    CBQ_BASE_PA, USER_CBQ_VA, SYS_CBQ_VA, USER_CBQ_VEC_PA, MAX_USER, USER_ENVIR_PA};
pub use memory_set::remap_test;
// pub use memory_set::SPACE_ID_SATP;

pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
    KERNEL_SPACE.lock().activate();
}


pub fn init_kernel_space(){
    KERNEL_SPACE.lock().activate();
}