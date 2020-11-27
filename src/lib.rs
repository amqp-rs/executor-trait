//! A collection of traits to define a common interface across executors

#![forbid(unsafe_code)]
#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]
#![no_std]
extern crate alloc;

use alloc::boxed::Box;
use async_trait::async_trait;
use core::{future::Future, ops::Deref, pin::Pin};

/// A common interface for spawning futures and blocking tasks on top of an executor
#[async_trait]
pub trait Executor {
    /// Spawn a future and return a handle to track its completion.
    ///
    /// Dropping the handle will cancel the future. You can call `detach()` to let it
    /// run without waiting for its completion.
    fn spawn(&self, f: Pin<Box<dyn Future<Output = ()> + Send>>) -> Box<dyn Task>;

    /// Spawn a non-Send future on the current thread and return a handle to track its completion.
    ///
    /// Dropping the handle will cancel the future. You can call `detach()` to let it
    /// run without waiting for its completion.
    fn spawn_local(&self, f: Pin<Box<dyn Future<Output = ()>>>) -> Box<dyn Task>;

    /// Convert a blocking task into a future, spawning it on a decicated thread pool
    async fn spawn_blocking(&self, f: Box<dyn FnOnce() + Send + 'static>);
}

#[async_trait]
impl<E: Deref + Sync> Executor for E
where
    E::Target: Executor + Sync,
{
    fn spawn(&self, f: Pin<Box<dyn Future<Output = ()> + Send>>) -> Box<dyn Task> {
        self.deref().spawn(f)
    }

    fn spawn_local(&self, f: Pin<Box<dyn Future<Output = ()>>>) -> Box<dyn Task> {
        self.deref().spawn_local(f)
    }

    async fn spawn_blocking(&self, f: Box<dyn FnOnce() + Send + 'static>) {
        self.deref().spawn_blocking(f).await
    }
}

/// A common interface to wait for a Task completion, let it run n the background or cancel it.
#[async_trait(?Send)]
pub trait Task: Future<Output = ()> {
    /// Let the task run in the background.
    fn detach(self: Box<Self>);

    /// Cancels the task and waits for it to stop running.
    ///
    /// Returns the task's output if it was completed just before it got canceled, or None if it
    /// didn't complete.
    async fn cancel(self: Box<Self>) -> Option<()>;
}
