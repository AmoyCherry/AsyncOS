pub mod context;
pub mod thread;
pub mod user_stack;

use thread::Thread;

pub mod fifo;

use alloc::boxed::Box;
use alloc::{sync::Arc};
use lazy_static::*;
use spin::Mutex;


use fifo::THREAD_MANAGER;
pub type Tid = usize;


pub mod scheduler;
pub mod thread_pool;
pub mod processor;

use processor::Processor;
use scheduler::{FifoScheduler, RRScheduler};
use scheduler::*;
use thread_pool::ThreadPool;

pub static CPU : Processor = Processor::new();


use crate::task::thread_main;
use crate::task::add_user_task_with_priority;
use crate::println;

pub fn init() {
    // 使用 Fifo Scheduler
    // let scheduler = FifoScheduler::new();
    // // 新建线程池
    // let thread_pool =Arc::new(ThreadPool::new(100, scheduler));

    // // 新建idle ，其入口为 Processor::idle_main
    // let idle = Thread::new_box_thread(Processor::idle_main as usize, &CPU as *const Processor as usize);

    // // 初始化 CPU
    // CPU.init(idle, thread_pool);

    // // 新建一个thread_main加入线程池
    
    // CPU.add_thread({
    //     let thread = Thread::new_box_thread(thread_main as usize, 1);
    //     thread
    // });
}





pub fn add_to_thread_pool(addr: usize, space_id:usize) {
    THREAD_MANAGER.lock().add(
        {
            let thread = Thread::new_thread(addr, space_id);
            thread
        }
    );
}




#[no_mangle]
pub fn init_cpu_test(){
    // println!("init_cpu_test");
    let scheduler = RRScheduler::new(50);
    
    // 新建线程池
    let thread_pool = Box::new(ThreadPool::new(10, scheduler));

    // 新建idle ，其入口为 Processor::idle_main
    let idle = Thread::new_box_thread(Processor::idle_main as usize, &CPU as *const Processor as usize);


    CPU.init(idle, thread_pool);

    CPU.add_thread(
        {
            let thread = Thread::new_box_thread(thread_main as usize, 1);
            thread
        }
    );

    // println!("add thread_main done");

}

#[no_mangle]
pub fn cpu_run(){
    CPU.run();
}




#[no_mangle]
pub fn thread_init() {
    println!("scheduler init");


    let scheduler = RRScheduler::new(50);
    
    // 新建线程池
    let thread_pool = Box::new(ThreadPool::new(10, scheduler));

    // 新建idle ，其入口为 Processor::idle_main
    let idle = Thread::new_box_thread(Processor::idle_main as usize, &CPU as *const Processor as usize);


    CPU.init(idle, thread_pool);

    CPU.add_thread(
        {
            let thread = Thread::new_box_thread(thread_main as usize, 1);
            thread
        }
    );

    println!("add thread_main done");
    println!("add_task");
    async fn foo_0(x:usize){
        println!("priority 0 task --- {:?}", x);
    }

    async fn foo_1(x:usize){
        println!("priority 1 task --- {:?}", x);
    }

    async fn foo_2(x:usize){
        println!("priority 2 task --- {:?}", x);
    }

    add_user_task_with_priority(Box::pin(foo_0(666)), 0);
    add_user_task_with_priority(Box::pin(foo_1(666)), 1);
    add_user_task_with_priority(Box::pin(foo_2(666)), 2);

    add_user_task_with_priority(Box::pin(foo_0(777)), 0);
    add_user_task_with_priority(Box::pin(foo_1(777)), 1);
    add_user_task_with_priority(Box::pin(foo_2(777)), 2);
    
    println!("scheduler cpu run");
    CPU.run();
}
