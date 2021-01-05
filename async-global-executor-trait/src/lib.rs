use async_trait::async_trait;
use executor_trait::{BlockingExecutor, Executor, LocalExecutorError, Task};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

/// Dummy object implementing executor-trait common interfaces on top of async-global-executor
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AsyncGlobalExecutor;

struct AGETask(Option<async_global_executor::Task<()>>);

impl Executor for AsyncGlobalExecutor {
    fn block_on(&self, f: Pin<Box<dyn Future<Output = ()>>>) {
        async_global_executor::block_on(f);
    }

    fn spawn(&self, f: Pin<Box<dyn Future<Output = ()> + Send>>) -> Box<dyn Task> {
        Box::new(AGETask(Some(async_global_executor::spawn(f))))
    }

    fn spawn_local(
        &self,
        f: Pin<Box<dyn Future<Output = ()>>>,
    ) -> Result<Box<dyn Task>, LocalExecutorError> {
        Ok(Box::new(AGETask(Some(async_global_executor::spawn_local(
            f,
        )))))
    }
}

#[async_trait]
impl BlockingExecutor for AsyncGlobalExecutor {
    async fn spawn_blocking(&self, f: Box<dyn FnOnce() + Send + 'static>) {
        async_global_executor::spawn_blocking(f).await;
    }
}

#[async_trait(?Send)]
impl Task for AGETask {
    async fn cancel(mut self: Box<Self>) -> Option<()> {
        self.0.take()?.cancel().await
    }
}

impl Drop for AGETask {
    fn drop(&mut self) {
        if let Some(task) = self.0.take() {
            task.detach();
        }
    }
}

impl Future for AGETask {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(self.0.as_mut().unwrap()).poll(cx)
    }
}
