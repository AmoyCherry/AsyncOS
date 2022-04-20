use super::thread_pool::ThreadPool;
use super::thread::Thread;
use super::scheduler::Scheduler;
use core::cell::UnsafeCell;
use alloc::boxed::Box;

//有了线程和进程，现在，我们再抽象出「处理器」来存放和管理线程池。
//同时，也需要存放和管理目前正在执行的线程（即中断前执行的线程，因为操作系统在工作时是处于中断、异常或系统调用服务之中）。
use super::Tid;

use crate::println;

use crate::task::USER_TASK_QUEUE;
use crate::task::thread_main;

unsafe impl Sync for Processor {}
pub struct Processor {
    pub inner: UnsafeCell<Option<ProcessorInner>>,
}
pub struct ProcessorInner {
    pub pool: Box<ThreadPool>,
    idle: Box<Thread>,
    current: Option<(Tid, Box<Thread>)>,
}


impl Processor {
    // 新建一个空的 Processor
    pub const fn new() -> Processor {
        Processor {
            inner: UnsafeCell::new(None),
        }
    }

    // 传入 idle 线程，以及线程池进行初始化
    pub fn init(&self, idle: Box<Thread>, pool: Box<ThreadPool>) {
        unsafe {
            *self.inner.get() = Some(
                ProcessorInner {
                    pool,
                    idle,
                    current: None,
                }
            );

        }
    }

    // 内部可变性：获取包裹的值的可变引用
    fn inner(&self) -> &mut ProcessorInner {
        unsafe { &mut *self.inner.get() }
            .as_mut()
            .expect("Processor is not initialized!")
    }

    pub fn thread_pool(&self) -> &mut ThreadPool {
        &mut *self.inner().pool
    }

    // 通过线程池新增线程
    pub fn add_thread(&self, thread: Box<Thread>) {
        self.inner().pool.add(thread);
    }

    pub fn idle_main(&self) -> ! {
        let inner = self.inner();
        loop {
            // 如果从线程池中获取到一个可运行线程
            if let Some(thread) = inner.pool.acquire() {

                // 将自身的正在运行线程设置为刚刚获取到的线程
                inner.current = Some(thread);

                // 从正在运行的线程 idle 切换到刚刚获取到的线程
                println!("\n>>>> will switch_to thread {} in idle_main!", inner.current.as_mut().unwrap().0);

                // 切换到刚刚获取到的线程
                inner.idle.switch_to(
                    &mut *inner.current.as_mut().unwrap().1
                );

                // 上个线程时间耗尽，切换回调度线程 idle
                println!("<<<< switch_back to idle in idle_main!");

                // 此时 current 还保存着上个线程
                let (tid, thread) = inner.current.take().unwrap();
                
                // 通知线程池这个线程需要将资源交还出去
                inner.pool.retrieve(tid, thread);
            }
            // 如果现在并无任何可运行线程.则检查协程队列是否为空
            else {

                let mut queue = USER_TASK_QUEUE.lock();

                if queue.is_all_empty() {
                    println!("finish task exit");
                    crate::sys_exit(0);
                } else {

                    //如果线程列表为空，但任务队列不空，创建一个线程
                    self.add_thread(        
                        {
                            let thread = Thread::new_box_thread(crate::task::thread_main as usize, 1);
                            thread
                        }
                    )
                }
                drop(queue);
                
            }
        }
    }

    pub fn tick(&self) {
        // let inner = self.inner();
        // if !inner.current.is_none() {
        //     // 如果当前有在运行线程
        //     if inner.pool.tick() {
        //         // 如果返回true, 表示当前运行线程时间耗尽，需要被调度出去

        //         // 我们要进入 idle 线程了，因此必须关闭异步中断
        //         // 我们可没保证 switch_to 前后 sstatus 寄存器不变
        //         // 因此必须手动保存
        //         let flags = disable_and_store();

        //         // 切换到 idle 线程进行调度
        //         inner.current
        //             .as_mut()
        //             .unwrap()
        //             .1
        //             .switch_to(&mut inner.idle);

        //         // 之后某个时候又从 idle 线程切换回来
        //         // 恢复 sstatus 寄存器继续中断处理
        //         restore(flags);
        //     }
        // }
    }


    pub fn exit(&self, code: usize) -> ! {
        // 由于要切换到 idle 线程，必须先关闭时钟中断
        // disable_and_store();


        // 由于自己正在执行，可以通过这种方式获取自身的 tid
        let inner = self.inner();
        let tid = inner.current.as_ref().unwrap().0;

        // 通知线程池这个线程退出啦！
        inner.pool.exit(tid);
        println!("thread {} exited, exit code = {}", tid, code);

        // 切换到 idle 线程决定下一个运行哪个线程
        inner.current
            .as_mut()
            .unwrap()
            .1
            .switch_to(&mut inner.idle);

        loop {}
    }

	pub fn run(&self) {
        Thread::new_idle().switch_to(&mut self.inner().idle);
    }
}



