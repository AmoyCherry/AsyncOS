use super::scheduler::Scheduler;
use super::thread::*;
use crate::alloc::{
    vec::Vec,
    boxed::Box,
};
use super::Tid;


pub type ExitCode = usize;
#[derive(Clone)]
pub enum Status {
    // 就绪：可以运行，但是要等到 CPU 的资源分配给它
    Ready,
    // 正在运行
    Running(Tid),
    // 睡眠：当前被阻塞，要满足某些条件才能继续运行
    Sleeping,
    // 退出：该线程执行完毕并退出
    Exited(ExitCode),
}


//调度算法 Scheduler 只管理 Tid ，和线程并没有关系。因此，我们使用线程池 ThreadPool 来给线程和 Tid 建立联系，将 Scheduler 的 Tid 调度变成线程调度。 
//事实上，每个线程刚被创建时并没有一个 Tid ，这是线程池给线程分配的。
struct ThreadInfo {
    // 占据这个位置的线程当前运行状态
    status: Status,
    // 占据这个位置的线程
    thread: Option<Box<Thread>>,
}

pub struct ThreadPool {
    threads: Vec<Option<ThreadInfo>>,
    scheduler: Box<dyn Scheduler>,
}


//作为一个线程池，需要实现调度相关的一系列操作：

// alloc_tid：为新线程分配一个新的 Tid
// add：添加一个可立即开始运行的线程
// acquire：从线程池中取一个线程开始运行
// retrieve：让当前线程交出 CPU 资源
// tick：时钟中断时查看当前所运行线程是否要切换出去
// exit：退出线程

//立即可以开始运行的线程池.(初始化完毕的)
impl ThreadPool {
    pub fn new(size: usize ,scheduler: impl Scheduler) -> Self {
        ThreadPool {
            threads: new_vec_default(size),
            scheduler: Box::new(scheduler),
        }
    }

    fn alloc_tid(&self) -> Tid {
        for (i, info) in self.threads.iter().enumerate() {
            if info.is_none() {
                return i;
            }
        }
        panic!("alloc tid failed!");
    }

    // 向调度器加入一个可立即开始运行的线程
    // 线程状态 Uninitialized -> Ready
    pub fn add(&mut self, _thread: Box<Thread>) {
        let tid = self.alloc_tid();
        self.threads[tid] = Some(
            ThreadInfo {
                status: Status::Ready,
                thread: Some(_thread),
            }
        );
        self.scheduler.push(tid);
    }
    // 从线程池中取一个线程开始运行
    // 线程状态 Ready -> Running
    pub fn acquire(&mut self) -> Option<(Tid, Box<Thread>)> {

        if let Some(tid) = self.scheduler.pop() {
            let mut thread_info = self.threads[tid].as_mut().expect("thread not exist!");
            thread_info.status = Status::Running(tid);
            return Some((tid, thread_info.thread.take().expect("thread not exist!")));
        }
        else {
            return None;
        }
    }

    // 这个线程已运行了太长时间或者已运行结束，需要交出CPU资源
    // 但是要提醒线程池它仍需要分配 CPU 资源
    pub fn retrieve(&mut self, tid: Tid, thread: Box<Thread>) {
        if self.threads[tid].is_none() {
            return;
        }
        let mut thread_info = self.threads[tid].as_mut().expect("thread not exist!");       
        thread_info.thread = Some(thread);
        if let Status::Running(_) = thread_info.status {
            thread_info.status = Status::Ready;
            self.scheduler.push(tid);
        }
    }
    // Scheduler 的简单包装：时钟中断时查看当前所运行线程是否要切换出去
    pub fn tick(&mut self) -> bool {
        let ret = self.scheduler.tick();
        ret
    }
    
    // 这个线程已经退出了，线程状态 Running -> Exited
    pub fn exit(&mut self, tid: Tid) {
        self.threads[tid] = None;
        self.scheduler.exit(tid);
    }
}




fn new_vec_default<T: Default>(size: usize) -> Vec<T> {
    let mut vec = Vec::new();
    vec.resize_with(size, Default::default);
    vec
}