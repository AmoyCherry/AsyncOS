use super::TaskControlBlock;
use alloc::collections::{VecDeque, BTreeSet};
use alloc::sync::Arc;

use lazy_static::*;
use spin::Mutex;

#[no_mangle]
lazy_static! {
    /// Update the highest priority process PIDS on each timer
    pub static ref PRIO_PIDS: Arc<Mutex<BTreeSet<usize>>> = Arc::new(Mutex::new(BTreeSet::new()));
}

pub struct TaskManager {
    ready_queue: VecDeque<Arc<TaskControlBlock>>,
}

/// A simple FIFO scheduler.
impl TaskManager {
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
    
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push_back(task);
    }

    #[allow(unused)]
    pub fn remove(&mut self, task: &Arc<TaskControlBlock>) {
        for (idx, task_item) in self.ready_queue.iter().enumerate() {
            if task_item.pid.0 == task.pid.0 {
                self.ready_queue.remove(idx);
                break;
            }
        }
    }

    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        let n = self.ready_queue.len();
        if n == 0 { return None; }

        if PRIO_PIDS.lock().len() == 0 { return self.ready_queue.pop_front(); }
        // May need to concern affinity
        let mut peek;
        let mut cnt = 0;
        loop {
           peek = self.ready_queue.pop_front().unwrap();
           if PRIO_PIDS.lock().contains(&peek.pid.0) { 
               //debug!("[kernel] next PID: {}", peek.pid.0);
               PRIO_PIDS.lock().remove(&peek.pid.0);
               return Some(peek); 
            }
           self.ready_queue.push_back(peek);
           cnt += 1;
           if cnt >= n { break; }
        }

        self.ready_queue.pop_front()
    }

    #[allow(unused)]
    pub fn prioritize(&mut self, pid: usize) {
        let q = &mut self.ready_queue;
        if q.is_empty() || q.len() == 1 {
            return;
        }
        let front_pid = q.front().unwrap().pid.0;
        if front_pid == pid {
            debug!("[Taskmgr] Task {} already at front", pid);

            return;
        }
        q.rotate_left(1);
        while {
            let f_pid = q.front().unwrap().pid.0;
            f_pid != pid && f_pid != front_pid
        } {
            q.rotate_left(1);
        }
        if q.front().unwrap().pid.0 == pid {
            debug!("[Taskmgr] Prioritized task {}", pid);
        }
    }
}
