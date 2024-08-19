use std::fmt::Debug;

use crate::{Ctx, Obj, ObjectProperty, Value};

pub trait Constructor<C: Ctx>: Debug + Obj<C> {
    /// Gets the constructor function for this object.
    fn get_constructor(&self) -> ObjectProperty<C>;

    /// Is this a special constructor? (we can call it without `new`)
    fn special_constructor(&self) -> bool {
        false
    }

    /// Gets the constructor value for this object (what gets fed into the constructor's this-value)
    fn value(&self, ctx: &mut C) -> Value<C>;

    /// Gets the constructor prototype for this object (useful for slightly cheaper `instanceof` checks)
    fn proto(&self, ctx: &mut C) -> Value<C> {
        if let Value::Object(obj) = self.value(ctx) {
            let Ok(o) = obj.get() else {
                return Value::Undefined;
            };

            let p = o.prototype();
            drop(o);

            p.resolve(Value::Object(obj), ctx)
                .unwrap_or(Value::Undefined)
        } else {
            Value::Undefined //TODO: return an error here
        }

        // if let Value::Object(obj) = self.value(ctx) {
        //TODO: this here causes an rust borrow checker bug, but the one above works somehow
        //     if let Ok(o) = obj.get() {
        //         let p = o.prototype();
        //         drop(o);
        //
        //         p.resolve(Value::Object(obj), ctx)
        //             .unwrap_or(Value::Undefined)
        //     } else {
        //         Value::Undefined
        //     }
        //
        // } else {
        //     Value::Undefined //TODO: return an error here
        // }
    }
}
