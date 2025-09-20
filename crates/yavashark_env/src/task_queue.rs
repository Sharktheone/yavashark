use crate::{Realm, Res};
use std::fmt::Debug;
use std::future::poll_fn;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::{fmt, mem};

pub trait AsyncTask {
    fn poll(self: Pin<&mut Self>, cx: &mut Context, realm: &mut Realm) -> Poll<Res>;

    fn run_first_sync(&mut self, realm: &mut Realm) -> Poll<Res>;
}

#[derive(Default)]
pub struct AsyncTaskQueue {
    microtasks: Vec<Microtask>,
    queue: Vec<Pin<Box<dyn AsyncTask>>>,
}

impl Debug for AsyncTaskQueue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AsyncTaskQueue")
            .field("microtasks", &self.microtasks.len())
            .field("queue", &self.queue.len())
            .finish()
    }
}

impl Clone for AsyncTaskQueue {
    fn clone(&self) -> Self {
        Self {
            microtasks: Vec::new(),
            queue: Vec::new(),
        }
    }
}

impl PartialEq for AsyncTaskQueue {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

impl AsyncTaskQueue {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn queue_microtask(&mut self, task: impl FnOnce(&mut Realm) + 'static) {
        self.microtasks.push(Box::new(task));
    }

    pub fn queue_task(mut task: impl AsyncTask + 'static, realm: &mut Realm) {
        if task.run_first_sync(realm).is_ready() {
            return;
        }

        let pinned: Pin<Box<dyn AsyncTask>> = Box::pin(task);


        realm.queue.queue.push(pinned);
    }

    pub fn flush_microtasks(&mut self) -> Microtasks {
        Microtasks {
            queue: mem::take(&mut self.microtasks),
        }
    }

    pub fn runner(&mut self) -> TaskQueueRunner {
        TaskQueueRunner {
            queue: mem::take(&mut self.queue),
        }
    }
}

pub type Microtask = Box<dyn FnOnce(&mut Realm)>;

pub struct Microtasks {
    queue: Vec<Microtask>,
}

pub struct TaskQueueRunner {
    queue: Vec<Pin<Box<dyn AsyncTask>>>,
}

impl TaskQueueRunner {
    pub async fn run(&mut self, realm: &mut Realm) {
        poll_fn(|cx| {
            Self::flush_microtasks(realm);

            self.queue.append(&mut realm.queue.queue);

            self.queue
                .retain_mut(|task| task.as_mut().poll(cx, realm).is_pending());

            while self.flush_queue(realm, cx) {
                Self::flush_microtasks(realm);
                self.queue
                    .retain_mut(|task| task.as_mut().poll(cx, realm).is_pending());
            }

            if self.queue.is_empty() {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;
    }

    fn flush_microtasks(realm: &mut Realm) {
        let micro = realm.queue.flush_microtasks();

        for task in micro.queue {
            task(realm);
        }

        if !realm.queue.microtasks.is_empty() {
            Self::flush_microtasks(realm);
        }
    }

    fn flush_queue(&mut self, realm: &mut Realm, cx: &mut Context) -> bool {
        let mut buf = Vec::new();

        let mut ran_tasks = false;

        while !realm.queue.queue.is_empty() {
            Self::flush_microtasks(realm);

            let mut queue = realm.queue.queue.drain(..).collect::<Vec<_>>();

            ran_tasks |= !queue.is_empty();

            queue.retain_mut(|task| task.as_mut().poll(cx, realm).is_pending());

            buf.append(&mut queue);
        }

        self.queue.append(&mut buf);

        ran_tasks
    }
}
