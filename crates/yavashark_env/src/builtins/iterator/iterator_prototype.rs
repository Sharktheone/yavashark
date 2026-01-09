// 27.1.2 The %IteratorPrototype% Object
//
// This is the base prototype that all iterator prototypes inherit from.
// It provides the @@iterator method that returns `this`.

use crate::{Realm, Res, Symbol, Value};
use yavashark_macro::props;

/// %IteratorPrototype% - The base iterator prototype
///
/// All iterator prototype objects (ArrayIterator.prototype, MapIterator.prototype, etc.)
/// inherit from this object.
///
/// Properties:
/// - [[Prototype]]: %Object.prototype%
/// - @@iterator: A function that returns `this`
/// - @@dispose: A function that calls `return()` if present
pub struct IteratorPrototype;

#[props(intrinsic_name = iterator_prototype)]
impl IteratorPrototype {
    /// 27.1.2.1 %IteratorPrototype% [ @@iterator ] ( )
    /// Returns `this` value.
    #[prop(Symbol::ITERATOR)]
    fn iterator(#[this] this: Value) -> Value {
        this
    }

    /// 27.1.2.14 %IteratorPrototype% [ @@dispose ] ( )
    /// Calls `return()` on the iterator if it exists.
    #[prop(Symbol::DISPOSE)]
    fn dispose(#[this] this: Value, #[realm] realm: &mut Realm) -> Res<Value> {
        // 1. Let O be the this value.
        let o = this.clone().to_object()?;

        // 2. Let return be ? GetMethod(O, "return").
        let return_method = o.get("return", realm)?;

        // 3. If return is not undefined, then
        if return_method.is_callable() {
            // a. Perform ? Call(return, O).
            return_method.call(realm, Vec::new(), this)?;
        }

        // 4. Return undefined.
        Ok(Value::Undefined)
    }
}
