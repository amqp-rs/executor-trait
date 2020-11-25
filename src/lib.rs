#![no_std]
extern crate alloc;

use alloc::boxed::Box;
use async_trait::async_trait;
use core::{future::Future, ops::Deref, pin::Pin};

pub trait Executor<T> {
    fn spawn(&self, f: Pin<Box<dyn Future<Output = T> + Send>>) -> Box<dyn Task<T>>;
}

pub trait BlockingExecutor<T> {
    fn spawn_blocking(&self, f: Box<dyn FnOnce() -> T + Send>) -> Box<dyn Task<T>>;
}

impl<T, E: Deref> Executor<T> for E
where
    E::Target: Executor<T>,
{
    fn spawn(&self, f: Pin<Box<dyn Future<Output = T> + Send>>) -> Box<dyn Task<T>> {
        self.deref().spawn(f)
    }
}

impl<T, E: Deref> BlockingExecutor<T> for E
where
    E::Target: BlockingExecutor<T>,
{
    fn spawn_blocking(&self, f: Box<dyn FnOnce() -> T + Send>) -> Box<dyn Task<T>> {
        self.deref().spawn_blocking(f)
    }
}

#[async_trait]
pub trait Task<T>: Future<Output = T> {
    fn detach(self);
    async fn cancel(self) -> Option<T>;
}
