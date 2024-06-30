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
    fn get_constructor_value(&self, ctx: &mut C) -> Option<Value<C>>;
    
    
    /// Gets the constructor prototype for this object (useful for slightly cheaper `instanceof` checks)
    fn get_constructor_proto(&self, ctx: &mut C) -> Option<Value<C>> {
        self.get_constructor_value(ctx).map(|v| v.get_proto(ctx))
    }
}