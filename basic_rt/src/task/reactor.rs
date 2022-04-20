#![no_std]

use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;

use alloc::collections::btree_map::BTreeMap;

use core::mem;
use spin::Mutex;
use woke::waker_ref;
use lazy_static::*;

use super::user_task::{TaskId, UserTask};


#[derive(PartialEq, Eq)]
pub enum TaskState {
    Ready,
    NotReady,
    Finish,
}

#[no_mangle]
lazy_static! {
    pub static ref REACTOR: Arc<Mutex<Box<Reactor>>> = Reactor::new();
}

pub struct Reactor {
    pub tasks: BTreeMap<TaskId, TaskState>,
}

impl Reactor {
    pub(crate) fn new() -> Arc<Mutex<Box<Self>>> {
        let reactor = Arc::new(Mutex::new(Box::new(Reactor {
            tasks: BTreeMap::new(),
        })));
        reactor
    }

    pub(crate) fn wake(&mut self, id: TaskId) {
        let state = self.tasks.get_mut(&id).unwrap();
        match mem::replace(state, TaskState::Ready) {
            TaskState::NotReady => (),
            TaskState::Finish => panic!("Called 'wake' twice on task: {:?}", id),
            _ => unreachable!()
        }
    }

    pub(crate) fn register(&mut self, id: TaskId) {
        if self.tasks.insert(id, TaskState::NotReady).is_some() {
            panic!("Tried to insert a task with id: '{:?}', twice!", id);
        }
    }

    pub(crate) fn is_ready(&self, id: TaskId) -> bool {
        self.tasks.get(&id).map(|state| match state {
            TaskState::Ready => true,
            _ => false,
        }).unwrap_or(false)
    }

    pub(crate) fn get_task(&self, task_id: TaskId) -> Option<&TaskState> {
        self.tasks.get(&task_id)
    }

    pub(crate) fn get_task_mut(&mut self, task_id: TaskId) -> Option<&mut TaskState> {
        self.tasks.get_mut(&task_id)
    }

    pub(crate) fn add_task(&mut self, task_id: TaskId) -> Option<TaskState> {
        self.tasks.insert(task_id, TaskState::NotReady)
    }

    pub(crate) fn contains_task(&self, task_id: TaskId) -> bool {
        self.tasks.contains_key(&task_id)
    }

    pub(crate) fn is_finish(&self, task_id: TaskId) -> bool {
        self.tasks.get(&task_id).map(|state| match state {
            TaskState::Finish => true,
            _ => false,
        }).unwrap_or(false)
    }

    pub(crate) fn finish_task(&mut self, task_id: TaskId) {
        self.tasks.insert(task_id, TaskState::Finish);
    }

    pub(crate) fn remove_task(&mut self, task_id: TaskId) -> Option<TaskState>{
        self.tasks.remove(&task_id)
    }

    /// 1. let the coroutine' state = NotReady,
    /// 2. todo! modify bitmap
    pub(crate) fn park(&mut self, task_id: TaskId) {
        let state = self.tasks.get_mut(&task_id).unwrap();
        match mem::replace(state, TaskState::NotReady) {
            TaskState::NotReady => (),
            TaskState::Finish => panic!("Called 'wake' twice on task: {:?}", task_id),
            _ => unreachable!()
        }
    }
}




impl woke::Woke for UserTask {
    fn wake_by_ref(task: &Arc<Self>) {
        task.do_wake()
    }
}

impl Drop for UserTask {
    fn drop(&mut self) {
        let r = self.reactor.clone();
        let mut r = r.lock();
        r.remove_task(self.id);
    }
}




//传递用户协程队列
pub fn diliver_to_kernel(){
    //to do
}


//检查kernel提供给用户的调度信息
pub fn check_kernel_clue(){
    //to do
    // println!("checking clue.");
}