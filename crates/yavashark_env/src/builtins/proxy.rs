use std::any::TypeId;
use std::ptr::NonNull;
use yavashark_garbage::GcRef;
use yavashark_string::YSString;
use yavashark_value::{BoxedObj, Obj, ObjectImpl};
use crate::{Error, Object, ObjectHandle, ObjectProperty, Realm, Res, Value, Variable};

#[derive(Debug)]
pub struct Proxy {
    inner: ObjectHandle,
    handler: ObjectHandle,
}


impl Obj<Realm> for Proxy {
    fn define_property(&self, name: Value, value: Value) -> Res {
        todo!()
    }

    fn define_variable(&self, name: Value, value: Variable) -> Res {
        todo!()
    }

    fn resolve_property(&self, name: &Value) -> Result<Option<ObjectProperty>, Error> {
        todo!()
    }

    fn get_property(&self, name: &Value) -> Result<Option<ObjectProperty>, Error> {
        todo!()
    }

    fn define_getter(&self, name: Value, value: Value) -> Res {
        todo!()
    }

    fn define_setter(&self, name: Value, value: Value) -> Res {
        todo!()
    }

    fn delete_property(&self, name: &Value) -> Result<Option<Value>, Error> {
        todo!()
    }

    fn contains_key(&self, name: &Value) -> Result<bool, Error> {
        todo!()
    }

    fn has_key(&self, name: &Value) -> Result<bool, Error> {
        todo!()
    }

    fn name(&self) -> String {
        self.inner.name()
    }

    fn to_string(&self, realm: &mut Realm) -> Result<YSString, Error> {
        self.inner.to_string(realm)
    }

    fn to_string_internal(&self) -> Result<YSString, Error> {
        self.inner.to_string_internal()
    }

    fn properties(&self) -> Result<Vec<(Value, Value)>, Error> {
        todo!()
    }

    fn keys(&self) -> Result<Vec<Value>, Error> {
        todo!()
    }

    fn values(&self) -> Result<Vec<Value>, Error> {
        todo!()
    }

    fn get_array_or_done(&self, index: usize) -> Result<(bool, Option<Value>), Error> {
        todo!()
    }

    fn clear_values(&self) -> Res {
        todo!()
    }

    fn call(&self, realm: &mut Realm, args: Vec<Value>, this: Value) -> Result<Value, Error> {
        todo!()
    }

    fn is_function(&self) -> bool {
        self.inner.is_function()
    }

    fn primitive(&self) -> Option<Value> {
        self.inner.primitive()
    }

    fn prototype(&self) -> Result<ObjectProperty, Error> {
        todo!()
    }

    fn set_prototype(&self, proto: ObjectProperty) -> Res {
        todo!()
    }

    fn constructor(&self) -> Result<ObjectProperty, Error> {
        todo!()
    }

    unsafe fn custom_gc_refs(&self) -> Vec<GcRef<BoxedObj<Realm>>> {
        todo!()
    }

    fn class_name(&self) -> &'static str {
        self.inner.class_name()
    }

    fn construct(&self, realm: &mut Realm, args: Vec<Value>) -> Result<Value, Error> {
        todo!()
    }

    fn is_constructor(&self) -> bool {
        self.inner.is_constructor()
    }

    unsafe fn inner_downcast(&self, ty: TypeId) -> Option<NonNull<()>> {
        self.inner.inner_downcast(ty)
    }
}