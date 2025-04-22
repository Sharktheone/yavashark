use crate::{AsyncPoll, ResumableVM, VmState};
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};
use tokio::sync::futures::Notified;
use yavashark_bytecode::BytecodeFunctionCode;
use yavashark_env::builtins::Promise;
use yavashark_env::conversion::FromValueOutput;
use yavashark_env::error::ErrorObj;
use yavashark_env::scope::Scope;
use yavashark_env::task_queue::AsyncTask;
use yavashark_env::value::{BoxedObj, Obj};
use yavashark_env::{ObjectHandle, Realm, Res, Value};
use yavashark_garbage::{OwningGcGuard, OwningGcGuardRefed};

pub struct BytecodeAsyncTask {
    state: Option<VmState>,
    await_promise:
        Option<OwningGcGuardRefed<BoxedObj<Realm>, (&'static Promise, Notified<'static>)>>,
    promise: OwningGcGuard<'static, BoxedObj<Realm>, Promise>,
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
        let promise_obj = Promise::new(realm).into_object();
        let promise = <&Promise>::from_value_out(promise_obj.clone().into())?;

        let this = Self {
            state: Some(state),
            await_promise: None,
            promise,
        };

        realm.queue.queue_task(this);

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

                state.continue_async(val)?;
            }
        }

        _ = inner.await_promise.take();

        if let Some(state) = inner.state.take() {
            let vm = ResumableVM::from_state(state, realm);
            match vm.poll() {
                AsyncPoll::Await(state, promise) => {
                    inner.state = Some(state);
                    let promise = match <&Promise>::from_value_out(promise.into()) {
                        Ok(promise) => promise,
                        Err(e) => return Poll::Ready(Err(e)),
                    };

                    let promise = promise.map_refed(|promise| (promise, promise.notify.notified()));

                    inner.await_promise = Some(promise);

                    Poll::Pending
                }
                AsyncPoll::Ret(state, ret) => {
                    match ret {
                        Ok(()) => inner.promise.resolve(&state.acc, realm)?,
                        Err(e) => {
                            let e = ErrorObj::error_to_value(e, realm);
                            inner.promise.reject(&e, realm)?;
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
