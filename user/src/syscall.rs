const SYSCALL_DUP: usize = 24;
const SYSCALL_OPEN: usize = 56;
const SYSCALL_CLOSE: usize = 57;
const SYSCALL_PIPE: usize = 59;
const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_GET_TIME: usize = 169;
const SYSCALL_GETPID: usize = 172;
const SYSCALL_FORK: usize = 220;
const SYSCALL_EXEC: usize = 221;
const SYSCALL_WAITPID: usize = 260;
const SYSCALL_V1: usize = 300;
const SYSCALL_GET_SYMBOL_ADDR: usize = 301;

pub const SYSCALL_GET_SATP: usize = 403;
pub const ASYNC_SYSCALL_READ: usize = 501;
pub const ASYNC_SYSCALL_WRITE: usize = 502;

fn syscall(id: usize, args: [usize; 6]) -> isize {
    let mut ret: isize;
    unsafe {
        llvm_asm!("ecall"
            : "={x10}" (ret)
            : "{x10}" (args[0]), "{x11}" (args[1]), "{x12}" (args[2]), "{x13}" (args[3]), "{x14}" (args[4]), "{x15}" (args[5]), "{x17}" (id)
            : "memory"
            : "volatile"
        );
    }
    ret
}

pub fn sys_dup(fd: usize) -> isize {
    syscall(SYSCALL_DUP, [fd, 0, 0, 0, 0, 0])
}

pub fn sys_open(path: &str, flags: u32) -> isize {
    syscall(SYSCALL_OPEN, [path.as_ptr() as usize, flags as usize, 0, 0, 0, 0])
}

pub fn sys_close(fd: usize) -> isize {
    syscall(SYSCALL_CLOSE, [fd, 0, 0, 0, 0, 0])
}

pub fn sys_pipe(pipe: &mut [usize]) -> isize {
    syscall(SYSCALL_PIPE, [pipe.as_mut_ptr() as usize, 0, 0, 0, 0, 0])
}

pub fn sys_read(fd: usize, buffer: &mut [u8]) -> isize {
    syscall(SYSCALL_READ, [fd, buffer.as_mut_ptr() as usize, buffer.len(), 0, 0, 0])
}

pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
    syscall(SYSCALL_WRITE, [fd, buffer.as_ptr() as usize, buffer.len(), 0, 0, 0])
}

pub fn sys_exit(exit_code: i32) -> ! {
    syscall(SYSCALL_EXIT, [exit_code as usize, 0, 0, 0, 0, 0]);
    panic!("sys_exit never returns!");
}

pub fn sys_yield() -> isize {
    syscall(SYSCALL_YIELD, [0, 0, 0, 0, 0, 0])
}

pub fn sys_get_time() -> isize {
    syscall(SYSCALL_GET_TIME, [0, 0, 0, 0, 0, 0])
}

pub fn sys_getpid() -> isize {
    syscall(SYSCALL_GETPID, [0, 0, 0, 0, 0, 0])
}

pub fn sys_fork() -> isize {
    syscall(SYSCALL_FORK, [0, 0, 0, 0, 0, 0])
}

pub fn sys_exec(path: &str, args: &[*const u8]) -> isize {
    syscall(SYSCALL_EXEC, [path.as_ptr() as usize, args.as_ptr() as usize, 0, 0, 0, 0])
}

pub fn sys_waitpid(pid: isize, exit_code: *mut i32) -> isize {
    syscall(SYSCALL_WAITPID, [pid as usize, exit_code as usize, 0, 0, 0, 0])
}



pub fn sys_get_symbol_addr(name: &str) -> isize {
    syscall(SYSCALL_GET_SYMBOL_ADDR, [name.as_ptr() as usize, 0, 0, 0, 0, 0])
}

pub fn sys_get_satp() -> isize {
    syscall(SYSCALL_GET_SATP, [0, 0, 0, 0, 0, 0])
}

pub fn async_sys_read(fd: usize, buffer_ptr: usize, buffer_len: usize, tid: usize, space_id: usize, k: usize) -> isize {
    syscall(ASYNC_SYSCALL_READ, [fd, buffer_ptr, buffer_len, tid, space_id, k])
}

pub fn async_sys_write(fd: usize, buffer_ptr: usize, buffer_len: usize, tid: usize, space_id: usize, rtid: usize) -> isize {
    syscall(ASYNC_SYSCALL_WRITE, [fd, buffer_ptr, buffer_len, tid, space_id, rtid])
}


pub struct AsyncCall {
    call_type: usize,

    fd: usize,
    buffer_ptr: usize,
    buffer_len: usize,
    tid: usize,
    space_id: usize,
    rtid: usize,

    cnt: usize,
}

impl AsyncCall {
    pub fn new(_type: usize, _fd: usize, _bt: usize, _bl: usize, _tid: usize, _sid: usize, _rtid: usize) -> Self {
        Self { call_type: _type, fd: _fd, buffer_ptr: _bt, buffer_len: _bl, tid: _tid, space_id: _sid, rtid: _rtid, cnt: 0}
    }
}

use futures::Future;
use spin::Mutex;
use core::{task::{Context, Poll}, pin::Pin};

use crate::{CHECK_CALLBACK};

impl Future for AsyncCall {
    type Output = (());

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // submit async task to kernel and return immediately
        if self.cnt == 0 {
            match self.call_type {
                ASYNC_SYSCALL_READ => async_sys_read(self.fd, self.buffer_ptr, self.buffer_len, self.tid, self.space_id, self.rtid),
                ASYNC_SYSCALL_WRITE => async_sys_write(self.fd, self.buffer_ptr, self.buffer_len, self.tid, self.space_id, self.rtid),
                _ => {0},
            };
            self.cnt += 1;
        }

        unsafe {
            let check_callback: fn(t: usize) -> bool = core::mem::transmute(CHECK_CALLBACK as usize );
            if check_callback(self.tid) {
                return Poll::Ready(());
            } else {
                return Poll::Pending;
            }
        }
    }
}
