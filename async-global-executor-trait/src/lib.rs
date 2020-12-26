use async_trait::async_trait;
use executor_trait::{Executor, Task};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

/// Dummy object implementing executor-trait common interfaces on top of async-global-executor
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AsyncGlobalExecutor;

struct AGETask(async_global_executor::Task<()>);

#[async_trait]
impl Executor for AsyncGlobalExecutor {
    fn block_on(&self, f: Pin<Box<dyn Future<Output = ()>>>) {
        async_global_executor::block_on(f);
    }

    fn spawn(&self, f: Pin<Box<dyn Future<Output = ()> + Send>>) -> Box<dyn Task> {
        Box::new(AGETask(async_global_executor::spawn(f)))
    }

    fn spawn_local(&self, f: Pin<Box<dyn Future<Output = ()>>>) -> Box<dyn Task> {
        Box::new(AGETask(async_global_executor::spawn_local(f)))
    }

    async fn spawn_blocking(&self, f: Box<dyn FnOnce() + Send + 'static>) {
        async_global_executor::spawn_blocking(f).await;
    }
}

#[async_trait(?Send)]
impl Task for AGETask {
    fn detach(self: Box<Self>) {
        self.0.detach();
    }

    async fn cancel(self: Box<Self>) -> Option<()> {
        self.0.cancel().await
    }
}

impl Future for AGETask {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.0).poll(cx)
    }
}
