use std::fmt::Debug;
use crate::{Ctx, Obj, Value};
pub trait ConstructValue<C: Ctx>: Debug + Obj<C> {
    fn get_constructor_value(
        &self,
        ctx: &mut C,
    ) -> Option<Value<C>>;
}
