use super::TaskContext;
// use core::arch::global_asm;

global_asm!(include_str!("switch.S"));

extern "C" {
    #[no_mangle]
    pub fn __switch(
        current_task_cx_ptr: *mut TaskContext, 
        next_task_cx_ptr: *mut TaskContext
    );
}
