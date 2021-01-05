//! A collection of traits to define a common interface across executors

#![forbid(unsafe_code)]
#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]
#![no_std]
extern crate alloc;

use alloc::boxed::Box;
use async_trait::async_trait;
use core::{fmt, future::Future, ops::Deref, pin::Pin};

/// An error to notify that the current executor doesn't support spawning local tasks.
/// Holds the task that we tried to spawn.
pub struct LocalExecutorError(pub Pin<Box<dyn Future<Output = ()>>>);

impl fmt::Debug for LocalExecutorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("LocalExecutorError")
            .field(&"Future")
            .finish()
    }
}

/// A common interface for spawning futures and blocking tasks on top of an executor
pub trait FullExecutor: Executor + BlockingExecutor {}

impl<E: Executor + BlockingExecutor> FullExecutor for E {}

/// A common interface for spawning futures on top of an executor
pub trait Executor {
    /// Block on a future until completion
    fn block_on(&self, f: Pin<Box<dyn Future<Output = ()>>>);

    /// Spawn a future and return a handle to track its completion.
    fn spawn(&self, f: Pin<Box<dyn Future<Output = ()> + Send>>) -> Box<dyn Task>;

    /// Spawn a non-Send future on the current thread and return a handle to track its completion.
    fn spawn_local(
        &self,
        f: Pin<Box<dyn Future<Output = ()>>>,
    ) -> Result<Box<dyn Task>, LocalExecutorError> {
        Err(LocalExecutorError(f))
    }
}

/// A common interface for spawning blocking tasks on top of an executor
#[async_trait]
pub trait BlockingExecutor {
    /// Convert a blocking task into a future, spawning it on a decicated thread pool
    async fn spawn_blocking(&self, f: Box<dyn FnOnce() + Send + 'static>);
}

impl<E: Deref + Sync> Executor for E
where
    E::Target: Executor + Sync,
{
    fn block_on(&self, f: Pin<Box<dyn Future<Output = ()>>>) {
        self.deref().block_on(f)
    }

    fn spawn(&self, f: Pin<Box<dyn Future<Output = ()> + Send>>) -> Box<dyn Task> {
        self.deref().spawn(f)
    }

    fn spawn_local(
        &self,
        f: Pin<Box<dyn Future<Output = ()>>>,
    ) -> Result<Box<dyn Task>, LocalExecutorError> {
        self.deref().spawn_local(f)
    }
}

#[async_trait]
impl<E: Deref + Sync> BlockingExecutor for E
where
    E::Target: BlockingExecutor + Sync,
{
    async fn spawn_blocking(&self, f: Box<dyn FnOnce() + Send + 'static>) {
        self.deref().spawn_blocking(f).await
    }
}

/// A common interface to wait for a Task completion, let it run n the background or cancel it.
#[async_trait(?Send)]
pub trait Task: Future<Output = ()> {
    /// Cancels the task and waits for it to stop running.
    ///
    /// Returns the task's output if it was completed just before it got canceled, or None if it
    /// didn't complete.
    async fn cancel(self: Box<Self>) -> Option<()>;
}
