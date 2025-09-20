use crate::builtins::{GcPromise, Promise};
use crate::conversion::downcast_obj;
use crate::task_queue::AsyncTask;
use crate::{Error, NativeFunction, ObjectHandle, Realm, Res, Value};
use pin_project::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::time::{sleep, Sleep};
use yavashark_value::Obj;

pub struct SleepDuration {
    dur: Duration,
    sleep: Option<Sleep>,
}

// impl Unpin for SleepDuration {}

impl SleepDuration {
    pub const fn new(duration: Duration) -> Self {
        Self {
            dur: duration,
            sleep: None,
        }
    }

    pub fn get_sleep(&mut self) -> &mut Sleep {
        self.sleep.get_or_insert_with(|| sleep(self.dur))
    }
}

#[pin_project(!Unpin)]
pub struct TimeoutTask {
    pub timer: SleepDuration,
    pub promise: GcPromise,
    pub cb: ObjectHandle,
}

impl TimeoutTask {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(timer: SleepDuration, cb: ObjectHandle, realm: &mut Realm) -> Res<ObjectHandle> {
        let promise_obj = Promise::new(realm).into_object();
        let promise = downcast_obj::<Promise>(promise_obj.clone().into())?;

        let this = Self { timer, promise, cb };

        realm.queue.queue_task(this);

        Ok(promise_obj)
    }
}

impl AsyncTask for TimeoutTask {
    fn poll(self: Pin<&mut Self>, cx: &mut Context, realm: &mut Realm) -> Poll<Res> {
        let proj = self.project();

        let sleep = unsafe { Pin::new_unchecked(proj.timer.get_sleep()) };

        match sleep.poll(cx) {
            Poll::Ready(()) => {
                let res = proj.cb.call(realm, Vec::new(), Value::Undefined);

                proj.promise.set_res(res, realm)?;

                Poll::Ready(Ok(()))
            }
            Poll::Pending => Poll::Pending,
        }
    }

    fn run_first_sync(&mut self, _realm: &mut Realm) -> Poll<Res> {
        Poll::Pending
    }
}

pub fn get_set_timeout(realm: &Realm) -> ObjectHandle {
    NativeFunction::new(
        "setTimeout",
        |args, _, realm| {
            let callback = args
                .first()
                .cloned()
                .unwrap_or(Value::Undefined)
                .to_object()?;

            if !callback.is_function() {
                return Err(Error::ty("Expected a function"));
            }

            let time = args
                .get(1)
                .cloned()
                .unwrap_or(Value::Undefined)
                .to_number(realm)?;

            let timer = SleepDuration::new(Duration::from_millis(time as u64));

            Ok(TimeoutTask::new(timer, callback, realm)?.into())
        },
        realm,
    )
}
