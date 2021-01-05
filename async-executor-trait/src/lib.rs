use async_trait::async_trait;
use executor_trait::{BlockingExecutor, Executor, LocalExecutorError, Task};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

/// Dummy object implementing executor-trait common interfaces on top of async-std
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AsyncStd;

struct ASTask(async_std::task::JoinHandle<()>);

impl Executor for AsyncStd {
    fn block_on(&self, f: Pin<Box<dyn Future<Output = ()>>>) {
        async_std::task::block_on(f);
    }

    fn spawn(&self, f: Pin<Box<dyn Future<Output = ()> + Send>>) -> Box<dyn Task> {
        Box::new(ASTask(async_std::task::spawn(f)))
    }

    fn spawn_local(&self, f: Pin<Box<dyn Future<Output = ()>>>) -> Result<Box<dyn Task>, LocalExecutorError> {
        Ok(Box::new(ASTask(async_std::task::spawn_local(f))))
    }
}

#[async_trait]
impl BlockingExecutor for AsyncStd {
    async fn spawn_blocking(&self, f: Box<dyn FnOnce() + Send + 'static>) {
        async_std::task::spawn_blocking(f).await;
    }
}

#[async_trait(?Send)]
impl Task for ASTask {
    async fn cancel(self: Box<Self>) -> Option<()> {
        self.0.cancel().await
    }
}

impl Future for ASTask {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.0).poll(cx)
    }
}
