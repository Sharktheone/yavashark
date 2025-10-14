use crate::error::Error;
use crate::value::{BoxedObj, Obj, Object, PrimitiveValue, Symbol, Value};
use half::f16;
use num_bigint::BigInt;
use std::any::type_name;
use std::rc::Rc;
use yavashark_garbage::OwningGcGuard;
use yavashark_string::YSString;

impl From<&'static str> for Value {
    fn from(s: &'static str) -> Self {
        Self::String(YSString::new_static(s))
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Self::String(YSString::from_string(s))
    }
}

impl From<&String> for Value {
    fn from(s: &String) -> Self {
        Self::String(YSString::from_ref(s))
    }
}

impl From<YSString> for Value {
    fn from(s: YSString) -> Self {
        Self::String(s)
    }
}

impl From<&YSString> for Value {
    fn from(s: &YSString) -> Self {
        Self::String(s.clone())
    }
}

impl From<()> for Value {
    fn from((): ()) -> Self {
        Self::Undefined
    }
}

impl From<f64> for Value {
    fn from(n: f64) -> Self {
        Self::Number(n)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Self::Boolean(b)
    }
}

impl From<u8> for Value {
    fn from(n: u8) -> Self {
        Self::Number(f64::from(n))
    }
}

impl From<u16> for Value {
    fn from(n: u16) -> Self {
        Self::Number(f64::from(n))
    }
}

impl From<u32> for Value {
    fn from(n: u32) -> Self {
        Self::Number(f64::from(n))
    }
}

impl From<u64> for Value {
    fn from(n: u64) -> Self {
        Self::Number(n as f64)
    }
}

impl From<i8> for Value {
    fn from(n: i8) -> Self {
        Self::Number(f64::from(n))
    }
}

impl From<i16> for Value {
    fn from(n: i16) -> Self {
        Self::Number(f64::from(n))
    }
}

impl From<i32> for Value {
    fn from(n: i32) -> Self {
        Self::Number(f64::from(n))
    }
}

impl From<i64> for Value {
    fn from(n: i64) -> Self {
        Self::Number(n as f64)
    }
}

impl From<usize> for Value {
    fn from(n: usize) -> Self {
        Self::Number(n as f64)
    }
}

impl From<isize> for Value {
    fn from(n: isize) -> Self {
        Self::Number(n as f64)
    }
}

impl From<f16> for Value {
    fn from(n: f16) -> Self {
        Self::Number(f64::from(n))
    }
}

impl From<f32> for Value {
    fn from(n: f32) -> Self {
        Self::Number(f64::from(n))
    }
}

impl From<BigInt> for Value {
    fn from(n: BigInt) -> Self {
        Self::BigInt(Rc::new(n))
    }
}

impl From<Rc<BigInt>> for Value {
    fn from(n: Rc<BigInt>) -> Self {
        Self::BigInt(n)
    }
}

impl From<Value> for Result<Value, Error> {
    fn from(value: Value) -> Self {
        Ok(value)
    }
}

impl<O: Into<Object>> From<O> for Value {
    fn from(o: O) -> Self {
        Self::Object(o.into())
    }
}

impl From<&'static str> for PrimitiveValue {
    fn from(s: &'static str) -> Self {
        Self::String(YSString::new_static(s))
    }
}

impl From<String> for PrimitiveValue {
    fn from(s: String) -> Self {
        Self::String(YSString::from_string(s))
    }
}

impl From<&String> for PrimitiveValue {
    fn from(s: &String) -> Self {
        Self::String(YSString::from_ref(s))
    }
}

impl From<YSString> for PrimitiveValue {
    fn from(s: YSString) -> Self {
        Self::String(s)
    }
}

impl From<&YSString> for PrimitiveValue {
    fn from(s: &YSString) -> Self {
        Self::String(s.clone())
    }
}

impl From<()> for PrimitiveValue {
    fn from((): ()) -> Self {
        Self::Undefined
    }
}

impl From<f64> for PrimitiveValue {
    fn from(n: f64) -> Self {
        Self::Number(n)
    }
}

impl From<bool> for PrimitiveValue {
    fn from(b: bool) -> Self {
        Self::Boolean(b)
    }
}

impl From<u8> for PrimitiveValue {
    fn from(n: u8) -> Self {
        Self::Number(f64::from(n))
    }
}

impl From<u16> for PrimitiveValue {
    fn from(n: u16) -> Self {
        Self::Number(f64::from(n))
    }
}

impl From<u32> for PrimitiveValue {
    fn from(n: u32) -> Self {
        Self::Number(f64::from(n))
    }
}

impl From<u64> for PrimitiveValue {
    fn from(n: u64) -> Self {
        Self::Number(n as f64)
    }
}

impl From<i8> for PrimitiveValue {
    fn from(n: i8) -> Self {
        Self::Number(f64::from(n))
    }
}

impl From<i16> for PrimitiveValue {
    fn from(n: i16) -> Self {
        Self::Number(f64::from(n))
    }
}

impl From<i32> for PrimitiveValue {
    fn from(n: i32) -> Self {
        Self::Number(f64::from(n))
    }
}

impl From<i64> for PrimitiveValue {
    fn from(n: i64) -> Self {
        Self::Number(n as f64)
    }
}

impl From<usize> for PrimitiveValue {
    fn from(n: usize) -> Self {
        Self::Number(n as f64)
    }
}

impl From<isize> for PrimitiveValue {
    fn from(n: isize) -> Self {
        Self::Number(n as f64)
    }
}

impl From<f16> for PrimitiveValue {
    fn from(n: f16) -> Self {
        Self::Number(f64::from(n))
    }
}

impl From<f32> for PrimitiveValue {
    fn from(n: f32) -> Self {
        Self::Number(f64::from(n))
    }
}

impl From<BigInt> for PrimitiveValue {
    fn from(n: BigInt) -> Self {
        Self::BigInt(Rc::new(n))
    }
}

impl From<Rc<BigInt>> for PrimitiveValue {
    fn from(n: Rc<BigInt>) -> Self {
        Self::BigInt(n)
    }
}

impl From<Symbol> for PrimitiveValue {
    fn from(n: Symbol) -> Self {
        Self::Symbol(n)
    }
}

impl From<&Symbol> for PrimitiveValue {
    fn from(n: &Symbol) -> Self {
        Self::Symbol(n.clone())
    }
}

// impl From<WeakValue> for PrimitiveValue {
//     fn from(v: WeakValue) -> Self {
//         match v.upgrade() {
//             Some(strong) => strong.into(),
//             None => PrimitiveValue::Undefined,
//         }
//     }
// }

pub trait FromValue: Sized {
    fn from_value(value: Value) -> Result<Self, Error>;
}

pub trait IntoValue {
    fn into_value(self) -> Value;
}

pub trait IntoValueRef {
    type ValueRef: AsRef<Value>;

    fn into_value_ref(self) -> Self::ValueRef;
}

impl<V: IntoValue> IntoValueRef for V {
    type ValueRef = Value;

    fn into_value_ref(self) -> Self::ValueRef {
        self.into_value()
    }
}

impl IntoValue for Value {
    fn into_value(self) -> Self {
        self
    }
}

impl FromValue for Value {
    fn from_value(value: Self) -> Result<Self, Error> {
        Ok(value)
    }
}

impl FromValue for YSString {
    fn from_value(value: Value) -> Result<Self, Error> {
        match value {
            Value::String(s) => Ok(s),
            _ => Err(Error::ty_error(format!(
                "Expected a string, found {value:?}"
            ))),
        }
    }
}

// impl FromValue for String {
//     fn from_value(value: Value) -> Result<Self, Error> {
//         match value {
//             Value::String(s) => Ok(s.to_string()),
//             _ => Err(Error::ty_error(format!(
//                 "Expected a string, found {value:?}"
//             ))),
//         }
//     }
// }

impl FromValue for bool {
    fn from_value(value: Value) -> Result<Self, Error> {
        match value {
            Value::Boolean(b) => Ok(b),
            _ => Err(Error::ty_error(format!(
                "Expected a boolean, found {value:?}"
            ))),
        }
    }
}

impl FromValue for BigInt {
    fn from_value(value: Value) -> Result<Self, Error> {
        match value {
            Value::BigInt(n) => Ok((*n).clone()),
            _ => Err(Error::ty_error(format!(
                "Expected a BigInt, found {value:?}"
            ))),
        }
    }
}

impl FromValue for Object {
    fn from_value(value: Value) -> Result<Self, Error> {
        match value {
            Value::Object(o) => Ok(o),
            _ => Err(Error::ty_error(format!(
                "Expected an object, found {value:?}"
            ))),
        }
    }
}

impl FromValue for () {
    fn from_value(value: Value) -> Result<Self, Error> {
        match value {
            Value::Undefined => Ok(()),
            _ => Err(Error::ty_error(format!(
                "Expected undefined, found {value:?}"
            ))),
        }
    }
}

impl IntoValue for String {
    fn into_value(self) -> Value {
        Value::String(self.into())
    }
}

impl IntoValue for YSString {
    fn into_value(self) -> Value {
        Value::String(self)
    }
}

impl IntoValue for &'static str {
    fn into_value(self) -> Value {
        Value::String(YSString::new_static(self))
    }
}

impl IntoValue for bool {
    fn into_value(self) -> Value {
        Value::Boolean(self)
    }
}


impl<T: Obj> IntoValue for T {
    fn into_value(self) -> Value {
        Value::Object(self.into_object())
    }
}
impl IntoValue for Object {
    fn into_value(self) -> Value {
        Value::Object(self)
    }
}

impl IntoValue for BigInt {
    fn into_value(self) -> Value {
        Value::BigInt(Rc::new(self))
    }
}

impl IntoValue for Rc<BigInt> {
    fn into_value(self) -> Value {
        Value::BigInt(self)
    }
}

impl IntoValue for Symbol {
    fn into_value(self) -> Value {
        Value::Symbol(self)
    }
}

impl IntoValue for &Symbol {
    fn into_value(self) -> Value {
        Value::Symbol(self.clone())
    }
}

impl IntoValue for () {
    fn into_value(self) -> Value {
        Value::Undefined
    }
}

impl<T: IntoValue> IntoValue for Option<T> {
    fn into_value(self) -> Value {
        self.map_or(Value::Undefined, IntoValue::into_value)
    }
}

macro_rules! impl_from_value {
    ($($t:ty),*) => {
        $(
            impl FromValue for $t {
                fn from_value(value: Value) -> Result<Self, Error> {
                    match value {
                        Value::Number(n) => Ok(n as $t),
                        _ => Err(Error::ty_error(format!("Expected a number, found {:?}", value))),
                    }
                }
            }

            impl IntoValue for $t {
                fn into_value(self) -> Value {
                    #[allow(clippy::cast_lossless)]
                    Value::Number(self as f64)
                }
            }
        )*
    };
    () => {};
}

impl_from_value!(u8, u16, u32, u64, i8, i16, i32, i64, i128, usize, isize, f32, f64);

impl<V: 'static> FromValue for OwningGcGuard<'_, BoxedObj, V> {
    fn from_value(value: Value) -> Result<Self, Error> {
        let obj = match value {
            Value::Object(obj) => Ok(obj.get_owning()),
            _ => Err(Error::ty_error(format!(
                "Expected an object, found {value:?}"
            ))),
        }?;

        obj.maybe_map(BoxedObj::downcast).map_err(|obj| {
            Error::ty_error(format!(
                "Expected an object of type {:?}, found {:?}",
                obj.class_name(),
                type_name::<V>(),
            ))
        })
    }
}
