use std::fmt::Debug;
use yavashark_garbage::GcRef;
use crate::error::Error;
use crate::Realm;
use crate::value::{BoxedObj, Obj, ObjectProperty, Value};

pub trait Constructor: Debug + Obj {
    fn construct(&self, realm: &mut Realm, args: Vec<Value>) -> Result<Value, Error>;

    fn construct_proto(&self) -> Result<ObjectProperty, Error> {
        Ok(self
            .resolve_property(&"prototype".into())?
            .unwrap_or(Value::Undefined.into()))
    }

    fn is_constructor(&self) -> bool {
        true
    }
}

pub trait ConstructorFn: Debug {
    fn gc_untyped_ref(&self) -> Option<GcRef<BoxedObj>>;
    fn construct(&self, args: Vec<Value>, this: Value, realm: &mut Realm)
        -> Result<(), Error>;
}

#[derive(Debug)]
pub struct NoOpConstructorFn;

impl ConstructorFn for NoOpConstructorFn {
    fn gc_untyped_ref(&self) -> Option<GcRef<BoxedObj>> {
        None
    }

    fn construct(
        &self,
        _args: Vec<Value>,
        _this: Value,
        _realm: &mut Realm,
    ) -> Result<(), Error> {
        Ok(())
    }
}
