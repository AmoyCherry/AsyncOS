use alloc::sync::Arc;
use alloc::{vec::Vec, vec};
use alloc::collections::{VecDeque, BTreeSet};
use spin::Mutex;
use lazy_static::*;

use super::{EXCUTOR, user_task::TaskId};


// ===================== CALLBACK QUEUE =====================

pub const CBQ_VA: usize = 0x8741_3000;
pub const V_PTR: usize = 0x8741_4000;
pub const CAP: usize = 500;

// VA 直接转换为 Vec
pub struct CBQueue {
    v_ptr: usize,
    head: usize,
    tail: usize,
    cap: usize,
}

impl CBQueue {
    /// clear tids
    pub fn init() {
        unsafe {
            let mut cbq = &mut *(CBQ_VA as *mut CBQueue);
            cbq.head = 0;
            cbq.cap = CAP;
            cbq.tail = 0;
            cbq.v_ptr = V_PTR;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.tail - self.head == 0
    }

    pub fn add(addr: usize, tid: usize, vptr: usize) {
        unsafe {
            let mut cbq = unsafe { &mut *(addr as *mut CBQueue) };
            cbq._add(tid, vptr);
        }     
    }

    pub fn _add(&mut self, tid: usize, vptr: usize) {
        unsafe {
            let ptr = vptr as *mut usize;
            *ptr.add(self.tail) = tid;
            self.tail += 1;
            //println!("[kernel] add callback tid = {}, len = {}", tid, self.tail);
        }
    }

    pub fn pop(&mut self) -> Vec<usize> {
        unsafe {

            let mut ret = Vec::new();
            let ptr = self.v_ptr as *mut usize;
            let tail = self.tail;
            for i in self.head..tail {
                ret.push(*ptr.add(i));
            }
            self.head = tail;

            ret   
        }
    }
}

pub fn wakeup_all(tids: &mut Vec<usize>) {
    let mut ex = EXCUTOR.lock();
    let len = tids.len();
    for i in 0..len {
        let tid = TaskId::get_tid_by_usize(tids[i]);
        CBTID.lock().add(tid.get_val());
        ex.wake_coroutine(tid);
    }
}


lazy_static! {
    pub static ref CBTID: Arc<Mutex<CBTid>> = Arc::new(Mutex::new(CBTid::new()));
}

pub struct CBTid {
    tids: Vec<bool>,
}

impl CBTid {
    pub fn new() -> Self {
        Self { tids: vec![false; 2000], }
    }

    pub fn add(&mut self, t: usize) {
        self.tids[t] = true;
    }

    pub fn contains_tid(&mut self, t: usize) -> bool {
        if self.tids[t] {
            self.tids[t] = false;
            return true;
        }
        false
    }
}
