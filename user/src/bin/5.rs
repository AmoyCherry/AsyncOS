#![no_std]
#![no_main]
#![feature(global_asm)]
#![feature(llvm_asm)]
#![feature(asm)]
#[macro_use]
extern crate user_lib;
use user_lib::{*, test_lib::{Counter, write_cnt, set_cnter_zero, write_cnt_without_wake}};



extern crate alloc;


#[no_mangle]
pub fn main() -> i32 {

    //println!("[user5 satp: {:#x}] main: Hello world from user mode program!", satp_read());

    let start = get_time();
    test_for_user();
    let end = get_time();

    println!(">>> {}", end - start);

    //println!("[user5 satp: {:#x}] main: end", satp_read());
    shut_done();
    0
}

const COROUTINE_NUM: usize = 
4000
;

pub fn test_for_user(){

    use core::future::Future;
    use core::pin::Pin;
    use alloc::boxed::Box;

    unsafe{
        
        let coroutine_run: fn() = core::mem::transmute(COROUTINE_RUN_VA as usize);
        let add_coroutine_with_prio : fn(future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>> , usize) -> () 
            = core::mem::transmute(ADD_COROUTINE_WITH_PRIO_VA as usize);


        //add_coroutine_with_prio(Box::pin(compute()), 0);
        for i in 0..COROUTINE_NUM {
            let ct = Counter::new(i + 1);
            async fn test(_ct: Counter) {
                //println!("start {}", &_ct.cnt);
                _ct.await;
                write_cnt();
                //println!("done {}", &_ct.cnt);
            }
            add_coroutine_with_prio(Box::pin(test(ct)), 1);
        }




        // tid == test_num
        let ct = Counter::new(COROUTINE_NUM + 1);
        /* async fn end_test(_ct: Counter, addr: usize, test_num: usize) {
            for _ in 0..test_num {
                async fn begin_test() {  
                    set_cnter_zero();
                    write_cnt();
                }
                let add_coroutine_with_prio : fn(future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>> , usize) -> () 
                        = unsafe { core::mem::transmute(addr as usize) };
                add_coroutine_with_prio(Box::pin(begin_test()), 1);

                _ct.await;
                write_cnt_without_wake();
            }
        }
        add_coroutine_with_prio(Box::pin(end_test(ct, ADD_COROUTINE_WITH_PRIO_VA, TEST_NUM)), 0); */

        async fn end_test2(_ct: Counter) {
            //println!("start {}", &_ct.cnt);
            set_cnter_zero();
            write_cnt();
            //println!("done {}", &_ct.cnt);
        }
        add_coroutine_with_prio(Box::pin(end_test2(ct)), 0);
        
        coroutine_run();
    }

}