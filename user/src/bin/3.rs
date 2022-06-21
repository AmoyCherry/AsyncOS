#![no_std]
#![no_main]
#![feature(global_asm)]
#![feature(llvm_asm)]
#![feature(asm)]
#[macro_use]
extern crate user_lib;
use user_lib::*;

extern crate alloc;


#[no_mangle]
pub fn main() -> i32 {

    println!("[user3 satp: {:#x}] main: Hello world from user mode program!, time: {}", satp_read(), get_time());

    init_coroutine_interface();

    test_for_user();

    println!("[user3 satp: {:#x}] main: end, time: {}", satp_read(), get_time());

    0
}


pub fn test_for_user(){

    use core::future::Future;
    use core::pin::Pin;
    use alloc::boxed::Box;

    unsafe{
        
        let coroutine_run: fn() = core::mem::transmute(COROUTINE_RUN_VA as usize);
        let add_coroutine_with_prio : fn(future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>> , usize) -> () 
            = core::mem::transmute(ADD_COROUTINE_WITH_PRIO_VA as usize);

        //add_coroutine_with_prio(Box::pin(compute()), 0);

        async fn test_num(prio: usize, no: usize) {
            println!("[hart {}] [user3 satp: {:#x}] prio: {}, No.: {}, total: 6", hart_id(), satp_read(), prio, no);
        }

        for i in 0..5{
            for j in 1..=6 {
                add_coroutine_with_prio(Box::pin(test_num(5 - i, j)), 5 - i as usize);
            }
        }
        // println!("cpu_run");
        coroutine_run();
    }

}