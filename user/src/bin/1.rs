#![no_std]
#![no_main]
#![feature(global_asm)]
#![feature(llvm_asm)]
#![feature(asm)]
#[macro_use]
extern crate user_lib;

use alloc::string::String;
use user_lib::{*, syscall::{AsyncCall, ASYNC_SYSCALL_READ, ASYNC_SYSCALL_WRITE}};

extern crate alloc;


#[no_mangle]
pub fn main() -> i32 {

    println!("[user1 satp: {:#x}] main: Hello world from user mode program!", satp_read());

    init_coroutine_interface();

    test_for_user();

    println!("[user1 satp: {:#x}] main: end, time", satp_read());

    0
}

pub const DATA: &str = "abcdefgabcdefgabasdfasdafsdfasdaabcdefgabcdegabasdfasdfsdasabcdefgabcdefgabasdfasdfsdfasdfasdfsadfaasgabcdefgabasdfasdfsdfasdas";
pub const BUFFER_SIZE: usize = 128;

#[allow(unused_mut)]
pub fn test_for_user(){

    use core::future::Future;
    use core::pin::Pin;
    use alloc::boxed::Box;

    unsafe{
        
        let coroutine_run: fn() = core::mem::transmute(COROUTINE_RUN_VA as usize);
        let add_coroutine_with_prio: fn(future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>> , usize) -> ()
            = core::mem::transmute(ADD_COROUTINE_WITH_PRIO_VA as usize);

        let mut pipe_fd = [0usize; 2];
        pipe(&mut pipe_fd);

        async fn work1(fd: usize) {
            let mut buffer = [0u8; BUFFER_SIZE];
            let ac = AsyncCall::new(ASYNC_SYSCALL_READ, fd, buffer.as_ptr() as usize, buffer.len(), 0, 1, 33);
            ac.await;
            println!("[user] read {:#?}", buffer);
        }
        add_coroutine_with_prio(Box::pin(work1(pipe_fd[0])), 0);
        

        for i in 0..1 {
            
            async fn work2(fd: usize, id: usize) {
                //let mut buffer = [0u8; 32];
                let str = DATA;
                let ac = AsyncCall::new(ASYNC_SYSCALL_WRITE, fd, str.as_bytes().as_ptr() as usize, str.len(), id, 1, 33);
                ac.await;
                //close(fd)
                println!("[user] write {} ok", id);
            }
            add_coroutine_with_prio(Box::pin(work2(pipe_fd[1], i + 1)), 0);
        }

        

        
        coroutine_run();
    }

}