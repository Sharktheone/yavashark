use crate::{Ctx, Obj, Value};
use std::fmt::Debug;
pub trait ConstructValue<C: Ctx>: Debug + Obj<C> {
    fn get_constructor_value(&self, ctx: &mut C) -> Option<Value<C>>;
}



pub trait Constructor<C: Ctx>: Debug + Obj<C> {
    fn get_constructor(&self) -> Value<C>;
}