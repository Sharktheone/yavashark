use std::fmt::Debug;
use crate::error::Error;
use crate::Realm;
use crate::value::{Obj, Value};

pub trait Func: Debug + Obj {
    fn call(
        &self,
        realm: &mut Realm,
        args: Vec<Value>,
        this: Value,
    ) -> Result<Value, Error>;
}
