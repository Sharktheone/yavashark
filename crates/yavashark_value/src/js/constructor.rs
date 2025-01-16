use std::fmt::Debug;
use yavashark_garbage::GcRef;
use crate::{BoxedObj, Error, Obj, ObjectProperty, Realm, Value};

pub trait Constructor<R: Realm>: Debug + Obj<R> {
    fn construct(&self, realm: &mut R, args: Vec<Value<R>>) -> Result<Value<R>, Error<R>>;
    
    fn construct_proto(&self) -> Result<ObjectProperty<R>, Error<R>> {
        Ok(self.resolve_property(&"prototype".into())?.unwrap_or(Value::Undefined.into()))
    }
}


pub trait ConstructorFn<R: Realm>: Debug {
    fn gc_untyped_ref(&self) -> Option<GcRef<BoxedObj<R>>>;
    fn construct(&self, args: Vec<Value<R>>, this: Value<R>, realm: &mut R) -> Result<(), Error<R>>;
}


#[derive(Debug)]
pub struct NoOpConstructorFn;

impl<R: Realm> ConstructorFn<R> for NoOpConstructorFn {
    
    fn gc_untyped_ref(&self) -> Option<GcRef<BoxedObj<R>>> {
        None
    }
    
    fn construct(&self, _args: Vec<Value<R>>, _this: Value<R>, _realm: &mut R) -> Result<(), Error<R>> {
        Ok(())
    }
}