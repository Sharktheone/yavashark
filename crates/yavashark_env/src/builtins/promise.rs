mod into_promise;

pub use into_promise::*;

use crate::conversion::FromValueOutput;
use crate::error::ErrorObj;
use crate::{MutObject, NativeFunction, ObjectHandle, Realm, Res, Value, ValueResult};
use std::cell::{Cell, RefCell};
use tokio::sync::Notify;
use yavashark_garbage::OwningGcGuard;
use yavashark_macro::{object, props};
use yavashark_value::{BoxedObj, Obj};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PromiseState {
    Pending,
    Fulfilled,
    Rejected,
}

#[object]
#[derive(Debug)]
pub struct Promise {
    pub notify: Notify,
    pub state: Cell<PromiseState>,
    #[mutable]
    pub value: Option<Value>,
    #[mutable]
    pub on_fulfilled: Vec<FullfilledHandler>, //TODO: this is a mem leak!
    #[mutable]
    pub on_rejected: Vec<RejectedHandler>,
    #[mutable]
    pub finally: Vec<ObjectHandle>,
}

#[object]
#[derive(Debug)]
pub struct FullfilledHandler {
    pub promise: OwningGcGuard<'static, BoxedObj<Realm>, Promise>,
    pub f: ObjectHandle,
}

#[object]
#[derive(Debug)]
pub struct RejectedHandler {
    pub promise: OwningGcGuard<'static, BoxedObj<Realm>, Promise>,
    pub f: ObjectHandle,
}

impl Promise {
    #[must_use]
    pub fn new(realm: &Realm) -> Self {
        Self {
            notify: Notify::new(),
            state: Cell::new(PromiseState::Pending),
            inner: RefCell::new(MutablePromise {
                object: MutObject::with_proto(realm.intrinsics.promise.clone().into()),
                value: None,
                on_fulfilled: Vec::new(),
                on_rejected: Vec::new(),
                finally: Vec::new(),
            }),
        }
    }

    pub async fn wait(&self) -> ValueResult {
        self.notify.notified().await;

        Ok(self
            .inner
            .try_borrow_mut()?
            .value
            .clone()
            .unwrap_or(Value::Undefined))
    }

    pub fn resolve(&self, value: &Value, realm: &mut Realm) -> Res {
        let mut inner = self.inner.try_borrow_mut()?;

        if self.state.get() != PromiseState::Pending {
            return Ok(());
        }

        self.state.set(PromiseState::Fulfilled);
        inner.value = Some(value.clone());

        for handler in inner.on_fulfilled.drain(..) {
            handler.handle(value.clone(), realm)?;
        }

        for handler in inner.finally.drain(..) {
            handler.call(realm, vec![], Value::Undefined)?;
        }

        self.notify.notify_waiters();

        Ok(())
    }

    pub fn reject(&self, value: &Value, realm: &mut Realm) -> Res {
        let mut inner = self.inner.try_borrow_mut()?;

        if self.state.get() != PromiseState::Pending {
            return Ok(());
        }

        self.state.set(PromiseState::Rejected);
        inner.value = Some(value.clone());

        for handler in inner.on_rejected.drain(..) {
            handler.handle(value.clone(), realm)?;
        }
        //TODO: handle unhandled rejection

        for handler in inner.finally.drain(..) {
            handler.call(realm, vec![], Value::Undefined)?;
        }

        self.notify.notify_waiters();

        Ok(())
    }

    pub fn set_res(&self, res: ValueResult, realm: &mut Realm) -> Res {
        match res {
            Ok(val) => self.resolve(&val, realm)?,
            Err(err) => {
                let val = ErrorObj::error_to_value(err, realm);
                self.reject(&val, realm)?;
            }
        }

        Ok(())
    }

    pub fn with_callback(callback: &ObjectHandle, realm: &mut Realm) -> Res<ObjectHandle> {
        let promise = Self::new(realm).into_object();

        let promise_clone = promise.clone();
        let promise_clone2 = promise.clone();

        let on_fullfilled = NativeFunction::new(
            "on_fullfilled",
            move |args, _, realm| {
                let this = <&Self>::from_value_out(promise_clone.clone().into())?;
                this.resolve(&args.first().cloned().unwrap_or(Value::Undefined), realm)?;

                Ok(Value::Undefined)
            },
            realm,
        );

        let on_rejected = NativeFunction::new(
            "on_rejected",
            move |args, _, realm| {
                let this = <&Self>::from_value_out(promise_clone2.clone().into())?;
                this.reject(&args.first().cloned().unwrap_or(Value::Undefined), realm)?;

                Ok(Value::Undefined)
            },
            realm,
        );

        callback.call(
            realm,
            vec![on_fullfilled.into(), on_rejected.into()],
            Value::Undefined,
        )?;

        Ok(promise)
    }
}

#[props]
impl Promise {
    #[constructor]
    pub fn construct(callback: &ObjectHandle, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        Self::with_callback(callback, realm)
    }

    pub fn then(
        &self,
        on_fulfilled: Option<ObjectHandle>,
        on_rejected: Option<ObjectHandle>,
        #[realm] realm: &mut Realm,
        #[this] this: Value,
    ) -> Res<ObjectHandle> {
        let mut inner = self.inner.try_borrow_mut()?;
        // let this_prom = <&Promise>::from_value_out(this.clone())?;

        let state = self.state.get();

        let promise = Self::new(realm).into_object();

        let promise_obj = <&Self>::from_value_out(promise.clone().into())?;

        if let Some(on_fulfilled) = on_fulfilled {
            match state {
                PromiseState::Fulfilled => {
                    let val = inner.value.clone().unwrap_or(Value::Undefined);
                    let ret = on_fulfilled.call(realm, vec![val], this.clone())?;
                    promise_obj.resolve(&ret, realm)?;
                }
                PromiseState::Pending => {
                    let handler = FullfilledHandler::new(promise_obj.clone(), on_fulfilled, realm);
                    inner.on_fulfilled.push(handler);
                }
                PromiseState::Rejected => {}
            }
        };

        if let Some(on_rejected) = on_rejected {
            match state {
                PromiseState::Rejected => {
                    let val = inner.value.clone().unwrap_or(Value::Undefined);
                    let ret = on_rejected.call(realm, vec![val], this)?;
                    promise_obj.reject(&ret, realm)?;
                }
                PromiseState::Pending => {
                    let handler = RejectedHandler::new(promise_obj.clone(), on_rejected, realm);
                    inner.on_rejected.push(handler);
                }
                PromiseState::Fulfilled => {}
            }
        };

        if promise_obj.state.get() != state {
            promise_obj.state.set(state);
            promise_obj
                .inner
                .try_borrow_mut()?
                .value
                .clone_from(&inner.value);
        }

        Ok(promise)
    }

    pub fn catch(&self, f: ObjectHandle, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        self.then(None, Some(f), realm, Value::Undefined)
    }

    pub fn finally(&self, f: ObjectHandle, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        let mut inner = self.inner.try_borrow_mut()?;

        let promise = Self::new(realm);

        match self.state.get() {
            PromiseState::Pending => {
                inner.finally.push(f);
            }
            _ => match f.call(realm, vec![], Value::Undefined) {
                Err(e) => {
                    let val = ErrorObj::error_to_value(e, realm);
                    promise.reject(&val, realm)?;
                }
                Ok(val) => {
                    if let Ok(prom) = <&Self>::from_value_out(val) {
                        if prom.state.get() == PromiseState::Rejected {
                            promise.reject(
                                &prom
                                    .inner
                                    .try_borrow()?
                                    .value
                                    .clone()
                                    .unwrap_or(Value::Undefined),
                                realm,
                            )?;
                        }
                    }
                }
            },
        }

        Ok(promise.into_object())
    }

    #[prop("resolve")]
    fn resolve_(val: &Value, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        let promise = Self::new(realm);

        promise.resolve(val, realm)?;

        Ok(promise.into_object())
    }

    #[prop("reject")]
    fn reject_(val: &Value, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        let promise = Self::new(realm);

        promise.reject(val, realm)?;

        Ok(promise.into_object())
    }

    #[prop("try")]
    fn try_(
        callback: &ObjectHandle,
        args: Vec<Value>,
        #[realm] realm: &mut Realm,
        #[this] this: Value,
    ) -> Res<ObjectHandle> {
        let promise = Self::new(realm);

        let ret = callback.call(realm, args, this);

        promise.set_res(ret, realm)?;

        Ok(promise.into_object())
    }
}

impl FullfilledHandler {
    #[must_use]
    pub fn new(
        promise: OwningGcGuard<'static, BoxedObj<Realm>, Promise>,
        f: ObjectHandle,
        realm: &Realm,
    ) -> Self {
        Self {
            inner: RefCell::new(MutableFullfilledHandler {
                object: MutObject::with_proto(realm.intrinsics.func.clone().into()),
            }),
            promise,
            f,
        }
    }

    pub fn handle(&self, value: Value, realm: &mut Realm) -> Res {
        match self.f.call(realm, vec![value], Value::Undefined) {
            Ok(ret) => self.promise.resolve(&ret, realm),
            Err(err) => {
                let val = ErrorObj::error_to_value(err, realm);

                self.promise.reject(&val, realm)
            }
        }
    }
}

impl RejectedHandler {
    #[must_use]
    pub fn new(
        promise: OwningGcGuard<'static, BoxedObj<Realm>, Promise>,
        f: ObjectHandle,
        realm: &Realm,
    ) -> Self {
        Self {
            inner: RefCell::new(MutableRejectedHandler {
                object: MutObject::with_proto(realm.intrinsics.func.clone().into()),
            }),
            promise,
            f,
        }
    }

    pub fn handle(&self, value: Value, realm: &mut Realm) -> Res {
        match self.f.call(realm, vec![value], Value::Undefined) {
            Ok(ret) => self.promise.resolve(&ret, realm),
            Err(err) => {
                let val = ErrorObj::error_to_value(err, realm);

                self.promise.reject(&val, realm)
            }
        }
    }
}
