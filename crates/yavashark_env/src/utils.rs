mod iterator;
mod protodefault;

pub use iterator::*;
pub use protodefault::*;
use yavashark_value::ObjectImpl;
use crate::{Object, ObjectHandle, Realm, Res, Value};
use crate::builtins::{BigIntObj, BooleanObj, NumberObj, StringObj, SymbolObj};

pub fn coerce_object(value: Value, realm: &mut Realm) -> Res<ObjectHandle> {
    Ok(match value {
        Value::Object(obj) => obj,
        Value::Number(num) => NumberObj::with_number(realm, num)?,
        Value::String(string) => StringObj::with_string(realm, string).into_object(),
        Value::Boolean(boolean) => BooleanObj::new(realm, boolean),
        Value::Symbol(symbol) => SymbolObj::new(realm, symbol),
        Value::BigInt(bigint) => BigIntObj::new(realm, bigint),
        Value::Undefined | Value::Null => Object::new(realm),
    })
}
