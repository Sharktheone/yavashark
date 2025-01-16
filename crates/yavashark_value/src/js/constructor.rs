use std::fmt::Debug;
use crate::{Error, Obj, ObjectProperty, Realm, Value};

pub trait Constructor<R: Realm>: Debug + Obj<R> {
    fn construct(&self, realm: &mut R, args: Vec<Value<R>>) -> Result<Value<R>, Error<R>>;
    
    fn construct_proto(&self) -> Result<ObjectProperty<R>, Error<R>> {
        Ok(self.resolve_property(&"prototype".into())?.unwrap_or(Value::Undefined.into()))
    }
}
