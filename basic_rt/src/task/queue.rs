#![no_std]

use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::collections::VecDeque;

use super::user_task::UserTask;
use super::bitmap::{PRIO_NUM,update_user_bitmap};
use super::NEXT_PRIO_NOT_EMPTY;


pub struct UserTaskQueue {
    pub queue: Vec<VecDeque<Arc<UserTask>>>,
}



//用户协程队列
impl UserTaskQueue {
    pub fn new() -> Self {
        let queue = (0..PRIO_NUM).map(|_| VecDeque::new() ).collect::<Vec<VecDeque<Arc<UserTask>>>>();
        UserTaskQueue {
            queue,
        }
    }

    pub fn add_task(&mut self, task: UserTask, priority: Option<usize>) {
        let p = priority.unwrap_or(0);
        self.queue[p].push_front(Arc::new(task));
        update_user_bitmap(p, true);
    }

    pub fn add_arc_task(&mut self, task: Arc<UserTask>, priority: usize) {
        self.queue[priority].push_back(task);
        update_user_bitmap(priority, true);
    }

    pub fn peek_task(&mut self) -> Option<Arc<UserTask>> {

        for i in 0..PRIO_NUM {
            if self.queue[i].len() !=0 {

                let x =  self.queue[i].pop_front();

                if self.queue[i].len() == 0 { 
                    update_user_bitmap(i, false);
                    if i < PRIO_NUM - 1 && self.queue[i + 1].len() != 0 { unsafe { NEXT_PRIO_NOT_EMPTY = true; } }
                    else { unsafe { NEXT_PRIO_NOT_EMPTY = false; } }
                }

                return x
            }
        }

        None
    }
    
    pub fn is_all_empty(&self) -> bool {

        for i in 0..self.queue.len() {
            if !self.queue[i].is_empty() {
                return false
            }
        }
        return  true;
    }

}