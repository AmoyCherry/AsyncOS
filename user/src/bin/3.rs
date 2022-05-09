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

    println!("[user3] main: Hello world from user mode program!");

    test_for_user();

    println!("[user3] main: end");

    0
}

pub fn hart_id() -> usize {
    let hart_id: usize;
    unsafe {
        asm!("mv {}, tp", out(reg) hart_id);
    }
    hart_id
}

/* pub fn get_satp() -> usize {
    let satp: usize;
    unsafe {
        asm!("mv {}, satp", out(reg) satp)
    }
    satp
} */

pub fn satp_read() -> usize {
    let ret: usize;
    unsafe {llvm_asm!("csrr $0, satp":"=r"(ret):::"volatile");}
    ret
}



pub fn test_for_user(){

    // let base = 0x86000000 - 0x87000000;
    let init_environment_addr = get_symbol_addr("init_environment\0") as usize - 0x1000000;
    let init_cpu_addr = get_symbol_addr("init_cpu_test\0") as usize - 0x1000000;
    let cpu_run_addr = get_symbol_addr("cpu_run\0") as usize    - 0x1000000;
    let add_user_task_with_priority_addr = get_symbol_addr("add_user_task_with_priority\0") as usize   - 0x1000000;
    // println!("init_environment at {:#x?}", init_environment_addr);
    // println!("init_cpu at {:#x?}", init_cpu_addr);
    // println!("cpu_run at {:#x?}", cpu_run_addr);
    // println!("add_user_task at {:#x?}", add_user_task_with_priority_addr);

    use spin::Mutex;
    use woke::waker_ref;
    use core::future::Future;
    use core::pin::Pin;
    use alloc::boxed::Box;


    unsafe{
        
        let init_environment: fn() = core::mem::transmute(init_environment_addr as usize );
        
        let init_cpu: fn()= core::mem::transmute(init_cpu_addr as usize);
        
        let cpu_run: fn() = core::mem::transmute(cpu_run_addr as usize);

        let add_task_with_priority : fn(future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>> , usize) -> () = unsafe {
            core::mem::transmute(add_user_task_with_priority_addr as usize)
        };

        // println!("init_environment");
        init_environment();
        
        // println!("init_cpu");
        init_cpu();

        async fn test(x: i32) {
            let test_inner = async { 
                let mut c: usize = 0;
                for i in 0..1000_000_000 {
                    c += i % 6;
                }
                println!("[hart {}] [user3] calc plus, sum = {}", hart_id(), c); 
            };
            test_inner.await;
            println!("[hart {}] [user3] await done", hart_id()); 
        }
        add_task_with_priority(Box::pin(test(666)), 0);
        // println!("test task addr :{:#x?}", test as usize);
        // println!("add_task");

        async fn test_num(prio: usize, no: usize) {
            println!("[hart {}] [user3] prio: {}, No.: {}, total: 6", hart_id(), prio, no);
        }

        for i in 0..5{
            for j in 1..=6 {
                add_task_with_priority(Box::pin(test_num(5 - i, j)), 5 - i as usize);
            }
        }
        // println!("cpu_run");
        cpu_run();
    }

}