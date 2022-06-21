#![no_std]


pub mod excutor;
pub mod bitmap;
pub mod cbq;
pub mod user_task;

use alloc::string::{String, ToString};

use excutor::Excutor;
use cbq::{CBQueue, CBQ_VA, wakeup_all};
use user_task::UserTask;

use alloc::sync::Arc;
use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;


use crate::syscall::sys_get_time;


use core::task::{Context, Poll};
use spin::Mutex;
use woke::waker_ref;
use lazy_static::*;
use crate::console::print;
//use crate::{println, print, CPU};

// use crate::println;

lazy_static!{
    pub static ref EXCUTOR: Arc<Mutex<Box<Excutor>>> = Arc::new(Mutex::new(Box::new(Excutor::new())));
    
}

// 应该使用thread_local
pub static mut CUR_COROUTINE: usize = 0;

#[no_mangle]
pub fn thread_main_ex() {
    println!(" > > > > > > > [kernel] thread_main < < < < < < < ");

    let mut cbq = unsafe { &mut *(CBQ_VA as *mut CBQueue) };

    loop {

        //if !cbq.is_empty() { 
        //    let mut tids = cbq.pop();
        //    wakeup_all(&mut tids); 
        //}

        let tid;
        let task;
        let waker;
        // get EXCUTOR lock
        {
            let mut ex = EXCUTOR.lock();
            if ex.is_empty() { continue; }

            let tid_wrap = ex.pop();
            if tid_wrap.is_none() { continue; }
            tid = tid_wrap.unwrap();

            let top = ex.get_task(&tid);
            if top.is_none() { continue; }
            task = top.unwrap().clone();
            
            waker = ex.get_waker(tid, task.prio);
        }
        unsafe { CUR_COROUTINE = tid.get_val(); }                                
        // creat Context
        let mut context = Context::from_waker(&*waker);
        match task.future.lock().as_mut().poll(&mut context) {
            Poll::Pending => { }
            Poll::Ready(()) => {
                // remove task
                EXCUTOR.lock().del_task(&tid);
            }
        }; 

        

        //if check_bitmap_should_yield() { sys_yield(); }
    }

    //println!("stats mean: {} , stats population_stddev {} ", stats.mean(), stats.population_stddev());

}


// add_user_task_with_priority(future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, prio: usize)
#[no_mangle]
pub fn add_user_task_with_priority(future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, prio: usize){
    let task = Arc::new(UserTask::spawn(Mutex::new(future), prio));
    EXCUTOR.lock().add_task(task, prio);
}

