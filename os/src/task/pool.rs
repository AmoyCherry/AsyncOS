use alloc::{collections::BTreeSet, sync::Arc};
use lazy_static::*;
use spin::Mutex;

use super::{manager::TaskManager, task::TaskControlBlock};

pub struct TaskPool {
    pub scheduler: TaskManager,
    pub sleeping_tasks: BTreeSet<Arc<TaskControlBlock>>,
}

lazy_static! {
    pub static ref TASK_POOL: Mutex<TaskPool> = Mutex::new(TaskPool::new());
}

impl TaskPool {
    pub fn new() -> Self {
        Self {
            scheduler: TaskManager::new(),
            sleeping_tasks: BTreeSet::new(),
        }
    }

    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.scheduler.add(task);
    }

    #[allow(unused)]
    pub fn remove(&mut self, task: Arc<TaskControlBlock>) {
        self.scheduler.remove(&task);
    }

    #[allow(unused)]
    pub fn wake(&mut self, task: Arc<TaskControlBlock>) {
        self.sleeping_tasks.remove(&task);
        self.scheduler.add(task);
    }

    #[allow(unused)]
    pub fn sleep(&mut self, task: Arc<TaskControlBlock>) {
        self.scheduler.remove(&task);
        self.sleeping_tasks.insert(task);
    }

    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.scheduler.fetch()
    }

    #[allow(unused)]
    pub fn prioritize(&mut self, pid: usize) {
        self.scheduler.prioritize(pid);
    }
}

pub fn add_task(task: Arc<TaskControlBlock>) {
    let token = task.acquire_inner_lock().memory_set.token();
    // debug!("task pid: {}, satp: {:#x} added to pool", task.pid.0, token);
    TASK_POOL.lock().add(task);
}

pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    TASK_POOL.lock().fetch()
}

#[allow(unused)]
pub fn prioritize_task(pid: usize) {
    TASK_POOL.lock().prioritize(pid);
}
