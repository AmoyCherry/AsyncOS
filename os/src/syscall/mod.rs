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
const SYSCALL_DO_YIELD: usize = 270;
const SYSCALL_V1: usize = 300;
const SYSCALL_GET_SYMBOL_ADDR: usize = 301;

const SYSCALL_GET_SATP: usize = 403;

const ASYNC_SYSCALL_READ: usize = 501;
const ASYNC_SYSCALL_WRITE: usize = 502;
pub const SYSCALL_SHUT_DONE: usize = 555;

mod fs;
pub mod process;


use fs::*;
pub use process::*;
use super::basic_rt::{add_user_task_with_priority, task::excutor::wakeup};


pub fn syscall(syscall_id: usize, args: [usize; 6]) -> isize {
    match syscall_id {
        SYSCALL_DUP=> sys_dup(args[0]),
        SYSCALL_OPEN => sys_open(args[0] as *const u8, args[1] as u32),
        SYSCALL_CLOSE => sys_close(args[0]),
        SYSCALL_PIPE => sys_pipe(args[0] as *mut usize),
        SYSCALL_READ => sys_read(args[0], args[1] as *const u8, args[2]),
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_GET_TIME => sys_get_time(),
        SYSCALL_GETPID => sys_getpid(),
        SYSCALL_FORK => sys_fork(),
        SYSCALL_EXEC => sys_exec(args[0] as *const u8, args[1] as *const usize),
        SYSCALL_WAITPID => sys_waitpid(args[0] as isize, args[1] as *mut i32),

        SYSCALL_GET_SATP => sys_get_satp(),
        ASYNC_SYSCALL_READ => async_sys_read(args[0], args[1] as *const u8, args[2], args[3], args[4], args[5]),
        ASYNC_SYSCALL_WRITE => async_sys_write(args[0], args[1] as *const u8, args[2], args[3], args[4], args[5]),
        SYSCALL_SHUT_DONE => sys_shut_done(),

        SYSCALL_DO_YIELD => sys_do_yield(args[0]),
        SYSCALL_GET_SYMBOL_ADDR=> sys_get_symbol_addr(args[0] as *const u8),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}

