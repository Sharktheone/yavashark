use crate::{Error, Obj, Realm, Value};
use std::fmt::Debug;

pub trait Func<C: Realm>: Debug + Obj<C> {
    fn call(
        &self,
        realm: &mut C,
        args: Vec<Value<C>>,
        this: Value<C>,
    ) -> Result<Value<C>, Error<C>>;
}
