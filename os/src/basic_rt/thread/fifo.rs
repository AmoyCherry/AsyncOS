use super::Thread;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use spin::Mutex;
use lazy_static::*;
use super::Tid;
use alloc::vec::Vec;

//old

pub struct ThreadManager {
    ready_queue: VecDeque<Thread>,
}

impl ThreadManager {
    pub fn new() -> Self {
        Self { ready_queue: VecDeque::new(), }
    }
    pub fn add(&mut self, thread: Thread) {
        self.ready_queue.push_back(thread);
    }
    pub fn fetch(&mut self) -> Option<Thread> {
        self.ready_queue.pop_front()
    }

    pub fn front(&mut self) -> Option<&Thread> {
        self.ready_queue.front()
    }
}

lazy_static! {
    pub static ref THREAD_MANAGER: Mutex<ThreadManager> = Mutex::new(ThreadManager::new());
}

pub fn add_thread(thread: Thread) {
    THREAD_MANAGER.lock().add(thread);
}

pub fn fetch_thread() -> Option<Thread> {
    THREAD_MANAGER.lock().fetch()
}

pub fn thread_space_id() -> usize {
    let x = THREAD_MANAGER.lock().ready_queue.front().unwrap().space_id;
    x
}