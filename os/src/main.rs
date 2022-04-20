#![no_std]
#![no_main]
#![feature(global_asm)]
#![feature(asm)]
#![feature(llvm_asm)]
#![feature(panic_info_message)]
#![feature(naked_functions)]
// #![feature(const_in_array_repeat_expressions)]
#![feature(alloc_error_handler)]
#![allow(unused)]

use alloc::alloc::dealloc;
// use std::println;

extern crate alloc;

#[macro_use]
extern crate bitflags;

#[macro_use]
mod console;
mod lang_items;
mod sbi;
mod syscall;
mod trap;
mod config;
mod task;
mod timer;
mod mm;
mod fs;
mod drivers;
mod loader;
mod lkm;


global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));


use core::sync::atomic::{AtomicBool, Ordering};
use core::hint::{spin_loop, self};


fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| {
        unsafe { (a as *mut u8).write_volatile(0) }
    });
}

static AP_CAN_INIT: AtomicBool = AtomicBool::new(false);

#[no_mangle]
pub fn rust_main(hart_id: usize) -> ! {
    
    if hart_id == 0{
        clear_bss();
        mm::init();
        println!("[kernel] Hello, world!");
        mm::remap_test();
        trap::init();
        trap::enable_timer_interrupt();
        timer::set_next_trigger();
        info!("loader list app");
        fs::list_apps();
        // test_for_kernel(0);
        debug!("trying to add user test");
        // task::add_initproc();
        task::add_user_test();

        send_ipi();
        AP_CAN_INIT.store(true, Ordering::Relaxed);

    }else{
        init_other_cpu();
    }

    println_hart!("Hello", hart_id);
    
    
    println_hart!("run user task", hart_id);
    task::run_tasks();
    
    panic!("Unreachable in rust_main!");
}

pub fn init_other_cpu(){

    let hart_id = hart_id();

    if hart_id != 0 {

        while !AP_CAN_INIT.load(Ordering::Relaxed) {
            hint::spin_loop();
        }

        others_main();
        
        unsafe {
            let satp: usize;
            let sp: usize;
            asm!("csrr {}, satp", out(reg) satp);
            println_hart!("init done", hart_id);
        }
    }
}

pub fn others_main(){
    mm::init_kernel_space();
    trap::init();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
}



pub fn send_ipi(){
    let hart_id = hart_id();
    for i in 1..1 {
        debug!("[hart {}] Start hart[{}]", hart_id, i);
        let mask: usize = 1 << i;
        sbi::send_ipi(&mask as *const _ as usize);
    }
}


pub fn hart_id() -> usize {
    let hart_id: usize;
    unsafe {
        asm!("mv {}, tp", out(reg) hart_id);
    }
    hart_id
}

pub fn test_for_kernel(base: usize){
    let init_environment_addr = lkm::get_symbol_addr_from_elf("basic_rt", "init_environment");
    println!("init_environment at {:#x?}", init_environment_addr);
    

    let init_cpu_addr = lkm::get_symbol_addr_from_elf("basic_rt", "init_cpu_test");
    println!("init_cpu at {:#x?}", init_cpu_addr);

    let cpu_run_addr = lkm::get_symbol_addr_from_elf("basic_rt", "cpu_run");
    println!("cpu_run at {:#x?}", cpu_run_addr);

    
    let add_user_task_with_priority_addr = lkm::get_symbol_addr_from_elf("basic_rt", "add_user_task_with_priority");
    println!("add_user_task at {:#x?}", add_user_task_with_priority_addr);
    
    use spin::Mutex;
    use woke::waker_ref;
    use core::future::Future;
    use core::pin::Pin;
    use alloc::boxed::Box;


    unsafe{
        
        let init_environment: fn() = core::mem::transmute(init_environment_addr as usize + base);
        
        let init_cpu: fn()= core::mem::transmute(init_cpu_addr as usize + base);
        
        // let add_user_task: fn() = core::mem::transmute(add_user_task_addr as usize + 0x87);
        let cpu_run: fn() = core::mem::transmute(cpu_run_addr as usize + base);



        println!("init_environment");
        init_environment();
        
        
        println!("init_cpu");
        init_cpu();

        async fn test(x: i32) {
            crate::println!("{}", x);
        }
        println!("test task addr :{:#x?}", test as usize);

        println!("add_task");
        let add_task_with_priority : fn(future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>> , Option<usize>) -> () = unsafe {
            core::mem::transmute(add_user_task_with_priority_addr as usize + base)
        };

        add_task_with_priority(Box::pin(test(666)), Some(0));


        let cpu_run_addr = lkm::get_symbol_addr_from_elf("basic_rt", "cpu_run");
        unsafe{
            let cpu_run: fn() = core::mem::transmute(cpu_run_addr as usize);
            println!("cpu_run");
            cpu_run();
        }
    }

}

