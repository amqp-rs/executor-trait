use async_trait::async_trait;
use executor_trait::{Executor, LocalExecutorError, Task};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::runtime::Handle;
use tokio_executor_trait_2::Tokio as OldTokio;

#[cfg(feature = "tracing")]
use {tracing::Span, tracing_futures::Instrument};

/// Dummy object implementing executor-trait common interfaces on top of tokio
#[derive(Default, Debug, Clone)]
pub struct Tokio {
    inner: OldTokio,
}

impl Tokio {
    pub fn with_handle(mut self, handle: Handle) -> Self {
        self.inner = self.inner.with_handle(handle);
        self
    }

    pub fn current() -> Self {
        Self {
            inner: OldTokio::current(),
        }
    }

    pub fn handle(&self) -> Option<Handle> {
        self.inner.handle()
    }
}

#[cfg(feature = "tracing")]
impl Tokio {
    pub fn with_span(mut self, span: Span) -> Self {
        self.inner = self.inner.with_span(span);
        self
    }

    pub fn with_current_span(mut self) -> Self {
        self.inner = self.inner.with_current_span();
        self
    }

    pub fn span(&self) -> Span {
        self.inner.span()
    }
}

struct TTask(tokio::task::JoinHandle<()>);

#[async_trait]
impl Executor for Tokio {
    fn block_on(&self, f: Pin<Box<dyn Future<Output = ()>>>) {
        #[cfg(feature = "tracing")]
        let f = f.instrument(self.span());
        if let Some(handle) = self.handle() {
            handle.block_on(f)
        } else {
            Handle::current().block_on(f);
        }
    }

    fn spawn(&self, f: Pin<Box<dyn Future<Output = ()> + Send>>) -> Box<dyn Task> {
        #[cfg(feature = "tracing")]
        let f = f.instrument(self.span());
        Box::new(TTask(if let Some(handle) = self.handle() {
            handle.spawn(f)
        } else {
            tokio::task::spawn(f)
        }))
    }

    fn spawn_local(
        &self,
        f: Pin<Box<dyn Future<Output = ()>>>,
    ) -> Result<Box<dyn Task>, LocalExecutorError> {
        // FIXME: how can we hook up spawn_local here?
        Err(LocalExecutorError(f))
    }

    async fn spawn_blocking(&self, f: Box<dyn FnOnce() + Send + 'static>) {
        #[cfg(feature = "tracing")]
        let f = {
            let span = self.span();
            move || {
                let _entered = span.enter();
                f()
            }
        };
        // FIXME: better handling of failure
        if let Some(handle) = self.handle() {
            handle.spawn_blocking(f).await
        } else {
            tokio::task::spawn_blocking(f).await
        }
        .expect("blocking task failed");
    }
}

#[async_trait(?Send)]
impl Task for TTask {
    async fn cancel(self: Box<Self>) -> Option<()> {
        self.0.abort();
        self.0.await.ok()
    }
}

impl Future for TTask {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Pin::new(&mut self.0).poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(res) => {
                res.expect("task has been canceled");
                Poll::Ready(())
            }
        }
    }
}

impl executor_trait_2::FullExecutor for Tokio {}

impl executor_trait_2::Executor for Tokio {
    fn block_on(&self, f: Pin<Box<dyn Future<Output = ()>>>) {
        self.inner.block_on(f)
    }

    fn spawn(
        &self,
        f: Pin<Box<dyn Future<Output = ()> + Send>>,
    ) -> Box<dyn executor_trait_2::Task> {
        self.inner.spawn(f)
    }

    fn spawn_local(
        &self,
        f: Pin<Box<dyn Future<Output = ()>>>,
    ) -> Result<Box<dyn executor_trait_2::Task>, executor_trait_2::LocalExecutorError> {
        self.inner.spawn_local(f)
    }
}

#[async_trait]
impl executor_trait_2::BlockingExecutor for Tokio {
    async fn spawn_blocking(&self, f: Box<dyn FnOnce() + Send + 'static>) {
        self.inner.spawn_blocking(f).await
    }
}
