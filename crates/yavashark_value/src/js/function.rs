use std::fmt::Debug;
use crate::{Ctx, Error, Obj, Value};

pub trait Func<C: Ctx>: Debug + Obj<C>  {
    fn call(
        &mut self,
        ctx: &mut C,
        args: Vec<Value<C>>,
        this: Value<C>,
    ) -> Result<Value<C>, Error<C>>;
}