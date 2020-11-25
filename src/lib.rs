#![no_std]
extern crate alloc;

use alloc::boxed::Box;
use async_trait::async_trait;
use core::{future::Future, ops::Deref, pin::Pin};

pub trait Executor {
    fn spawn<T>(&self, f: Pin<Box<dyn Future<Output = T> + Send>>) -> Box<dyn Task<T>>;
}

pub trait LocalExecutor {
    fn spawn_local<T>(&self, f: Pin<Box<dyn Future<Output = T>>>) -> Box<dyn Task<T>>;
}

pub trait BlockingExecutor {
    fn spawn_blocking<T>(&self, f: Box<dyn FnOnce() -> T + Send>) -> Box<dyn Task<T>>;
}

impl<E: Deref> Executor for E
where
    E::Target: Executor,
{
    fn spawn<T>(&self, f: Pin<Box<dyn Future<Output = T> + Send>>) -> Box<dyn Task<T>> {
        self.deref().spawn(f)
    }
}

impl<E: Deref> LocalExecutor for E
where
    E::Target: LocalExecutor,
{
    fn spawn_local<T>(&self, f: Pin<Box<dyn Future<Output = T>>>) -> Box<dyn Task<T>> {
        self.deref().spawn_local(f)
    }
}

impl<E: Deref> BlockingExecutor for E
where
    E::Target: BlockingExecutor,
{
    fn spawn_blocking<T>(&self, f: Box<dyn FnOnce() -> T + Send>) -> Box<dyn Task<T>> {
        self.deref().spawn_blocking(f)
    }
}

#[async_trait]
pub trait Task<T>: Future<Output = T> {
    fn detach(self);
    async fn cancel(self) -> Option<T>;
}
