
pub mod bitmap;
pub mod queue;
pub mod reactor;
pub mod user_task;

use bitmap::check_bitmap_should_yield;
use queue::UserTaskQueue;
use user_task::UserTask;
use reactor::TaskState;


use alloc::sync::Arc;
use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;

use core::task::{Context, Poll};
use spin::Mutex;
use woke::waker_ref;
use lazy_static::*;
use crate::println;
use crate::syscall::sys_yield;



// use crate::println;

lazy_static! {
    pub static ref USER_TASK_QUEUE: Arc<Mutex<Box<UserTaskQueue>>> =
        Arc::new(
            Mutex::new(
                Box::new(
                    UserTaskQueue::new(),
                )
            )
        );
}

pub static mut NEXT_PRIO_NOT_EMPTY: bool = true;

pub fn hart_id() -> usize {
    let hart_id: usize;
    unsafe {
        asm!("mv {}, tp", out(reg) hart_id);
    }
    hart_id
}


#[no_mangle]
pub fn thread_main() {

    //println!(" > > > > > > > thread_main < < < < < < < ");
    loop {
        //let mut queue = USER_TASK_QUEUE.lock();
        let task = USER_TASK_QUEUE.lock().peek_task();
        //println!("thread_main running, coroutine queue is not empty -> {:?}", !task.is_none());

        match task {
            // have any task
            Some(task) => {
                let mywaker = task.clone();
                let waker = waker_ref(&mywaker);
                let mut context = Context::from_waker(&*waker);

                let r = task.reactor.clone();
                let mut r = r.lock();

                // 如果是not ready, 直接插回
                let prio = task.prio;
                if r.contains_task(task.id) && *r.get_task(task.id).unwrap() == TaskState::NotReady {
                    USER_TASK_QUEUE.lock().queue[prio].push_back(task);
                    continue;
                }

                /* match task.future.lock().as_mut().poll(&mut context) {
                    Poll::Ready(_) => {  }
                    Poll::Pending => {
                        queue.add_task(task, task.)
                    }
                } */

                if r.is_ready(task.id) {
                    //let mut future = task.future.lock();
                    let ret = task.future.lock().as_mut().poll(&mut context);
                    if ret == Poll::Pending {
                        r.add_task(task.id);
                        USER_TASK_QUEUE.lock().queue[prio].push_back(task); 
                    } else {
                        r.finish_task(task.id);
                    }
                    /* match  {
                        Poll::Ready(_) => {
                            // 任务完成
                            r.finish_task(task.id);
                        }
                        Poll::Pending => {
                            r.add_task(task.id);
                            USER_TASK_QUEUE.lock().queue[prio].push_back(task);
                        }
                    } */
                } else if r.contains_task(task.id) {
                    r.register(task.id);
                } else {
                    let ret = task.future.lock().as_mut().poll(&mut context);
                    if ret == Poll::Pending {
                        r.park(task.id);
                        USER_TASK_QUEUE.lock().queue[prio].push_back(task);
                    }
                    /* let mut future = task.future.lock();
                    match future.as_mut().poll(&mut context) {
                        Poll::Ready(_) => {
                            // 任务完成
                            // println!("task completed");
                        }
                        Poll::Pending => {
                            USER_TASK_QUEUE.lock().queue[prio].push_back(task);
                            r.park(task.id);
                        }
                    } */
                }

                if unsafe { !NEXT_PRIO_NOT_EMPTY } && check_bitmap_should_yield() { 
                    //println!("!!!!!! sys_yield, switch to another space !!!!!!");
                    sys_yield(); 
                }
            }
            None => {
                println!("no task");
                // let mut queue = USER_TASK_QUEUE.lock();
                // if queue.is_all_empty(){
                //     crate::sys_exit(0);
                // }
                crate::sys_exit(0);
                break;

            }
                
        }

        // crate::sys_exit(0);
    }
}

#[no_mangle]
pub fn add_user_task(future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>){
    let mut queue = USER_TASK_QUEUE.lock();
    let task = UserTask::spawn(Mutex::new(future), 0);
    queue.add_task(task , Some(0));
    drop(queue);
}


#[no_mangle]
pub fn add_user_task_with_priority(future: Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>, priority: usize){
    let mut queue = USER_TASK_QUEUE.lock();
    let task = UserTask::spawn(Mutex::new(future), priority);
    queue.add_task(task , Some(priority));
    drop(queue);
}

