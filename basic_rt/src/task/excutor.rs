#![no_std]

use core::task::{Context, Poll, Waker};
use alloc::boxed::Box;
use alloc::task::Wake;
use lazy_static::*;

use alloc::collections::btree_map::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::collections::VecDeque;
use spin::Mutex;

use crate::bitmap::{update_user_bitmap, PRIO_NUM};
use crate::user_task::{TaskId, UserTask};
use crate::println;

pub struct Excutor {
    pub tasks: BTreeMap<TaskId, Arc<UserTask>>,
    pub task_queue: Arc<Mutex<Box<TaskQueue>>>,
    pub waker_cache: BTreeMap<TaskId, Arc<Waker>>,
    pub task_num: usize,
}

impl Excutor {
    pub fn new() -> Self {
        Excutor {
            tasks: BTreeMap::new(),
            task_queue: Arc::new(Mutex::new(Box::new(TaskQueue::new()))),
            waker_cache: BTreeMap::new(),
            task_num: 0,
        }
    }

    pub fn add_task(&mut self, task: Arc<UserTask>, prio: usize) {
        let tid = task.id;
        self.task_queue.lock().add_tid(tid, prio);
        self.tasks.insert(tid, task);
        self.task_num += 1;
    }

    pub fn del_task(&mut self, tid: &TaskId) {
        //self.tasks.remove(&tid);
        //self.waker_cache.remove(&tid);
        self.task_num -= 1;
    }

    pub fn get_waker(&mut self, tid: TaskId, prio: usize) -> Arc<Waker> {
        let que = self.task_queue.clone();
        self.waker_cache
                        .entry(tid)
                        .or_insert_with(|| Arc::new(TaskWaker::new(tid, prio, que)))
                        .clone()
    }

    pub fn pop(&mut self) -> Option<TaskId> {
        self.task_queue.lock().pop_tid()
    }

    pub fn get_task(&self, tid: &TaskId) -> Option<Arc<UserTask>> {
        if let Some(ret) = self.tasks.get(tid) {
            return Some(ret.clone());
        } else {
            return None;
        }
    }

    pub fn is_empty(&self) -> bool { self.task_num == 0 }
}


// ===================== WAKER =====================
pub struct TaskWaker {
    tid: TaskId,
    prio: usize,
    queue: Arc<Mutex<Box<TaskQueue>>>,
}

impl TaskWaker {
    pub fn new(id: TaskId, p: usize, q: Arc<Mutex<Box<TaskQueue>>>) -> Waker {
        Waker::from(
            Arc::new(TaskWaker {
                    tid: id,
                    prio: p,
                    queue: q,
                }
            )
        )
    }

    fn wake_task(&self) {
        println!("------------------ wake task ------------------");
        self.queue.lock().add_tid(self.tid, self.prio);
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}



// ===================== QUEUE =====================
pub struct TaskQueue {
    queue: Vec<VecDeque<TaskId>>,
}

impl TaskQueue {
    pub fn new() -> Self {
        let q = (0..PRIO_NUM).map(|_| VecDeque::new() ).collect::<Vec<VecDeque<TaskId>>>();
        Self {
            queue: q,
        }
    }

    pub fn add_tid(&mut self,  tid: TaskId, prio: usize) {
        self.queue[prio].push_back(tid);
        // update bitmap
        update_user_bitmap(prio, true);
    }

    pub fn pop_tid(&mut self) -> Option<TaskId> {
        for i in 0..PRIO_NUM {
            if self.queue[i].len() == 0 { continue; }

            let ret = self.queue[i].pop_front();
            // update bitmap
            if self.queue[i].len() == 0 { update_user_bitmap(i, false); }
            return ret;
        }

        None
    }
}