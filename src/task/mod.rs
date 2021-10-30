mod simple_executor;

use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

pub use simple_executor::SimpleExecutor;

struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    fn new(future: impl Future<Output = ()> + 'static) -> Self {
        Self { future: Box::pin(future) }
    }

    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}
