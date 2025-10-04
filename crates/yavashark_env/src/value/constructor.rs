use crate::error::Error;
use crate::value::{BoxedObj, Obj, ObjectProperty, Value};
use crate::{ObjectHandle, Realm};
use std::fmt::Debug;
use yavashark_garbage::GcRef;

pub trait Constructor: Debug + Obj {
    fn construct(&self, realm: &mut Realm, args: Vec<Value>) -> Result<ObjectHandle, Error>;

    fn is_constructable(&self) -> bool {
        true
    }
}

pub trait ConstructorFn: Debug {
    fn gc_untyped_ref(&self) -> Option<GcRef<BoxedObj>>;
    fn construct(&self, args: Vec<Value>, this: Value, realm: &mut Realm) -> Result<(), Error>;
}

#[derive(Debug)]
pub struct NoOpConstructorFn;

impl ConstructorFn for NoOpConstructorFn {
    fn gc_untyped_ref(&self) -> Option<GcRef<BoxedObj>> {
        None
    }

    fn construct(&self, _args: Vec<Value>, _this: Value, _realm: &mut Realm) -> Result<(), Error> {
        Ok(())
    }
}
