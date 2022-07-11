#![no_std]

use lazy_static::lazy_static;
use alloc::{vec, sync::Arc};
use futures::Future;
use spin::Mutex;
use core::{task::{Context, Poll}, pin::Pin};

use crate::WAKE_COROUTINE_VA;


pub async fn compute() {
    let G = [[1.0, 0.0, 0.0], 
                            [0.5, 0.5, 0.5],
                            [0.2, -0.5, 0.5],
                            [0.0, 0.0, 1.0]];
    let B_T = [[1.0, 0.0, -1.0, 0.0],
                              [0.0, 1.0, 1.0, 0.0],
                              [0.0, -1.0, 1.0, 0.0]];

    // Matrix Multiplication: out[3, 3] = G[4, 3] * B_T[3, 4] 
    let mut out = [[0.0;4];4];
    for i in 0..4 {
        for j in 0..4 {
            for k in 0..3 {
                out[i][j] += G[i][k] * B_T[k][j];
            }
        }
    }
}



lazy_static! {
    pub static ref COUNTER: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
}

pub fn write_cnt() {
    *COUNTER.lock() += 1;
    // async writing will wake up the corresponding reading
    unsafe {
        let wake_coroutine: fn(tid: usize) = core::mem::transmute(WAKE_COROUTINE_VA as usize);
        wake_coroutine(*COUNTER.lock() - 1);
    }
}

pub fn write_cnt_without_wake() {
    *COUNTER.lock() += 1;
}

pub fn set_cnter_zero() {
    *COUNTER.lock() = 0;
}

#[derive(Clone, Copy)]
pub struct Counter {
    pub cnt: usize,
}

impl Counter {
    pub fn new(v: usize) -> Self {
        Self { cnt: v }
    }

    pub fn write(&mut self) {
        self.cnt += 1;
    }
}

impl Future for Counter {
    type Output = (());

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
       if self.cnt != *COUNTER.lock() {
           //println!("cnt: {}, waiting for wake up", self.cnt);
           return Poll::Pending;
       } else {
           return Poll::Ready(());
       }
    }
}

