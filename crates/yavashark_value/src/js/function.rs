use crate::{Ctx, Error, Obj, Value};
use std::fmt::Debug;

pub trait Func<C: Ctx>: Debug + Obj<C> {
    fn call(
        &mut self,
        ctx: &mut C,
        args: Vec<Value<C>>,
        this: Value<C>,
    ) -> Result<Value<C>, Error<C>>;
}
