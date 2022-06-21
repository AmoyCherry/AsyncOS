use crate::basic_rt::excutor::WRMAP;
use crate::config::PAGE_SIZE;
use crate::mm::{
    UserBuffer,
    translated_byte_buffer,
    translated_refmut,
    translated_str,
};

use crate::basic_rt::{task::{cbq::CBQueue, excutor::wakeup}, add_user_task_with_priority};
use crate::mm::{CBQ_BASE_PA, USER_CBQ_VEC_PA};
use crate::task::{current_user_token, current_task, TaskControlBlock};
use crate::fs::{make_pipe, OpenFlags, open_file};
use alloc::sync::Arc;


use core::future::Future;
use alloc::boxed::Box;
use core::pin::Pin;
use spin::Mutex;
use core::{task::{Context, Poll}};

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.acquire_inner_lock();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        if !file.writable() {
            return -1;
        }
        let file = file.clone();
        // release Task lock manually to avoid deadlock
        drop(inner);
        file.write(
            UserBuffer::new(translated_byte_buffer(token, buf, len))
        ) as isize

    } else {
        -1
    }
}

fn write(fd: usize, buf: *const u8, len: usize, token: usize, task: Arc<TaskControlBlock>) -> isize {
    let inner = task.acquire_inner_lock();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        if !file.writable() {
            return -1;
        }
        let file = file.clone();
        // release Task lock manually to avoid deadlock
        drop(inner);
        file.write(
            UserBuffer::new(translated_byte_buffer(token, buf, len))
        ) as isize
    } else {
        -1
    }
}

pub fn async_sys_write(fd: usize, buf: *const u8, len: usize, tid: usize, space_id: usize, rtid: usize) -> isize {
    /* async fn work(_fd: usize, __buf: usize, _len: usize, _tid: usize, _space_id: usize, _rtid: usize, token: usize, task: Arc<TaskControlBlock>) {
        let _buf = __buf as *const u8;
        write(_fd, _buf, _len, token, task);
        sys_close(_fd);

        let rid = WRMAP.lock().get_rid(_rtid);
        if rid.is_some() { wakeup(rid.unwrap()); }
        

        let addr = CBQ_BASE_PA + PAGE_SIZE * _space_id;
        let vptr = USER_CBQ_VEC_PA + PAGE_SIZE * _space_id;
        CBQueue::add(addr, _tid, vptr);
    }
    
    let token = current_user_token();
    let task = current_task().unwrap();
    add_user_task_with_priority(Box::pin(work(fd, buf as usize, len, tid, space_id, rtid, token, task)), 0);
    0 */

    sys_write(fd, buf, len);
    sys_close(fd);
    let rid = WRMAP.lock().get_rid(rtid);
    if rid.is_some() { wakeup(rid.unwrap()); }
    //println!("write done rtid = {}", rtid);
    0
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.acquire_inner_lock();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        if !file.readable() {
            return -1;
        }
        // release Task lock manually to avoid deadlock
        drop(inner);
        file.read(
            UserBuffer::new(translated_byte_buffer(token, buf, len))
        ) as isize
    } else {
        -1
    }
}


pub fn async_sys_read(fd: usize, buf: *const u8, len: usize, tid: usize, space_id: usize, k: usize) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.acquire_inner_lock();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        if !file.readable() {
            return -1;
        }
        // release Task lock manually to avoid deadlock
        drop(inner);
        //file.read(
        //    UserBuffer::new(translated_byte_buffer(token, buf, len))
        //) as isize
        let work = file.aread(UserBuffer::new(translated_byte_buffer(token, buf, len)), space_id, tid, k);
        add_user_task_with_priority(work, 0);
        0
    } else {
        -1
    }

    
}

/// return fd
pub fn sys_open(path: *const u8, flags: u32) -> isize {
    let task = current_task().unwrap();
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(inode) = open_file(
        path.as_str(),
        OpenFlags::from_bits(flags).unwrap()
    ) {
        let mut inner = task.acquire_inner_lock();
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    let task = current_task().unwrap();
    let mut inner = task.acquire_inner_lock();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

pub fn sys_pipe(pipe: *mut usize) -> isize {
    let task = current_task().unwrap();
    let token = current_user_token();
    let mut inner = task.acquire_inner_lock();
    let (pipe_read, pipe_write) = make_pipe();
    let read_fd = inner.alloc_fd();
    inner.fd_table[read_fd] = Some(pipe_read);
    let write_fd = inner.alloc_fd();
    inner.fd_table[write_fd] = Some(pipe_write);
    *translated_refmut(token, pipe) = read_fd;
    *translated_refmut(token, unsafe { pipe.add(1) }) = write_fd;
    0
}

pub fn sys_dup(fd: usize) -> isize {
    let task = current_task().unwrap();
    let mut inner = task.acquire_inner_lock();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    let new_fd = inner.alloc_fd();
    inner.fd_table[new_fd] = Some(Arc::clone(inner.fd_table[fd].as_ref().unwrap()));
    new_fd as isize
}