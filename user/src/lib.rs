#![no_std]
#![no_main]
#![feature(global_asm)]
#![feature(llvm_asm)]
#![feature(asm)]
#![feature(linkage)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![allow(unused)]
#![feature(naked_functions)]

#[macro_use]
pub mod console;
pub mod test_lib;

mod syscall;
mod lang_items;

extern crate alloc;
#[macro_use]
extern crate bitflags;

use syscall::*;
use buddy_system_allocator::LockedHeap;
use alloc::vec::Vec;

const USER_HEAP_SIZE: usize = 65536;

static mut HEAP_SPACE: [u8; USER_HEAP_SIZE] = [0; USER_HEAP_SIZE];

#[global_allocator]
static HEAP: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start(argc: usize, argv: usize) -> ! {

    let mut space_id :usize;
    unsafe {
        HEAP.lock()
            .init(HEAP_SPACE.as_ptr() as usize, USER_HEAP_SIZE);
    }
    let mut v: Vec<&'static str> = Vec::new();
    for i in 0..argc {
        let str_start = unsafe {
            ((argv + i * core::mem::size_of::<usize>()) as *const usize).read_volatile()
        };
        let len = (0usize..).find(|i| unsafe {
            ((str_start + *i) as *const u8).read_volatile() == 0
        }).unwrap();
        v.push(
            core::str::from_utf8(unsafe {
                core::slice::from_raw_parts(str_start as *const u8, len)
            }).unwrap()
        );
    }

    unsafe{asm!("mv {}, tp", out(reg) space_id, options(nomem, nostack));}

    // println!(" space_id : {:#x}", space_id);
    exit(main(argc, v.as_slice()));
}

#[linkage = "weak"]
#[no_mangle]
fn main(_argc: usize, _argv: &[&str]) -> i32 {
    panic!("Cannot find main!");
}

bitflags! {
    pub struct OpenFlags: u32 {
        const RDONLY = 0;
        const WRONLY = 1 << 0;
        const RDWR = 1 << 1;
        const CREATE = 1 << 9;
        const TRUNC = 1 << 10;
    }
}

pub fn dup(fd: usize) -> isize { sys_dup(fd) }
/// return fd
pub fn open(path: &str, flags: OpenFlags) -> isize { sys_open(path, flags.bits) }
pub fn close(fd: usize) -> isize { sys_close(fd) }
pub fn pipe(pipe_fd: &mut [usize]) -> isize { sys_pipe(pipe_fd) }
/// read the file with fd
pub fn read(fd: usize, buf: &mut [u8]) -> isize { sys_read(fd, buf) }
pub fn write(fd: usize, buf: &[u8]) -> isize { sys_write(fd, buf) }
pub fn exit(exit_code: i32) -> ! { sys_exit(exit_code); }
pub fn yield_() -> isize { sys_yield() }
pub fn get_time() -> isize { sys_get_time() }
pub fn getpid() -> isize { sys_getpid() }
pub fn fork() -> isize { sys_fork() }
pub fn exec(path: &str, args: &[*const u8]) -> isize { sys_exec(path, args) }
pub fn wait(exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(-1, exit_code as *mut _) {
            -2 => { yield_(); }
            // -1 or a real pid
            exit_pid => return exit_pid,
        }
    }
}

pub fn waitpid(pid: usize, exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(pid as isize, exit_code as *mut _) {
            -2 => { yield_(); }
            // -1 or a real pid
            exit_pid => return exit_pid,
        }
    }
}
pub fn sleep(period_ms: usize) {
    let start = sys_get_time();
    while sys_get_time() < start + period_ms as isize {
        sys_yield();
    }
}


pub fn get_symbol_addr(name: &str) -> usize{

    sys_get_symbol_addr(name) as usize
}




