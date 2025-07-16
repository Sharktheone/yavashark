use crate::{fmt_num, Realm, Symbol, Value};
use yavashark_string::{ToYSString, YSString};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PropertyKey {
    String(YSString),
    Symbol(Symbol),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InternalPropertyKey {
    String(YSString),
    Symbol(Symbol),
    Index(usize),
}

impl<R: Realm> From<Value<R>> for PropertyKey {
    fn from(value: Value<R>) -> Self {
        match value {
            Value::String(s) => Self::String(s),
            Value::Symbol(s) => Self::Symbol(s),
            Value::Null => Self::String("null".into()),
            Value::Undefined => Self::String("undefined".into()),
            Value::Number(n) => Self::String(n.to_string().into()),
            Value::Boolean(b) => Self::String(b.to_string().into()),
            Value::BigInt(b) => Self::String(b.to_string().into()),
            Value::Object(obj) => Self::String(obj.to_ys_string()),
        }
    }
}

impl<R: Realm> From<PropertyKey> for Value<R> {
    fn from(key: PropertyKey) -> Self {
        match key {
            PropertyKey::String(s) => Value::String(s),
            PropertyKey::Symbol(s) => Value::Symbol(s),
        }
    }
}

impl<R: Realm> From<Value<R>> for InternalPropertyKey {
    fn from(value: Value<R>) -> Self {
        match value {
            Value::String(s) => {
                s.parse::<usize>()
                    .map_or_else(|_| Self::String(s), Self::Index)
                //TODO: this is a hack, we should not parse strings to usize
            }
            Value::Symbol(s) => Self::Symbol(s),
            Value::Null => Self::String("null".into()),
            Value::Undefined => Self::String("undefined".into()),
            Value::Number(n) => {
                if !n.is_nan() && !n.is_infinite() && n.fract() == 0.0 {
                    Self::Index(n as usize)
                } else {
                    Self::String(fmt_num(n))
                }
            }
            Value::Boolean(b) => Self::String(b.to_string().into()),
            Value::BigInt(b) => Self::String(b.to_string().into()),
            Value::Object(obj) => Self::String(obj.to_ys_string()),
        }
    }
}

impl<R: Realm> From<InternalPropertyKey> for Value<R> {
    fn from(key: InternalPropertyKey) -> Self {
        match key {
            InternalPropertyKey::String(s) => Value::String(s),
            InternalPropertyKey::Symbol(s) => Value::Symbol(s),
            InternalPropertyKey::Index(i) => Value::Number(i as f64),
        }
    }
}

impl From<InternalPropertyKey> for PropertyKey {
    fn from(key: InternalPropertyKey) -> Self {
        match key {
            InternalPropertyKey::String(s) => PropertyKey::String(s),
            InternalPropertyKey::Symbol(s) => PropertyKey::Symbol(s),
            InternalPropertyKey::Index(i) => PropertyKey::String(i.to_string().into()),
        }
    }
}
