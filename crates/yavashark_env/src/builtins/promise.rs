mod into_promise;

pub use into_promise::*;

use crate::conversion::FromValueOutput;
use crate::error::ErrorObj;
use crate::{Error, MutObject, NativeFunction, Object, ObjectHandle, Realm, Res, Value, ValueResult};
use std::cell::{Cell, RefCell};
use std::fmt::Debug;
use futures::future::join_all;
use tokio::sync::futures::Notified;
use tokio::sync::Notify;
use yavashark_garbage::OwningGcGuard;
use yavashark_macro::{object, props};
use yavashark_value::{BoxedObj, Obj};
use crate::array::Array;
use crate::utils::ValueIterator;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PromiseState {
    Pending,
    Fulfilled,
    Rejected,
}


pub enum PromiseResult {
    Fulfilled(Value),
    Rejected(Value),
}

#[object]
#[derive(Debug)]
pub struct Promise {
    pub notify: PromiseNotify,
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

#[derive(Debug)]
pub struct PromiseNotify {
    notify: Notify,
    finished: Cell<bool>,
}

impl PromiseNotify {
    pub fn new() -> Self {
        Self {
            notify: Notify::new(),
            finished: Cell::new(false),
        }
    }

    pub fn finished(&self) {
        self.notify.notify_waiters();
        self.finished.set(true);
    }

    pub fn notified(&self) -> Option<Notified<'_>> {
        if self.finished.get() {
            None
        } else {
            Some(self.notify.notified())
        }
    }
}

pub enum Callable {
    JsFunction(ObjectHandle),
    NativeFunction(Box<dyn Fn(Value, Value, &mut Realm) -> Res>),
}

impl Debug for Callable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Callable::JsFunction(func) => write!(f, "JsFunction({:?})", func),
            Callable::NativeFunction(_) => write!(f, "NativeFunction"),
        }
    }
}

impl Callable {
    pub fn call(&self, realm: &mut Realm, arg: Value, this: Value) -> ValueResult {
        match self {
            Callable::JsFunction(func) => func.call(realm, vec![arg], this),
            Callable::NativeFunction(func) => {
                func(arg, this, realm)?;

                Ok(Value::Undefined)
            }
        }
    }
}

#[object]
#[derive(Debug)]
pub struct FullfilledHandler {
    pub promise: OwningGcGuard<'static, BoxedObj<Realm>, Promise>,
    pub f: Callable,
}

#[object]
#[derive(Debug)]
pub struct RejectedHandler {
    pub promise: OwningGcGuard<'static, BoxedObj<Realm>, Promise>,
    pub f: Callable,
}

impl Promise {
    #[must_use]
    pub fn new(realm: &Realm) -> Self {
        Self {
            notify: PromiseNotify::new(),
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
        if let Some(notify) = self.notify.notified() {
            notify.await
        }

        Ok(self
            .inner
            .try_borrow_mut()?
            .value
            .clone()
            .unwrap_or(Value::Undefined))
    }

    pub async fn wait_to_res(&self) -> Res<PromiseResult> {
        if let Some(notify) = self.notify.notified() {
            notify.await;
        }

        let inner = self.inner.try_borrow_mut()?;
        let value = inner.value.clone().unwrap_or(Value::Undefined);

        match self.state.get() {
            PromiseState::Fulfilled => Ok(PromiseResult::Fulfilled(value)),
            PromiseState::Rejected => Ok(PromiseResult::Rejected(value)),
            PromiseState::Pending => Err(Error::new("Promise is still pending")),
        }
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

        self.notify.finished();

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

        self.notify.finished();

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

    pub fn get_fullfilled(this: GcPromise, realm: &mut Realm) -> ObjectHandle {
        NativeFunction::new(
            "on_fullfilled",
            move |args, _, realm| {
                this.resolve(&args.first().cloned().unwrap_or(Value::Undefined), realm)?;

                Ok(Value::Undefined)
            },
            realm,
        )
    }

    pub fn get_rejected(this: GcPromise, realm: &mut Realm) -> ObjectHandle {
        NativeFunction::new(
            "on_rejected",
            move |args, _, realm| {
                this.reject(&args.first().cloned().unwrap_or(Value::Undefined), realm)?;

                Ok(Value::Undefined)
            },
            realm,
        )
    }

    pub fn get_gc(this: ObjectHandle) -> Res<GcPromise> {
        <&Self>::from_value_out(this.into())
    }

    pub fn with_callback(callback: &ObjectHandle, realm: &mut Realm) -> Res<ObjectHandle> {
        let promise = Self::new(realm).into_object();

        let gc = Self::get_gc(promise.clone())?;

        let on_fullfilled = Self::get_fullfilled(gc.clone(), realm);
        let on_rejected = Self::get_rejected(gc, realm);

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

    #[prop("withResolvers")]
    fn with_resolvers(#[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        let ret = Object::new(realm);

        let promise = Self::new(realm).into_object();
        ret.set("promise", promise.clone(), realm)?;

        let gc = Self::get_gc(promise)?;

        let resolve = Self::get_fullfilled(gc.clone(), realm);
        let reject = Self::get_rejected(gc, realm);

        ret.set("resolve", resolve, realm)?;
        ret.set("reject", reject, realm)?;

        Ok(ret)
    }


    pub fn all(
        promises: &Value,
        #[realm] realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let iter = match ValueIterator::new(promises, realm) {
            Ok(iter) => iter,
            Err(err) => {
                let err = ErrorObj::error_to_value(err, realm);

                return Promise::reject_(&err, realm);
            }
        };

        let mut promises = Vec::new();

        while let Some(p) = match iter.next(realm) {
            Ok(p) => p,
            Err(err) => {
                let err = ErrorObj::error_to_value(err, realm);

                return Promise::reject_(&err, realm);
            }
        } {
            if let Ok(prom) = <&Self>::from_value_out(p) {
                promises.push(prom);
            }
        }

        let futures = promises.into_iter().map(|p| {
            p.map_refed(Self::wait_to_res)
        });

        let fut = join_all(
            futures
        );

        let array_proto = realm.intrinsics.array.clone().into();

        let fut = async move {
            let results = fut.await;

            let mut values = Vec::new();
            for res in results {
                match res? {
                    PromiseResult::Fulfilled(val) => values.push(val),
                    PromiseResult::Rejected(val) => {
                        return Err(Error::throw(val))
                    }
                }
            }

            let array = Array::with_elements_and_proto(array_proto, values)?;

            Ok(array.into_object())
        };





        Ok(fut.into_promise(realm))
    }
    #[prop("allSettled")]
    pub fn all_settled(
        promises: &Value,
        #[realm] realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let iter = match ValueIterator::new(promises, realm) {
            Ok(iter) => iter,
            Err(err) => {
                let err = ErrorObj::error_to_value(err, realm);

                return Promise::reject_(&err, realm);
            }
        };

        let mut promises = Vec::new();

        while let Some(p) = match iter.next(realm) {
            Ok(p) => p,
            Err(err) => {
                let err = ErrorObj::error_to_value(err, realm);

                return Promise::reject_(&err, realm);
            }
        } {
            if let Ok(prom) = <&Self>::from_value_out(p) {
                promises.push(prom);
            }
        }

        let futures = promises.into_iter().map(|p| {
            p.map_refed(Self::wait_to_res)
        });

        let fut = join_all(
            futures
        );

        let array_proto = realm.intrinsics.array.clone().into();
        let obj_proto: Value = realm.intrinsics.obj.clone().into();

        let fut = async move {
            let results = fut.await;

            let mut values = Vec::new();
            for res in results {
                match res? {
                    PromiseResult::Fulfilled(val) => {
                        let obj = Object::with_proto(obj_proto.clone());

                        obj.define_property("status".into(), "fulfilled".into())?;
                        obj.define_property("value".into(), val)?;

                        values.push(obj.into())
                    },
                    PromiseResult::Rejected(val) => {
                        let obj = Object::with_proto(obj_proto.clone());

                        obj.define_property("status".into(), "rejected".into())?;
                        obj.define_property("reason".into(), val)?;

                        values.push(obj.into())
                    },
                }
            }

            let array = Array::with_elements_and_proto(array_proto, values)?;

            Ok(array.into_object())
        };

        Ok(fut.into_promise(realm))
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
            f: Callable::JsFunction(f),
        }
    }

    pub fn new_native(
        promise: OwningGcGuard<'static, BoxedObj<Realm>, Promise>,
        f: impl Fn(Value, Value, &mut Realm) -> Res + 'static,
        realm: &Realm,
    ) -> Self {
        Self {
            inner: RefCell::new(MutableFullfilledHandler {
                object: MutObject::with_proto(realm.intrinsics.func.clone().into()),
            }),
            promise,
            f: Callable::NativeFunction(Box::new(f)),
        }
    }

    pub fn handle(&self, value: Value, realm: &mut Realm) -> Res {
        match self.f.call(realm, value, Value::Undefined) {
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
            f: Callable::JsFunction(f),
        }
    }

    pub fn handle(&self, value: Value, realm: &mut Realm) -> Res {
        match self.f.call(realm, value, Value::Undefined) {
            Ok(ret) => self.promise.resolve(&ret, realm),
            Err(err) => {
                let val = ErrorObj::error_to_value(err, realm);

                self.promise.reject(&val, realm)
            }
        }
    }
}
