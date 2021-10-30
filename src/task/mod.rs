mod executor;
pub mod keyboard;

use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

pub use executor::Executor;

struct Task {
    id:     TaskId,
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    fn new(future: impl Future<Output = ()> + 'static) -> Self {
        Self { id: TaskId::new(), future: Box::pin(future) }
    }

    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TaskId(u64);

impl TaskId {
    fn new() -> Self {
        use core::sync::atomic::{AtomicU64, Ordering};
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}
