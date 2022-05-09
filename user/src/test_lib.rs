#![no_std]

use alloc::{boxed::Box};
use core::task::{Context, Poll};
use core::{future::Future, pin::Pin};

pub struct AsyncTest {
    cur: usize,
    end: usize,
}

impl AsyncTest {
    pub fn new(times: usize) -> Self {
        Self {
            cur: 0,
            end: times,
        }
    }
}

impl Future for AsyncTest {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.cur >= self.end {
            return Poll::Ready(());
        } else {
            self.cur += 1;
            return Poll::Pending;
        }
    }
}