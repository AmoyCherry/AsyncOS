
pub mod bitmap;
//pub mod queue;
pub mod user_task;
pub mod excutor;

use bitmap::check_bitmap_should_yield;
//use queue::UserTaskQueue;
use user_task::UserTask;
//use reactor::TaskState;
use excutor::{Excutor,TaskWaker};


use alloc::sync::Arc;
use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;

use core::task::{Context, Poll};
use spin::Mutex;
use woke::waker_ref;
use lazy_static::*;
use crate::{println, CPU};
use crate::syscall::sys_yield;



// use crate::println;

lazy_static!{
    pub static ref EXCUTOR: Arc<Mutex<Box<Excutor>>> = Arc::new(Mutex::new(Box::new(Excutor::new())));
    
}


#[no_mangle]
pub fn thread_main_ex() {
    println!(" > > > > > > > thread_main < < < < < < < ");

    loop {
        let tid;
        let task;
        let waker;
        // get EXCUTOR lock
        {
            let mut ex = EXCUTOR.lock();
            if ex.is_empty() { break; }

            let tid_wrap = ex.pop();
            if tid_wrap.is_none() { continue; }
            tid = tid_wrap.unwrap();

            let top = ex.get_task(&tid);
            if top.is_none() { continue; }
            task = top.unwrap().clone();

            waker = ex.get_waker(tid, task.prio);
        }
                                        
        // creat Context
        let mut context = Context::from_waker(&*waker);

        match task.future.lock().as_mut().poll(&mut context) {
            Poll::Pending => { }
            Poll::Ready(()) => {
                // remove task
                EXCUTOR.lock().del_task(&tid);
            }
        }; 

        if check_bitmap_should_yield() { sys_yield(); }
    }
}


#[no_mangle]
pub fn add_user_task_with_priority(future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, prio: usize){
    let task = Arc::new(UserTask::spawn(Mutex::new(future), prio));
    EXCUTOR.lock().add_task(task, prio);
}

