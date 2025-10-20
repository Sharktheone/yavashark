mod into_promise;

pub use into_promise::*;

use crate::array::Array;
use crate::conversion::downcast_obj;
use crate::error_obj::ErrorObj;
use crate::utils::ValueIterator;
use crate::value::{BoxedObj, IntoValue, Obj};
use crate::{
    Error, MutObject, NativeFunction, Object, ObjectHandle, Realm, Res, Value, ValueResult,
};
use futures::future::{join_all, select_all};
use std::cell::{Cell, RefCell};
use std::fmt::Debug;
use tokio::sync::futures::Notified;
use tokio::sync::Notify;
use yavashark_garbage::OwningGcGuard;
use yavashark_macro::{object, props};

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

impl Default for PromiseNotify {
    fn default() -> Self {
        Self::new()
    }
}

impl PromiseNotify {
    #[must_use]
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
            Self::JsFunction(func) => write!(f, "JsFunction({func:?})"),
            Self::NativeFunction(_) => write!(f, "NativeFunction"),
        }
    }
}

impl Callable {
    pub fn call(&self, realm: &mut Realm, arg: Value, this: Value) -> ValueResult {
        match self {
            Self::JsFunction(func) => func.call(vec![arg], this, realm),
            Self::NativeFunction(func) => {
                func(arg, this, realm)?;

                Ok(Value::Undefined)
            }
        }
    }
}

#[derive(Debug)]
pub struct FullfilledHandler {
    pub promise: OwningGcGuard<'static, BoxedObj, Promise>,
    pub f: Callable,
}

#[derive(Debug)]
pub struct RejectedHandler {
    pub promise: OwningGcGuard<'static, BoxedObj, Promise>,
    pub f: Callable,
}

impl Promise {
    pub fn new(realm: &mut Realm) -> Res<Self> {
        Ok(Self {
            notify: PromiseNotify::new(),
            state: Cell::new(PromiseState::Pending),
            inner: RefCell::new(MutablePromise {
                object: MutObject::with_proto(
                    realm.intrinsics.clone_public().promise.get(realm)?.clone(),
                ),
                value: None,
                on_fulfilled: Vec::new(),
                on_rejected: Vec::new(),
                finally: Vec::new(),
            }),
        })
    }

    pub async fn wait(&self) -> ValueResult {
        if let Some(notify) = self.notify.notified() {
            notify.await;
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
            handler.call(vec![], Value::Undefined, realm)?;
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
            handler.call(vec![], Value::Undefined, realm)?;
        }

        self.notify.finished();

        Ok(())
    }

    pub fn set_res(&self, res: ValueResult, realm: &mut Realm) -> Res {
        match res {
            Ok(val) => self.resolve(&val, realm)?,
            Err(err) => {
                let val = ErrorObj::error_to_value(err, realm)?;
                self.reject(&val, realm)?;
            }
        }

        Ok(())
    }

    pub fn get_fullfilled(this: GcPromise, realm: &mut Realm) -> ObjectHandle {
        NativeFunction::with_len(
            "on_fullfilled",
            move |args, _, realm| {
                this.resolve(&args.first().cloned().unwrap_or(Value::Undefined), realm)?;

                Ok(Value::Undefined)
            },
            realm,
            1,
        )
    }

    pub fn get_rejected(this: GcPromise, realm: &mut Realm) -> ObjectHandle {
        NativeFunction::with_len(
            "on_rejected",
            move |args, _, realm| {
                this.reject(&args.first().cloned().unwrap_or(Value::Undefined), realm)?;

                Ok(Value::Undefined)
            },
            realm,
            1,
        )
    }

    pub fn get_gc(this: ObjectHandle) -> Res<GcPromise> {
        downcast_obj::<Self>(this.into())
    }

    pub fn with_callback(callback: &ObjectHandle, realm: &mut Realm) -> Res<ObjectHandle> {
        let promise = Self::new(realm)?.into_object();

        let gc = Self::get_gc(promise.clone())?;

        let on_fullfilled = Self::get_fullfilled(gc.clone(), realm);
        let on_rejected = Self::get_rejected(gc, realm);

        if let Err(e) = callback.call(
            vec![on_fullfilled.into(), on_rejected.into()],
            Value::Undefined,
            realm,
        ) {
            let val = ErrorObj::error_to_value(e, realm)?;
            let gc = Self::get_gc(promise.clone())?;
            gc.reject(&val, realm)?;
        }

        Ok(promise)
    }

    pub fn rejected(val: &Value, realm: &mut Realm) -> Res<ObjectHandle> {
        let promise = Self::new(realm)?;
        promise.reject(val, realm)?;
        Ok(promise.into_object())
    }

    pub fn resolved(val: &Value, realm: &mut Realm) -> Res<ObjectHandle> {
        let promise = Self::new(realm)?;
        promise.resolve(val, realm)?;
        Ok(promise.into_object())
    }
}

#[props(intrinsic_name = promise, to_string_tag = "Promise")]
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
    ) -> Res<ObjectHandle> {
        let mut inner = self.inner.try_borrow_mut()?;
        // let this_prom = <&Promise>::from_value_out(this.clone())?;

        let state = self.state.get();

        let promise = Self::new(realm)?.into_object();

        let promise_obj = downcast_obj::<Self>(promise.clone().into())?;

        if let Some(on_fulfilled) = on_fulfilled {
            match state {
                PromiseState::Fulfilled => {
                    let val = inner.value.clone().unwrap_or(Value::Undefined);
                    let handler = FullfilledHandler::new(promise_obj.clone(), on_fulfilled);

                    handler.handle(val, realm)?;
                }
                PromiseState::Pending => {
                    let handler = FullfilledHandler::new(promise_obj.clone(), on_fulfilled);
                    inner.on_fulfilled.push(handler);
                }
                PromiseState::Rejected => {}
            }
        }

        if let Some(on_rejected) = on_rejected {
            match state {
                PromiseState::Rejected => {
                    let val = inner.value.clone().unwrap_or(Value::Undefined);
                    let handler = RejectedHandler::new(promise_obj.clone(), on_rejected);
                    handler.handle(val, realm)?;
                }
                PromiseState::Pending => {
                    let handler = RejectedHandler::new(promise_obj.clone(), on_rejected);
                    inner.on_rejected.push(handler);
                }
                PromiseState::Fulfilled => {}
            }
        }

        Ok(promise)
    }

    pub fn catch(&self, f: ObjectHandle, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        self.then(None, Some(f), realm)
    }

    pub fn finally(&self, f: ObjectHandle, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        let mut inner = self.inner.try_borrow_mut()?;

        let promise = Self::new(realm)?;

        match self.state.get() {
            PromiseState::Pending => {
                inner.finally.push(f);
            }
            _ => match f.call(vec![], Value::Undefined, realm) {
                Err(e) => {
                    let val = ErrorObj::error_to_value(e, realm)?;
                    promise.reject(&val, realm)?;
                }
                Ok(val) => {
                    if let Ok(prom) = downcast_obj::<Self>(val) {
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
    fn resolve_(val: &Value, #[realm] realm: &mut Realm, this: &Value) -> Res<ObjectHandle> {
        if !this.is_constructable() {
            return Err(Error::ty("Promise capability must be a constructor"));
        }

        let promise = Self::new(realm)?;

        promise.resolve(val, realm)?;

        Ok(promise.into_object())
    }

    #[prop("reject")]
    fn reject_(val: &Value, #[realm] realm: &mut Realm, this: &Value) -> Res<ObjectHandle> {
        if !this.is_constructable() {
            return Err(Error::ty("Promise capability must be a constructor"));
        }

        Self::rejected(val, realm)
    }

    #[prop("try")]
    fn try_(
        callback: &ObjectHandle,
        args: Vec<Value>,
        #[realm] realm: &mut Realm,
        #[this] this: Value,
    ) -> Res<ObjectHandle> {
        if !this.is_constructable() {
            return Err(Error::ty("Promise capability must be a constructor"));
        }

        let promise = Self::new(realm)?;

        let ret = callback.call(args, this, realm);

        promise.set_res(ret, realm)?;

        Ok(promise.into_object())
    }

    #[prop("withResolvers")]
    fn with_resolvers(#[realm] realm: &mut Realm, this: Value) -> Res<ObjectHandle> {
        if !this.is_constructable() {
            return Err(Error::ty("Promise capability must be a constructor"));
        }

        let ret = Object::new(realm);

        let promise = Self::new(realm)?.into_object();
        ret.set("promise", promise.clone(), realm)?;

        let gc = Self::get_gc(promise)?;

        let resolve = Self::get_fullfilled(gc.clone(), realm);
        let reject = Self::get_rejected(gc, realm);

        ret.set("resolve", resolve, realm)?;
        ret.set("reject", reject, realm)?;

        Ok(ret)
    }

    pub fn all(promises: &Value, #[realm] realm: &mut Realm, this: &Value) -> Res<ObjectHandle> {
        if !this.is_constructable() {
            return Err(Error::ty("Promise capability must be a constructor"));
        }

        let iter = match ValueIterator::new(promises, realm) {
            Ok(iter) => iter,
            Err(err) => {
                let err = ErrorObj::error_to_value(err, realm)?;

                return Self::rejected(&err, realm);
            }
        };

        let mut promises = Vec::new();

        while let Some(mut p) = match iter.next(realm) {
            Ok(p) => p,
            Err(err) => {
                let err = ErrorObj::error_to_value(err, realm)?;

                iter.close(realm)?;

                return Self::rejected(&err, realm);
            }
        } {
            let then = p
                .get_property_opt("then", realm)?
                .unwrap_or(Value::Undefined);

            if !then.is_callable() {
                p = Self::resolved(&p, realm)?.into();
            }

            if let Ok(prom) = downcast_obj::<Self>(p) {
                promises.push(prom);
            }
        }

        iter.close(realm)?;

        let futures = promises.into_iter().map(|p| p.map_refed(Self::wait_to_res));

        let fut = join_all(futures);

        let array_proto = realm
            .intrinsics
            .clone_public()
            .array
            .get(realm)?
            .clone()
            .into();

        let fut = async move {
            let results = fut.await;

            let mut values = Vec::new();
            for res in results {
                match res? {
                    PromiseResult::Fulfilled(val) => values.push(val),
                    PromiseResult::Rejected(val) => return Err(Error::throw(val)),
                }
            }

            let array = Array::with_elements_and_proto(array_proto, values)?;

            Ok(array.into_object())
        };

        fut.into_promise(realm)
    }
    #[prop("allSettled")]
    pub fn all_settled(
        promises: &Value,
        #[realm] realm: &mut Realm,
        this: &Value,
    ) -> Res<ObjectHandle> {
        if !this.is_constructable() {
            return Err(Error::ty("Promise capability must be a constructor"));
        }

        let iter = match ValueIterator::new(promises, realm) {
            Ok(iter) => iter,
            Err(err) => {
                let err = ErrorObj::error_to_value(err, realm)?;

                return Self::rejected(&err, realm);
            }
        };

        let mut promises = Vec::new();

        while let Some(mut p) = match iter.next(realm) {
            Ok(p) => p,
            Err(err) => {
                let err = ErrorObj::error_to_value(err, realm)?;

                iter.close(realm)?;

                return Self::rejected(&err, realm);
            }
        } {
            let then = p
                .get_property_opt("then", realm)?
                .unwrap_or(Value::Undefined);

            if !then.is_callable() {
                p = Self::resolved(&p, realm)?.into();
            }

            if let Ok(prom) = downcast_obj::<Self>(p) {
                promises.push(prom);
            }
        }

        iter.close(realm)?;

        let futures = promises.into_iter().map(|p| p.map_refed(Self::wait_to_res));

        let fut = join_all(futures);

        let array_proto = realm.intrinsics.clone_public().array.get(realm)?.clone();
        let obj_proto = realm.intrinsics.obj.clone();

        let fut = async move {
            let results = fut.await;

            let mut values = Vec::new();
            for res in results {
                match res? {
                    PromiseResult::Fulfilled(val) => {
                        let obj = Object::from_values_with_proto(
                            vec![("status".into(), "fulfilled".into()), ("value".into(), val)],
                            obj_proto.clone(),
                        )?;

                        values.push(obj.into());
                    }
                    PromiseResult::Rejected(val) => {
                        let obj = Object::from_values_with_proto(
                            vec![("status".into(), "rejected".into()), ("reason".into(), val)],
                            obj_proto.clone(),
                        )?;

                        values.push(obj.into());
                    }
                }
            }

            let array = Array::with_elements_and_proto(array_proto, values)?;

            Ok(array.into_object())
        };

        fut.into_promise(realm)
    }

    fn any(promises: &Value, #[realm] realm: &mut Realm, this: &Value) -> Res<ObjectHandle> {
        if !this.is_constructable() {
            return Err(Error::ty("Promise capability must be a constructor"));
        }

        let iter = match ValueIterator::new(promises, realm) {
            Ok(iter) => iter,
            Err(err) => {
                let err = ErrorObj::error_to_value(err, realm)?;

                return Self::rejected(&err, realm);
            }
        };

        let mut promises = Vec::new();

        while let Some(mut p) = match iter.next(realm) {
            Ok(p) => p,
            Err(err) => {
                let err = ErrorObj::error_to_value(err, realm)?;

                iter.close(realm)?;

                return Self::rejected(&err, realm);
            }
        } {
            let then = p
                .get_property_opt("then", realm)?
                .unwrap_or(Value::Undefined);

            if !then.is_callable() {
                p = Self::resolved(&p, realm)?.into();
            }

            if let Ok(prom) = downcast_obj::<Self>(p) {
                promises.push(prom);
            }
        }

        iter.close(realm)?;

        if promises.is_empty() {
            return Self::resolved(&Array::from_realm(realm)?.into_value(), realm);
        }

        for prom in &promises {
            if prom.state.get() == PromiseState::Fulfilled {
                let val = prom
                    .inner
                    .try_borrow()?
                    .value
                    .clone()
                    .unwrap_or(Value::Undefined);
                return Self::resolved(&val, realm);
            }
        }

        let futures = promises
            .into_iter()
            .map(|p| Box::pin(p.map_refed(Self::wait_to_res)));

        let mut fut = select_all(futures);

        let fut = async move {
            loop {
                let (res, _, others) = fut.await;

                match res? {
                    PromiseResult::Fulfilled(val) => return Ok(val),
                    PromiseResult::Rejected(_) => {
                        if others.is_empty() {
                            return Err(Error::throw(Value::Undefined));
                        }

                        fut = select_all(others);
                    }
                }
            }
        };

        fut.into_promise(realm)
    }

    fn race(promises: &Value, #[realm] realm: &mut Realm, this: &Value) -> Res<ObjectHandle> {
        if !this.is_constructable() {
            return Err(Error::ty("Promise capability must be a constructor"));
        }

        let iter = match ValueIterator::new(promises, realm) {
            Ok(iter) => iter,
            Err(err) => {
                let err = ErrorObj::error_to_value(err, realm)?;

                return Self::rejected(&err, realm);
            }
        };

        let mut promises = Vec::new();

        while let Some(mut p) = match iter.next(realm) {
            Ok(p) => p,
            Err(err) => {
                let err = ErrorObj::error_to_value(err, realm)?;

                iter.close(realm)?;

                return Self::rejected(&err, realm);
            }
        } {
            let then = p
                .get_property_opt("then", realm)?
                .unwrap_or(Value::Undefined);

            if !then.is_callable() {
                p = Self::resolved(&p, realm)?.into();
            }

            if let Ok(prom) = downcast_obj::<Self>(p) {
                promises.push(prom);
            }
        }

        iter.close(realm)?;

        if promises.is_empty() {
            return Self::resolved(&Array::from_realm(realm)?.into_value(), realm);
        }

        for prom in &promises {
            match prom.state.get() {
                PromiseState::Fulfilled => {
                    let val = prom
                        .inner
                        .try_borrow()?
                        .value
                        .clone()
                        .unwrap_or(Value::Undefined);
                    return Self::resolved(&val, realm);
                }
                PromiseState::Rejected => {
                    let val = prom
                        .inner
                        .try_borrow()?
                        .value
                        .clone()
                        .unwrap_or(Value::Undefined);
                    return Self::rejected(&val, realm);
                }
                PromiseState::Pending => {}
            }
        }

        let futures = promises
            .into_iter()
            .map(|p| Box::pin(p.map_refed(Self::wait_to_res)));

        let fut = select_all(futures);

        let fut = async move {
            let (res, _, _) = fut.await;

            match res? {
                PromiseResult::Fulfilled(val) => Ok(val),
                PromiseResult::Rejected(val) => Err(Error::throw(val)),
            }
        };

        fut.into_promise(realm)
    }
}

impl FullfilledHandler {
    #[must_use]
    pub const fn new(promise: OwningGcGuard<'static, BoxedObj, Promise>, f: ObjectHandle) -> Self {
        Self {
            promise,
            f: Callable::JsFunction(f),
        }
    }

    pub fn new_native(
        promise: OwningGcGuard<'static, BoxedObj, Promise>,
        f: impl Fn(Value, Value, &mut Realm) -> Res + 'static,
    ) -> Self {
        Self {
            promise,
            f: Callable::NativeFunction(Box::new(f)),
        }
    }

    pub fn handle(&self, value: Value, realm: &mut Realm) -> Res {
        match self.f.call(realm, value, Value::Undefined) {
            Ok(ret) => {
                if let Ok(prom) = downcast_obj::<Promise>(ret.clone()) {
                    return match prom.state.get() {
                        PromiseState::Fulfilled => {
                            let val = prom
                                .inner
                                .try_borrow()?
                                .value
                                .clone()
                                .unwrap_or(Value::Undefined);
                            self.promise.resolve(&val, realm)
                        }
                        PromiseState::Rejected => {
                            let val = prom
                                .inner
                                .try_borrow()?
                                .value
                                .clone()
                                .unwrap_or(Value::Undefined);
                            self.promise.reject(&val, realm)
                        }
                        PromiseState::Pending => {
                            let mut inner = self.promise.inner.try_borrow_mut()?;

                            let mut other = prom.inner.try_borrow_mut()?;

                            other.on_fulfilled.append(&mut inner.on_fulfilled);
                            other.on_rejected.append(&mut inner.on_rejected);
                            Ok(())
                        }
                    };
                }

                self.promise.resolve(&ret, realm)
            }
            Err(err) => {
                let val = ErrorObj::error_to_value(err, realm)?;

                self.promise.reject(&val, realm)
            }
        }
    }
}

impl RejectedHandler {
    #[must_use]
    pub fn new(promise: OwningGcGuard<'static, BoxedObj, Promise>, f: ObjectHandle) -> Self {
        Self {
            promise,
            f: Callable::JsFunction(f),
        }
    }

    pub fn handle(&self, value: Value, realm: &mut Realm) -> Res {
        match self.f.call(realm, value, Value::Undefined) {
            Ok(ret) => self.promise.resolve(&ret, realm),
            Err(err) => {
                let val = ErrorObj::error_to_value(err, realm)?;

                self.promise.reject(&val, realm)
            }
        }
    }
}
