#![no_std]
extern crate alloc;

use alloc::boxed::Box;
use core::{future::Future, ops::Deref, pin::Pin};

pub trait Executor {
    fn spawn(&self, f: Pin<Box<dyn Future<Output = ()> + Send>>);
    fn spawn_blocking(&self, f: Box<dyn FnOnce() + Send>);
}

impl<E: Deref> Executor for E
where
    E::Target: Executor,
{
    fn spawn(&self, f: Pin<Box<dyn Future<Output = ()> + Send>>) {
        self.deref().spawn(f);
    }

    fn spawn_blocking(&self, f: Box<dyn FnOnce() + Send>) {
        self.deref().spawn_blocking(f)
    }
}
