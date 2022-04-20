use alloc::alloc::{alloc, dealloc, Layout};

// #[derive(Debug)]
#[derive(Clone)]
pub struct UserStack(usize);

const STACK_SIZE: usize = 0x8000;

impl UserStack {
    pub fn new() -> UserStack {
        let bottom =
            unsafe {
                alloc(Layout::from_size_align(STACK_SIZE, STACK_SIZE).unwrap())
            } as usize;
        UserStack(bottom)
    }

    pub fn top(&self) -> usize {
        self.0 + STACK_SIZE
    }
}


use core::fmt::{self, Debug, Formatter};
impl Debug for UserStack {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("UserStack:{:#x}", self.0))
    }
}

impl Drop for UserStack {
    fn drop(&mut self) {
        unsafe {
            dealloc(
                self.0 as _,
                Layout::from_size_align(STACK_SIZE, STACK_SIZE).unwrap()
            );
        }
    }
}