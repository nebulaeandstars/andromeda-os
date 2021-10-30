use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::task::Wake;
use core::future::Future;
use core::task::{Context, Poll, Waker};

use crossbeam_queue::ArrayQueue;

use super::{Task, TaskId};

struct TaskWaker {
    task_id:    TaskId,
    task_queue: Arc<ArrayQueue<TaskId>>,
}

impl TaskWaker {
    fn new(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(TaskWaker { task_id, task_queue }))
    }

    fn wake_task(&self) {
        self.task_queue.push(self.task_id).expect("task_queue full");
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}

pub struct Executor {
    tasks:  BTreeMap<TaskId, Task>,
    wakers: BTreeMap<TaskId, Waker>,
    queue:  Arc<ArrayQueue<TaskId>>,
}

impl Executor {
    pub fn new(max_tasks: usize) -> Self {
        Executor {
            tasks:  BTreeMap::new(),
            wakers: BTreeMap::new(),
            queue:  Arc::new(ArrayQueue::new(max_tasks)),
        }
    }

    pub fn spawn(&mut self, future: impl Future<Output = ()> + 'static) {
        let task = Task::new(future);
        let task_id = task.id;
        if self.tasks.insert(task.id, task).is_some() {
            panic!("task with same ID already in tasks");
        }
        self.queue.push(task_id).expect("queue full");
    }

    pub fn run(&mut self) -> ! {
        loop {
            self.run_ready_tasks();
            self.sleep_if_idle();
        }
    }

    fn sleep_if_idle(&self) {
        use x86_64::instructions::interrupts;

        interrupts::disable();
        if self.queue.is_empty() {
            interrupts::enable_and_hlt();
        }
        else {
            interrupts::enable();
        }
    }

    fn run_ready_tasks(&mut self) {
        let Self { tasks, wakers, queue } = self;

        while let Some(task_id) = queue.pop() {
            let task = match tasks.get_mut(&task_id) {
                Some(task) => task,
                None => continue, // task no longer exists
            };
            let waker = wakers
                .entry(task_id)
                .or_insert_with(|| TaskWaker::new(task_id, queue.clone()));
            let mut context = Context::from_waker(waker);
            match task.poll(&mut context) {
                Poll::Pending => {},
                Poll::Ready(()) => {
                    tasks.remove(&task_id);
                    wakers.remove(&task_id);
                },
            }
        }
    }
}
