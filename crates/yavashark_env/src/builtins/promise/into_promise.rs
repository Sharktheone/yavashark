use crate::builtins::Promise;
use crate::conversion::{FromValueOutput, TryIntoValue};
use crate::task_queue::AsyncTask;
use crate::{ObjectHandle, Realm, Res};
use pin_project::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use yavashark_garbage::OwningGcGuard;
use yavashark_value::{BoxedObj, Obj};

pub type GcPromise = OwningGcGuard<'static, BoxedObj<Realm>, Promise>;

pub trait IntoPromise {
    fn into_promise(self, realm: &mut Realm) -> ObjectHandle;
}

impl IntoPromise for Promise {
    fn into_promise(self, _: &mut Realm) -> ObjectHandle {
        self.into_object()
    }
}

#[pin_project]
pub struct FutureTask<F: Future<Output = O>, O: TryIntoValue> {
    #[pin]
    future: F,
    promise: GcPromise,
}

impl<F: Future<Output = O>, O: TryIntoValue> AsyncTask for FutureTask<F, O> {
    fn poll(self: Pin<&mut Self>, cx: &mut Context, realm: &mut Realm) -> Poll<Res> {
        let projected = self.project();

        match projected.future.poll(cx) {
            Poll::Ready(value) => {
                let value = value.try_into_value();
                projected.promise.set_res(value, realm)?;
                Poll::Ready(Ok(()))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<F: Future<Output = O> + 'static, O: TryIntoValue + 'static> IntoPromise for F {
    fn into_promise(self, realm: &mut Realm) -> ObjectHandle {
        let promise_obj = Promise::new(realm).into_object();

        #[allow(clippy::expect_used)]
        let promise = <&Promise>::from_value_out(promise_obj.clone().into()).expect("unreachable");

        let task = FutureTask {
            future: self,
            promise,
        };

        realm.queue.queue_task(task);

        promise_obj
    }
}
