use crate::async_generator::AsyncGenerator;
use crate::{AsyncGeneratorPoll, ResumableVM, VmState};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::futures::Notified;
use yavashark_env::builtins::Promise;
use yavashark_env::conversion::downcast_obj;
use yavashark_env::error_obj::ErrorObj;
use yavashark_env::task_queue::{AsyncTask, AsyncTaskQueue};
use yavashark_env::value::{BoxedObj, Obj};
use yavashark_env::{Object, ObjectHandle, Realm, Res, Value};
use yavashark_garbage::{OwningGcGuard, OwningGcGuardRefed};

pub struct AsyncGeneratorTask {
    state: Option<VmState>,
    await_promise:
        Option<OwningGcGuardRefed<BoxedObj, (&'static Promise, Notified<'static>, bool)>>,
    promise: OwningGcGuard<'static, BoxedObj, Promise>,
    gen: OwningGcGuard<'static, BoxedObj, AsyncGenerator>,
    gen_notify: Option<OwningGcGuardRefed<BoxedObj, Notified<'static>>>,
}

impl Unpin for AsyncGeneratorTask {}

impl AsyncGeneratorTask {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        realm: &mut Realm,
        state: Option<VmState>,
        gen: OwningGcGuard<'static, BoxedObj, AsyncGenerator>,
    ) -> Res<ObjectHandle> {
        let promise_obj = Promise::new(realm).into_object();
        let promise = downcast_obj::<Promise>(promise_obj.clone().into())?;

        let gen_notify = if state.is_none() {
            Some(gen.clone().map_refed(|gen| gen.notify.notified()))
        } else {
            None
        };

        let this = Self {
            state,
            await_promise: None,
            promise,
            gen,
            gen_notify,
        };

        AsyncTaskQueue::queue_task(this, realm);

        Ok(promise_obj)
    }
}

impl AsyncTask for AsyncGeneratorTask {
    fn poll(self: Pin<&mut Self>, cx: &mut Context, realm: &mut Realm) -> Poll<Res> {
        let inner = Pin::into_inner(self);

        if let Some(gen_notify) = &mut inner.gen_notify {
            let pinned = unsafe { Pin::new_unchecked(&mut **gen_notify) };
            if pinned.poll(cx).is_pending() {
                return Poll::Pending;
            }
            inner.state = inner.gen.state.take();
            inner.gen_notify = None;
        }

        if let Some(promise) = &mut inner.await_promise {
            let pinned = unsafe { Pin::new_unchecked(&mut promise.1) };
            if pinned.poll(cx).is_pending() {
                return Poll::Pending;
            } else if let Some(state) = inner.state.as_mut() {
                let val = promise
                    .0
                    .inner
                    .borrow()
                    .value
                    .clone()
                    .unwrap_or(Value::Undefined);

                if promise.2 {
                    inner.promise.resolve(&val, realm)?;
                    return Poll::Ready(Ok(()));
                }
                state.continue_async(val, realm)?;
            }
        }

        _ = inner.await_promise.take();

        inner.poll_next(realm)
    }
    fn run_first_sync(&mut self, realm: &mut Realm) -> Poll<Res> {
        self.poll_next(realm)
    }
}

impl AsyncGeneratorTask {
    fn poll_next(&mut self, realm: &mut Realm) -> Poll<Res> {
        if let Some(state) = self.state.take() {
            let vm = ResumableVM::from_state(state, realm);
            match vm.poll_next() {
                AsyncGeneratorPoll::Await(state, promise) => {
                    self.state = Some(state);
                    let promise = match downcast_obj::<Promise>(promise.into()) {
                        Ok(promise) => promise,
                        Err(e) => return Poll::Ready(Err(e)),
                    };

                    let promise = promise.try_map_refed(|promise| {
                        let Some(notify) = promise.notify.notified() else {
                            return Err(());
                        };

                        Ok((promise, notify, false))
                    });

                    match promise {
                        Ok(promise) => {
                            self.await_promise = Some(promise);
                        }
                        Err((promise, ())) => {
                            let val = promise
                                .inner
                                .borrow()
                                .value
                                .clone()
                                .unwrap_or(Value::Undefined);

                            self.state.as_mut().map(|state| state.continue_async(val, realm));

                            return self.poll_next(realm);
                        }
                    }

                    Poll::Pending
                }
                AsyncGeneratorPoll::Ret(_, ret) => {
                    match ret {
                        Ok(val) => {
                            self.gen.notify.notify_waiters();
                            let obj = Object::new(realm);

                            obj.define_property("done".into(), true.into(), realm)?;
                            obj.define_property("value".into(), val, realm)?;

                            self.promise.resolve(&obj.into(), realm)?;
                        }
                        Err(e) => {
                            self.gen.notify.notify_waiters();
                            let e = ErrorObj::error_to_value(e, realm);
                            self.promise.reject(&e, realm)?;
                        }
                    }

                    Poll::Ready(Ok(()))
                }
                AsyncGeneratorPoll::Yield(state, mut val) => {
                    self.gen.state.replace(Some(state));
                    self.gen.notify.notify_one();

                    if let Value::Object(obj) = &val {
                        if let Some(promise) = obj.downcast::<Promise>() {
                            let promise = promise.try_map_refed(|promise| {
                                let Some(notify) = promise.notify.notified() else {
                                    return Err(());
                                };

                                Ok((promise, notify, true))
                            });

                            match promise {
                                Ok(promise) => {
                                    self.await_promise = Some(promise);
                                    return Poll::Pending;
                                }
                                Err((promise, ())) => {
                                    let value = promise
                                        .inner
                                        .borrow()
                                        .value
                                        .clone()
                                        .unwrap_or(Value::Undefined);

                                    val = value;
                                }
                            }
                        }
                    }

                    let obj = Object::new(realm);

                    obj.define_property("done".into(), false.into(), realm)?;
                    obj.define_property("value".into(), val, realm)?;

                    self.promise.resolve(&obj.into(), realm)?;

                    Poll::Ready(Ok(()))
                }
            }
        } else {
            Poll::Ready(Ok(()))
        }
    }
}
