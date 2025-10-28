#![allow(unused)]
use yavashark_macro::{object, props};
use crate::{Object, ObjectHandle, Realm, Value, Symbol};

#[object]
#[derive(Debug)]
pub struct AsyncDisposableStack {}


#[props(intrinsic_name = async_disposable_stack)]
impl AsyncDisposableStack {

    pub fn adopt(&self, value: Value, on_dispose: ObjectHandle) -> Value {
        value
    }

    pub fn defer(&self, on_dispose: ObjectHandle) {

    }

    #[prop("disposeAsync")]
    pub fn dispose_async(&self) {

    }

    pub fn move_(&self) -> ObjectHandle {
        Object::null()
    }

    #[prop("use")]
    pub fn use_(&self, value: Value) -> Value {
        value
    }

    #[prop(Symbol::ASYNC_DISPOSE)]
    pub fn symbol_async_dispose(&self) {

    }
}