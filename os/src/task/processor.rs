use super::TaskControlBlock;
use super::{fetch_task, TaskStatus};
use super::TaskContext;
use super::__switch;
use super::add_task;

use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::Mutex;
use core::cell::RefCell;
use crate::trap::TrapContext;
use crate::config::CPU_NUM;
use lazy_static::*;
// use core::arch::asm;
pub struct Processor {
    inner: RefCell<ProcessorInner>,
}

impl Default for Processor {
    fn default() -> Self {
        Self {
            inner: RefCell::new(ProcessorInner {
                current: None,
                idle_task_cx: TaskContext::zero_init(),
                idle_task_cx_ptr: 0,
            }),
        }
    }
}


unsafe impl Sync for Processor {}

struct ProcessorInner {
    current: Option<Arc<TaskControlBlock>>,
    idle_task_cx: TaskContext,
    idle_task_cx_ptr: usize,
}

impl Processor {
    pub fn new() -> Self {
        Self {
            inner: RefCell::new(ProcessorInner {
                current: None,
                idle_task_cx: TaskContext::zero_init(),
                idle_task_cx_ptr: 0,
            }),
        }
    }

    fn get_idle_task_cx_ptr(&self) -> *mut TaskContext {
        let mut inner = self.inner.borrow_mut();
        &mut inner.idle_task_cx as *mut TaskContext
    }


    fn get_idle_task_cx_ptr2(&self) -> *const usize {
        let inner = self.inner.borrow();
        &inner.idle_task_cx_ptr as *const usize
    }

    pub fn run_next(&self, task: Arc<TaskControlBlock>){
        
        let idle_task_cx_ptr = self.get_idle_task_cx_ptr();
        // acquire
        let mut task_inner = task.acquire_inner_lock();
        let next_task_cx_ptr = task_inner.get_task_cx_ptr();
        task_inner.task_status = TaskStatus::Running(hart_id());
        
        let task_cx = unsafe { &*next_task_cx_ptr };

        // release
        drop(task_inner);
        self.inner.borrow_mut().current = Some(task);

        // println_hart!("switching idle:{:#x?} to:{:#x?}", hart_id(), idle_task_cx_ptr, next_task_cx_ptr );
        unsafe {
            __switch(idle_task_cx_ptr, next_task_cx_ptr);
        }
    }

    #[no_mangle]
    fn suspend_current(&self) {
        
        // info!("[suspend current]");
        if let Some(task) = take_current_task() {

            // info!("task pid: {} suspend", task.pid.0);

            // ---- hold current PCB lock
            let mut task_inner = task.acquire_inner_lock();
            // Change status to Ready
            task_inner.task_status = TaskStatus::Ready;

            drop(task_inner);
            // ---- release current PCB lock

            // push back to ready queue.
            add_task(task);
        }
    }

    #[no_mangle]
    pub fn run(&self) {
        static CNT: Mutex<usize> = Mutex::new(0);
        loop {
            let task = fetch_task();
                
                match task {
                    Some(task) => {
                        unsafe { riscv::asm::sfence_vma_all()}
                        self.run_next(task);
                        // println_hart!("idel----", hart_id());
                        self.suspend_current();

                    }
                    None => {
                        //info!("all user process finished!");
                        
                        /* let c = *CNT.lock();
                        if c == 0 {
                            *CNT.lock() += 1;
                            super::add_initproc();
                        } */
                    }
                }
        }
    }
    
    pub fn take_current(&self) -> Option<Arc<TaskControlBlock>> {
        self.inner.borrow_mut().current.take()
    }
    pub fn take_current_mut(&self) -> Option<Arc<TaskControlBlock>> {
        self.inner.borrow_mut().current.take()
    }

    pub fn current(&self) -> Option<Arc<TaskControlBlock>> {
        self.inner.borrow().current.as_ref().map(|task| Arc::clone(task))
    }
}

lazy_static! {
    pub static ref PROCESSORS: [Processor; CPU_NUM] = Default::default();
}


pub fn hart_id() -> usize {
    let hart_id: usize;
    unsafe {
        asm!("mv {}, tp", out(reg) hart_id);
    }
    hart_id
}

pub fn run_tasks() {
    debug!("run_tasks");
    PROCESSORS[hart_id()].run();
}

pub fn take_current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSORS[hart_id()].take_current()
}

pub fn current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSORS[hart_id()].current()
}

#[allow(unused)]
pub fn current_tasks() -> Vec<Option<Arc<TaskControlBlock>>> {
    PROCESSORS
        .iter()
        .map(|processor| processor.current())
        .collect()
}

pub fn current_user_token() -> usize {
    let task = current_task().unwrap();
    let token = task.acquire_inner_lock().get_user_token();
    token
}

pub fn current_trap_cx() -> &'static mut TrapContext {
    current_task().unwrap().acquire_inner_lock().get_trap_cx()
}

#[no_mangle]
pub fn schedule(switched_task_cx_ptr: *mut TaskContext) {
    let idle_task_cx_ptr = PROCESSORS[hart_id()].get_idle_task_cx_ptr();
    unsafe {
        __switch(switched_task_cx_ptr, idle_task_cx_ptr);
    }
}
