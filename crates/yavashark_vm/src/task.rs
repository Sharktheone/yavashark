use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use tokio::sync::futures::Notified;
use std::task::{Context, Poll};
use yavashark_env::{ObjectHandle, Realm, Res};
use yavashark_env::builtins::Promise;
use yavashark_env::conversion::FromValueOutput;
use yavashark_env::error::ErrorObj;
use yavashark_env::scope::Scope;
use yavashark_env::task_queue::AsyncTask;
use yavashark_env::value::{BoxedObj, Obj};
use yavashark_garbage::{OwningGcGuard, OwningGcGuardRefed};
use crate::{AsyncPoll, AsyncVM, VmState, VM};
use crate::function_code::BytecodeFunctionCode;

pub struct BytecodeAsyncTask {
    state: Option<VmState>,
    await_promise: Option<OwningGcGuardRefed<BoxedObj<Realm>, (&'static Promise, Notified<'static>)>>,
    promise: OwningGcGuard<'static, BoxedObj<Realm>, Promise>,
}


impl Unpin for BytecodeAsyncTask {}

impl BytecodeAsyncTask {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(code: Rc<BytecodeFunctionCode>, realm: &mut Realm, scope: Scope) -> Res<ObjectHandle> {
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
            }
        }
        
        
        
        if let Some(state) = inner.state.take() {
            let vm = AsyncVM::from_state(state, realm);
            match vm.run() {
                AsyncPoll::Await(state, promise) => {
                    inner.state = Some(state);
                    let promise = match <&Promise>::from_value_out(promise.into()) {
                        Ok(promise) => promise,
                        Err(e) => return Poll::Ready(Err(e.into()))
                    };
                    
                    
                    let promise = promise.map_refed(|promise| {
                        (promise, promise.notify.notified())
                    });
                    
                    inner.await_promise = Some(promise);
                    
                    Poll::Pending
                }
                AsyncPoll::Ret(state, ret) => {
                    match ret {
                        Ok(_) => inner.promise.resolve(state.acc, realm)?,
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