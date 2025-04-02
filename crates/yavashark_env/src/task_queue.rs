use std::fmt::Debug;
use crate::builtins::Promise;
use crate::{Realm, Res, ValueResult};
use std::future::{poll_fn, Future};
use std::{fmt, mem};
use std::pin::Pin;
use std::task::{Context, Poll};
use yavashark_garbage::OwningGcGuard;
use yavashark_value::BoxedObj;

pub trait AsyncTask {
    fn poll(self: Pin<&mut Self>, cx: &mut Context, realm: &mut Realm) -> Poll<Res>;
}

#[derive(Default)]
pub struct AsyncTaskQueue {
    microtasks: Vec<Box<dyn FnOnce(&mut Realm)>>,
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
    
    pub fn queue_task(&mut self, task: impl AsyncTask + 'static) {
        let boxed: Box<dyn AsyncTask> = Box::new(task);
        
        self.queue.push(Box::into_pin(boxed));
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

pub struct Microtasks {
    queue: Vec<Box<dyn FnOnce(&mut Realm)>>,
}


pub struct TaskQueueRunner {
    queue: Vec<Pin<Box<dyn AsyncTask>>>,
}

impl TaskQueueRunner {
    pub async fn run(&mut self, realm: &mut Realm) {
        
        poll_fn(|cx| {
            self.flush_microtasks(realm);
            
            self.queue.append(&mut realm.queue.queue);
            
            
            self.queue
                .retain_mut(|task| task.as_mut().poll(cx, realm).is_pending());
            

            if self.queue.is_empty() {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        }).await;
    }
    
    fn flush_microtasks(&mut self, realm: &mut Realm) {
        let micro = realm.queue.flush_microtasks();
        
        for task in micro.queue {
            task(realm);
        }
        
        if !realm.queue.microtasks.is_empty() {
            self.flush_microtasks(realm);
        }
    }
}
