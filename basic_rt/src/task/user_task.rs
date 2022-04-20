#![no_std]

use alloc::boxed::Box;
use alloc::sync::Arc;
use core::future::Future;
use core::pin::Pin;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};


use core::task::{Context, Poll};
use spin::Mutex;

use super::reactor::{Reactor, REACTOR};


#[derive(Eq, PartialEq, Debug, Clone, Copy, Hash, Ord, PartialOrd)]
pub struct TaskId(usize);

impl TaskId {
    pub(crate) fn generate() -> TaskId {
        // 任务编号计数器，任务编号自增
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        if id > usize::MAX / 2 {
            // TODO: 不让系统 Panic
            panic!("too many tasks!")
        }
        TaskId(id)
    }
}



//Task包装协程
pub struct UserTask{
    // 任务编号
    pub id: TaskId,
    // future
    pub future: Mutex<Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>>, 
    // reactor
    pub reactor: Arc<Mutex<Box<Reactor>>>,
    
    pub prio: usize,
}

impl UserTask{
    //创建一个协程
    pub fn spawn(future: Mutex<Pin<Box<dyn Future<Output=()> + 'static + Send + Sync>>>, p: usize) -> Self{
        UserTask{
            id: TaskId::generate(),
            future: future,
            reactor: REACTOR.clone(),
            prio: p,
        }
    }

    pub fn do_wake(self: &Arc<Self>) {
        self.reactor.lock().wake(self.id);
    }
}


impl Future for UserTask {
    type Output = usize;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut r = self.reactor.lock();
        if r.is_ready(self.id) {
            //r.finish_task(self.id);
            Poll::Ready(self.id.0)
        } else if !r.contains_task(self.id) {
            r.add_task(self.id);
            Poll::Pending
        } else {
            let mut f = self.future.lock();
            match f.as_mut().poll(cx) {
                Poll::Ready(_) => {
                    Poll::Ready(0)
                },
                Poll::Pending => {
                    r.register(self.id); // fixme
                    Poll::Pending
                }
            }
    
        }
    }
}