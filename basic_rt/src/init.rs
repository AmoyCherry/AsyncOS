
#![no_std]
#![no_main]

#![allow(unused)]

use spin::Mutex;
use core::usize::MAX;

use crate::task::{USER_TASK_QUEUE, user_task::UserTask};


use crate::thread::*;
use crate::println;


extern crate alloc;
use alloc::boxed::Box;


async fn foo(x:usize){
    println!("{:?}", x);
}

pub fn task(){
    let mut queue = USER_TASK_QUEUE.lock();
    for i in 0..100_000_000 {
        queue.add_task(UserTask::spawn(Mutex::new(Box::pin( foo(i) )), 0) , Some(0));
        if i % 10_000_000 == 0 {
            println!("count {:?}", i);
        }
    }
    drop(queue);
    // runtime::run();
}


pub fn fooo(){
    println!("---");
}

pub fn thread(){
    for i in 0..1000_000{
        add_to_thread_pool(fooo as usize,0);
        if i % 100_000 == 0 { println!("count {:?}", i); }
    }
}



fn main(){
    crate::thread::init();
    crate::thread::init_cpu_test();
    panic!("!!");
}


use core::{mem::MaybeUninit, ptr::NonNull};
const USER_HEAP_SIZE: usize = 65536;

static mut HEAP_SPACE: [u8; USER_HEAP_SIZE] = [0; USER_HEAP_SIZE];

const HEAP_SIZE: usize = 1024 * 1024;
static HEAP_MEMORY: MaybeUninit<[u8; HEAP_SIZE]> = core::mem::MaybeUninit::uninit();

use buddy_system_allocator::LockedHeap;



#[global_allocator]
static HEAP: LockedHeap = LockedHeap::empty();


#[no_mangle]
unsafe fn init_environment() {
    let heap_start = HEAP_MEMORY.as_ptr() as usize;
    HEAP.lock().init(heap_start, HEAP_SIZE);
}


#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}
