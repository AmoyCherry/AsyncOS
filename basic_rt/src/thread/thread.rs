use super::user_stack::UserStack;
use super::context::Context;


use alloc::boxed::Box;
use alloc::sync::Arc;

#[derive(Debug, Clone)]
pub struct Thread {
    pub context: Context,
    pub ustack: UserStack,
    pub space_id: usize,
}

pub struct ThreadInner {
    pub context: Context,
}
impl Thread {
    pub fn new_idle() -> Thread {
        unsafe {
            Thread {
                context: Context::null(),
                ustack: UserStack::new(),
                space_id: 0,
            }
        }
    }
    pub fn new_thread(entry: usize, arg: usize) -> Thread {
        unsafe {
            let ustack_ = UserStack::new();
            Thread {
                context: Context::new_thread_context(entry, arg, ustack_.top()),
                ustack: ustack_,
                space_id: arg
            }
        }
    }
    pub fn switch_to(&mut self, target: &mut Thread) {
       unsafe {
           self.context.switch(&mut target.context);
       }
    }


    pub fn new_box_thread(entry: usize, arg: usize) -> Box<Thread> {
        unsafe {
            let ustack_ = UserStack::new();
            Box::new(
                Thread {
                    context: Context::new_thread_context(entry, arg, ustack_.top()),
                    ustack: ustack_,
                    space_id: arg
                }
            )
        }
    }
}