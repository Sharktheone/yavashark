#![allow(unused)]
use crate::{Object, ObjectHandle, Realm, Symbol, Value};
use yavashark_macro::{object, props};

#[object]
#[derive(Debug)]
pub struct DisposableStack {

}

#[props(intrinsic_name = disposable_stack)]
impl DisposableStack {
    pub fn adopt(&self, value: Value, on_dispose: ObjectHandle) -> Value {
        value
    }

    pub fn defer(&self, on_dispose: ObjectHandle) {}

    pub fn dispose(&self) {}

    pub fn move_(&self) -> ObjectHandle {
        Object::null()
    }

    #[prop("use")]
    pub fn use_(&self, value: Value) -> Value {
        value
    }

    #[prop(Symbol::DISPOSE)]
    pub fn symbol_dispose(&self) {}
}
