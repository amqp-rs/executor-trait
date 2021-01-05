use async_trait::async_trait;
use executor_trait::{BlockingExecutor, Executor, LocalExecutorError, Task};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

/// Dummy object implementing executor-trait common interfaces on top of bastion
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bastion;

struct BTask(lightproc::recoverable_handle::RecoverableHandle<()>);

impl Executor for Bastion {
    fn block_on(&self, f: Pin<Box<dyn Future<Output = ()>>>) {
        bastion::run!(f);
    }

    fn spawn(&self, f: Pin<Box<dyn Future<Output = ()> + Send>>) -> Box<dyn Task> {
        Box::new(BTask(bastion::executor::spawn(f)))
    }

    fn spawn_local(&self, f: Pin<Box<dyn Future<Output = ()>>>) -> Result<Box<dyn Task>, LocalExecutorError> {
        Err(LocalExecutorError(f))
    }
}

#[async_trait]
impl BlockingExecutor for Bastion {
    async fn spawn_blocking(&self, f: Box<dyn FnOnce() + Send + 'static>) {
        bastion::executor::blocking(async move { f() }).await;
    }
}

#[async_trait(?Send)]
impl Task for BTask {
    async fn cancel(self: Box<Self>) -> Option<()> {
        self.0.cancel();
        self.0.await
    }
}

impl Future for BTask {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Pin::new(&mut self.0).poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(res) => Poll::Ready(res.expect("task has been canceled")),
        }
    }
}
