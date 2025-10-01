use crate::error::Error;
use crate::value::{Obj, Value};
use crate::Realm;
use std::fmt::Debug;

pub trait Func: Debug + Obj {
    fn call(&self, realm: &mut Realm, args: Vec<Value>, this: Value) -> Result<Value, Error>;
}
