use super::File;
use alloc::rc::Rc;
use alloc::sync::{Arc, Weak};
use spin::Mutex;
use crate::basic_rt::CUR_COROUTINE;
use crate::basic_rt::excutor::WRMAP;
use crate::mm::{
    UserBuffer,
};
use crate::task::suspend_current_and_run_next;
use crate::config::PAGE_SIZE;
use crate::mm::{CBQ_BASE_PA, USER_CBQ_VA, USER_CBQ_VEC_PA};
use crate::basic_rt::cbq::CBQueue;

#[derive(Clone)]
pub struct Pipe {
    readable: bool,
    writable: bool,
    buffer: Arc<Mutex<PipeRingBuffer>>,
}

impl Pipe {
    pub fn read_end_with_buffer(buffer: Arc<Mutex<PipeRingBuffer>>) -> Self {
        Self {
            readable: true,
            writable: false,
            buffer,
        }
    }
    pub fn write_end_with_buffer(buffer: Arc<Mutex<PipeRingBuffer>>) -> Self {
        Self {
            readable: false,
            writable: true,
            buffer,
        }
    }
}

const RING_BUFFER_SIZE: usize = 4 * 1024;
//const RING_BUFFER_SIZE: usize = 8;

#[derive(Copy, Clone, PartialEq)]
enum RingBufferStatus {
    FULL,
    EMPTY,
    NORMAL,
}

pub struct PipeRingBuffer {
    arr: [u8; RING_BUFFER_SIZE],
    head: usize,
    tail: usize,
    status: RingBufferStatus,
    write_end: Option<Weak<Pipe>>,
}

impl PipeRingBuffer {
    pub fn new() -> Self {
        Self {
            arr: [0; RING_BUFFER_SIZE],
            head: 0,
            tail: 0,
            status: RingBufferStatus::EMPTY,
            write_end: None,
        }
    }
    pub fn set_write_end(&mut self, write_end: &Arc<Pipe>) {
        self.write_end = Some(Arc::downgrade(write_end));
    }
    pub fn write_byte(&mut self, byte: u8) {
        self.status = RingBufferStatus::NORMAL;
        self.arr[self.tail] = byte;
        self.tail = (self.tail + 1) % RING_BUFFER_SIZE;
        if self.tail == self.head {
            self.status = RingBufferStatus::FULL;
        }
    }
    pub fn read_byte(&mut self) -> u8 {
        self.status = RingBufferStatus::NORMAL;
        let c = self.arr[self.head];
        self.head = (self.head + 1) % RING_BUFFER_SIZE;
        if self.head == self.tail {
            self.status = RingBufferStatus::EMPTY;
        }
        c
    }
    pub fn available_read(&self) -> usize {
        if self.status == RingBufferStatus::EMPTY {
            0
        } else {
            if self.tail > self.head {
                self.tail - self.head
            } else {
                self.tail + RING_BUFFER_SIZE - self.head
            }
        }
    }
    pub fn available_write(&self) -> usize {
        if self.status == RingBufferStatus::FULL {
            0
        } else {
            RING_BUFFER_SIZE - self.available_read()
        }
    }
    pub fn all_write_ends_closed(&self) -> bool {
        self.write_end.as_ref().unwrap().upgrade().is_none()
    }
}

/// Return (read_end, write_end)
pub fn make_pipe() -> (Arc<Pipe>, Arc<Pipe>) {
    let buffer = Arc::new(Mutex::new(PipeRingBuffer::new()));
    let read_end = Arc::new(
        Pipe::read_end_with_buffer(buffer.clone())
    );
    let write_end = Arc::new(
        Pipe::write_end_with_buffer(buffer.clone())
    );
    buffer.lock().set_write_end(&write_end);
    (read_end, write_end)
}

impl File for Pipe {
    fn readable(&self) -> bool { self.readable }
    fn writable(&self) -> bool { self.writable }
    fn read(&self, buf: UserBuffer) -> usize {
        assert_eq!(self.readable(), true);
        let mut buf_iter = buf.into_iter();
        let mut read_size = 0usize;

        loop {
            let mut ring_buffer = self.buffer.lock();
            let loop_read = ring_buffer.available_read();
            if loop_read == 0 {
                if ring_buffer.all_write_ends_closed() {
                    return read_size;
                }
                drop(ring_buffer);
                suspend_current_and_run_next();
                continue;
            }
            // read at most loop_read bytes
            for _ in 0..loop_read {
                if let Some(byte_ref) = buf_iter.next() {
                    unsafe { *byte_ref = ring_buffer.read_byte(); }
                    read_size += 1;
                } else {
                    return read_size;
                }
            }
        }
    }
    fn write(&self, buf: UserBuffer) -> usize {
        assert_eq!(self.writable(), true);
        let mut buf_iter = buf.into_iter();
        let mut write_size = 0usize;
        loop {
            let mut ring_buffer = self.buffer.lock();
            let loop_write = ring_buffer.available_write();
            if loop_write == 0 {
                drop(ring_buffer);
                suspend_current_and_run_next();
                continue;
            }
            // write at most loop_write bytes
            for _ in 0..loop_write {
                if let Some(byte_ref) = buf_iter.next() {
                    ring_buffer.write_byte(unsafe { *byte_ref });
                    write_size += 1;
                } else {
                    return write_size;
                }
            }
        }
    }

    fn aread(&self, buf: UserBuffer, space_id: usize, tid: usize, k: usize) -> Pin<Box<dyn Future<Output = ()>  + 'static + Send + Sync>> {
        async fn aread_work(s: Pipe, _buf: UserBuffer, space_id: usize, tid: usize, k: usize) {
            assert_eq!(s.readable(), true);
            let mut buf_iter = _buf.into_iter();
            let mut read_size = 0usize;

            let mut helper = Box::new(ReadHelper::new());
            loop {
                let mut ring_buffer = s.buffer.lock();
                let loop_read = ring_buffer.available_read();
                if loop_read == 0 {
                    if ring_buffer.all_write_ends_closed() {
                        break ;
                        //return read_size;
                    }
                    drop(ring_buffer);
                    //suspend_current_and_run_next();
                    unsafe { WRMAP.lock().register(k, CUR_COROUTINE); }
                    helper.as_mut().await;
                    continue;
                }   
                // read at most loop_read bytes
                for _ in 0..loop_read {
                    if let Some(byte_ref) = buf_iter.next() {
                        unsafe { *byte_ref = ring_buffer.read_byte(); }
                        read_size += 1;
                    } else {
                        break; ;
                        //return read_size;
                    }
                }
            }
            let addr = CBQ_BASE_PA + PAGE_SIZE * space_id;
            let vptr = USER_CBQ_VEC_PA + PAGE_SIZE * space_id;
            CBQueue::add(addr, tid, vptr);
        }

        Box::pin(aread_work(self.clone(), buf, space_id, tid, k))
    }
}


use core::cell::RefCell;
use core::future::Future;
use alloc::boxed::Box;
use core::pin::Pin;
use core::{task::{Context, Poll}};

pub struct ReadHelper(usize);

impl ReadHelper {
    pub fn new() -> Self {
        Self(0)
    }
}

impl Future for ReadHelper {
    type Output = (());

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.0 += 1;
        if (self.0 & 1) == 1 {
            return Poll::Pending;
        } else {
            return Poll::Ready(());
        }
    }
}