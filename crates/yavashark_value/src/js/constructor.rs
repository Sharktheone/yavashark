use std::fmt::Debug;

use crate::{Ctx, Obj, Value};

pub trait Constructor<C: Ctx>: Debug + Obj<C> {
    /// Gets the constructor function for this object.
    fn get_constructor(&self) -> Value<C>;

    /// Is this a special constructor? (we can call it without `new`)
    fn special_constructor(&self) -> bool {
        false
    }

    /// Gets the constructor value for this object (what gets fed into the constructor's this-value)
    fn value(&self, ctx: &mut C) -> Value<C>;


    /// Gets the constructor prototype for this object (useful for slightly cheaper `instanceof` checks)
    fn proto(&self, ctx: &mut C) -> Value<C> {
        if let Value::Object(o) = self.value(ctx) {
            o.get().map_or(Value::Undefined, |o| o.prototype())
        } else {
            Value::Undefined
        }
    }
}