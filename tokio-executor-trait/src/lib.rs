use async_trait::async_trait;
use executor_trait::{BlockingExecutor, Executor, FullExecutor, LocalExecutorError, Task};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::runtime::Handle;

#[cfg(feature = "tracing")]
use {tracing::Span, tracing_futures::Instrument};

/// Dummy object implementing executor-trait common interfaces on top of tokio
#[derive(Debug, Clone)]
#[cfg_attr(not(feature = "tracing"), derive(Default))]
pub struct Tokio {
    handle: Option<Handle>,
    #[cfg(feature = "tracing")]
    span: Span,
}

#[cfg(feature = "tracing")]
impl Default for Tokio {
    fn default() -> Self {
        Self {
            handle: None,
            span: Span::none(),
        }
    }
}

impl Tokio {
    pub fn with_handle(mut self, handle: Handle) -> Self {
        self.handle = Some(handle);
        self
    }

    pub fn current() -> Self {
        Self::default().with_handle(Handle::current())
    }

    pub(crate) fn handle(&self) -> Option<Handle> {
        Handle::try_current().or_else(|| self.0.clone())
    }
}

#[cfg(feature = "tracing")]
impl Tokio {
    pub fn with_span(mut self, span: Span) -> Self {
        self.span = span;
        self
    }

    pub fn with_current_span(mut self) -> Self {
        self.span = Span::current();
        self
    }
}

struct TTask(tokio::task::JoinHandle<()>);

#[async_trait]
impl Executor for Tokio {
    fn block_on(&self, f: Pin<Box<dyn Future<Output = ()>>>) {
        #[cfg(feature = "tracing")]
        let f = f.instrument(self.span.clone());
        if let Some(handle) = self.handle() {
            handle.block_on(f)
        } else {
            Handle::current().block_on(f);
        }
    }

    fn spawn(&self, f: Pin<Box<dyn Future<Output = ()> + Send>>) -> Box<dyn Task> {
        #[cfg(feature = "tracing")]
        let f = f.instrument(self.span.clone());
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
            let span = self.span.clone();
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
