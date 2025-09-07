mod iterator;
mod protodefault;

use crate::builtins::{BigIntObj, BooleanObj, NumberObj, StringObj, SymbolObj};
use crate::{Object, ObjectHandle, Realm, Res, Value};
pub use iterator::*;
pub use protodefault::*;
use yavashark_value::ObjectImpl;

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


pub fn coerce_object_strict(value: Value, realm: &mut Realm) -> Res<ObjectHandle> {
    Ok(match value {
        Value::Object(obj) => obj,
        Value::Number(num) => NumberObj::with_number(realm, num)?,
        Value::String(string) => StringObj::with_string(realm, string).into_object(),
        Value::Boolean(boolean) => BooleanObj::new(realm, boolean),
        Value::Symbol(symbol) => SymbolObj::new(realm, symbol),
        Value::BigInt(bigint) => BigIntObj::new(realm, bigint),
        Value::Undefined | Value::Null => return Err(yavashark_value::Error::ty("Cannot convert undefined or null to object")),
    })
}
