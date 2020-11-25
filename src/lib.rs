//! A collection of traits to define a common interface across executors

#![forbid(unsafe_code)]
#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]
#![no_std]
extern crate alloc;

use alloc::boxed::Box;
use async_trait::async_trait;
use core::{future::Future, ops::Deref, pin::Pin};

/// A common interface for spawning futures on top of an executor
pub trait Executor {
    /// Spawn a future and return a handle to get its result on completion.
    ///
    /// Dropping the handle will cancel the future. You can call `detach()` to let it
    /// run without waiting for its completion.
    fn spawn<T: Send + 'static>(&self, f: Pin<Box<dyn Future<Output = T> + Send>>) -> Box<dyn Task<T>>;
}

/// A common interface for spawning non-Send futures on top of an executor, on the current thread
pub trait LocalExecutor {
    /// Spawn a non-Send future on the current thread and return a handle to get its result on completion.
    ///
    /// Dropping the handle will cancel the future. You can call `detach()` to let it
    /// run without waiting for its completion.
    fn spawn_local<T: 'static>(&self, f: Pin<Box<dyn Future<Output = T>>>) -> Box<dyn Task<T>>;
}

/// A common interface for spawning blocking tasks on top of an executor
#[async_trait]
pub trait BlockingExecutor {
    /// Convert a blocking task into a future, spawning it on a decicated thread pool
    async fn spawn_blocking<F: FnOnce() -> T + Send + 'static, T: Send + 'static>(&self, f: F) -> T;
}

impl<E: Deref> Executor for E
where
    E::Target: Executor,
{
    fn spawn<T: Send + 'static>(&self, f: Pin<Box<dyn Future<Output = T> + Send>>) -> Box<dyn Task<T>> {
        self.deref().spawn(f)
    }
}

impl<E: Deref> LocalExecutor for E
where
    E::Target: LocalExecutor,
{
    fn spawn_local<T: 'static>(&self, f: Pin<Box<dyn Future<Output = T>>>) -> Box<dyn Task<T>> {
        self.deref().spawn_local(f)
    }
}

#[async_trait]
impl<E: Deref + Sync> BlockingExecutor for E
where
    E::Target: BlockingExecutor + Sync,
{
    async fn spawn_blocking<F: FnOnce() -> T + Send + 'static, T: Send + 'static>(&self, f: F) -> T {
        self.deref().spawn_blocking(f).await
    }
}

/// A common interface to wait for a Task completion, let it run n the background or cancel it.
#[async_trait(?Send)]
pub trait Task<T>: Future<Output = T> {
    /// Let the task run in the background, discarding its return value
    fn detach(self);
    /// Cancels the task and waits for it to stop running.
    ///
    /// Returns the task's output if it was completed just before it got canceled, or None if it
    /// didn't complete.
    async fn cancel(self) -> Option<T>;
}
