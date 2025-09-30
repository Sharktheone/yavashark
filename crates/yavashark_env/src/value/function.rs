use std::fmt::Debug;
use crate::error::Error;
use crate::value::{Obj, Realm, Value};

pub trait Func<C: Realm>: Debug + Obj<C> {
    fn call(
        &self,
        realm: &mut C,
        args: Vec<Value<C>>,
        this: Value<C>,
    ) -> Result<Value<C>, Error<C>>;
}
