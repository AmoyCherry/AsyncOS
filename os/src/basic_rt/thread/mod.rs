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


use crate::basic_rt::{cbq::CBQueue, thread_main_ex, add_user_task_with_priority, EXCUTOR};
use crate::{println};

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




/// 在用户态程序中获取地址直接调用
#[no_mangle]
pub fn init_cpu_test() {
    CBQueue::init();
    
    let scheduler = RRScheduler::new(50);
    
    // 新建线程池
    let thread_pool = Box::new(ThreadPool::new(10, scheduler));

    // 新建idle ，其入口为 Processor::idle_main
    let idle = Thread::new_box_thread(Processor::idle_main as usize, &CPU as *const Processor as usize);
    
    //println!("[basic] init cpu test");
    CPU.init(idle, thread_pool);
    // 初始化线程池时先创建一个线程, 启动线程执行器之后可以直接使用
    CPU.add_thread(
        {
            let thread = Thread::new_box_thread(thread_main_ex as usize, 1);
            thread
        }
    );

}

/// 启动协程执行器
/// 
/// 现阶段, 每个进程只使用一个线程工作, 所以此处的协程执行器直接作为函数调用
/// 
/// 未来的多线程的工作方式为: 调用函数idle_main, 在某些特定的条件之下, 如没有可用的线程, 但存在可以执行的协程, 
/// 则可以创建线程去执行协程
#[no_mangle]
pub fn cpu_run(){
    thread_main_ex();
}

