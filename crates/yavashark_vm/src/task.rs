use crate::{AsyncPoll, ResumableVM, VmState};
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};
use tokio::sync::futures::Notified;
use yavashark_bytecode::BytecodeFunctionCode;
use yavashark_env::builtins::{Promise, PromiseState};
use yavashark_env::conversion::downcast_obj;
use yavashark_env::error_obj::ErrorObj;
use yavashark_env::scope::Scope;
use yavashark_env::task_queue::{AsyncTask, AsyncTaskQueue};
use yavashark_env::value::{BoxedObj, Obj};
use yavashark_env::{ObjectHandle, Realm, Res, Value};
use yavashark_garbage::{OwningGcGuard, OwningGcGuardRefed};

pub struct BytecodeAsyncTask {
    state: Option<VmState>,
    await_promise: Option<OwningGcGuardRefed<BoxedObj, (&'static Promise, Notified<'static>)>>,
    promise: OwningGcGuard<'static, BoxedObj, Promise>,
}

impl Unpin for BytecodeAsyncTask {}

impl BytecodeAsyncTask {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        code: Rc<BytecodeFunctionCode>,
        realm: &mut Realm,
        scope: Scope,
    ) -> Res<ObjectHandle> {
        let state = VmState::new(code, scope);
        let promise_obj = Promise::new(realm)?.into_object();
        let promise = downcast_obj::<Promise>(promise_obj.clone().into())?;

        let this = Self {
            state: Some(state),
            await_promise: None,
            promise,
        };

        AsyncTaskQueue::queue_task(this, realm);

        Ok(promise_obj)
    }
}

impl AsyncTask for BytecodeAsyncTask {
    fn poll(self: Pin<&mut Self>, cx: &mut Context, realm: &mut Realm) -> Poll<Res> {
        let inner = Pin::into_inner(self);

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


                if promise.0.state.get() == PromiseState::Rejected {
                    state.handle_root_error(val)?;
                } else {
                    state.continue_async(val, realm)?;
                }
            }
        }

        _ = inner.await_promise.take();

        inner.poll_next(realm)
    }

    fn run_first_sync(&mut self, realm: &mut Realm) -> Poll<Res> {
        self.poll_next(realm)
    }
}

impl BytecodeAsyncTask {
    fn poll_next(&mut self, realm: &mut Realm) -> Poll<Res> {
        if let Some(state) = self.state.take() {
            let vm = ResumableVM::from_state(state, realm);
            match vm.poll() {
                AsyncPoll::Await(state, promise) => {
                    self.state = Some(state);
                    let promise = match downcast_obj::<Promise>(promise.into()) {
                        Ok(promise) => promise,
                        Err(e) => return Poll::Ready(Err(e)),
                    };

                    let promise = promise.try_map_refed(|promise| {
                        let Some(notify) = promise.notify.notified() else {
                            return Err(());
                        };

                        Ok((promise, notify))
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

                            self.state
                                .as_mut()
                                .map(|state| {
                                    if promise.state.get() == PromiseState::Rejected {
                                        _ = state.handle_root_error(val);
                                    } else {
                                        _ = state.continue_async(val, realm);
                                    }
                                });

                            return self.poll_next(realm);
                        }
                    }

                    Poll::Pending
                }
                AsyncPoll::Ret(state, ret) => {
                    match ret {
                        Ok(()) => self.promise.resolve(&state.acc, realm)?,
                        Err(e) => {
                            let e = ErrorObj::error_to_value(e, realm)?;
                            self.promise.reject(&e, realm)?;
                        }
                    }

                    Poll::Ready(Ok(()))
                }
            }
        } else {
            Poll::Ready(Ok(()))
        }
    }
}
