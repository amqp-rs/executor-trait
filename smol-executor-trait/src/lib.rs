use async_trait::async_trait;
use executor_trait::{BlockingExecutor, Executor, FullExecutor, LocalExecutorError, Task};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

/// Dummy object implementing executor-trait common interfaces on top of smol
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Smol;

struct STask(Option<smol::Task<()>>);

#[async_trait]
impl Executor for Smol {
    fn block_on(&self, f: Pin<Box<dyn Future<Output = ()>>>) {
        smol::block_on(f);
    }

    fn spawn(&self, f: Pin<Box<dyn Future<Output = ()> + Send>>) -> Box<dyn Task> {
        Box::new(STask(Some(smol::spawn(f))))
    }

    fn spawn_local(
        &self,
        f: Pin<Box<dyn Future<Output = ()>>>,
    ) -> Result<Box<dyn Task>, LocalExecutorError> {
        Err(LocalExecutorError(f))
    }

    async fn spawn_blocking(&self, f: Box<dyn FnOnce() + Send + 'static>) {
        smol::unblock(f).await;
    }
}

#[async_trait(?Send)]
impl Task for STask {
    async fn cancel(mut self: Box<Self>) -> Option<()> {
        self.0.take()?.cancel().await
    }
}

impl Drop for STask {
    fn drop(&mut self) {
        if let Some(task) = self.0.take() {
            task.detach();
        }
    }
}

impl Future for STask {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(self.0.as_mut().unwrap()).poll(cx)
    }
}
